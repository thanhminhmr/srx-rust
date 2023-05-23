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

use crate::basic::Byte;
use crate::primary_context::{ByteHistory, HistoryState, PrimaryContext};
use crate::secondary_context::SecondaryContext;

// -----------------------------------------------

pub const PRIMARY_CONTEXT_SIZE: usize = 1 << 24;
pub const SECONDARY_CONTEXT_SIZE: usize = 0x4000 * 256 + (1024 + 32) * 768;

// -----------------------------------------------

pub type BridgedPrimaryContext = PrimaryContext<PRIMARY_CONTEXT_SIZE>;
pub type BridgedSecondaryContext = SecondaryContext<SECONDARY_CONTEXT_SIZE>;

// -----------------------------------------------

pub struct BridgedContextInfo {
	bit_context: usize,
	literal_context: usize,
	current_history: ByteHistory,
	current_state: HistoryState,
}

impl BridgedContextInfo {
	pub fn new(current_history: ByteHistory, previous_byte: Byte, hash_value: usize) -> Self {
		let current_state: HistoryState = current_history.get_state();
		let match_count: usize = current_state.match_count();
		Self {
			bit_context: 0x4000 * 256
				+ if match_count < 4 {
					(usize::from(previous_byte) << 2) | match_count
				} else {
					1024 + if match_count - 4 <= 63 {
						(match_count - 4) >> 1
					} else {
						31
					}
				} * 768,
			literal_context: (hash_value & 0x3FFF) * 256,
			current_history,
			current_state,
		}
	}

	pub fn first_context(&self) -> usize {
		return self.bit_context + usize::from(self.current_history.first_byte());
	}

	pub fn second_context(&self) -> usize {
		return self.bit_context
			+ 0x100 + ((usize::from(self.current_history.second_byte())
			+ usize::from(self.current_history.third_byte()))
			& 0xFF);
	}

	pub fn third_context(&self) -> usize {
		return self.bit_context
			+ 0x200 + ((usize::from(self.current_history.second_byte()) * 2)
			.wrapping_sub(usize::from(self.current_history.third_byte()))
			& 0xFF);
	}

	pub fn literal_context(&self) -> usize {
		self.literal_context
	}

	pub fn first_byte(&self) -> Byte {
		self.current_history.first_byte()
	}

	pub fn second_byte(&self) -> Byte {
		self.current_history.second_byte()
	}

	pub fn third_byte(&self) -> Byte {
		self.current_history.third_byte()
	}

	pub fn current_state(&self) -> HistoryState {
		self.current_state
	}
}
