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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Bit {
	Zero,
	One,
}

impl From<Bit> for bool {
	fn from(value: Bit) -> Self {
		match value {
			Bit::Zero => false,
			Bit::One => true,
		}
	}
}

impl From<bool> for Bit {
	fn from(value: bool) -> Self {
		match value {
			false => Bit::Zero,
			true => Bit::One,
		}
	}
}

macro_rules! impl_from_for_bit {
    ($($t:ty),*) => {
        $(
            impl From<Bit> for $t {
				fn from(value: Bit) -> Self {
					match value {
						Bit::Zero => 0,
						Bit::One => 1,
					}
				}
            }
        )*
    };
}

impl_from_for_bit!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
