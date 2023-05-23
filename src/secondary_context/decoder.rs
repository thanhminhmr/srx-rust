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

use crate::basic::{AnyResult, Closable, PipedReader, Reader};
use crate::secondary_context::Bit;

// -----------------------------------------------

pub struct BitDecoder<const SIZE: usize> {
	value: u32,
	low: u32,
	high: u32,
	reader: PipedReader<u8, SIZE>,
}

impl<const SIZE: usize> BitDecoder<SIZE> {
	pub fn new(reader: PipedReader<u8, SIZE>) -> Self {
		Self {
			value: 0,
			low: 0,
			high: 0,
			reader,
		}
	}

	#[cold]
	#[inline(always)]
	fn flush(&mut self) -> AnyResult<()> {
		debug_assert!((self.high ^ self.low) < 0x01000000);
		while {
			// shift byte in
			self.value = (self.value << 8)
				| match self.reader.read()? {
					None => 0xFF,
					Some(byte) => byte as u32,
				};
			// shift new bits into high/low
			self.low <<= 8;
			self.high = (self.high << 8) | 0xFF;
			// check condition again
			(self.high ^ self.low) < 0x01000000
		} {}
		Ok(())
	}

	#[inline(always)]
	pub fn bit(&mut self, prediction: u32) -> AnyResult<Bit> {
		// shift bits in
		if (self.high ^ self.low) < 0x01000000 {
			self.flush()?;
		}
		// checking
		debug_assert!(self.low < self.high);
		debug_assert!(self.low <= self.value && self.value <= self.high);
		// get delta
		let delta: u32 = (((self.high - self.low) as u64 * prediction as u64) >> 32) as u32;
		// calculate middle
		let middle: u32 = self.low + delta;
		debug_assert!(self.low <= middle && middle < self.high);
		// calculate bit
		let bit: Bit = match self.value <= middle {
			true => Bit::One,
			false => Bit::Zero,
		};
		// update high/low
		*(match bit {
			Bit::Zero => &mut self.low,
			Bit::One => &mut self.high,
		}) = middle + (u32::from(bit) ^ 1);
		// return the value
		return Ok(bit);
	}
}

impl<const SIZE: usize> Closable<()> for BitDecoder<SIZE> {
	fn close(self) -> AnyResult<()> {
		self.reader.close()
	}
}
