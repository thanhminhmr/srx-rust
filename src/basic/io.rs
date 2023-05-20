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

use crate::basic::AnyResult;

// -----------------------------------------------

pub trait Closable<T> {
	fn close(self) -> AnyResult<T>;
}

// -----------------------------------------------

pub trait Reader<T> {
	fn read(&mut self) -> AnyResult<Option<T>>;
}

// -----------------------------------------------

pub trait Writer<T> {
	fn write(&mut self, value: T) -> AnyResult<()>;
}

// -----------------------------------------------

pub trait Consumer<T> {
	fn consume(&mut self, buffer: &[T]) -> AnyResult<usize>;
}

// -----------------------------------------------

pub trait Producer<T> {
	fn produce(&mut self, buffer: &mut [T]) -> AnyResult<usize>;
}

// -----------------------------------------------

pub trait ToConsumer<T> {
	fn consume<C: Consumer<T>>(&mut self, consumer: &mut C) -> AnyResult<usize>;
}

// -----------------------------------------------

pub trait FromProducer<T> {
	fn produce<P: Producer<T>>(&mut self, producer: &mut P) -> AnyResult<usize>;
}
