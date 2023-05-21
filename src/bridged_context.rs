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

use crate::primary_context::{ByteMatched, PrimaryContext};
use crate::secondary_context::SecondaryContext;

// -----------------------------------------------

pub const PRIMARY_CONTEXT_SIZE: usize = 1 << 24;
pub const SECONDARY_CONTEXT_SIZE: usize = 0x4000 * 256 + (1024 + 32) * 768;

// -----------------------------------------------

pub type BridgedSecondaryContext = SecondaryContext<SECONDARY_CONTEXT_SIZE>;

// -----------------------------------------------

#[derive(Copy, Clone)]
pub struct BridgedContextInfo {
	first_byte: u8,
	second_byte: u8,
	third_byte: u8,
	bit_context: usize,
	literal_context: usize,
}

impl BridgedContextInfo {
	const fn min(a: usize, b: usize) -> usize {
		if a <= b {
			a
		} else {
			b
		}
	}

	const fn new(
		previous_byte: u8,
		first_byte: u8,
		second_byte: u8,
		third_byte: u8,
		match_count: usize,
		hash_value: usize,
	) -> Self {
		Self {
			first_byte,
			second_byte,
			third_byte,
			bit_context: 0x4000 * 256
				+ if match_count < 4 {
					((previous_byte as usize) << 2) | match_count
				} else {
					1024 + (Self::min(match_count - 4, 63) >> 1)
				} * 768,
			literal_context: (hash_value & 0x3FFF) * 256,
		}
	}

	pub const fn first_context(&self) -> usize {
		return self.bit_context + self.first_byte as usize;
	}

	pub const fn second_context(&self) -> usize {
		return self.bit_context + 0x100 + self.second_byte.wrapping_add(self.third_byte) as usize;
	}

	pub const fn third_context(&self) -> usize {
		return self.bit_context
			+ 0x200 + self
			.second_byte
			.wrapping_mul(2)
			.wrapping_sub(self.third_byte) as usize;
	}

	pub const fn literal_context(&self) -> usize {
		self.literal_context
	}

	pub const fn first_byte(&self) -> u8 {
		self.first_byte
	}

	pub const fn second_byte(&self) -> u8 {
		self.second_byte
	}

	pub const fn third_byte(&self) -> u8 {
		self.third_byte
	}
}

// -----------------------------------------------

pub struct BridgedPrimaryContext(PrimaryContext<PRIMARY_CONTEXT_SIZE>);

impl BridgedPrimaryContext {
	pub fn new() -> Self {
		Self(PrimaryContext::new())
	}

	pub fn context_info(&self) -> BridgedContextInfo {
		let (first_byte, second_byte, third_byte, match_count): (u8, u8, u8, usize) = self.0.get();
		BridgedContextInfo::new(
			self.0.previous_byte(),
			first_byte,
			second_byte,
			third_byte,
			match_count,
			self.0.hash_value(),
		)
	}

	pub fn matching(&mut self, next_byte: u8) -> ByteMatched {
		self.0.matching(next_byte)
	}

	pub fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
		self.0.matched(next_byte, matched)
	}
}
