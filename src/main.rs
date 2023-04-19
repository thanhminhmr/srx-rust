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

use std::{env, fmt, thread};
use std::cmp::min;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread::JoinHandle;
use std::time::Instant;

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
	0x80000000, 0x55555555, 0x40000000, 0x33333333, 0x2AAAAAAA, 0x24924924, 0x20000000, 0x1C71C71C,
	0x19999999, 0x1745D174, 0x15555555, 0x13B13B13, 0x12492492, 0x11111111, 0x10000000, 0x0F0F0F0F,
	0x0E38E38E, 0x0D79435E, 0x0CCCCCCC, 0x0C30C30C, 0x0BA2E8BA, 0x0B21642C, 0x0AAAAAAA, 0x0A3D70A3,
	0x09D89D89, 0x097B425E, 0x09249249, 0x08D3DCB0, 0x08888888, 0x08421084, 0x08000000, 0x07C1F07C,
	0x07878787, 0x07507507, 0x071C71C7, 0x06EB3E45, 0x06BCA1AF, 0x06906906, 0x06666666, 0x063E7063,
	0x06186186, 0x05F417D0, 0x05D1745D, 0x05B05B05, 0x0590B216, 0x0572620A, 0x05555555, 0x05397829,
	0x051EB851, 0x05050505, 0x04EC4EC4, 0x04D4873E, 0x04BDA12F, 0x04A7904A, 0x04924924, 0x047DC11F,
	0x0469EE58, 0x0456C797, 0x04444444, 0x04325C53, 0x04210842, 0x04104104, 0x04000000, 0x03F03F03,
	0x03E0F83E, 0x03D22635, 0x03C3C3C3, 0x03B5CC0E, 0x03A83A83, 0x039B0AD1, 0x038E38E3, 0x0381C0E0,
	0x03759F22, 0x0369D036, 0x035E50D7, 0x03531DEC, 0x03483483, 0x033D91D2, 0x03333333, 0x0329161F,
	0x031F3831, 0x03159721, 0x030C30C3, 0x03030303, 0x02FA0BE8, 0x02F14990, 0x02E8BA2E, 0x02E05C0B,
	0x02D82D82, 0x02D02D02, 0x02C8590B, 0x02C0B02C, 0x02B93105, 0x02B1DA46, 0x02AAAAAA, 0x02A3A0FD,
	0x029CBC14, 0x0295FAD4, 0x028F5C28, 0x0288DF0C, 0x02828282, 0x027C4597, 0x02762762, 0x02702702,
	0x026A439F, 0x02647C69, 0x025ED097, 0x02593F69, 0x0253C825, 0x024E6A17, 0x02492492, 0x0243F6F0,
	0x023EE08F, 0x0239E0D5, 0x0234F72C, 0x02302302, 0x022B63CB, 0x0226B902, 0x02222222, 0x021D9EAD,
	0x02192E29, 0x0214D021, 0x02108421, 0x020C49BA, 0x02082082, 0x02040810, 0x02000000, 0x01FC07F0,
	0x01F81F81, 0x01F44659, 0x01F07C1F, 0x01ECC07B, 0x01E9131A, 0x01E573AC, 0x01E1E1E1, 0x01DE5D6E,
	0x01DAE607, 0x01D77B65, 0x01D41D41, 0x01D0CB58, 0x01CD8568, 0x01CA4B30, 0x01C71C71, 0x01C3F8F0,
	0x01C0E070, 0x01BDD2B8, 0x01BACF91, 0x01B7D6C3, 0x01B4E81B, 0x01B20364, 0x01AF286B, 0x01AC5701,
	0x01A98EF6, 0x01A6D01A, 0x01A41A41, 0x01A16D3F, 0x019EC8E9, 0x019C2D14, 0x01999999, 0x01970E4F,
	0x01948B0F, 0x01920FB4, 0x018F9C18, 0x018D3018, 0x018ACB90, 0x01886E5F, 0x01861861, 0x0183C977,
	0x01818181, 0x017F405F, 0x017D05F4, 0x017AD220, 0x0178A4C8, 0x01767DCE, 0x01745D17, 0x01724287,
	0x01702E05, 0x016E1F76, 0x016C16C1, 0x016A13CD, 0x01681681, 0x01661EC6, 0x01642C85, 0x01623FA7,
	0x01605816, 0x015E75BB, 0x015C9882, 0x015AC056, 0x0158ED23, 0x01571ED3, 0x01555555, 0x01539094,
	0x0151D07E, 0x01501501, 0x014E5E0A, 0x014CAB88, 0x014AFD6A, 0x0149539E, 0x0147AE14, 0x01460CBC,
	0x01446F86, 0x0142D662, 0x01414141, 0x013FB013, 0x013E22CB, 0x013C995A, 0x013B13B1, 0x013991C2,
	0x01381381, 0x013698DF, 0x013521CF, 0x0133AE45, 0x01323E34, 0x0130D190, 0x012F684B, 0x012E025C,
	0x012C9FB4, 0x012B404A, 0x0129E412, 0x01288B01, 0x0127350B, 0x0125E227, 0x01249249, 0x01234567,
	0x0121FB78, 0x0120B470, 0x011F7047, 0x011E2EF3, 0x011CF06A, 0x011BB4A4, 0x011A7B96, 0x01194538,
	0x01181181, 0x0116E068, 0x0115B1E5, 0x011485F0, 0x01135C81, 0x0112358E, 0x01111111, 0x010FEF01,
	0x010ECF56, 0x010DB20A, 0x010C9714, 0x010B7E6E, 0x010A6810, 0x010953F3, 0x01084210, 0x01073260,
	0x010624DD, 0x0105197F, 0x01041041, 0x0103091B, 0x01020408, 0x01010101, 0x01000000, 0x00FF00FF,
];

// lower 8-bit is a counter, higher 24-bit is prediction
#[derive(Clone)]
struct BitPrediction(u32);

impl BitPrediction {
	fn new() -> Self { Self(0x80000000) }

	fn get_prediction(&self) -> u64 { (self.0 >> 8) as u64 }

	// return current prediction and then update the prediction with new bit
	fn update(&mut self, bit: usize) -> u64 {
		assert!(bit == 0 || bit == 1);
		// get bit 0-7 as count
		let count: usize = (self.0 & 0xFF) as usize;
		// get bit 8-31 as current prediction
		let current_prediction: i64 = (self.0 >> 8) as i64;
		// create bit shift
		let bit_shift: i64 = (bit as i64) << 24;
		// get multiplier
		let multiplier: i64 = MULTIPLIER[count] as i64;
		// calculate new prediction
		let new_prediction: u32 = (((bit_shift - current_prediction) * multiplier) >> 24) as u32;
		// update state
		self.0 = self.0.wrapping_add((new_prediction & 0xFFFFFF00)
			+ if count < 255 { 1 } else { 0 });
		// return current prediction (before update)
		return current_prediction as u64;
	}
}

//endregion BitPrediction
// -----------------------------------------------
//region BitEncoder

struct BitEncoder<W: Write> {
	low: u64,
	high: u64,
	states: Vec<BitPrediction>,
	writer: W,
}

impl<W: Write> BitEncoder<W> {
	fn new(size: usize, writer: W) -> Self {
		Self {
			low: 0,
			high: 0xFFFFFFFF_FFFFFFFF,
			states: vec![BitPrediction::new(); size],
			writer,
		}
	}

	fn bit(&mut self, context: usize, bit: usize) -> AnyResult<()> {
		// checking
		assert!(self.low < self.high);
		assert!(context < self.states.len());
		assert!(bit == 0 || bit == 1);
		// get prediction
		let prediction: u64 = self.states[context].update(bit);
		// get delta
		let delta: u64 = (((self.high - self.low) as u128 * prediction as u128) >> 24) as u64;
		// calculate middle
		let middle: u64 = self.low + delta;
		assert!(self.low <= middle && middle < self.high);
		// set new range limit
		*(if bit != 0 { &mut self.high } else { &mut self.low }) = middle + (bit ^ 1) as u64;
		// shift bits out
		while (self.high ^ self.low) & 0xFF000000_00000000 == 0 {
			// write byte
			self.writer.write_all(&[(self.low >> 56) as u8])?;
			// shift new bits into high/low
			self.low = self.low << 8;
			self.high = (self.high << 8) | 0xFF;
		}
		// oke
		return Ok(());
	}

	// unused, moved to the threaded one
	#[allow(dead_code)]
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
		self.writer.write_all(&[(self.low >> 56) as u8])?;
		// oke, give back the writer
		Ok(self.writer)
	}
}

//endregion BitEncoder
// -----------------------------------------------
//region BitDecoder

struct BitDecoder<R: Read> {
	value: u64,
	low: u64,
	high: u64,
	states: Vec<BitPrediction>,
	reader: R,
}

impl<R: Read> BitDecoder<R> {
	fn new(size: usize, reader: R) -> Self {
		Self {
			value: 0,
			low: 0,
			high: 0,
			states: vec![BitPrediction::new(); size],
			reader,
		}
	}

	fn bit(&mut self, context: usize) -> AnyResult<usize> {
		// shift bits in
		while (self.high ^ self.low) & 0xFF000000_00000000 == 0 {
			// read byte
			let mut byte: [u8; 1] = [0];
			let read: usize = self.reader.read(&mut byte)?;
			// shift new bits into high/low/value
			self.value = (self.value << 8) | if read > 0 { byte[0] as u64 } else { 0xFF };
			self.low = self.low << 8;
			self.high = (self.high << 8) | 0xFF;
		}
		// checking
		assert!(context < self.states.len());
		assert!(self.low < self.high);
		// get prediction
		let bit_prediction: &mut BitPrediction = &mut self.states[context];
		let prediction: u64 = bit_prediction.get_prediction();
		// get delta
		let delta: u64 = (((self.high - self.low) as u128 * prediction as u128) >> 24) as u64;
		// calculate middle
		let middle: u64 = self.low + delta;
		assert!(self.low <= middle && middle < self.high);
		// calculate bit
		let bit: usize = if self.value <= middle { 1 } else { 0 };
		// update high/low
		*(if bit != 0 { &mut self.high } else { &mut self.low }) = middle + (bit ^ 1) as u64;
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

	fn flush(self) -> R { self.reader }
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
const BUFFER_SAFE_GUARD: usize = 0x10;

type Buffer = Box<[u32]>;
type BufferGuarded<'local> = MutexGuard<'local, Buffer>;

#[derive(Clone)]
struct BufferContainer(Arc<Mutex<Buffer>>);

impl Deref for BufferContainer {
	type Target = Arc<Mutex<Buffer>>;
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl BufferContainer {
	fn new() -> Self { Self(Arc::new(Mutex::new(vec![0; BUFFER_SIZE].into_boxed_slice()))) }
}

//endregion Shared Buffer
// -----------------------------------------------
//region BufferedEncoder

struct BufferedEncoder<'local> {
	buffer: BufferGuarded<'local>,
	count: usize,
}

impl<'local> BufferedEncoder<'local> {
	fn new(buffer: BufferGuarded<'local>) -> Self { Self { buffer, count: 0 } }

	fn full(&self) -> bool { self.count + BUFFER_SAFE_GUARD >= BUFFER_SIZE }

	fn count(&self) -> usize { self.count }

	fn bit(&mut self, context: usize, bit: usize) {
		assert!(bit == 0 || bit == 1);
		assert!(context < 0x7FFFFFFF);
		assert!(self.count < BUFFER_SIZE);
		self.buffer[self.count] = (context + context + bit) as u32;
		self.count += 1;
	}

	fn byte(&mut self, context: usize, byte: u8) {
		assert!(self.count + 8 < BUFFER_SIZE);
		assert!(context < 0x7FFFFFFF);
		// code high 4 bits in first 15 contexts
		let high: usize = ((byte >> 4) | 16) as usize;
		self.bit(context + 1, high >> 3 & 1);
		self.bit(context + (high >> 3), high >> 2 & 1);
		self.bit(context + (high >> 2), high >> 1 & 1);
		self.bit(context + (high >> 1), high & 1);
		// code low 4 bits in one of 16 blocks of 15 contexts (to reduce cache misses)
		let low_context: usize = context + (15 * (high - 15)) as usize;
		let low: usize = ((byte & 15) | 16) as usize;
		self.bit(low_context + 1, low >> 3 & 1);
		self.bit(low_context + (low >> 3), low >> 2 & 1);
		self.bit(low_context + (low >> 2), low >> 1 & 1);
		self.bit(low_context + (low >> 1), low & 1);
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
	fn new(buffer: BufferContainer, count: usize) -> Self { Self { buffer, count } }
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
		let (sender, receiver): (SyncSender<ThreadMessage>, Receiver<ThreadMessage>)
			= sync_channel(1);
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
				// the sender is closed, breaking out
				Err(_) => break,
			};
			// encode every bit in buffer
			let buffer: MutexGuard<Buffer> = message.buffer.lock()?;
			for i in 0..message.count {
				encoder.bit((buffer[i] >> 1) as usize, (buffer[i] & 1) as usize)?;
			}
		}
		return encoder.flush();
	}

	fn buffer(&self) -> &BufferContainer {
		if self.buffer_which { &self.buffer_one } else { &self.buffer_two }
	}

	fn flip(&mut self) {
		self.buffer_which = !self.buffer_which;
	}

	fn begin(&self) -> AnyResult<BufferedEncoder> {
		Ok(BufferedEncoder::new(self.buffer().lock()?))
	}

	fn end(&self, buffer: BufferedEncoder) -> AnyResult<()> {
		self.sender.send(ThreadMessage::new(self.buffer().clone(), buffer.count()))?;
		Ok(())
	}

	fn flush(self) -> AnyResult<W> {
		drop(self.sender);
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
struct MatchingContext {
	value: u32,
}

impl MatchingContext {
	fn new() -> Self {
		Self {
			value: 0
		}
	}

	fn get(&self) -> (u8, u8, u8, usize) {
		(
			self.value as u8, // first byte
			(self.value >> 8) as u8, // second byte
			(self.value >> 16) as u8, // third byte
			(self.value >> 24) as usize // count
		)
	}

	fn matching(&mut self, next_byte: u8) -> ByteMatched {
		let mask: u32 = self.value ^ (0x10101 * next_byte as u32);
		return if (mask & 0x0000FF) == 0 { // mask for the first byte
			// increase count by 1, capped at 255
			self.value += if self.value < 0xFF000000 { 0x01000000 } else { 0 };

			ByteMatched::FIRST
		} else if (mask & 0x00FF00) == 0 { // mask for the second byte
			self.value = (self.value & 0xFF0000) // keep the third byte
				| ((self.value << 8) & 0xFF00) // bring the old first byte to second place
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

			ByteMatched::SECOND
		} else if (mask & 0xFF0000) == 0 {  // mask for the third byte
			self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32 // set the first byte
				| 0x1000000; // set count to 1

			ByteMatched::THIRD
		} else { // not match
			self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
				| next_byte as u32; // set the first byte

			ByteMatched::NONE
		};
	}

	fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
		match matched {
			ByteMatched::FIRST => { // first byte
				// increase count by 1, capped at 255
				self.value += if self.value < 0xFF000000 { 0x01000000 } else { 0 };
			}
			ByteMatched::SECOND => { // second byte
				self.value = (self.value & 0xFF0000) // keep the third byte
					| ((self.value << 8) & 0xFF00) // bring the old first byte to second place
					| next_byte as u32 // set the first byte
					| 0x1000000; // set count to 1
			}
			ByteMatched::THIRD => { // third byte
				self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
					| next_byte as u32 // set the first byte
					| 0x1000000; // set count to 1
			}
			ByteMatched::NONE => { // not match
				self.value = ((self.value << 8) & 0xFFFF00) // move old first/second to second/third
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
	contexts: Vec<MatchingContext>,
}

impl MatchingContexts {
	fn new(size_log: usize) -> Self {
		Self {
			last_byte: 0,
			hash_value: 0,
			contexts: vec![MatchingContext::new(); 1 << size_log],
		}
	}

	fn get_last_byte(&self) -> u8 { self.last_byte }
	fn get_hash_value(&self) -> usize { self.hash_value }
	fn get_context(&self) -> &MatchingContext { &self.contexts[self.hash_value] }

	fn matching(&mut self, next_byte: u8) -> ByteMatched {
		let matching_byte: ByteMatched = self.contexts[self.hash_value].matching(next_byte);
		self.last_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + next_byte as usize + 1)
			& (self.contexts.len() - 1);
		assert!(self.hash_value < self.contexts.len());
		return matching_byte;
	}

	fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
		self.contexts[self.hash_value].matched(next_byte, matched);
		self.last_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + next_byte as usize + 1)
			& (self.contexts.len() - 1);
		assert!(self.hash_value < self.contexts.len());
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
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for StreamContexts {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl StreamContexts {
	fn new() -> Self { Self(MatchingContexts::new(PRIMARY_CONTEXT_SIZE_LOG)) }

	fn calculate_context(&self) -> (u8, u8, u8, usize, usize, usize, usize) {
		let (first_byte, second_byte, third_byte, count) = self.get_context().get();

		let bit_context: usize = if count < 4 {
			((self.get_last_byte() as usize) << 2) | count
		} else {
			1024 + (min(count - 4, 63) >> 1)
		} * 768 + 0x400000;

		let first_context: usize = bit_context + first_byte as usize;
		let second_context: usize = bit_context + 256
			+ second_byte.wrapping_add(third_byte) as usize;
		let third_context: usize = bit_context + 512
			+ second_byte.wrapping_mul(2).wrapping_sub(third_byte) as usize;
		let literal_context: usize = (self.get_hash_value() & 0x3FFF) * 256;

		return (first_byte, second_byte, third_byte,
			first_context, second_context, third_context, literal_context);
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
				let (first_byte, _, _,
					first_context, second_context,
					third_context, literal_context)
					= self.contexts.calculate_context();

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

				if encoder.full() { break; }
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
			let (first_byte, second_byte, third_byte,
				first_context, second_context,
				third_context, literal_context)
				= self.contexts.calculate_context();

			let (next_byte, matched) =
				if self.decoder.bit(first_context)? == 0 {
					// match first
					(first_byte, ByteMatched::FIRST)
				} else if self.decoder.bit(second_context)? == 0 {
					// literal
					let next_byte: u8 = self.decoder.byte(literal_context)?;
					if next_byte == first_byte {
						// eof, gave the reader/writer back
						let reader = self.decoder.flush();
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

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 4 || !(args[1].starts_with("c") || args[1].starts_with("d")) {
		println!("\
		srx: The fast Symbol Ranking based compressor.\n\
		Copyright (C) 2023  Mai Thanh Minh (a.k.a. thanhminhmr)\n\n\
		To   compress: srx c <input-file> <output-file>\n\
		To decompress: srx d <input-file> <output-file>");
		exit(0);
	}

	// open file
	let reader: File = File::open(Path::new(&args[2])).unwrap();
	let writer: File = File::create(Path::new(&args[3])).unwrap();

	// wrap it in buffered reader/writer
	let buffered_reader: BufReader<File> = BufReader::with_capacity(1 << 20, reader);
	let buffered_writer: BufWriter<File> = BufWriter::with_capacity(1 << 20, writer);

	// start the timer
	let start: Instant = Instant::now();

	// do the compression/decompression
	let (mut done_reader, mut done_writer): (BufReader<File>, BufWriter<File>) =
		if args[1].starts_with("c") {
			StreamEncoder::new(buffered_reader, buffered_writer).encode().unwrap()
		} else {
			StreamDecoder::new(buffered_reader, buffered_writer).decode().unwrap()
		};

	// stop the timer and calculate the duration
	let duration: f64 = start.elapsed().as_millis() as f64 / 1000.0;

	let input_size: u64 = done_reader.stream_position().unwrap();
	let output_size: u64 = done_writer.stream_position().unwrap();

	let percentage: f64 = output_size as f64 / input_size as f64 * 100.0;
	let speed: f64 = input_size as f64 / duration / (1 << 20) as f64;

	println!("{} -> {} ({:.2}%) in {:.2} seconds ({:.2} MB/s)",
		input_size, output_size, percentage, duration, speed);
}
