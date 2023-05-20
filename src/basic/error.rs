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

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

// -----------------------------------------------

pub type AnyResult<T> = Result<T, AnyError>;

// -----------------------------------------------

#[derive(Debug, Clone)]
pub struct AnyError(String);

impl AnyError {
    pub fn new<S: Into<String>>(into_string: S) -> Self {
        Self(into_string.into())
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E: Error> From<E> for AnyError {
    fn from(e: E) -> Self {
        Self(e.to_string())
    }
}
