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

use crate::basic::{AnyError, AnyResult};
use std::cmp::min;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{env, thread};

mod basic;
#[cfg(test)]
mod tests;

// =================================================================================================
//region Secondary Context

// -------------------------------------------------------------------------------------------------
//region Direct Bit Encoding/Decoding

// -----------------------------------------------
// region BitPrediction

// MULTIPLIER[i] == 0x1_0000_0000 / (i + 2) but rounded
const MULTIPLIER: [u32; 256] = {
    // const-for loops not yet supported
    let mut table = [0; 256];
    let mut i: usize = 0;
    while i < 256 {
        let div = (1 << 33) / (i as u64 + 2);
        table[i] = ((div >> 1) + (div & 1)) as u32; // rounding
        i += 1;
    }
    table
};

// lower 8-bit is a counter, higher 24-bit is prediction
#[derive(Clone)]
struct BitPrediction(u32);

impl BitPrediction {
    fn new() -> Self {
        Self(0x80000000)
    }

    fn get_prediction(&self) -> u32 {
        self.0 & 0xFFFFFF00
    }

    // return current prediction and then update the prediction with new bit
    fn update(&mut self, bit: usize) -> u32 {
        debug_assert!(bit == 0 || bit == 1);
        // get bit 0-7 as count
        let count: usize = (self.0 & 0xFF) as usize;
        // masking bit 8-31 as old prediction
        let old_prediction: u32 = self.0 & 0xFFFFFF00;
        // create bit shift
        let bit_shift: i64 = (bit as i64) << 32;
        // get multiplier
        let multiplier: i64 = MULTIPLIER[count] as i64;
        // calculate new prediction
        let new_prediction: u32 = (((bit_shift - old_prediction as i64) * multiplier) >> 32) as u32;
        // update state
        self.0 = self
            .0
            .wrapping_add((new_prediction & 0xFFFFFF00) + if count < 255 { 1 } else { 0 });
        // return old prediction (before update)
        return old_prediction;
    }
}

//endregion BitPrediction
// -----------------------------------------------
//region BitEncoder

struct BitEncoder<W: Write> {
    low: u32,
    high: u32,
    states: Box<[BitPrediction]>,
    writer: W,
}

impl<W: Write> BitEncoder<W> {
    fn new(size: usize, writer: W) -> Self {
        Self {
            low: 0,
            high: 0xFFFFFFFF,
            states: vec![BitPrediction::new(); size].into_boxed_slice(),
            writer,
        }
    }

    fn bit(&mut self, context: usize, bit: usize) -> AnyResult<()> {
        // checking
        debug_assert!(self.low < self.high);
        debug_assert!(context < self.states.len());
        debug_assert!(bit == 0 || bit == 1);
        // get prediction
        let prediction: u32 = self.states[context].update(bit);
        // get delta
        let delta: u32 = (((self.high - self.low) as u64 * prediction as u64) >> 32) as u32;
        // calculate middle
        let middle: u32 = self.low + delta;
        debug_assert!(self.low <= middle && middle < self.high);
        // set new range limit
        *(if bit != 0 {
            &mut self.high
        } else {
            &mut self.low
        }) = middle + (bit ^ 1) as u32;
        // shift bits out
        while (self.high ^ self.low) & 0xFF000000 == 0 {
            // write byte
            self.writer.write_all(&[(self.low >> 24) as u8])?;
            // shift new bits into high/low
            self.low = self.low << 8;
            self.high = (self.high << 8) | 0xFF;
        }
        // oke
        return Ok(());
    }

    fn byte(&mut self, context: usize, byte: u8) -> AnyResult<()> {
        // code high 4 bits in first 15 contexts
        let high: usize = ((byte >> 4) | 16) as usize;
        self.bit(context + 1, high >> 3 & 1)?;
        self.bit(context + (high >> 3), high >> 2 & 1)?;
        self.bit(context + (high >> 2), high >> 1 & 1)?;
        self.bit(context + (high >> 1), high & 1)?;
        // code low 4 bits in one of 16 blocks of 15 contexts (to reduce cache misses)
        let low_context: usize = context + (15 * (high - 15)) as usize;
        let low: usize = ((byte & 15) | 16) as usize;
        self.bit(low_context + 1, low >> 3 & 1)?;
        self.bit(low_context + (low >> 3), low >> 2 & 1)?;
        self.bit(low_context + (low >> 2), low >> 1 & 1)?;
        self.bit(low_context + (low >> 1), low & 1)?;
        // oke
        return Ok(());
    }

    fn flush(mut self) -> AnyResult<W> {
        // write then get out
        self.writer.write_all(&[(self.low >> 24) as u8])?;
        // oke, give back the writer
        Ok(self.writer)
    }
}

//endregion BitEncoder
// -----------------------------------------------
//region BitDecoder

struct BitDecoder<R: Read> {
    value: u32,
    low: u32,
    high: u32,
    states: Box<[BitPrediction]>,
    reader: R,
}

impl<R: Read> BitDecoder<R> {
    fn new(size: usize, reader: R) -> Self {
        Self {
            value: 0,
            low: 0,
            high: 0,
            states: vec![BitPrediction::new(); size].into_boxed_slice(),
            reader,
        }
    }

    fn bit(&mut self, context: usize) -> AnyResult<usize> {
        // shift bits in
        while (self.high ^ self.low) & 0xFF000000 == 0 {
            // read byte
            let mut byte: [u8; 1] = [0];
            let read: usize = self.reader.read(&mut byte)?;
            // shift new bits into high/low/value
            self.value = (self.value << 8) | if read > 0 { byte[0] as u32 } else { 0xFF };
            self.low = self.low << 8;
            self.high = (self.high << 8) | 0xFF;
        }
        // checking
        debug_assert!(context < self.states.len());
        debug_assert!(self.low < self.high);
        // get prediction
        let bit_prediction: &mut BitPrediction = &mut self.states[context];
        let prediction: u32 = bit_prediction.get_prediction();
        // get delta
        let delta: u32 = (((self.high - self.low) as u64 * prediction as u64) >> 32) as u32;
        // calculate middle
        let middle: u32 = self.low + delta;
        debug_assert!(self.low <= middle && middle < self.high);
        // calculate bit
        let bit: usize = if self.value <= middle { 1 } else { 0 };
        // update high/low
        *(if bit != 0 {
            &mut self.high
        } else {
            &mut self.low
        }) = middle + (bit ^ 1) as u32;
        // update bit prediction
        bit_prediction.update(bit);
        // return the value
        return Ok(bit);
    }

    fn byte(&mut self, context: usize) -> AnyResult<u8> {
        let mut high: usize = 1;
        high += high + self.bit(context + high)?;
        high += high + self.bit(context + high)?;
        high += high + self.bit(context + high)?;
        high += high + self.bit(context + high)?;
        let low_context: usize = context + (15 * (high - 15)) as usize;
        let mut low: usize = 1;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        low += low + self.bit(low_context + low)?;
        return Ok((((high - 16) << 4) | (low - 16)) as u8);
    }

    fn flush(self) -> R {
        self.reader
    }
}

//endregion BitDecoder
// -----------------------------------------------

//endregion Direct Bit Encoding/Decoding
// -------------------------------------------------------------------------------------------------

// -------------------------------------------------------------------------------------------------
//region Threaded Bit Encoding

// -----------------------------------------------
//region Shared Buffer

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

//endregion Shared Buffer
// -----------------------------------------------
//region BufferedEncoder

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

//endregion BufferedEncoder
// -----------------------------------------------
//region ThreadMessage

struct ThreadMessage {
    buffer: BufferContainer,
    count: usize,
}

impl ThreadMessage {
    fn new(buffer: BufferContainer, count: usize) -> Self {
        Self { buffer, count }
    }
}

//endregion ThreadMessage
// -----------------------------------------------
//region ThreadedEncoder

struct ThreadedEncoder<W: Write + Send + 'static> {
    buffer_which: bool,
    buffer_one: BufferContainer,
    buffer_two: BufferContainer,
    sender: SyncSender<ThreadMessage>,
    thread: JoinHandle<AnyResult<W>>,
}

impl<W: Write + Send + 'static> ThreadedEncoder<W> {
    fn new(size: usize, output: W) -> Self {
        let (sender, receiver): (SyncSender<ThreadMessage>, Receiver<ThreadMessage>) =
            sync_channel(1);
        Self {
            buffer_which: true,
            buffer_one: BufferContainer::new(),
            buffer_two: BufferContainer::new(),
            sender,
            thread: thread::spawn(move || ThreadedEncoder::thread(size, output, receiver)),
        }
    }

    fn thread(size: usize, writer: W, receiver: Receiver<ThreadMessage>) -> AnyResult<W> {
        let mut encoder: BitEncoder<W> = BitEncoder::new(size, writer);
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
                    encoder.bit(value >> 9, value & 1)?;
                } else {
                    encoder.byte(value >> 9, (value & 0xFF) as u8)?;
                }
            }
        }
        return encoder.flush();
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

//endregion ThreadedEncoder
// -----------------------------------------------

//endregion Threaded Bit Encoding
// -------------------------------------------------------------------------------------------------

//endregion Secondary Context
// =================================================================================================
//region Symbol Ranking

// -------------------------------------------------------------------------------------------------
//region Matching Context

enum ByteMatched {
    FIRST,
    SECOND,
    THIRD,
    NONE,
}

#[derive(Clone)]
struct MatchingContext(u32);

impl MatchingContext {
    fn new() -> Self {
        Self(0)
    }

    fn get(&self) -> (u8, u8, u8, usize) {
        (
            self.0 as u8,            // first byte
            (self.0 >> 8) as u8,     // second byte
            (self.0 >> 16) as u8,    // third byte
            (self.0 >> 24) as usize, // count
        )
    }

    fn matching(&mut self, next_byte: u8) -> ByteMatched {
        let mask: u32 = self.0 ^ (0x10101 * next_byte as u32);
        return if (mask & 0x0000FF) == 0 {
            // mask for the first byte
            // increase count by 1, capped at 255
            self.0 += if self.0 < 0xFF000000 { 0x01000000 } else { 0 };

            ByteMatched::FIRST
        } else if (mask & 0x00FF00) == 0 {
            // mask for the second byte
            self.0 = (self.0 & 0xFF0000) // keep the third byte
				| ((self.0 << 8) & 0xFF00) // bring the old first byte to second place
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

            ByteMatched::SECOND
        } else if (mask & 0xFF0000) == 0 {
            // mask for the third byte
            self.0 = ((self.0 << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

            ByteMatched::THIRD
        } else {
            // not match
            self.0 = ((self.0 << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32; // set the first byte

            ByteMatched::NONE
        };
    }

    fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
        match matched {
            ByteMatched::FIRST => {
                // first byte
                // increase count by 1, capped at 255
                self.0 += if self.0 < 0xFF000000 { 0x01000000 } else { 0 };
            }
            ByteMatched::SECOND => {
                // second byte
                self.0 = (self.0 & 0xFF0000) // keep the third byte
					| ((self.0 << 8) & 0xFF00) // bring the old first byte to second place
					| next_byte as u32 // set the first byte
					| 0x1000000; // set count to 1
            }
            ByteMatched::THIRD => {
                // third byte
                self.0 = ((self.0 << 8) & 0xFFFF00) // move old first/second to second/third
					| next_byte as u32 // set the first byte
					| 0x1000000; // set count to 1
            }
            ByteMatched::NONE => {
                // not match
                self.0 = ((self.0 << 8) & 0xFFFF00) // move old first/second to second/third
					| next_byte as u32; // set the first byte
            }
        }
    }
}

// endregion Matching Context
// -------------------------------------------------------------------------------------------------
//region Matching Contexts

struct MatchingContexts {
    last_byte: u8,
    hash_value: usize,
    contexts: Box<[MatchingContext]>,
}

impl MatchingContexts {
    fn new(size_log: usize) -> Self {
        Self {
            last_byte: 0,
            hash_value: 0,
            contexts: vec![MatchingContext::new(); 1 << size_log].into_boxed_slice(),
        }
    }

    fn get_last_byte(&self) -> u8 {
        self.last_byte
    }
    fn get_hash_value(&self) -> usize {
        self.hash_value
    }
    fn get_context(&self) -> &MatchingContext {
        &self.contexts[self.hash_value]
    }

    fn matching(&mut self, next_byte: u8) -> ByteMatched {
        let matching_byte: ByteMatched = self.contexts[self.hash_value].matching(next_byte);
        self.last_byte = next_byte;
        self.hash_value =
            (self.hash_value * (5 << 5) + next_byte as usize + 1) & (self.contexts.len() - 1);
        debug_assert!(self.hash_value < self.contexts.len());
        return matching_byte;
    }

    fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
        self.contexts[self.hash_value].matched(next_byte, matched);
        self.last_byte = next_byte;
        self.hash_value =
            (self.hash_value * (5 << 5) + next_byte as usize + 1) & (self.contexts.len() - 1);
        debug_assert!(self.hash_value < self.contexts.len());
    }
}

// endregion Matching Contexts
// -------------------------------------------------------------------------------------------------

//endregion Symbol Ranking
// =================================================================================================
//region Stream Encoder/Decoder

// -------------------------------------------------------------------------------------------------
//region StreamContext

const PRIMARY_CONTEXT_SIZE_LOG: usize = 24;
const SECONDARY_CONTEXT_SIZE: usize = (1024 + 32) * 768 + 0x400000;

struct StreamContexts(MatchingContexts);

impl Deref for StreamContexts {
    type Target = MatchingContexts;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StreamContexts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl StreamContexts {
    fn new() -> Self {
        Self(MatchingContexts::new(PRIMARY_CONTEXT_SIZE_LOG))
    }

    fn calculate_context(&self) -> (u8, u8, u8, usize, usize, usize, usize) {
        let (first_byte, second_byte, third_byte, count) = self.get_context().get();

        let bit_context: usize = if count < 4 {
            ((self.get_last_byte() as usize) << 2) | count
        } else {
            1024 + (min(count - 4, 63) >> 1)
        } * 768
            + 0x400000;

        let first_context: usize = bit_context + first_byte as usize;
        let second_context: usize =
            bit_context + 256 + second_byte.wrapping_add(third_byte) as usize;
        let third_context: usize =
            bit_context + 512 + second_byte.wrapping_mul(2).wrapping_sub(third_byte) as usize;
        let literal_context: usize = (self.get_hash_value() & 0x3FFF) * 256;

        return (
            first_byte,
            second_byte,
            third_byte,
            first_context,
            second_context,
            third_context,
            literal_context,
        );
    }
}

//endregion StreamEncoder
// -------------------------------------------------------------------------------------------------
//region StreamEncoder

struct StreamEncoder<R: Read, W: Write + Send + 'static> {
    contexts: StreamContexts,
    encoder: ThreadedEncoder<W>,
    reader: R,
}

impl<R: Read, W: Write + Send + 'static> StreamEncoder<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            contexts: StreamContexts::new(),
            encoder: ThreadedEncoder::new(SECONDARY_CONTEXT_SIZE, writer),
            reader,
        }
    }

    #[inline(never)]
    fn encode(mut self) -> AnyResult<(R, W)> {
        loop {
            let mut encoder: BufferedEncoder = self.encoder.begin()?;
            loop {
                let (
                    first_byte,
                    _,
                    _,
                    first_context,
                    second_context,
                    third_context,
                    literal_context,
                ) = self.contexts.calculate_context();

                let mut byte_result: [u8; 1] = [0];
                if self.reader.read(&mut byte_result)? == 0 {
                    // eof, encoded using first byte as literal
                    encoder.bit(first_context, 1);
                    encoder.bit(second_context, 0);
                    encoder.byte(literal_context, first_byte);

                    self.encoder.end(encoder)?;
                    let writer: W = self.encoder.flush()?;
                    // gave the reader/writer back
                    return Ok((self.reader, writer));
                }

                let current_byte: u8 = byte_result[0];
                match self.contexts.matching(current_byte) {
                    ByteMatched::FIRST => {
                        encoder.bit(first_context, 0);
                    }
                    ByteMatched::SECOND => {
                        encoder.bit(first_context, 1);
                        encoder.bit(second_context, 1);
                        encoder.bit(third_context, 0);
                    }
                    ByteMatched::THIRD => {
                        encoder.bit(first_context, 1);
                        encoder.bit(second_context, 1);
                        encoder.bit(third_context, 1);
                    }
                    ByteMatched::NONE => {
                        encoder.bit(first_context, 1);
                        encoder.bit(second_context, 0);
                        encoder.byte(literal_context, current_byte);
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
    contexts: StreamContexts,
    decoder: BitDecoder<R>,
    writer: W,
}

impl<R: Read, W: Write> StreamDecoder<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            contexts: StreamContexts::new(),
            decoder: BitDecoder::new(SECONDARY_CONTEXT_SIZE, reader),
            writer,
        }
    }

    #[inline(never)]
    fn decode(mut self) -> AnyResult<(R, W)> {
        loop {
            let (
                first_byte,
                second_byte,
                third_byte,
                first_context,
                second_context,
                third_context,
                literal_context,
            ) = self.contexts.calculate_context();

            let (next_byte, matched) = if self.decoder.bit(first_context)? == 0 {
                // match first
                (first_byte, ByteMatched::FIRST)
            } else if self.decoder.bit(second_context)? == 0 {
                // literal
                let next_byte: u8 = self.decoder.byte(literal_context)?;
                if next_byte == first_byte {
                    // eof, gave the reader/writer back
                    let reader: R = self.decoder.flush();
                    return Ok((reader, self.writer));
                }
                (next_byte, ByteMatched::NONE)
            } else if self.decoder.bit(third_context)? == 0 {
                // match second
                (second_byte, ByteMatched::SECOND)
            } else {
                // match third
                (third_byte, ByteMatched::THIRD)
            };
            self.writer.write_all(&[next_byte])?;
            self.contexts.matched(next_byte, matched);
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
