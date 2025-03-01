use core::fmt;
use std::cmp::min;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::ops::Index;
use std::collections::hash_map::{IntoIter, IntoKeys, Iter};
use std::fs::File;
use std::path::Path;

use crate::alc_error::AlcError;

use super::keycode::{Keycode, KeycodeOptions};
use super::ngram::Ngram;

pub trait Frequencies {}
impl Frequencies for f64 {}
impl Frequencies for u32 {}

#[derive(Debug, PartialEq, Clone)]
pub enum TopFrequenciesToTake {
	All,
	Num(usize),
}

use serde_derive::{Deserialize, Serialize};
use TopFrequenciesToTake::*;
impl fmt::Display for TopFrequenciesToTake {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			All => write!(f, "All"),
			Num(i) => write!(f, "{}", i),
		}
	}
}

/// single as in only holds one length of n-gram
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct SingleGramFrequencies<T> where T: Frequencies {
	frequencies: HashMap<Ngram, T>,
	n: usize,
	pub total: f64,
}
impl<T> SingleGramFrequencies<T> where T: Frequencies {
	pub fn new(n: usize) -> Self {
		Self { 
			frequencies:  Default::default(), 
			n,
			total: 0.0 
		}
	}
	pub fn get(&self, k: &Ngram) -> Option<&T> {
		self.frequencies.get(k)
	}
	pub fn into_keys(self) -> IntoKeys<Ngram, T> {
		self.frequencies.into_keys()
	}	
	pub fn iter(&self) -> Iter<'_, Ngram, T> {
		self.frequencies.iter()
	}
}

/// u32 for raw ngram counts
/// should only contain one ngram length
impl SingleGramFrequencies<u32> {
	// fn increment(&mut self, ngram: Ngram) -> Result<(), AlcError> {
	// 	if self.n != ngram.clone().len() {
	// 		Err(AlcError::NgramMatchError(ngram.len(), self.n))
	// 	} else {
	// 		Ok(*self.frequencies.entry(ngram).or_insert(0) += 1)
	// 	}
	// }

	/// Error if trying to add an Ngram with a different length
	fn add_from_key_value(&mut self, key: Ngram, value: u32) -> Result<(), AlcError> {
		if self.n != key.len() {
			Err(AlcError::NgramMatchError(key.len(), self.n))
		} else {
			self.total += value as f64;
			*self.frequencies.entry(key).or_insert(0) += value;
			Ok(())
		}
	}
	/// This might be faster if the bigger holder is on the left?
	/// consumes other (I think?)
	/// will give an error if trying to combine containers with different ngram lengths
	pub fn combine_with(&mut self, other: Self) -> Result<(), AlcError> {
		for (key, value) in other {
			self.add_from_key_value(key, value)?;
		}
		Ok(())
	}

	pub fn take_top_frequencies(&mut self, amount: TopFrequenciesToTake) {
		let mut hash_vec: Vec<(&Ngram, &u32)> = self.frequencies.iter().collect();
    	hash_vec.sort_by(|a, b| b.1.cmp(a.1));
		let amount_to_take = match amount {
			All => hash_vec.len(),
			Num(n) => min(hash_vec.len(), n),
		};
		let mut temp_freqs: HashMap<Ngram, u32> = Default::default();
		let mut new_total = 0.0;
		for item in hash_vec.iter().take(amount_to_take) {
			// println!("{:?}", item);
			let k = item.0.clone();
			let v = *item.1;
			new_total += v as f64;
			temp_freqs.insert(k, v);
		}
		self.frequencies = temp_freqs;
		self.total = new_total;
	}

	/// might want to rename because it isn't really a conversion, once something turns into frequencies it can't be turned back
	pub fn try_from_string(s: &str, n: usize, options: &KeycodeOptions) -> Result<Option<SingleGramFrequencies<u32>>, AlcError> {
		let mut ngram_to_counts: HashMap<Ngram, u32> = HashMap::new();
		let keycodes = Keycode::from_string(s, options)?;
		// if keycodes.contains(&Keycode::_AMPR) {
		// 	pnic!("found &");
		// }
		if keycodes.len() < n {
			// this particular string was not long enough to create an N-gram out of
			return Ok(None);
			// return Err(AlcError::GenericError(format!("{} was not long enough to create an {}-gram out of", s, n)));
		}
		let mut total = 0.0;
		for i in 0..(keycodes.len() - n + 1) {
			// should handle error here or change ngram to not error
			let ngram = Ngram::new(keycodes[i..i + n].to_vec());
			*ngram_to_counts.entry(ngram).or_insert(0) += 1;
			total += 1.0;
		}
		Ok(Some(SingleGramFrequencies { 
			frequencies: ngram_to_counts, 
			n, 
			total,
		}))
	}

	pub fn try_from_file<P>(filename: P, n: usize, options: &KeycodeOptions) -> Result<SingleGramFrequencies<u32>, AlcError> where P: AsRef<Path> {
		let file = match File::open(filename) {
			Ok(v) => v,
			Err(e) => return Err(AlcError::GenericError(format!("{}", e))),
		};
		let lines = io::BufReader::new(file).lines();
		let mut ngram_to_counts = Self::new(n);
		for line in lines.map_while(Result::ok) {
			if let Some(holder_from_line) = Self::try_from_string(line.as_str(), n, options)? {
				ngram_to_counts.combine_with(holder_from_line).unwrap();
			}
		}
		Ok(ngram_to_counts)
	}

	pub fn sum(&self) -> u32 {
		self.frequencies.values().sum()
	}

	pub fn len(&self) -> usize {
		self.frequencies.len()
	}
	pub fn is_empty(&self) -> bool {
		self.frequencies.is_empty()
	}
}
impl<T> Index<Ngram> for SingleGramFrequencies<T> where T: Frequencies {
	type Output = T;
	fn index(&self, ngram: Ngram) -> &Self::Output {
		&self.frequencies[&ngram]
	}
}
impl<T> IntoIterator for SingleGramFrequencies<T> where T: Frequencies {
	type Item = (Ngram, T);
	type IntoIter = IntoIter<Ngram, T>;
	fn into_iter(self) -> Self::IntoIter {
		self.frequencies.into_iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use Keycode::*;

	#[test]
	fn test_frequency_holder_construction () {
		let ngram = Ngram::new(vec![_A, _B]);
		let mut freq_holder = SingleGramFrequencies::<u32>::new(2);
		freq_holder.add_from_key_value(ngram.clone(), 1).unwrap();
		assert_eq!(freq_holder[ngram.clone()], 1);
		freq_holder.add_from_key_value(ngram.clone(), 1).unwrap();
		assert_eq!(freq_holder[ngram.clone()], 2);

		freq_holder.add_from_key_value(ngram.clone(), 5).unwrap();
		assert_eq!(freq_holder[ngram.clone()], 7);
		freq_holder.add_from_key_value(Ngram::new(vec![_C, _D]), 3).unwrap();
		assert_eq!(freq_holder[Ngram::new(vec![_C, _D])], 3);

		assert_eq!(freq_holder.total, 10.0);

		let mut freq_holder2 = SingleGramFrequencies::<u32>::new(2);
		freq_holder2.add_from_key_value(Ngram::new(vec![_A, _E]), 2).unwrap();
		freq_holder2.add_from_key_value(Ngram::new(vec![_C, _D]), 3).unwrap();

		assert_eq!(freq_holder2.total, 5.0);

		freq_holder.combine_with(freq_holder2).unwrap();
		assert_eq!(freq_holder[Ngram::new(vec![_A, _B])], 7);
		assert_eq!(freq_holder[Ngram::new(vec![_C, _D])], 6);
		assert_eq!(freq_holder[Ngram::new(vec![_A, _E])], 2);

		assert_eq!(freq_holder.total, 15.0);
	}

	#[test]
	fn test_frequency_holder_ab() {
		let ngram = Ngram::new(vec![_A, _B]);
		let mut expected_ngram_to_counts: HashMap<Ngram, u32> = HashMap::new();
		expected_ngram_to_counts.insert(ngram, 1);
		assert_eq!(SingleGramFrequencies::try_from_string("ab", 2, &KeycodeOptions::default()).unwrap().unwrap(), SingleGramFrequencies { frequencies: expected_ngram_to_counts, n: 2, total: 1.0 });
	}

	#[test]
	fn test_frequency_holder_abab() {
		let ngram = Ngram::new(vec![_A, _B]);
		// let map: HashMap<Ngram<2>, u32> = HashMap::new();
		// let holder = NgramFrequencyHolder { frequencies: map };
		let holder2 = SingleGramFrequencies::<u32>::try_from_string("abab", 2, &KeycodeOptions::default()).unwrap().unwrap();
		let holder2_ab = holder2[ngram];
		assert_eq!(holder2_ab, 2);
		let holder2_ba = holder2[Ngram::new(vec![_B, _A])];
		assert_eq!(holder2_ba, 1);
		let holder4 = SingleGramFrequencies::<u32>::try_from_string("abab", 4, &KeycodeOptions::default()).unwrap().unwrap();
		let holder4_val = holder4[Ngram::new(vec![_A, _B, _A, _B])];
		assert_eq!(holder4_val, 1)
	}

	#[test]
	fn test_read_from_file() {
		let mut holder = SingleGramFrequencies::<u32>::try_from_file("./data/rust_book_test/ch04-02-references-and-borrowing.md", 2, &KeycodeOptions::default()).unwrap();
		let holder_clone = holder.clone();
		// println!("{:?}", holder);
		// this value is found by control + F "he" and seeing how many matches there are
		assert_eq!(holder[Ngram::new(vec![_H, _E])], 145);
		holder.take_top_frequencies(All);
		assert_eq!(holder, holder_clone);

		holder.take_top_frequencies(Num(10));
		println!("{:?}", holder);
		assert_eq!(holder.len(), 10);
	}

}