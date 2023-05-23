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

use crate::secondary_context::Bit;
use super::info::{StateInfo, STATE_TABLE};

// -----------------------------------------------

#[derive(Copy, Clone)]
pub struct BitState(u16);

impl Default for BitState {
	fn default() -> Self {
		Self(0)
	}
}

impl BitState {
	pub fn get_info(&self) -> StateInfo {
		STATE_TABLE[self.0 as usize]
	}

	pub fn update(&mut self, current_state: StateInfo, bit: Bit) {
		debug_assert!(STATE_TABLE[self.0 as usize] == current_state);
		self.0 = current_state.next(bit);
	}
}
