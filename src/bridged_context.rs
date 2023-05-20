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

pub struct BridgedPrimaryContext(PrimaryContext<PRIMARY_CONTEXT_SIZE>);

impl BridgedPrimaryContext {
    pub fn new() -> Self {
        Self(PrimaryContext::new())
    }

    fn bit_context(&self) -> usize {
        use std::cmp::min;
        let count: usize = self.0.match_count();
        let bit_context: usize = 0x4000 * 256
            + if count < 4 {
                ((self.0.previous_byte() as usize) << 2) | count
            } else {
                1024 + (min(count - 4, 63) >> 1)
            } * 768;
        bit_context
    }

    pub fn first_context(&self) -> usize {
        return self.bit_context() + self.0.first_byte() as usize;
    }

    pub fn second_context(&self) -> usize {
        return self.bit_context()
            + 0x100
            + self.0.second_byte().wrapping_add(self.0.third_byte()) as usize;
    }

    pub fn third_context(&self) -> usize {
        return self.bit_context()
            + 0x200
            + self
                .0
                .second_byte()
                .wrapping_mul(2)
                .wrapping_sub(self.0.third_byte()) as usize;
    }

    pub fn literal_context(&self) -> usize {
        (self.0.hash_value() & 0x3FFF) * 256
    }

    pub fn first_byte(&self) -> u8 {
        self.0.first_byte()
    }

    pub fn second_byte(&self) -> u8 {
        self.0.second_byte()
    }

    pub fn third_byte(&self) -> u8 {
        self.0.third_byte()
    }

    pub fn matching(&mut self, next_byte: u8) -> ByteMatched {
        self.0.matching(next_byte)
    }

    pub fn matched(&mut self, next_byte: u8, matched: ByteMatched) {
        self.0.matched(next_byte, matched)
    }
}
