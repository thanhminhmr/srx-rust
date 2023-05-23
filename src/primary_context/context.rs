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

use crate::basic::{Buffer, Byte};
use super::history::{ByteHistory, HistoryState};
use super::matched::ByteMatched;

// -----------------------------------------------

pub struct PrimaryContext<const SIZE: usize> {
	previous_byte: Byte,
	hash_value: usize,
	context: Buffer<ByteHistory, SIZE>,
}

impl<const SIZE: usize> PrimaryContext<SIZE> {
	// assert that SIZE is power of 2
	const _SIZE_CHECK: () = assert!(SIZE != 0 && (SIZE & (SIZE - 1)) == 0);

	pub fn new() -> Self {
		Self {
			previous_byte: Byte::from(0),
			hash_value: 0,
			context: Buffer::new(),
		}
	}

	pub fn get_history(&self) -> ByteHistory {
		self.context[self.hash_value]
	}

	pub fn previous_byte(&self) -> Byte {
		self.previous_byte
	}

	pub fn hash_value(&self) -> usize {
		self.hash_value
	}

	pub fn matching(&mut self, current_state: HistoryState, next_byte: Byte) -> ByteMatched {
		let current_history: &mut ByteHistory = &mut self.context[self.hash_value];
		let matching_byte: ByteMatched = current_history.matching(current_state, next_byte);
		self.previous_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + usize::from(next_byte) + 1) % SIZE;
		debug_assert!(self.hash_value < SIZE);
		return matching_byte;
	}

	pub fn matched(&mut self, current_state: HistoryState, next_byte: Byte, matched: ByteMatched) {
		let current_history: &mut ByteHistory = &mut self.context[self.hash_value];
		current_history.matched(current_state, next_byte, matched);
		self.previous_byte = next_byte;
		self.hash_value = (self.hash_value * (5 << 5) + usize::from(next_byte) + 1) % SIZE;
		debug_assert!(self.hash_value < SIZE);
	}
}
