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

use crate::primary_context::ByteMatched;
use std::ops::{Index, IndexMut};

// -----------------------------------------------

include!("state_table.inc");

// -----------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateElement(u64);

impl StateElement {
	pub const fn new(
		first_count: u8,
		next_if_first: u8,
		next_if_second: u8,
		next_if_third: u8,
		next_if_miss: u8,
	) -> Self {
		Self(
			(next_if_first as u64)
				| ((next_if_second as u64) << 8)
				| ((next_if_third as u64) << 16)
				| ((next_if_miss as u64) << 24)
				| ((first_count as u64) << 32),
		)
	}

	pub fn next(&self, matched: ByteMatched) -> u8 {
		match matched {
			ByteMatched::FIRST => self.0 as u8,
			ByteMatched::SECOND => (self.0 >> 8) as u8,
			ByteMatched::THIRD => (self.0 >> 16) as u8,
			ByteMatched::NONE => (self.0 >> 24) as u8,
		}
	}

	pub fn get(&self) -> usize {
		(self.0 >> 32) as usize
	}
}

// -----------------------------------------------

#[derive(Eq, PartialEq, Debug)]
pub struct StateTable<const SIZE: usize>([StateElement; SIZE]);

#[cfg(test)]
impl<const SIZE: usize> StateTable<SIZE> {
	pub fn new() -> Self {
		Self([StateElement(0); SIZE])
	}
}

impl<const SIZE: usize> Index<usize> for StateTable<SIZE> {
	type Output = StateElement;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<const SIZE: usize> IndexMut<usize> for StateTable<SIZE> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}
