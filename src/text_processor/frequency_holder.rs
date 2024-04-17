use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{self, BufRead};
use std::ops::Index;
use std::collections::hash_map::Keys;
use std::fs::File;
use std::path::Path;

use super::keycode::{string_to_keycode, Keycode, Keycode::*};
use super::ngram::Ngram;

trait Frequencies {}
impl Frequencies for f32 {}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FrequenciesError {
	#[error("trying to add an ngram of length {0} to a holder with ngrams of length {1}, the ngram lengths must match")]
	NgramMatchError(usize, usize),
}

/// single as in only holds one length of n-gram
#[derive(Debug, PartialEq, Clone)]
pub struct SingleGramFrequencies<T> {
	frequencies: HashMap<Ngram, T>,
	n: usize,
}
impl<T>  SingleGramFrequencies<T> {
	fn new(n: usize) -> Self {
		Self { frequencies:  Default::default(), n: n }
	}
	fn keys(&self) -> Keys<'_, Ngram, T> {
		self.frequencies.keys()	
	}
	fn get(&self, k: &Ngram) -> Option<&T> {
		self.frequencies.get(k)
	}
}
/// u32 for raw ngram counts
impl  SingleGramFrequencies<u32> {
	fn increment(&mut self, ngram: Ngram) -> Result<(), FrequenciesError> {
		if self.n != ngram.clone().len() {
			Err(FrequenciesError::NgramMatchError(ngram.len(), self.n))
		} else {
			Ok(*self.frequencies.entry(ngram).or_insert(0) += 1)
		}
	}
	fn add_from_key_value(&mut self, key: Ngram, value: u32) -> Result<(), FrequenciesError> {
		if self.n != key.clone().len() {
			Err(FrequenciesError::NgramMatchError(key.len(), self.n))
		} else {
			Ok(*self.frequencies.entry(key).or_insert(0) += value)
		}
	}
	/// This might be faster if the bigger holder is on the left?
	fn combine_with(&mut self, holder: Self) -> () {
		for key in holder.keys() {
			self.add_from_key_value(key.clone(), *holder.get(key).unwrap());
		}
	}
	/// might want to rename this since try_from is from TryFrom
	fn try_from_string(s: &str, n: usize) -> Option<SingleGramFrequencies<u32>> {
		let mut ngram_to_counts: HashMap<Ngram, u32> = HashMap::new();
		let keycodes = string_to_keycode(s);
		if keycodes.len() < n {
			// this particular string was not long enough to create an N-gram out of
			return None;
		}
		for i in 0..(keycodes.len() - n + 1) {
			// should handle error here or change ngram to not error
			let ngram = Ngram::new(keycodes[i..i + n].to_vec());
			*ngram_to_counts.entry(ngram).or_insert(0) += 1;
		}
		Some(SingleGramFrequencies { frequencies: ngram_to_counts, n: n })
	}

	fn try_from_file<P>(&mut self, filename: P, n: usize) -> Result<(), io::Error> where P: AsRef<Path> {
		let file = File::open(filename)?;
		let lines = io::BufReader::new(file).lines();
		for line in lines.flatten() {
			if let Some(holder_from_line) = Self::try_from_string(line.as_str(), n)
			{
				self.combine_with(holder_from_line);
			}
		}
		Ok(())
	}
}
impl<T> Index<Ngram> for SingleGramFrequencies<T> {
	type Output = T;
	fn index(&self, ngram: Ngram) -> &Self::Output {
		&self.frequencies[&ngram]
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_frequency_holder_construction () {
		let ngram = Ngram::new(vec![_A, _B]);
		let mut freq_holder = SingleGramFrequencies::<u32>::new(2);
		freq_holder.increment(ngram.clone());
		assert_eq!(freq_holder[ngram.clone()], 1);
		freq_holder.increment(ngram.clone());
		assert_eq!(freq_holder[ngram.clone()], 2);

		freq_holder.add_from_key_value(ngram.clone(), 5);
		assert_eq!(freq_holder[ngram.clone()], 7);
		freq_holder.add_from_key_value(Ngram::new(vec![_C, _D]), 3);
		assert_eq!(freq_holder[Ngram::new(vec![_C, _D])], 3);

		let mut freq_holder2 = SingleGramFrequencies::<u32>::new(2);
		freq_holder2.add_from_key_value(Ngram::new(vec![_A, _E]), 2);
		freq_holder2.add_from_key_value(Ngram::new(vec![_C, _D]), 3);

		freq_holder.combine_with(freq_holder2);
		assert_eq!(freq_holder[Ngram::new(vec![_A, _B])], 7);
		assert_eq!(freq_holder[Ngram::new(vec![_C, _D])], 6);
		assert_eq!(freq_holder[Ngram::new(vec![_A, _E])], 2);
	}

	#[test]
	fn test_frequency_holder_ab() {
		let ngram = Ngram::new(vec![_A, _B]);
		let mut expected_ngram_to_counts: HashMap<Ngram, u32> = HashMap::new();
		expected_ngram_to_counts.insert(ngram, 1);
		assert_eq!(SingleGramFrequencies::try_from_string("ab", 2).unwrap(), SingleGramFrequencies { frequencies: expected_ngram_to_counts, n: 2 });
	}

	#[test]
	fn test_frequency_holder_abab() {
		let ngram = Ngram::new(vec![_A, _B]);
		// let map: HashMap<Ngram<2>, u32> = HashMap::new();
		// let holder = NgramFrequencyHolder { frequencies: map };
		let holder2 = SingleGramFrequencies::<u32>::try_from_string("abab", 2).unwrap();
		let holder2_ab = holder2[ngram];
		assert_eq!(holder2_ab, 2);
		let holder2_ba = holder2[Ngram::new(vec![_B, _A])];
		assert_eq!(holder2_ba, 1);
		let holder4 = SingleGramFrequencies::<u32>::try_from_string("abab", 4).unwrap();
		let holder4_val = holder4[Ngram::new(vec![_A, _B, _A, _B])];
		assert_eq!(holder4_val, 1)
	}

	#[test]
	fn test_read_from_file() {
		let mut holder = SingleGramFrequencies::<u32>::new(2);
		holder.try_from_file("./data/ch04-02-references-and-borrowing.md", 2);
		println!("{:?}", holder);
		// this value is found by control + F "he" and seeing how many matches there are
		assert_eq!(holder[Ngram::new(vec![_H, _E])], 145);
	}

}