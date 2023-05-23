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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Byte(usize);

impl From<Byte> for u8 {
	fn from(value: Byte) -> Self {
		value.0 as u8
	}
}

impl From<u8> for Byte {
	fn from(value: u8) -> Self {
		Byte(value as usize)
	}
}

macro_rules! impl_from_unsigned_for_byte {
    ($($t:ty),*) => {
        $(
            impl From<Byte> for $t {
				fn from(value: Byte) -> Self {
					value.0 as $t
				}
            }

            impl From<$t> for Byte {
				fn from(value: $t) -> Self {
					debug_assert!(value <= 255, "Unexpected value for Byte!");
					Byte(value as usize)
				}
            }
        )*
    };
}

macro_rules! impl_from_signed_for_byte {
    ($($t:ty),*) => {
        $(
            impl From<Byte> for $t {
				fn from(value: Byte) -> Self {
					value.0 as $t
				}
            }

            impl From<$t> for Byte {
				fn from(value: $t) -> Self {
					debug_assert!(value >= 0 && value <= 255, "Unexpected value for Byte!");
					Byte(value as usize)
				}
            }
        )*
    };
}

impl_from_unsigned_for_byte!(u16, u32, u64, u128, usize);
impl_from_signed_for_byte!(i16, i32, i64, i128, isize);
