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

use std::cmp::min;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{env, fmt, thread};

// =================================================================================================
//region AnyError
type AnyResult<T> = Result<T, AnyError>;

#[derive(Debug, Clone)]
struct AnyError(String);

impl AnyError {
    fn new(s: &str) -> Self {
        Self(String::from(s))
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E: Error> From<E> for AnyError {
    fn from(e: E) -> Self {
        Self(e.to_string())
    }
}
//endregion AnyError
// =================================================================================================
//region Secondary Context

// -------------------------------------------------------------------------------------------------
//region Direct Bit Encoding/Decoding

// -----------------------------------------------
// region BitPrediction

// MULTIPLIER[i] == 0x1_0000_0000 / (i + 2)
const MULTIPLIER: [u32; 256] = [
    0x80000000, 0x55555555, 0x40000000, 0x33333333, 0x2AAAAAAB, 0x24924925, 0x20000000, 0x1C71C71C,
    0x1999999A, 0x1745D174, 0x15555555, 0x13B13B14, 0x12492492, 0x11111111, 0x10000000, 0x0F0F0F0F,
    0x0E38E38E, 0x0D79435E, 0x0CCCCCCD, 0x0C30C30C, 0x0BA2E8BA, 0x0B21642D, 0x0AAAAAAB, 0x0A3D70A4,
    0x09D89D8A, 0x097B425F, 0x09249249, 0x08D3DCB1, 0x08888889, 0x08421084, 0x08000000, 0x07C1F07C,
    0x07878788, 0x07507507, 0x071C71C7, 0x06EB3E45, 0x06BCA1AF, 0x06906907, 0x06666666, 0x063E7064,
    0x06186186, 0x05F417D0, 0x05D1745D, 0x05B05B06, 0x0590B216, 0x0572620B, 0x05555555, 0x0539782A,
    0x051EB852, 0x05050505, 0x04EC4EC5, 0x04D4873F, 0x04BDA12F, 0x04A7904A, 0x04924925, 0x047DC11F,
    0x0469EE58, 0x0456C798, 0x04444444, 0x04325C54, 0x04210842, 0x04104104, 0x04000000, 0x03F03F04,
    0x03E0F83E, 0x03D22635, 0x03C3C3C4, 0x03B5CC0F, 0x03A83A84, 0x039B0AD1, 0x038E38E4, 0x0381C0E0,
    0x03759F23, 0x0369D037, 0x035E50D8, 0x03531DEC, 0x03483483, 0x033D91D3, 0x03333333, 0x03291620,
    0x031F3832, 0x03159722, 0x030C30C3, 0x03030303, 0x02FA0BE8, 0x02F14990, 0x02E8BA2F, 0x02E05C0C,
    0x02D82D83, 0x02D02D03, 0x02C8590B, 0x02C0B02C, 0x02B93105, 0x02B1DA46, 0x02AAAAAB, 0x02A3A0FD,
    0x029CBC15, 0x0295FAD4, 0x028F5C29, 0x0288DF0D, 0x02828283, 0x027C4598, 0x02762762, 0x02702702,
    0x026A439F, 0x02647C69, 0x025ED098, 0x02593F6A, 0x0253C825, 0x024E6A17, 0x02492492, 0x0243F6F0,
    0x023EE090, 0x0239E0D6, 0x0234F72C, 0x02302302, 0x022B63CC, 0x0226B902, 0x02222222, 0x021D9EAD,
    0x02192E2A, 0x0214D021, 0x02108421, 0x020C49BA, 0x02082082, 0x02040810, 0x02000000, 0x01FC07F0,
    0x01F81F82, 0x01F4465A, 0x01F07C1F, 0x01ECC07B, 0x01E9131B, 0x01E573AD, 0x01E1E1E2, 0x01DE5D6E,
    0x01DAE607, 0x01D77B65, 0x01D41D42, 0x01D0CB59, 0x01CD8569, 0x01CA4B30, 0x01C71C72, 0x01C3F8F0,
    0x01C0E070, 0x01BDD2B9, 0x01BACF91, 0x01B7D6C4, 0x01B4E81B, 0x01B20364, 0x01AF286C, 0x01AC5702,
    0x01A98EF6, 0x01A6D01A, 0x01A41A42, 0x01A16D40, 0x019EC8E9, 0x019C2D15, 0x0199999A, 0x01970E50,
    0x01948B10, 0x01920FB5, 0x018F9C19, 0x018D3019, 0x018ACB91, 0x01886E5F, 0x01861862, 0x0183C978,
    0x01818182, 0x017F4060, 0x017D05F4, 0x017AD221, 0x0178A4C8, 0x01767DCE, 0x01745D17, 0x01724288,
    0x01702E06, 0x016E1F77, 0x016C16C1, 0x016A13CD, 0x01681681, 0x01661EC7, 0x01642C86, 0x01623FA7,
    0x01605816, 0x015E75BC, 0x015C9883, 0x015AC057, 0x0158ED23, 0x01571ED4, 0x01555555, 0x01539095,
    0x0151D07F, 0x01501501, 0x014E5E0A, 0x014CAB88, 0x014AFD6A, 0x0149539E, 0x0147AE14, 0x01460CBC,
    0x01446F86, 0x0142D662, 0x01414141, 0x013FB014, 0x013E22CC, 0x013C995A, 0x013B13B1, 0x013991C3,
    0x01381381, 0x013698DF, 0x013521D0, 0x0133AE46, 0x01323E35, 0x0130D190, 0x012F684C, 0x012E025C,
    0x012C9FB5, 0x012B404B, 0x0129E413, 0x01288B01, 0x0127350C, 0x0125E227, 0x01249249, 0x01234568,
    0x0121FB78, 0x0120B471, 0x011F7048, 0x011E2EF4, 0x011CF06B, 0x011BB4A4, 0x011A7B96, 0x01194538,
    0x01181181, 0x0116E069, 0x0115B1E6, 0x011485F1, 0x01135C81, 0x0112358E, 0x01111111, 0x010FEF01,
    0x010ECF57, 0x010DB20B, 0x010C9715, 0x010B7E6F, 0x010A6811, 0x010953F4, 0x01084211, 0x01073261,
    0x010624DD, 0x0105197F, 0x01041041, 0x0103091B, 0x01020408, 0x01010101, 0x01000000, 0x00FF00FF,
];

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

const SRX_HEADER: [u8; 4] = ['s' as u8, 'R' as u8, 'x' as u8, 0];

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
        buffered_writer.write_all(&SRX_HEADER)?;
        StreamEncoder::new(buffered_reader, buffered_writer).encode()?
    } else {
        let mut buffer: [u8; 4] = [0; 4];
        buffered_reader.read_exact(&mut buffer)?;
        if !buffer.eq(&SRX_HEADER) {
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
