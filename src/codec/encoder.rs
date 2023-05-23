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

use super::shared::{run_file_reader, run_file_writer, thread_join};
use crate::basic::{pipe, AnyResult, Byte, Closable, PipedReader, PipedWriter, Reader, Writer};
use crate::bridged_context::{BridgedContextInfo, BridgedPrimaryContext, BridgedSecondaryContext};
use crate::primary_context::ByteMatched;
use crate::secondary_context::{Bit, BitEncoder, StateInfo};
use std::io::{Read, Write};
use std::thread::{scope, ScopedJoinHandle};

// -----------------------------------------------

#[derive(Copy, Clone)]
enum Message {
	Bit(usize, Bit),
	Byte(usize, Byte),
}

#[derive(Copy, Clone)]
struct PackedMessage(u32);

impl Default for PackedMessage {
	fn default() -> Self {
		Self(0)
	}
}

impl PackedMessage {
	fn bit(context: usize, bit: Bit) -> Self {
		Self(u32::from(bit) << 30 | context as u32)
	}

	fn byte(context: usize, byte: Byte) -> Self {
		Self(0x80000000 | context as u32 | u32::from(byte))
	}

	fn get(&self) -> Message {
		if self.0 < 0x80000000 {
			Message::Bit((self.0 & 0x3FFFFFFF) as usize, Bit::from(self.0 >> 30))
		} else {
			Message::Byte((self.0 & 0x7FFFFF00) as usize, Byte::from(self.0 & 0xFF))
		}
	}
}

// -----------------------------------------------

fn run_primary_context_encoder<const IO_BUFFER_SIZE: usize, const MESSAGE_BUFFER_SIZE: usize>(
	mut reader: PipedReader<u8, IO_BUFFER_SIZE>,
	mut writer: PipedWriter<PackedMessage, MESSAGE_BUFFER_SIZE>,
) -> AnyResult<()> {
	let mut context: BridgedPrimaryContext = BridgedPrimaryContext::new();
	loop {
		let info: BridgedContextInfo = BridgedContextInfo::new(
			context.get_history(),
			context.previous_byte(),
			context.hash_value(),
		);
		match reader.read()? {
			None => {
				writer.write(PackedMessage::bit(info.first_context(), Bit::One))?;
				writer.write(PackedMessage::bit(info.second_context(), Bit::Zero))?;
				writer.write(PackedMessage::byte(
					info.literal_context(),
					info.first_byte(),
				))?;
				reader.close()?;
				writer.close()?;
				return Ok(());
			}
			Some(current_byte) => {
				match context.matching(info.current_state(), Byte::from(current_byte)) {
					ByteMatched::FIRST => {
						writer.write(PackedMessage::bit(info.first_context(), Bit::Zero))?;
					}
					ByteMatched::NONE => {
						writer.write(PackedMessage::bit(info.first_context(), Bit::One))?;
						writer.write(PackedMessage::bit(info.second_context(), Bit::Zero))?;
						writer.write(PackedMessage::byte(
							info.literal_context(),
							Byte::from(current_byte),
						))?;
					}
					ByteMatched::SECOND => {
						writer.write(PackedMessage::bit(info.first_context(), Bit::One))?;
						writer.write(PackedMessage::bit(info.second_context(), Bit::One))?;
						writer.write(PackedMessage::bit(info.third_context(), Bit::Zero))?;
					}
					ByteMatched::THIRD => {
						writer.write(PackedMessage::bit(info.first_context(), Bit::One))?;
						writer.write(PackedMessage::bit(info.second_context(), Bit::One))?;
						writer.write(PackedMessage::bit(info.third_context(), Bit::One))?;
					}
				}
			}
		}
	}
}

// -----------------------------------------------

struct SecondaryContextEncoder<const IO_BUFFER_SIZE: usize, const MESSAGE_BUFFER_SIZE: usize> {
	context: BridgedSecondaryContext,
	reader: PipedReader<PackedMessage, MESSAGE_BUFFER_SIZE>,
	encoder: BitEncoder<IO_BUFFER_SIZE>,
}

impl<const IO_BUFFER_SIZE: usize, const MESSAGE_BUFFER_SIZE: usize>
	SecondaryContextEncoder<IO_BUFFER_SIZE, MESSAGE_BUFFER_SIZE>
{
	#[inline(always)]
	fn bit(&mut self, context_index: usize, bit: Bit) -> AnyResult<()> {
		let current_state: StateInfo = self.context.get_info(context_index);
		self.context.update(current_state, context_index, bit);
		self.encoder.bit(current_state.prediction(), bit)
	}

	fn byte(&mut self, context_index: usize, byte: Byte) -> AnyResult<()> {
		// code high 4 bits in first 15 contexts
		let high: usize = (usize::from(byte) >> 4) | 16;
		self.bit(context_index + 1, Bit::from(high >> 3 & 1))?;
		self.bit(context_index + (high >> 3), Bit::from(high >> 2 & 1))?;
		self.bit(context_index + (high >> 2), Bit::from(high >> 1 & 1))?;
		self.bit(context_index + (high >> 1), Bit::from(high & 1))?;
		// code low 4 bits in one of 16 blocks of 15 contexts (to reduce cache misses)
		let low_context: usize = context_index + 15 * (high - 15);
		let low: usize = (usize::from(byte) & 15) | 16;
		self.bit(low_context + 1, Bit::from(low >> 3 & 1))?;
		self.bit(low_context + (low >> 3), Bit::from(low >> 2 & 1))?;
		self.bit(low_context + (low >> 2), Bit::from(low >> 1 & 1))?;
		self.bit(low_context + (low >> 1), Bit::from(low & 1))?;
		// oke
		return Ok(());
	}

	fn encode(mut self) -> AnyResult<()> {
		loop {
			match self.reader.read()? {
				None => {
					self.reader.close()?;
					self.encoder.close()?;
					return Ok(());
				}
				Some(message) => match message.get() {
					Message::Bit(context_index, bit) => self.bit(context_index, bit)?,
					Message::Byte(context_index, value) => self.byte(context_index, value)?,
				},
			}
		}
	}
}

// -----------------------------------------------

fn run_secondary_context_encoder<const IO_BUFFER_SIZE: usize, const MESSAGE_BUFFER_SIZE: usize>(
	reader: PipedReader<PackedMessage, MESSAGE_BUFFER_SIZE>,
	writer: PipedWriter<u8, IO_BUFFER_SIZE>,
) -> AnyResult<()> {
	let encoder: SecondaryContextEncoder<IO_BUFFER_SIZE, MESSAGE_BUFFER_SIZE> =
		SecondaryContextEncoder {
			context: BridgedSecondaryContext::new(),
			reader,
			encoder: BitEncoder::new(writer),
		};
	encoder.encode()
}

// -----------------------------------------------

pub fn encode<
	R: Read + Send,
	W: Write + Send,
	const IO_BUFFER_SIZE: usize,
	const MESSAGE_BUFFER_SIZE: usize,
>(
	reader: R,
	writer: W,
) -> AnyResult<(R, W)> {
	scope(|scope| {
		let (input_writer, input_reader): (
			PipedWriter<u8, IO_BUFFER_SIZE>,
			PipedReader<u8, IO_BUFFER_SIZE>,
		) = pipe::<u8, IO_BUFFER_SIZE>();
		let (message_writer, message_reader): (
			PipedWriter<PackedMessage, MESSAGE_BUFFER_SIZE>,
			PipedReader<PackedMessage, MESSAGE_BUFFER_SIZE>,
		) = pipe::<PackedMessage, MESSAGE_BUFFER_SIZE>();
		let (output_writer, output_reader): (
			PipedWriter<u8, IO_BUFFER_SIZE>,
			PipedReader<u8, IO_BUFFER_SIZE>,
		) = pipe::<u8, IO_BUFFER_SIZE>();
		let file_reader: ScopedJoinHandle<AnyResult<R>> =
			scope.spawn(|| run_file_reader(reader, input_writer));
		let primary_context_encoder: ScopedJoinHandle<AnyResult<()>> =
			scope.spawn(|| run_primary_context_encoder(input_reader, message_writer));
		let secondary_context_encoder: ScopedJoinHandle<AnyResult<()>> =
			scope.spawn(|| run_secondary_context_encoder(message_reader, output_writer));
		let file_writer: ScopedJoinHandle<AnyResult<W>> =
			scope.spawn(|| run_file_writer(output_reader, writer));
		let returned_reader: R = thread_join(file_reader)?;
		thread_join(primary_context_encoder)?;
		thread_join(secondary_context_encoder)?;
		let returned_writer: W = thread_join(file_writer)?;
		Ok((returned_reader, returned_writer))
	})
}
