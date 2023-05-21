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

use crate::basic::Buffer;
use crate::primary_context::history::ByteHistory;
use crate::primary_context::matched::ByteMatched;

// -----------------------------------------------

pub struct PrimaryContext<const SIZE: usize> {
	previous_byte: u8,
	hash_value: usize,
	context: Buffer<ByteHistory, SIZE>,
}

impl<const SIZE: usize> PrimaryContext<SIZE> {
	// assert that SIZE is power of 2
	const _SIZE_CHECK: () = assert!(SIZE != 0 && (SIZE & (SIZE - 1)) == 0);

	pub fn new() -> Self {
		Self {
			previous_byte: 0,
			hash_value: 0,
			context: Buffer::new(ByteHistory::new()),
		}
	}

	pub fn get(&self) -> (u8, u8, u8, usize) {
		self.context[self.hash_value].get()
	}

	pub fn previous_byte(&self) -> u8 {
		self.previous_byte as u8
	}

	pub fn hash_value(&self) -> usize {
		self.hash_value
	}

	fn update(&mut self, next_byte: u8) {
		self.previous_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + next_byte as usize + 1) % SIZE;
		debug_assert!(self.hash_value < SIZE);
	}

	pub fn matching(&mut self, next_byte: u8) -> ByteMatched {
		let matching_byte: ByteMatched = self.context[self.hash_value].matching(next_byte);
		self.update(next_byte);
		return matching_byte;
	}

	pub fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
		self.context[self.hash_value].matched(next_byte, matched);
		self.update(next_byte);
	}
}
