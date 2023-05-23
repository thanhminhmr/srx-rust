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

use super::state::{HistoryState, STATE_TABLE};
use crate::basic::Byte;
use crate::primary_context::ByteMatched;

// -----------------------------------------------

#[derive(Clone, Copy)]
pub struct ByteHistory(u32);

impl Default for ByteHistory {
	fn default() -> Self {
		ByteHistory(0)
	}
}

impl ByteHistory {
	pub fn first_byte(&self) -> Byte {
		Byte::from((self.0 >> 8) & 0xFF)
	}

	pub fn second_byte(&self) -> Byte {
		Byte::from((self.0 >> 16) & 0xFF)
	}

	pub fn third_byte(&self) -> Byte {
		Byte::from(self.0 >> 24)
	}

	pub fn get_state(&self) -> HistoryState {
		STATE_TABLE[(self.0 & 0xFF) as usize]
	}

	pub fn matching(&mut self, current_state: HistoryState, next_byte: Byte) -> ByteMatched {
		let mask: u32 = self.0 ^ (0x01_01_01_00 * u32::from(next_byte));
		let matched: ByteMatched = if (mask & 0x00_00_FF_00) == 0 {
			ByteMatched::FIRST
		} else if (mask & 0x00_FF_00_00) == 0 {
			ByteMatched::SECOND
		} else if (mask & 0xFF_00_00_00) == 0 {
			ByteMatched::THIRD
		} else {
			ByteMatched::NONE
		};
		self.matched(current_state, next_byte, matched);
		matched
	}

	pub fn matched(&mut self, current_state: HistoryState, next_byte: Byte, matched: ByteMatched) {
		let byte_history: u32 = self.0;
		debug_assert!(STATE_TABLE[(byte_history & 0xFF) as usize] == current_state);
		let updated_history: u32 = match matched {
			ByteMatched::FIRST => {
				// matched the first byte, keep the order of bytes
				byte_history & 0xFF_FF_FF_00
			}
			ByteMatched::SECOND => {
				// matched the second byte, swap the first and the second place
				(byte_history & 0xFF_00_00_00)
					| (((byte_history & 0x00_00_FF_00) | u32::from(next_byte)) << 8)
			}
			ByteMatched::THIRD => {
				// matched the third byte, move old first/second to second/third and set the first byte
				((byte_history & 0x00_FF_FF_00) | u32::from(next_byte)) << 8
			}
			ByteMatched::NONE => {
				// not match, move old first/second to second/third and set the first byte
				((byte_history & 0x00_FF_FF_00) | u32::from(next_byte)) << 8
			}
		};
		self.0 = updated_history | current_state.next(matched) as u32;
	}
}
