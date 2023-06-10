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

use super::info::{StateInfo, STATE_TABLE};
use crate::basic::AnyResult;
use crate::secondary_context::Bit;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::path::Path;

// -----------------------------------------------

const fn gcd_remaining(first: u64, second: u64) -> (u64, u64, u64) {
	let gcd: u64 = gcd(first, second);
	(first / gcd, second / gcd, gcd)
}

const fn gcd(first: u64, second: u64) -> u64 {
	let first_power: u32 = first.trailing_zeros();
	let second_power: u32 = second.trailing_zeros();
	fast_gcd(first >> first_power, second >> second_power)
		<< if first_power <= second_power {
			first_power
		} else {
			second_power
		}
}

const fn fast_gcd(mut first: u64, mut second: u64) -> u64 {
	loop {
		debug_assert!(first % 2 == 1 && second % 2 == 1);
		if first > second {
			let temp: u64 = first;
			first = second;
			second = temp;
		}
		second -= first;
		if second == 0 {
			return first;
		}
		second >>= second.trailing_zeros();
	}
}

const fn gcd_reduce(first: u64, second: u64) -> (u64, u64) {
	let (reduced_first, reduced_second, _): (u64, u64, u64) = gcd_remaining(first, second);
	(reduced_first, reduced_second)
}

// -----------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Fraction {
	numerator: u64,
	denominator: u64,
}

impl Fraction {
	const fn new(numerator: u64, denominator: u64) -> Self {
		assert!(denominator > 0, "Fraction denominator is zero!");
		let (numerator, denominator): (u64, u64) = gcd_reduce(numerator, denominator);
		Self {
			numerator,
			denominator,
		}
	}

	fn normalize(&mut self) {
		let (numerator, denominator): (u64, u64) = gcd_reduce(self.numerator, self.denominator);
		self.numerator = numerator;
		self.denominator = denominator;
	}
}

impl Add for Fraction {
	type Output = Fraction;

	fn add(mut self, rhs: Self) -> Self::Output {
		self.add_assign(rhs);
		self
	}
}

impl AddAssign for Fraction {
	fn add_assign(&mut self, rhs: Self) {
		let (self_rem, rhs_rem, gcd): (u64, u64, u64) =
			gcd_remaining(self.denominator, rhs.denominator);
		self.numerator = self.numerator * rhs_rem + self_rem * rhs.numerator;
		self.denominator = self_rem * rhs_rem * gcd; // LCM
		self.normalize()
	}
}

impl Sub for Fraction {
	type Output = Fraction;

	fn sub(mut self, rhs: Self) -> Self::Output {
		self.sub_assign(rhs);
		self
	}
}

impl SubAssign for Fraction {
	fn sub_assign(&mut self, rhs: Self) {
		let (self_rem, rhs_rem, gcd): (u64, u64, u64) =
			gcd_remaining(self.denominator, rhs.denominator);
		self.numerator = self.numerator * rhs_rem - self_rem * rhs.numerator;
		self.denominator = self_rem * rhs_rem * gcd; // LCM
		self.normalize()
	}
}

impl Mul for Fraction {
	type Output = Fraction;

	fn mul(mut self, rhs: Self) -> Self::Output {
		self.mul_assign(rhs);
		self
	}
}

impl MulAssign for Fraction {
	fn mul_assign(&mut self, rhs: Self) {
		let (self_num, rhs_denom): (u64, u64) = gcd_reduce(self.numerator, rhs.denominator);
		let (rhs_num, self_denom): (u64, u64) = gcd_reduce(rhs.numerator, self.denominator);
		self.numerator = self_num * rhs_num;
		self.denominator = self_denom * rhs_denom;
	}
}

impl Div for Fraction {
	type Output = Fraction;

	fn div(mut self, rhs: Self) -> Self::Output {
		self.mul_assign(Fraction {
			numerator: rhs.denominator,
			denominator: rhs.numerator,
		});
		self
	}
}

impl DivAssign for Fraction {
	fn div_assign(&mut self, rhs: Self) {
		self.mul_assign(Fraction {
			numerator: rhs.denominator,
			denominator: rhs.numerator,
		});
	}
}

impl From<Fraction> for f64 {
	fn from(value: Fraction) -> Self {
		value.numerator as f64 / value.denominator as f64
	}
}

impl PartialOrd for Fraction {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Fraction {
	fn cmp(&self, other: &Self) -> Ordering {
		f64::from(*self).total_cmp(&f64::from(*other))
	}
}

// -----------------------------------------------

#[derive(Copy, Clone, Debug)]
enum Value {
	Fraction(Fraction),
	Prediction(f64),
}

impl PartialEq<Self> for Value {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Value::Fraction(my_fraction) => match other {
				Value::Fraction(other_fraction) => my_fraction.eq(other_fraction),
				Value::Prediction(_) => false,
			},
			Value::Prediction(my_prediction) => match other {
				Value::Fraction(_) => false,
				Value::Prediction(other_prediction) => {
					my_prediction.total_cmp(other_prediction) == Ordering::Equal
				}
			},
		}
	}
}

impl Eq for Value {}

impl PartialOrd<Self> for Value {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Value {
	fn cmp(&self, other: &Self) -> Ordering {
		match self {
			Value::Fraction(my_fraction) => match other {
				Value::Fraction(other_fraction) => my_fraction.cmp(other_fraction),
				Value::Prediction(_) => Ordering::Less,
			},
			Value::Prediction(my_prediction) => match other {
				Value::Fraction(_) => Ordering::Greater,
				Value::Prediction(other_prediction) => my_prediction.total_cmp(other_prediction),
			},
		}
	}
}

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Value::Fraction(fraction) => fraction.hash(state),
			Value::Prediction(value) => value.to_bits().hash(state),
		}
	}
}

impl From<Value> for f64 {
	fn from(value: Value) -> Self {
		match value {
			Value::Fraction(fraction) => fraction.numerator as f64 / fraction.denominator as f64,
			Value::Prediction(value) => value,
		}
	}
}

impl From<Value> for u32 {
	fn from(value: Value) -> Self {
		let fx: f64 = match value {
			Value::Fraction(fraction) => fraction.numerator as f64 / fraction.denominator as f64,
			Value::Prediction(value) => value,
		};
		(fx * (1u64 << 32) as f64).round() as u32
	}
}

// -----------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct StateIndex {
	count: u64,
	value: Value,
}

impl PartialOrd<Self> for StateIndex {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for StateIndex {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.count.cmp(&other.count) {
			Ordering::Less => Ordering::Less,
			Ordering::Greater => Ordering::Greater,
			Ordering::Equal => self.value.cmp(&other.value),
		}
	}
}

// -----------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct PrimitiveState {
	current_state: StateIndex,
	next_if_zero: StateIndex,
	next_if_one: StateIndex,
}

// -----------------------------------------------

fn state(count: u64, value: Value) -> StateIndex {
	StateIndex { count, value }
}

fn fraction(numerator: u64, denominator: u64) -> Value {
	Value::Fraction(Fraction::new(numerator, denominator))
}

fn prediction_rescaling(numerator: u64, denominator: u64) -> f64 {
	let x: f64 = numerator as f64 / denominator as f64;
	let sqr_x: f64 = x * x;
	let sqr_x_m_1: f64 = (1.0 - x) * (1.0 - x);
	sqr_x / (sqr_x + sqr_x_m_1)
}

fn prediction_next(predictions: &Vec<f64>, current_state: StateIndex, bit: Bit) -> StateIndex {
	let (count, value): (u64, f64) = match current_state.value {
		Value::Fraction(fraction) => (current_state.count + 1, f64::from(fraction)),
		Value::Prediction(value) => (current_state.count, value),
	};
	let prediction: f64 = if bit.into() {
		value + (1.0 - value) / (count + 2) as f64
	} else {
		value - value / (count + 2) as f64
	};
	let normalized_prediction: f64 =
		match predictions.binary_search_by(|value| value.total_cmp(&prediction)) {
			Ok(index) => *predictions.get(index).unwrap(),
			Err(index) => {
				if index == 0 {
					*predictions.get(index).unwrap()
				} else if index == predictions.len() {
					*predictions.get(index - 1).unwrap()
				} else {
					let prev: f64 = *predictions.get(index - 1).unwrap();
					let next: f64 = *predictions.get(index).unwrap();
					if index * 2 < predictions.len() {
						prev
					} else {
						next
					}
				}
			}
		};
	StateIndex {
		count,
		value: Value::Prediction(normalized_prediction),
	}
}

// -----------------------------------------------

#[derive(Debug)]
struct PrimitiveStateTable {
	map: HashMap<StateIndex, PrimitiveState>,
}

impl PrimitiveStateTable {
	fn new() -> Self {
		Self {
			map: HashMap::new(),
		}
	}

	fn state(
		&mut self,
		current_state: StateIndex,
		next_if_zero: StateIndex,
		next_if_one: StateIndex,
	) {
		let full_state = PrimitiveState {
			current_state,
			next_if_zero,
			next_if_one,
		};
		if let Some(old_state) = self.map.insert(current_state, full_state) {
			assert_eq!(old_state, full_state, "State already exist!");
		}
	}

	fn state_auto(&mut self, current_state: StateIndex) {
		const ONE: Fraction = Fraction::new(1, 1);
		let count: u64 = current_state.count;
		let value: Fraction = match current_state.value {
			Value::Fraction(fraction) => fraction,
			Value::Prediction(_) => panic!("Invalid value!"),
		};
		let fraction: Fraction = Fraction::new(1, count + 2);
		self.state(
			current_state,
			state(count + 1, Value::Fraction(value - value * fraction)),
			state(count + 1, Value::Fraction(value + (ONE - value) * fraction)),
		);
	}

	fn state_manual(&mut self, prediction: &Vec<f64>, current_state: StateIndex) {
		self.state(
			current_state,
			prediction_next(prediction, current_state, Bit::Zero),
			prediction_next(prediction, current_state, Bit::One),
		);
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
		<attributes class="node">\
			<attribute id="0" title="level" type="integer"/>
			<attribute id="1" title="probablity" type="double"/>
		</attributes>
		<attributes class="edge">
			<attribute id="0" title="bit" type="boolean"/>
		</attributes>
		<nodes>"#)?;

		for (state, _) in self.map.iter() {
			let mut hasher: DefaultHasher = DefaultHasher::new();
			state.hash(&mut hasher);
			let id = hasher.finish();

			let level: usize = state.count as usize;
			let prediction: f64 = f64::from(state.value);

			writer.write(
				format!(
					r#"
			<node id="{}" label="{},{}">
				<attvalues>
					<attvalue for="0" value="{}"/>
					<attvalue for="1" value="{}"/>
				</attvalues>
			</node>"#,
					id, level, prediction, level, prediction,
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
			let mut hasher: DefaultHasher = DefaultHasher::new();
			state.current_state.hash(&mut hasher);
			let id = hasher.finish();

			let mut hasher: DefaultHasher = DefaultHasher::new();
			state.next_if_zero.hash(&mut hasher);
			let zero_id = hasher.finish();

			let mut hasher: DefaultHasher = DefaultHasher::new();
			state.next_if_one.hash(&mut hasher);
			let one_id = hasher.finish();

			writer.write(
				format!(
					r#"
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="false"/>
				</attvalues>
			</edge>
			<edge source="{}" target="{}">
				<attvalues>
					<attvalue for="0" value="true"/>
				</attvalues>
			</edge>"#,
					id, zero_id, id, one_id
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

fn table() -> AnyResult<PrimitiveStateTable> {
	let mut table: PrimitiveStateTable = PrimitiveStateTable::new();

	let limit_level: u64 = 64;

	// table.state_auto(state(0, fraction(1, 2)));

	for level in 0..limit_level - 1 {
		let denominator: u64 = (level + 1) * 2;
		for index in 0..level + 1 {
			let numerator: u64 = index * 2 + 1;
			table.state_auto(state(level, fraction(numerator, denominator)));
		}
	}

	let limit_denominator: u64 = (1 << 16) - limit_level - table.map.len() as u64;

	let mut predictions: Vec<f64> = Vec::with_capacity(limit_denominator as usize);
	for index in 1..limit_denominator + 1 {
		// predictions.push(prediction_f64(index, limit_denominator + 1));
		// predictions.push(index as f64 / (limit_denominator + 1) as f64);
		predictions.push(prediction_rescaling(index, limit_denominator + 1));
	}
	predictions.sort_by(f64::total_cmp);

	// dbg!(&predictions);

	for index in 0..limit_level {
		let numerator: u64 = index * 2 + 1;
		table.state_manual(
			&predictions,
			state(limit_level - 1, fraction(numerator, limit_level * 2)),
		);
	}

	for prediction in predictions.iter() {
		table.state_manual(
			&predictions,
			state(limit_level, Value::Prediction(*prediction)),
		);
	}

	Ok(table)
}

// -----------------------------------------------

#[test]
fn test_and_generate_state_table() -> AnyResult<()> {
	// create table
	let table = table()?;

	// dbg!(&table);

	// export to Gephi (*.gexf)
	// table.export()?;

	// check for valid table
	assert_eq!(table.map.len(), 1 << 16);

	// get the states as an array
	let mut data: Vec<&PrimitiveState> = table.map.iter().map(|(_, state)| state).collect();
	data.sort_by_key(|x| x.current_state);

	// create index for states
	let mut data_index: HashMap<StateIndex, usize> = HashMap::new();
	data.iter().enumerate().for_each(|(index, state)| {
		data_index.insert(state.current_state, index);
	});

	// create next states array
	println!("pub const STATE_TABLE: &[StateInfo] = &[ // length = {}", data.len());
	let mut state_table: Vec<StateInfo> = Vec::new();
	for index in 0..65536 {
		let state: &PrimitiveState = data[index];
		let level: usize = state.current_state.count as usize;
		let prediction: u32 = u32::from(state.current_state.value);
		let next_if_zero: u16 = *data_index.get(&state.next_if_zero).unwrap() as u16;
		let next_if_one: u16 = *data_index.get(&state.next_if_one).unwrap() as u16;
		state_table.push(StateInfo::new(prediction, next_if_zero, next_if_one));
		println!(
			"\tStateInfo::new(0x{:08X}, 0x{:04X}, 0x{:04X}), // 0x{:04X}, {}",
			prediction, next_if_zero, next_if_one, index, level,
		);
	}
	println!("];");

	debug_assert!(state_table.eq(STATE_TABLE));

	Ok(())
}
