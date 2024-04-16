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

#[derive(Debug, PartialEq)]
pub struct NgramFrequencyHolder<const N: usize, T> {
	frequencies: HashMap<Ngram<N>, T>
}
impl<const N: usize, T>  NgramFrequencyHolder<N, T> {
	fn new() -> Self {
		Self { frequencies:  Default::default() }
	}
	fn keys(&self) -> Keys<'_, Ngram<N>, T> {
		self.frequencies.keys()	
	}
	fn get(&self, k: &Ngram<N>) -> Option<&T> {
		self.frequencies.get(k)
	}
}
impl<const N: usize>  NgramFrequencyHolder<N, u32> {
	fn increment(&mut self, ngram: Ngram<N>) -> () {
		*self.frequencies.entry(ngram).or_insert(0) += 1
	}
	fn add_from_key_value(&mut self, key: Ngram<N>, value: u32) -> () {
		*self.frequencies.entry(key).or_insert(0) += value
	}
	/// This might be faster if the bigger holder is on the left?
	fn combine_with(&mut self, holder: Self) -> () {
		for key in holder.keys() {
			self.add_from_key_value(key.clone(), *holder.get(key).unwrap());
		}
	}
	fn try_from_file<P>(&mut self, filename: P) -> Result<(), io::Error> where P: AsRef<Path> {
		let file = File::open(filename)?;
		let lines = io::BufReader::new(file).lines();
		for line in lines.flatten() {
			if let Some(holder_from_line) = Self::try_from(line.as_str())
			{
				self.combine_with(holder_from_line);
			}
		}
		Ok(())
	}

	/// might want to rename this since try_from is from TryFrom
	fn try_from(s: &str) -> Option<NgramFrequencyHolder<N, u32>> {
		let mut ngram_to_counts: HashMap<Ngram<N>, u32> = HashMap::new();
		let keycodes = string_to_keycode(s);
		if keycodes.len() < N {
			// this particular string was not long enough to create an N-gram out of
			return None;
		}
		for i in 0..(keycodes.len() - N + 1) {
			// should handle error here or change ngram to not error
			let ngram = Ngram::<N>::try_from(&keycodes[i..i + N]).unwrap();
			*ngram_to_counts.entry(ngram).or_insert(0) += 1;
		}
		Some(NgramFrequencyHolder { frequencies: ngram_to_counts })
	}
}
impl<const N: usize, T> Index<Ngram<N>> for NgramFrequencyHolder<N, T> {
	type Output = T;
	fn index(&self, ngram: Ngram<N>) -> &Self::Output {
		&self.frequencies[&ngram]
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_frequency_holder_construction () {
		let ngram = Ngram::new([_A, _B]);
		let mut freq_holder = NgramFrequencyHolder::<2, u32>::new();
		freq_holder.increment(ngram.clone());
		assert_eq!(freq_holder[ngram.clone()], 1);
		freq_holder.increment(ngram.clone());
		assert_eq!(freq_holder[ngram.clone()], 2);

		freq_holder.add_from_key_value(ngram.clone(), 5);
		assert_eq!(freq_holder[ngram.clone()], 7);
		freq_holder.add_from_key_value(Ngram::new([_C, _D]), 3);
		assert_eq!(freq_holder[Ngram::new([_C, _D])], 3);

		let mut freq_holder2 = NgramFrequencyHolder::<2, u32>::new();
		freq_holder2.add_from_key_value(Ngram::new([_A, _E]), 2);
		freq_holder2.add_from_key_value(Ngram::new([_C, _D]), 3);

		freq_holder.combine_with(freq_holder2);
		assert_eq!(freq_holder[Ngram::new([_A, _B])], 7);
		assert_eq!(freq_holder[Ngram::new([_C, _D])], 6);
		assert_eq!(freq_holder[Ngram::new([_A, _E])], 2);
	}

	#[test]
	fn test_frequency_holder_ab() {
		let ngram = Ngram::new([_A, _B]);
		let mut expected_ngram_to_counts: HashMap<Ngram<2>, u32> = HashMap::new();
		expected_ngram_to_counts.insert(ngram, 1);
		assert_eq!(NgramFrequencyHolder::try_from("ab").unwrap(), NgramFrequencyHolder { frequencies: expected_ngram_to_counts });
	}

	#[test]
	fn test_frequency_holder_abab() {
		let ngram = Ngram::new([_A, _B]);
		// let map: HashMap<Ngram<2>, u32> = HashMap::new();
		// let holder = NgramFrequencyHolder { frequencies: map };
		let holder2 = NgramFrequencyHolder::<2, u32>::try_from("abab").unwrap();
		let holder2_ab = holder2[ngram];
		assert_eq!(holder2_ab, 2);
		let holder2_ba = holder2[Ngram::new([_B, _A])];
		assert_eq!(holder2_ba, 1);
		let holder4 = NgramFrequencyHolder::<4, u32>::try_from("abab").unwrap();
		let holder4_val = holder4[Ngram::new([_A, _B, _A, _B])];
		assert_eq!(holder4_val, 1)
	}

	#[test]
	fn test_read_from_file() {
		let mut holder = NgramFrequencyHolder::<2, u32>::new();
		holder.try_from_file("./data/ch04-02-references-and-borrowing.md");
		println!("{:?}", holder);
		// this value is found by control + F "he" and seeing how many matches there are
		assert_eq!(holder[Ngram::new([_H, _E])], 145);
	}
}