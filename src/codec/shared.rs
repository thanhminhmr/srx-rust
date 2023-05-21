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

use crate::basic::{
	AnyError, AnyResult, Closable, Consumer, FromProducer, PipedReader, PipedWriter, Producer,
	ToConsumer,
};
use std::io::{Read, Write};
use std::thread::ScopedJoinHandle;

// -----------------------------------------------

struct WrappedReader<R: Read>(R);

impl<R: Read> Producer<u8> for WrappedReader<R> {
	fn produce(&mut self, buffer: &mut [u8]) -> AnyResult<usize> {
		Ok(self.0.read(buffer)?)
	}
}

pub fn run_file_reader<R: Read, const IO_BUFFER_SIZE: usize>(
	std_reader: R,
	mut writer: PipedWriter<u8, IO_BUFFER_SIZE>,
) -> AnyResult<R> {
	let mut reader: WrappedReader<R> = WrappedReader(std_reader);
	while writer.produce(&mut reader)? > 0 {}
	writer.close()?;
	Ok(reader.0)
}

// -----------------------------------------------

struct WrappedWriter<W: Write>(W);

impl<W: Write> Consumer<u8> for WrappedWriter<W> {
	fn consume(&mut self, buffer: &[u8]) -> AnyResult<usize> {
		self.0.write_all(buffer)?;
		Ok(buffer.len())
	}
}

pub fn run_file_writer<W: Write, const IO_BUFFER_SIZE: usize>(
	mut reader: PipedReader<u8, IO_BUFFER_SIZE>,
	std_writer: W,
) -> AnyResult<W> {
	let mut writer: WrappedWriter<W> = WrappedWriter(std_writer);
	while reader.consume(&mut writer)? > 0 {}
	reader.close()?;
	Ok(writer.0)
}

// -----------------------------------------------

pub fn thread_join<T>(thread_handle: ScopedJoinHandle<AnyResult<T>>) -> AnyResult<T> {
	match thread_handle.join() {
		Ok(value) => Ok(value?),
		Err(error) => Err(match error.downcast_ref::<String>() {
			Some(string) => AnyError::from_string(string),
			None => match error.downcast_ref::<&'static str>() {
				Some(&string) => AnyError::from_string(string),
				None => AnyError::from_box(error),
			},
		}),
	}
}
