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

use crate::primary_context::matched::ByteMatched;

// -----------------------------------------------

#[derive(Clone, Copy)]
pub struct ByteHistory(u32);

impl ByteHistory {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn first_byte(&self) -> u8 {
        self.0 as u8
    }

    pub fn second_byte(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn third_byte(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn match_count(&self) -> usize {
        // TODO can a state machine perform better than a simple match count here?
        (self.0 >> 24) as usize
    }

    pub fn matching(&mut self, next_byte: u8) -> ByteMatched {
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

    pub fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
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
