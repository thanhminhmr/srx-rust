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

mod buffer;
mod byte;
mod error;
mod io;
mod pipe;

pub use self::buffer::Buffer;
pub use self::byte::Byte;
pub use self::error::{AnyError, AnyResult};
pub use self::io::{Closable, Consumer, FromProducer, Producer, Reader, ToConsumer, Writer};
pub use self::pipe::{pipe, PipedReader, PipedWriter};
