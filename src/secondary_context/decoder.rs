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

use crate::basic::{AnyResult, Closable};
use crate::secondary_context::bit::Bit;
use std::array::from_mut;
use std::io::Read;

// -----------------------------------------------

pub struct BitDecoder<R: Read> {
    value: u32,
    low: u32,
    high: u32,
    reader: R,
}

impl<R: Read> BitDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            value: 0,
            low: 0,
            high: 0,
            reader,
        }
    }

    pub fn bit(&mut self, prediction: u32) -> AnyResult<Bit> {
        // shift bits in
        while (self.high ^ self.low) < 0x01000000 {
            // read byte
            let mut byte: u8 = 0;
            let read: usize = self.reader.read(from_mut(&mut byte))?;
            // shift new bits into high/low/value
            self.value = (self.value << 8) | if read > 0 { byte as u32 } else { 0xFF };
            self.low = self.low << 8;
            self.high = (self.high << 8) | 0xFF;
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
        }) = middle
            + match bit {
                Bit::Zero => 1,
                Bit::One => 0,
            };
        // return the value
        return Ok(bit);
    }
}

impl<R: Read> Closable<R> for BitDecoder<R> {
    fn close(self) -> AnyResult<R> {
        Ok(self.reader)
    }
}
