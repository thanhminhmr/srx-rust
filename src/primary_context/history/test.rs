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
use super::state::{HistoryState, STATE_TABLE};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufWriter, Write};
use std::path::Path;

// -----------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct StateInfo {
	first: u8,
	second: u8,
	third: u8,
}

impl StateInfo {
	fn id(&self) -> u64 {
		((self.first as u64) << 16) | ((self.second as u64) << 8) | (self.third as u64)
	}
}

impl PartialOrd<Self> for StateInfo {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for StateInfo {
	fn cmp(&self, other: &Self) -> Ordering {
		self.first
			.cmp(&other.first)
			.then(self.second.cmp(&other.second))
			.then(self.third.cmp(&other.third))
	}
}

// -----------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct PrimitiveState {
	current_state: StateInfo,
	next_if_first: StateInfo,
	next_if_second: StateInfo,
	next_if_third: StateInfo,
	next_if_miss: StateInfo,
}

// -----------------------------------------------

fn range(value: u8, max: u8) -> u8 {
	if value >= max {
		max
	} else {
		value
	}
}

fn increase(mut value: u8, max: u8) -> u8 {
	value += 1;
	if value >= max {
		max
	} else {
		value
	}
}

fn decrease(mut value: u8, max: u8) -> u8 {
	value = value.saturating_sub(1);
	if value >= max {
		max
	} else {
		value
	}
}

fn dec_nz(mut value: u8, max: u8) -> u8 {
	value = if value > 1 {
		value.saturating_sub(1)
	} else {
		value
	};
	if value >= max {
		max
	} else {
		value
	}
}

// -----------------------------------------------

#[derive(Debug)]
struct PrimitiveStateTable {
	map: HashMap<StateInfo, PrimitiveState>,
}

impl PrimitiveStateTable {
	const MAX_FIRST: u8 = 67;
	const MAX_SECOND: u8 = 7;
	const MAX_THIRD: u8 = 3;
	// const MAX_MISS: u8 = 3;

	fn new() -> Self {
		Self {
			map: HashMap::new(),
		}
	}

	fn state(
		&mut self,
		current_state: StateInfo,
		next_if_first: StateInfo,
		next_if_second: StateInfo,
		next_if_third: StateInfo,
		next_if_miss: StateInfo,
	) -> bool {
		let full_state = PrimitiveState {
			current_state,
			next_if_first,
			next_if_second,
			next_if_third,
			next_if_miss,
		};
		if let Some(old_state) = self.map.insert(current_state, full_state) {
			assert_eq!(old_state, full_state, "State already exist!");
			false
		} else {
			true
		}
	}

	fn next_if_first(current: StateInfo) -> StateInfo {
		if current.first <= 31 {
			StateInfo {
				first: increase(current.first, Self::MAX_FIRST),
				second: dec_nz(current.second, Self::MAX_SECOND),
				third: dec_nz(current.third, Self::MAX_THIRD),
			}
		} else {
			StateInfo {
				first: increase(current.first, Self::MAX_FIRST),
				second: 1,
				third: 1,
			}
		}
	}

	fn next_if_second(current: StateInfo) -> StateInfo {
		StateInfo {
			first: range(current.second, Self::MAX_FIRST),
			second: range(current.first, Self::MAX_SECOND),
			third: dec_nz(current.third, Self::MAX_THIRD),
		}
	}

	fn next_if_third(current: StateInfo) -> StateInfo {
		StateInfo {
			first: range(current.third, Self::MAX_FIRST),
			second: range(current.first, Self::MAX_SECOND),
			third: dec_nz(current.second, Self::MAX_THIRD),
		}
	}

	fn next_if_miss(current: StateInfo) -> StateInfo {
		StateInfo {
			first: 0,
			second: range(current.first, Self::MAX_SECOND),
			third: dec_nz(current.second, Self::MAX_THIRD),
		}
	}

	fn state_auto(&mut self, current: StateInfo) {
		let next_if_first: StateInfo = Self::next_if_first(current);
		let next_if_second: StateInfo = Self::next_if_second(current);
		let next_if_third: StateInfo = Self::next_if_third(current);
		let next_if_miss: StateInfo = Self::next_if_miss(current);
		if self.state(
			current,
			next_if_first,
			next_if_second,
			next_if_third,
			next_if_miss,
		) {
			self.state_auto(next_if_first);
			self.state_auto(next_if_second);
			self.state_auto(next_if_third);
			self.state_auto(next_if_miss);
		}
	}

	#[allow(dead_code)]
	fn export(&self) -> AnyResult<()> {
		let mut writer: BufWriter<File> = BufWriter::new(File::create(Path::new("map.gexf"))?);

		writer.write(br#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://gexf.net/1.3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://gexf.net/1.3 http://gexf.net/1.3/gexf.xsd" version="1.3">
	<meta lastmodifieddate="2009-03-20">
		<creator>Gephi.org</creator>
		<description>A Web network</description>
	</meta>
	<graph defaultedgetype="directed">
		<attributes class="node">
			<attribute id="0" title="first" type="integer"/>
			<attribute id="1" title="second" type="integer"/>
			<attribute id="2" title="third" type="integer"/>
		</attributes>
		<attributes class="edge">
			<attribute id="0" title="match" type="string"/>
		</attributes>
		<nodes>"#)?;

		for (state, _) in self.map.iter() {
			let id = state.id();
			let first: usize = state.first as usize;
			let second: usize = state.second as usize;
			let third: usize = state.third as usize;

			writer.write(
				format!(
					r#"
			<node id="{}" label="{},{},{}">
				<attvalues>
					<attvalue for="0" value="{}"/>
					<attvalue for="1" value="{}"/>
					<attvalue for="2" value="{}"/>
				</attvalues>
			</node>"#,
					id, first, second, third, first, second, third,
				)
				.as_bytes(),
			)?;
		}

		writer.write(
			br#"
		</nodes>
		<edges>"#,
		)?;

		for (_, state) in self.map.iter() {
			let current_state = state.current_state.id();
			let next_if_first = state.next_if_first.id();
			let next_if_second = state.next_if_second.id();
			let next_if_third = state.next_if_third.id();
			let next_if_miss = state.next_if_miss.id();

			writer.write(
				format!(
					r#"
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="FIRST"/>
				</attvalues>
			</edge>
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="SECOND"/>
				</attvalues>
			</edge>
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="THIRD"/>
				</attvalues>
			</edge>
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="MISS"/>
				</attvalues>
			</edge>"#,
					current_state,
					next_if_first,
					current_state,
					next_if_second,
					current_state,
					next_if_third,
					current_state,
					next_if_miss
				)
				.as_bytes(),
			)?;
		}

		writer.write(
			br#"
		</edges>
	</graph>
</gexf>"#,
		)?;
		Ok(())
	}
}

// -----------------------------------------------

#[test]
fn test_and_generate_state_table() -> AnyResult<()> {
	let mut table: PrimitiveStateTable = PrimitiveStateTable::new();
	table.state_auto(StateInfo {
		first: 0,
		second: 0,
		third: 0,
	});

	// table.export()?;

	let mut states: Vec<&PrimitiveState> = table.map.values().collect();
	states.sort_by_key(|x| x.current_state);

	let mut states_index: HashMap<&StateInfo, usize> = HashMap::new();
	for (index, &state) in states.iter().enumerate() {
		states_index.insert(&state.current_state, index);
	}

	// create next states array
	println!(
		"pub const STATE_TABLE: &[HistoryState] = &[ // length = {}",
		states.len()
	);
	let mut state_table: Vec<HistoryState> = Vec::new();
	for (index, &state) in states.iter().enumerate() {
		let first_count = state.current_state.first;
		let second_count = state.current_state.second;
		let third_count = state.current_state.third;
		let &next_if_first = states_index.get(&state.next_if_first).unwrap();
		let &next_if_second = states_index.get(&state.next_if_second).unwrap();
		let &next_if_third = states_index.get(&state.next_if_third).unwrap();
		let &next_if_miss = states_index.get(&state.next_if_miss).unwrap();
		state_table.push(HistoryState::new(
			first_count,
			next_if_first as u8,
			next_if_second as u8,
			next_if_third as u8,
			next_if_miss as u8,
		));
		println!(
			"\tHistoryState::new({:2}, {:3}, {:3}, {:3}, {:3}), // {:3}, {:2}, {:2}, {:2}",
			first_count,
			next_if_first,
			next_if_second,
			next_if_third,
			next_if_miss,
			index,
			first_count,
			second_count,
			third_count,
		);
	}
	println!("];");

	debug_assert!(state_table.eq(STATE_TABLE));

	Ok(())
}
