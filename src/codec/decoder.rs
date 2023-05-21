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

use crate::basic::{pipe, AnyResult, Closable, PipedReader, PipedWriter, Writer};
use crate::bridged_context::{BridgedContextInfo, BridgedPrimaryContext, BridgedSecondaryContext};
use crate::codec::shared::{run_file_reader, run_file_writer, thread_join};
use crate::primary_context::ByteMatched;
use crate::secondary_context::{Bit, BitDecoder};
use std::io::{Read, Write};
use std::thread::{scope, ScopedJoinHandle};

// -----------------------------------------------

struct CombinedContextDecoder<const IO_BUFFER_SIZE: usize> {
	primary_context: BridgedPrimaryContext,
	secondary_context: BridgedSecondaryContext,
	decoder: BitDecoder<IO_BUFFER_SIZE>,
	writer: PipedWriter<u8, IO_BUFFER_SIZE>,
}

impl<const IO_BUFFER_SIZE: usize> CombinedContextDecoder<IO_BUFFER_SIZE> {
	fn bit(&mut self, context_index: usize) -> AnyResult<Bit> {
		let prediction: u32 = self.secondary_context.get(context_index);
		let bit: Bit = self.decoder.bit(prediction)?;
		self.secondary_context.update(context_index, bit);
		Ok(bit)
	}

	fn byte(&mut self, context_index: usize) -> AnyResult<u8> {
		let mut high: usize = 1;
		high += high + usize::from(self.bit(context_index + high)?);
		high += high + usize::from(self.bit(context_index + high)?);
		high += high + usize::from(self.bit(context_index + high)?);
		high += high + usize::from(self.bit(context_index + high)?);
		let low_context: usize = context_index + (15 * (high - 15)) as usize;
		let mut low: usize = 1;
		low += low + usize::from(self.bit(low_context + low)?);
		low += low + usize::from(self.bit(low_context + low)?);
		low += low + usize::from(self.bit(low_context + low)?);
		low += low + usize::from(self.bit(low_context + low)?);
		return Ok((((high - 16) << 4) | (low - 16)) as u8);
	}

	fn decode(mut self) -> AnyResult<()> {
		loop {
			let info: BridgedContextInfo = self.primary_context.context_info();
			let (next_byte, matched): (u8, ByteMatched) = match self.bit(info.first_context())? {
				// match first
				Bit::Zero => (info.first_byte(), ByteMatched::FIRST),
				// match next
				Bit::One => match self.bit(info.second_context())? {
					// literal
					Bit::Zero => {
						let next_byte: u8 = self.byte(info.literal_context())?;
						if next_byte == info.first_byte() {
							// eof, gave the reader/writer back
							self.decoder.close()?;
							self.writer.close()?;
							return Ok(());
						}
						(next_byte, ByteMatched::NONE)
					}
					// match next
					Bit::One => match self.bit(info.third_context())? {
						// match second
						Bit::Zero => (info.second_byte(), ByteMatched::SECOND),
						// match third
						Bit::One => (info.third_byte(), ByteMatched::THIRD),
					},
				},
			};
			self.writer.write(next_byte)?;
			self.primary_context.matched(next_byte, matched);
		}
	}
}

// -----------------------------------------------

fn run_combined_context_decoder<const IO_BUFFER_SIZE: usize>(
	reader: PipedReader<u8, IO_BUFFER_SIZE>,
	writer: PipedWriter<u8, IO_BUFFER_SIZE>,
) -> AnyResult<()> {
	let decoder: CombinedContextDecoder<IO_BUFFER_SIZE> = CombinedContextDecoder {
		primary_context: BridgedPrimaryContext::new(),
		secondary_context: BridgedSecondaryContext::new(),
		decoder: BitDecoder::new(reader),
		writer,
	};
	decoder.decode()
}

// -----------------------------------------------

pub fn decode<R: Read + Send, W: Write + Send, const IO_BUFFER_SIZE: usize>(
	reader: R,
	writer: W,
) -> AnyResult<(R, W)> {
	scope(|scope| {
		let (input_writer, input_reader): (
			PipedWriter<u8, IO_BUFFER_SIZE>,
			PipedReader<u8, IO_BUFFER_SIZE>,
		) = pipe::<u8, IO_BUFFER_SIZE>(0);
		let (output_writer, output_reader): (
			PipedWriter<u8, IO_BUFFER_SIZE>,
			PipedReader<u8, IO_BUFFER_SIZE>,
		) = pipe::<u8, IO_BUFFER_SIZE>(0);
		let file_reader: ScopedJoinHandle<AnyResult<R>> =
			scope.spawn(|| run_file_reader(reader, input_writer));
		let combined_context_decoder: ScopedJoinHandle<AnyResult<()>> =
			scope.spawn(|| run_combined_context_decoder(input_reader, output_writer));
		let file_writer: ScopedJoinHandle<AnyResult<W>> =
			scope.spawn(|| run_file_writer(output_reader, writer));
		let returned_reader: R = thread_join(file_reader)?;
		thread_join(combined_context_decoder)?;
		let returned_writer: W = thread_join(file_writer)?;
		Ok((returned_reader, returned_writer))
	})
}
