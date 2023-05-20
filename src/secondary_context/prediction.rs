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

use crate::secondary_context::bit::Bit;

// -----------------------------------------------

// MULTIPLIER[i] == 0x1_0000_0000 / (i + 2) but rounded
pub const MULTIPLIER: [u32; 256] = {
    // const-for loops not yet supported
    let mut table = [0; 256];
    let mut i: usize = 0;
    while i < 256 {
        let div = (1 << 33) / (i as u64 + 2);
        table[i] = ((div >> 1) + (div & 1)) as u32; // rounding
        i += 1;
    }
    table
};

// -----------------------------------------------

// lower 8-bit is a counter, higher 24-bit is prediction
#[derive(Clone)]
pub struct BitPrediction(u32);

impl BitPrediction {
    pub fn new() -> Self {
        Self(0x80000000)
    }

    pub fn get(&self) -> u32 {
        self.0 & 0xFFFFFF00
    }

    // return current prediction and then update the prediction with new bit
    pub fn update(&mut self, bit: Bit) -> u32 {
        // get bit 0-7 as count
        let count: usize = (self.0 & 0xFF) as usize;
        // masking bit 8-31 as old prediction
        let old_prediction: u32 = self.0 & 0xFFFFFF00;
        // create bit shift
        let bit_shift: i64 = match bit {
            Bit::Zero => 0,
            Bit::One => 1 << 32,
        };
        // get multiplier
        let multiplier: i64 = MULTIPLIER[count] as i64;
        // calculate new prediction
        let new_prediction: u32 = (((bit_shift - old_prediction as i64) * multiplier) >> 32) as u32;
        // update state
        self.0 = self
            .0
            .wrapping_add((new_prediction & 0xFFFFFF00) + if count < 255 { 1 } else { 0 });
        // return old prediction (before update)
        return old_prediction;
    }
}
