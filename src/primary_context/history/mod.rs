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

mod state;
#[cfg(test)]
mod test;

use crate::primary_context::history::state::{StateElement, STATE_TABLE};
use crate::primary_context::matched::ByteMatched;

// -----------------------------------------------

#[derive(Clone, Copy)]
pub struct ByteHistory(u32);

impl ByteHistory {
	pub fn new() -> Self {
		Self(0)
	}

	pub fn get(&self) -> (u8, u8, u8, usize) {
		(
			// first byte
			(self.0 >> 8) as u8,
			// second byte
			(self.0 >> 16) as u8,
			// third byte
			(self.0 >> 24) as u8,
			// match_count
			STATE_TABLE[(self.0 & 0xFF) as usize].get(),
		)
	}

	pub fn matching(&mut self, next_byte: u8) -> ByteMatched {
		let mask: u32 = self.0 ^ (0x01_01_01_00 * next_byte as u32);
		let matched : ByteMatched = if (mask & 0x00_00_FF_00) == 0 {
			ByteMatched::FIRST
		} else if (mask & 0x00_FF_00_00) == 0 {
			ByteMatched::SECOND
		} else if (mask & 0xFF_00_00_00) == 0 {
			ByteMatched::THIRD
		} else {
			ByteMatched::NONE
		};
		self.matched(next_byte, matched);
		matched
	}

	pub fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
		let byte_history: u32 = self.0;
		let current_state: StateElement = STATE_TABLE[(byte_history & 0xFF) as usize];
		let updated_history: u32 = match matched {
			ByteMatched::FIRST => {
				// matched the first byte, keep the order of bytes
				byte_history & 0xFF_FF_FF_00
			}
			ByteMatched::SECOND => {
				// matched the second byte, swap the first and the second place
				(byte_history & 0xFF_00_00_00) | (((byte_history & 0x00_00_FF_00) | next_byte as u32) << 8)
			}
			ByteMatched::THIRD => {
				// matched the third byte, move old first/second to second/third and set the first byte
				((byte_history & 0x00_FF_FF_00) | next_byte as u32) << 8
			}
			ByteMatched::NONE => {
				// not match, move old first/second to second/third and set the first byte
				((byte_history & 0x00_FF_FF_00) | next_byte as u32) << 8
			}
		};
		self.0 = updated_history | current_state.next(matched) as u32;
	}
}
