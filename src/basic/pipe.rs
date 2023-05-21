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

use crate::basic::buffer::Buffer;
use crate::basic::error::{AnyError, AnyResult};
use crate::basic::io::{Closable, Consumer, FromProducer, Producer, Reader, ToConsumer, Writer};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

// -----------------------------------------------

type WriterToReader<T, const SIZE: usize> = (Buffer<T, SIZE>, usize);
type ReaderToWriter<T, const SIZE: usize> = Buffer<T, SIZE>;

// -----------------------------------------------

pub fn pipe<T: Copy + Send + 'static, const SIZE: usize>(
	value: T,
) -> (PipedWriter<T, SIZE>, PipedReader<T, SIZE>) {
	let (writer_sender, reader_receiver): (
		SyncSender<WriterToReader<T, SIZE>>,
		Receiver<WriterToReader<T, SIZE>>,
	) = sync_channel(1);
	let (reader_sender, writer_receiver): (
		SyncSender<ReaderToWriter<T, SIZE>>,
		Receiver<ReaderToWriter<T, SIZE>>,
	) = sync_channel(1);
	(
		PipedWriter {
			sender: writer_sender,
			receiver: writer_receiver,
			buffer: Some(Buffer::new(value)),
			index: 0,
		},
		PipedReader {
			sender: reader_sender,
			receiver: reader_receiver,
			buffer: Some(Buffer::new(value)),
			index: 0,
			length: 0,
		},
	)
}

// -----------------------------------------------

pub struct PipedWriter<T: Copy + Send + 'static, const SIZE: usize> {
	sender: SyncSender<WriterToReader<T, SIZE>>,
	receiver: Receiver<ReaderToWriter<T, SIZE>>,
	buffer: Option<Buffer<T, SIZE>>,
	index: usize,
}

impl<T: Copy + Send + 'static, const SIZE: usize> PipedWriter<T, SIZE> {
	// private sync
	fn sync(&mut self) -> AnyResult<()> {
		debug_assert!(self.buffer.is_some());
		debug_assert!(self.index > 0 && self.index <= SIZE);
		let buffer: Buffer<T, SIZE> = self.buffer.take().unwrap();
		self.sender.send((buffer, self.index))?;
		self.buffer = Some(self.receiver.recv()?);
		self.index = 0;
		Ok(())
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> Writer<T> for PipedWriter<T, SIZE> {
	fn write(&mut self, value: T) -> AnyResult<()> {
		match &mut self.buffer {
			None => Err(AnyError::from_string("Broken pipe!")),
			Some(buffer) => {
				debug_assert!(self.index < SIZE);
				buffer[self.index] = value;
				self.index += 1;
				debug_assert!(self.index <= SIZE);
				if self.index == SIZE {
					self.sync()?;
				}
				debug_assert!(self.index < SIZE);
				Ok(())
			}
		}
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> FromProducer<T> for PipedWriter<T, SIZE> {
	fn produce<P: Producer<T>>(&mut self, producer: &mut P) -> AnyResult<usize> {
		match &mut self.buffer {
			None => Err(AnyError::from_string("Broken pipe!")),
			Some(buffer) => {
				debug_assert!(self.index < SIZE);
				let sliced_buffer: &mut [T] = &mut buffer[self.index..SIZE];
				let produced_length: usize = producer.produce(sliced_buffer)?;
				debug_assert!(produced_length <= sliced_buffer.len());
				self.index += produced_length;
				debug_assert!(self.index <= SIZE);
				if self.index == SIZE {
					self.sync()?;
				}
				Ok(produced_length)
			}
		}
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> Closable<()> for PipedWriter<T, SIZE> {
	fn close(mut self) -> AnyResult<()> {
		if self.buffer.is_some() && self.index > 0 {
			debug_assert!(self.index <= SIZE);
			self.sync()
		} else {
			Ok(())
		}
	}
}

// -----------------------------------------------

pub struct PipedReader<T: Copy + Send + 'static, const SIZE: usize> {
	sender: SyncSender<ReaderToWriter<T, SIZE>>,
	receiver: Receiver<WriterToReader<T, SIZE>>,
	buffer: Option<Buffer<T, SIZE>>,
	length: usize,
	index: usize,
}

impl<T: Copy + Send + 'static, const SIZE: usize> PipedReader<T, SIZE> {
	fn sync(&mut self) {
		debug_assert!(self.index <= self.length && self.length <= SIZE);
		if !self.buffer.is_none() && self.index >= self.length {
			// take the old buffer and set it to None
			let old_buffer: Buffer<T, SIZE> = self.buffer.take().unwrap();
			// receive the new buffer
			if let Ok((new_buffer, length)) = self.receiver.recv() {
				debug_assert!(length > 0 && length <= SIZE);
				// set the new buffer and its length
				self.buffer = Some(new_buffer);
				self.length = length;
				self.index = 0;
				// send the old buffer away, maybe print something to log if error?
				let _error_ignored_ = self.sender.send(old_buffer);
			}
		}
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> Reader<T> for PipedReader<T, SIZE> {
	fn read(&mut self) -> AnyResult<Option<T>> {
		debug_assert!(self.index <= self.length && self.length <= SIZE);
		self.sync();
		match &mut self.buffer {
			None => Ok(None),
			Some(buffer) => {
				debug_assert!(self.index < self.length && self.length <= SIZE);
				let value: T = buffer[self.index];
				self.index += 1;
				debug_assert!(self.index <= self.length);
				Ok(Some(value))
			}
		}
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> ToConsumer<T> for PipedReader<T, SIZE> {
	fn consume<C: Consumer<T>>(&mut self, consumer: &mut C) -> AnyResult<usize> {
		debug_assert!(self.index <= self.length && self.length <= SIZE);
		self.sync();
		match &mut self.buffer {
			None => Ok(0),
			Some(buffer) => {
				debug_assert!(self.index < self.length && self.length <= SIZE);
				let sliced_buffer: &[T] = &buffer[self.index..self.length];
				let consumed_length: usize = consumer.consume(sliced_buffer)?;
				if consumed_length <= sliced_buffer.len() {
					self.index += consumed_length;
					debug_assert!(self.index <= SIZE);
					Ok(consumed_length)
				} else {
					Err(AnyError::from_string(
						"Consumed length is greater than available length!",
					))
				}
			}
		}
	}
}

impl<T: Copy + Send + 'static, const SIZE: usize> Closable<()> for PipedReader<T, SIZE> {
	fn close(self) -> AnyResult<()> {
		Ok(())
	}
}
