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
use crate::secondary_context::bit::Bit;
use crate::secondary_context::prediction::BitPrediction;

pub struct SecondaryContext<const SIZE: usize> {
    context: Buffer<BitPrediction, SIZE>,
}

impl<const SIZE: usize> SecondaryContext<SIZE> {
    pub fn new() -> Self {
        Self {
            context: Buffer::new(BitPrediction::new()),
        }
    }

    pub fn get(&self, context_index: usize) -> u32 {
        debug_assert!(context_index < SIZE);
        self.context[context_index].get()
    }

    // return current prediction and then update the prediction with new bit
    pub fn update(&mut self, context_index: usize, bit: Bit) -> u32 {
        debug_assert!(context_index < SIZE);
        self.context[context_index].update(bit)
    }
}
