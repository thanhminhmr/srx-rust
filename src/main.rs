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

use crate::basic::{AnyError, AnyResult};
use crate::codec::{decode, encode};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::process::exit;
use std::time::Instant;

mod basic;
mod bridged_context;
mod codec;
mod primary_context;
mod secondary_context;

// -----------------------------------------------

const IO_BUFFER_SIZE: usize = 0x400000;
const MESSAGE_BUFFER_SIZE: usize = 0x40000;

// -----------------------------------------------

const SRX_HEADER: &[u8; 4] = b"sRx\x00";

fn run(input_path: &Path, output_path: &Path, is_compress: bool) -> AnyResult<(u64, u64, f64)> {
	// open file
	let mut reader: File = File::open(input_path)?;
	let mut writer: File = File::create(output_path)?;

	// start the timer
	let start: Instant = Instant::now();

	// do the compression/decompression
	let (mut done_reader, mut done_writer): (File, File) = if is_compress {
		writer.write_all(SRX_HEADER)?;
		encode::<File, File, IO_BUFFER_SIZE, MESSAGE_BUFFER_SIZE>(reader, writer)?
	} else {
		let mut buffer: [u8; 4] = [0; 4];
		reader.read_exact(&mut buffer)?;
		if !buffer.eq(SRX_HEADER) {
			return Err(AnyError::from_string("Not a SRX compressed file!"));
		}
		decode::<File, File, IO_BUFFER_SIZE>(reader, writer)?
	};

	// stop the timer and calculate the duration in seconds
	let duration: f64 = start.elapsed().as_millis() as f64 / 1000.0;

	// get the input and output size
	let input_size: u64 = done_reader.stream_position()?;
	let output_size: u64 = done_writer.stream_position()?;

	// oke
	Ok((input_size, output_size, duration))
}

fn help() -> ! {
	println!(
		"\
		srx: The fast Symbol Ranking based compressor, version {}.\n\
		Copyright (C) 2023  Mai Thanh Minh (a.k.a. thanhminhmr)\n\n\
		To   compress: srx c <input-file> <output-file>\n\
		To decompress: srx d <input-file> <output-file>",
		env!("CARGO_PKG_VERSION")
	);
	exit(0);
}

fn main() {
	let args: Vec<String> = env::args().collect();

	// check and parse arguments
	if args.len() != 4 {
		help()
	}
	let is_compress: bool = match args[1].as_str() {
		"c" => true,
		"d" => false,
		_ => help(),
	};
	let input_path: &Path = Path::new(&args[2]);
	let output_path: &Path = Path::new(&args[3]);

	// run the compression
	match run(input_path, output_path, is_compress) {
		Ok((input_size, output_size, duration)) => {
			// calculating and report
			let (percentage, speed) = if is_compress {
				(
					output_size as f64 / input_size as f64 * 100.0,
					input_size as f64 / duration / (1 << 20) as f64,
				)
			} else {
				(
					input_size as f64 / output_size as f64 * 100.0,
					output_size as f64 / duration / (1 << 20) as f64,
				)
			};
			println!(
				"{} -> {} ({:.2}%) in {:.2} seconds ({:.2} MiB/s)",
				input_size, output_size, percentage, duration, speed
			);
		}
		Err(error) => {
			// something unexpected happened
			println!("Error occurred! {}", error);
			exit(1);
		}
	};
}
