/*
 * srx: The fast Symbol Ranking based compressor.
 * Copyright (C) 2023  Mai Thanh Minh (a.k.a. thanhminhmr)
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either  version 3 of the  License,  or (at your option) any later
 * version.
 *
 * This program  is distributed in the hope  that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR  A PARTICULAR PURPOSE. See  the  GNU  General  Public   License  for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::basic::{AnyError, AnyResult, Closable};
use crate::bridged_context::{BridgedPrimaryContext, BridgedSecondaryContext};
use crate::primary_context::ByteMatched;
use crate::secondary_context::{Bit, BitDecoder, BitEncoder};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{env, thread};

mod basic;
mod bridged_context;
mod primary_context;
mod secondary_context;

// -----------------------------------------------

const BUFFER_SIZE: usize = 0x10000;
const BUFFER_SAFE_GUARD: usize = 0x8;

type Buffer = Box<[u32]>;
type BufferGuarded<'local> = MutexGuard<'local, Buffer>;

#[derive(Clone)]
struct BufferContainer(Arc<Mutex<Buffer>>);

impl Deref for BufferContainer {
    type Target = Arc<Mutex<Buffer>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BufferContainer {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(
            vec![0; BUFFER_SIZE].into_boxed_slice(),
        )))
    }
}

// -----------------------------------------------

struct BufferedEncoder<'local> {
    buffer: BufferGuarded<'local>,
    count: usize,
}

impl<'local> BufferedEncoder<'local> {
    fn new(buffer: BufferGuarded<'local>) -> Self {
        Self { buffer, count: 0 }
    }

    fn full(&self) -> bool {
        self.count + BUFFER_SAFE_GUARD >= BUFFER_SIZE
    }

    fn count(&self) -> usize {
        self.count
    }

    fn bit(&mut self, context: usize, bit: usize) {
        debug_assert!(bit == 0 || bit == 1);
        debug_assert!(context <= 0x007FFFFF);
        debug_assert!(self.count < BUFFER_SIZE);
        self.buffer[self.count] = ((context << 9) + bit) as u32;
        self.count += 1;
    }

    fn byte(&mut self, context: usize, byte: u8) {
        debug_assert!(self.count < BUFFER_SIZE);
        debug_assert!(context <= 0x007FFFFF);
        self.buffer[self.count] = ((context << 9) + (byte as usize) + 0x100) as u32;
        self.count += 1;
    }
}

// -----------------------------------------------

struct ThreadMessage {
    buffer: BufferContainer,
    count: usize,
}

impl ThreadMessage {
    fn new(buffer: BufferContainer, count: usize) -> Self {
        Self { buffer, count }
    }
}

// -----------------------------------------------

struct ThreadedEncoder<W: Write + Send + 'static> {
    buffer_which: bool,
    buffer_one: BufferContainer,
    buffer_two: BufferContainer,
    sender: SyncSender<ThreadMessage>,
    thread: JoinHandle<AnyResult<W>>,
}

impl<W: Write + Send + 'static> ThreadedEncoder<W> {
    fn new(output: W) -> Self {
        let (sender, receiver): (SyncSender<ThreadMessage>, Receiver<ThreadMessage>) =
            sync_channel(1);
        Self {
            buffer_which: true,
            buffer_one: BufferContainer::new(),
            buffer_two: BufferContainer::new(),
            sender,
            thread: thread::spawn(move || ThreadedEncoder::thread(output, receiver)),
        }
    }

    fn bit(
        encoder: &mut BitEncoder<W>,
        context: &mut BridgedSecondaryContext,
        context_index: usize,
        bit: usize,
    ) -> AnyResult<()> {
        debug_assert!(bit == 0 || bit == 1);
        let bit_enum: Bit = if bit == 0 { Bit::Zero } else { Bit::One };
        let prediction = context.update(context_index, bit_enum);
        encoder.bit(prediction, bit_enum)
    }

    fn byte(
        encoder: &mut BitEncoder<W>,
        context: &mut BridgedSecondaryContext,
        context_index: usize,
        byte: u8,
    ) -> AnyResult<()> {
        // code high 4 bits in first 15 contexts
        let high: usize = ((byte >> 4) | 16) as usize;
        Self::bit(encoder, context, context_index + 1, high >> 3 & 1)?;
        Self::bit(encoder, context, context_index + (high >> 3), high >> 2 & 1)?;
        Self::bit(encoder, context, context_index + (high >> 2), high >> 1 & 1)?;
        Self::bit(encoder, context, context_index + (high >> 1), high & 1)?;
        // code low 4 bits in one of 16 blocks of 15 contexts (to reduce cache misses)
        let low_context: usize = context_index + (15 * (high - 15)) as usize;
        let low: usize = ((byte & 15) | 16) as usize;
        Self::bit(encoder, context, low_context + 1, low >> 3 & 1)?;
        Self::bit(encoder, context, low_context + (low >> 3), low >> 2 & 1)?;
        Self::bit(encoder, context, low_context + (low >> 2), low >> 1 & 1)?;
        Self::bit(encoder, context, low_context + (low >> 1), low & 1)?;
        // oke
        return Ok(());
    }

    fn thread(writer: W, receiver: Receiver<ThreadMessage>) -> AnyResult<W> {
        let mut encoder: BitEncoder<W> = BitEncoder::new(writer);
        let mut context: BridgedSecondaryContext = BridgedSecondaryContext::new();
        loop {
            // receive message
            let message: ThreadMessage = match receiver.recv() {
                // receive normally
                Ok(value) => value,
                // the sender is closed, something wrong happened
                Err(_) => return Err(AnyError::new("Broken connection between threads!")),
            };
            // check if this is the end, exit if it is
            if message.count > BUFFER_SIZE {
                break;
            }
            // encode every bit in buffer
            let buffer: BufferGuarded = message.buffer.lock()?;
            for i in 0..message.count {
                let value: usize = buffer[i] as usize;
                if value & 0x100 == 0 {
                    Self::bit(&mut encoder, &mut context, value >> 9, value & 1)?;
                } else {
                    Self::byte(&mut encoder, &mut context, value >> 9, (value & 0xFF) as u8)?;
                }
            }
        }
        return encoder.close();
    }

    fn buffer(&self) -> &BufferContainer {
        if self.buffer_which {
            &self.buffer_one
        } else {
            &self.buffer_two
        }
    }

    fn flip(&mut self) {
        self.buffer_which = !self.buffer_which;
    }

    fn begin(&self) -> AnyResult<BufferedEncoder> {
        Ok(BufferedEncoder::new(self.buffer().lock()?))
    }

    fn end(&self, buffer: BufferedEncoder) -> AnyResult<()> {
        self.sender
            .send(ThreadMessage::new(self.buffer().clone(), buffer.count()))?;
        Ok(())
    }

    fn flush(self) -> AnyResult<W> {
        self.sender
            .send(ThreadMessage::new(self.buffer().clone(), BUFFER_SIZE + 1))?;
        match self.thread.join() {
            Ok(value) => value,
            Err(_) => Err(AnyError::new("Thread join failed!")),
        }
    }
}

// -----------------------------------------------

struct StreamEncoder<R: Read, W: Write + Send + 'static> {
    context: BridgedPrimaryContext,
    encoder: ThreadedEncoder<W>,
    reader: R,
}

impl<R: Read, W: Write + Send + 'static> StreamEncoder<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            context: BridgedPrimaryContext::new(),
            encoder: ThreadedEncoder::new(writer),
            reader,
        }
    }

    #[inline(never)]
    fn encode(mut self) -> AnyResult<(R, W)> {
        loop {
            let mut encoder: BufferedEncoder = self.encoder.begin()?;
            loop {
                let mut byte_result: [u8; 1] = [0];
                if self.reader.read(&mut byte_result)? == 0 {
                    // eof, encoded using first byte as literal
                    encoder.bit(self.context.first_context(), 1);
                    encoder.bit(self.context.second_context(), 0);
                    encoder.byte(self.context.literal_context(), self.context.first_byte());

                    self.encoder.end(encoder)?;
                    let writer: W = self.encoder.flush()?;
                    // gave the reader/writer back
                    return Ok((self.reader, writer));
                }

                let current_byte: u8 = byte_result[0];
                match self.context.matching(current_byte) {
                    ByteMatched::FIRST => {
                        encoder.bit(self.context.first_context(), 0);
                    }
                    ByteMatched::SECOND => {
                        encoder.bit(self.context.first_context(), 1);
                        encoder.bit(self.context.second_context(), 1);
                        encoder.bit(self.context.third_context(), 0);
                    }
                    ByteMatched::THIRD => {
                        encoder.bit(self.context.first_context(), 1);
                        encoder.bit(self.context.second_context(), 1);
                        encoder.bit(self.context.third_context(), 1);
                    }
                    ByteMatched::NONE => {
                        encoder.bit(self.context.first_context(), 1);
                        encoder.bit(self.context.second_context(), 0);
                        encoder.byte(self.context.literal_context(), current_byte);
                    }
                };

                if encoder.full() {
                    break;
                }
            }

            self.encoder.end(encoder)?;
            self.encoder.flip();
        }
    }
}

//endregion StreamEncoder
// -------------------------------------------------------------------------------------------------
//region StreamDecoder

struct StreamDecoder<R: Read, W: Write> {
    primary_context: BridgedPrimaryContext,
    secondary_context: BridgedSecondaryContext,
    decoder: BitDecoder<R>,
    writer: W,
}

impl<R: Read, W: Write> StreamDecoder<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            primary_context: BridgedPrimaryContext::new(),
            secondary_context: BridgedSecondaryContext::new(),
            decoder: BitDecoder::new(reader),
            writer,
        }
    }

    fn bit(&mut self, context_index: usize) -> AnyResult<usize> {
        let prediction: u32 = self.secondary_context.get(context_index);
        let bit: Bit = self.decoder.bit(prediction)?;
        self.secondary_context.update(context_index, bit);
        Ok(match bit {
            Bit::Zero => 0,
            Bit::One => 1,
        })
    }

    fn byte(&mut self, context_index: usize) -> AnyResult<u8> {
        let mut high: usize = 1;
        high += high + self.bit(context_index + high)?;
        high += high + self.bit(context_index + high)?;
        high += high + self.bit(context_index + high)?;
        high += high + self.bit(context_index + high)?;
        let low_context: usize = context_index + (15 * (high - 15)) as usize;
        let mut low: usize = 1;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        return Ok((((high - 16) << 4) | (low - 16)) as u8);
    }

    #[inline(never)]
    fn decode(mut self) -> AnyResult<(R, W)> {
        loop {
            let (next_byte, matched): (u8, ByteMatched) =
                if self.bit(self.primary_context.first_context())? == 0 {
                    // match first
                    (self.primary_context.first_byte(), ByteMatched::FIRST)
                } else if self.bit(self.primary_context.second_context())? == 0 {
                    // literal
                    let next_byte: u8 = self.byte(self.primary_context.literal_context())?;
                    if next_byte == self.primary_context.first_byte() {
                        // eof, gave the reader/writer back
                        let reader: R = self.decoder.close()?;
                        return Ok((reader, self.writer));
                    }
                    (next_byte, ByteMatched::NONE)
                } else if self.bit(self.primary_context.third_context())? == 0 {
                    // match second
                    (self.primary_context.second_byte(), ByteMatched::SECOND)
                } else {
                    // match third
                    (self.primary_context.third_byte(), ByteMatched::THIRD)
                };
            self.writer.write_all(&[next_byte])?;
            self.primary_context.matched(next_byte, matched);
        }
    }
}

//endregion StreamDecoder
// -------------------------------------------------------------------------------------------------

//endregion Stream Encoder/Decoder
// =================================================================================================

const SRX_HEADER: &[u8; 4] = b"sRx\x00";

fn run(input_path: &Path, output_path: &Path, is_compress: bool) -> AnyResult<(u64, u64, f64)> {
    // open file
    let reader: File = File::open(input_path)?;
    let writer: File = File::create(output_path)?;

    // wrap it in buffered reader/writer
    let mut buffered_reader: BufReader<File> = BufReader::with_capacity(1 << 20, reader);
    let mut buffered_writer: BufWriter<File> = BufWriter::with_capacity(1 << 20, writer);

    // start the timer
    let start: Instant = Instant::now();

    // do the compression/decompression
    let (mut done_reader, mut done_writer): (BufReader<File>, BufWriter<File>) = if is_compress {
        buffered_writer.write_all(SRX_HEADER)?;
        StreamEncoder::new(buffered_reader, buffered_writer).encode()?
    } else {
        let mut buffer: [u8; 4] = [0; 4];
        buffered_reader.read_exact(&mut buffer)?;
        if !buffer.eq(SRX_HEADER) {
            return Err(AnyError::new("Not a SRX compressed file!"));
        }
        StreamDecoder::new(buffered_reader, buffered_writer).decode()?
    };

    // stop the timer and calculate the duration in seconds
    let duration: f64 = start.elapsed().as_millis() as f64 / 1000.0;

    // get the input and output size
    let input_size: u64 = done_reader.stream_position()?;
    let output_size: u64 = done_writer.stream_position()?;

    // oke
    Ok((input_size, output_size, duration))
}

fn help() -> ! {
    println!(
        "\
		srx: The fast Symbol Ranking based compressor, version {}.\n\
		Copyright (C) 2023  Mai Thanh Minh (a.k.a. thanhminhmr)\n\n\
		To   compress: srx c <input-file> <output-file>\n\
		To decompress: srx d <input-file> <output-file>",
        env!("CARGO_PKG_VERSION")
    );
    exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // check and parse arguments
    if args.len() != 4 {
        help()
    }
    let is_compress: bool = match args[1].as_str() {
        "c" => true,
        "d" => false,
        _ => help(),
    };
    let input_path: &Path = Path::new(&args[2]);
    let output_path: &Path = Path::new(&args[3]);

    // run the compression
    match run(input_path, output_path, is_compress) {
        Ok((input_size, output_size, duration)) => {
            // calculating and report
            let (percentage, speed) = if is_compress {
                (
                    output_size as f64 / input_size as f64 * 100.0,
                    input_size as f64 / duration / (1 << 20) as f64,
                )
            } else {
                (
                    input_size as f64 / output_size as f64 * 100.0,
                    output_size as f64 / duration / (1 << 20) as f64,
                )
            };
            println!(
                "{} -> {} ({:.2}%) in {:.2} seconds ({:.2} MiB/s)",
                input_size, output_size, percentage, duration, speed
            );
        }
        Err(error) => {
            // something unexpected happened
            println!("Error occurred! {}", error);
            exit(1);
        }
    };
}
