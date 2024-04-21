use std::fs::read_dir;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::alc_error::AlcError;

use super::frequency_holder::{Frequencies, SingleGramFrequencies, TopFrequenciesToTake};
use super::keycode::KeycodeOptions;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FrequencyDatasetError {

}

// #[derive(Debug, PartialEq)]
// pub enum TextPresets {
// 	TheRustBook,
// }

type MultipleNgramFrequencies<T> = HashMap<usize, SingleGramFrequencies<T>>;
#[derive(Debug, PartialEq, Clone)]
pub struct FrequencyDataset<T> where T: Frequencies {
	pub name: String,
	pub ngram_frequencies: MultipleNgramFrequencies<T>,
}

impl FrequencyDataset<u32> {
	fn new(name: String, ngram_frequencies: MultipleNgramFrequencies<u32>) -> Self {
		Self {name, ngram_frequencies}
	}

	/// Because some ngrams are so infrequent and would only serve to increase computation time without affecting layout score very much, `top_n_to_take` allows you to choose how many of the most frequent ngrams you want to include.
	pub fn from_dir(dir: PathBuf, max_ngram_size: usize, top_frequencies_to_take: TopFrequenciesToTake, options: &KeycodeOptions) -> Result<Self, AlcError> {
		let metadata = dir.metadata().unwrap();
		if !metadata.is_dir() {
			Err(AlcError::ExpectedDirectoryError(dir))
		} else {
			// be a bit lazy for now and don't check for directories recursively
			let mut ngram_frequencies: HashMap<usize, SingleGramFrequencies<u32>> = Default::default();
			for n in 1..=max_ngram_size {
				ngram_frequencies.insert(n, SingleGramFrequencies::new(n));
			}
			for n in 1..=max_ngram_size {
				let files = read_dir(dir.clone()).unwrap();
				for file in files {
					let single_gram_frequencies = SingleGramFrequencies::<u32>::try_from_file(file.unwrap().path(), n, options)?;
					ngram_frequencies.get_mut(&n).unwrap().combine_with(single_gram_frequencies).unwrap(); // 
				}
				ngram_frequencies.get_mut(&n).unwrap().take_top_frequencies(top_frequencies_to_take.clone());
			}
			// do something about this
			let name = dir.file_name().unwrap().to_str().unwrap().to_string();
			Ok(FrequencyDataset::new(name, ngram_frequencies))
		}
	}

	pub fn get(&self, k: &usize) -> Option<&SingleGramFrequencies<u32>> {
		self.ngram_frequencies.get(k)
	}
}


#[cfg(test)]
mod tests {
	use crate::text_processor::keycode::Keycode::*;
	use crate::text_processor::ngram::Ngram;
	use super::*;
	use TopFrequenciesToTake::*;
	

	#[test]
	fn test_from_directory() {
		let frequency_dataset = FrequencyDataset::from_dir(PathBuf::try_from("./data/rust_book_test/").unwrap(), 4, All, &KeycodeOptions::default()).unwrap();
		let twogram_frequency = frequency_dataset.ngram_frequencies.get(&(2 as usize)).unwrap();
		assert_eq!(twogram_frequency[Ngram::new(vec![_H, _E])], 145 + 201);
		assert_eq!(twogram_frequency[Ngram::new(vec![_B, _E])], 34 + 23);
		let threegram_frequency = frequency_dataset.ngram_frequencies.get(&(3 as usize)).unwrap();
		// assert_eq!(threegram_frequency.len(), 1000);
		assert_eq!(threegram_frequency[Ngram::new(vec![_T, _H, _E])], 114 + 175);
		assert_eq!(threegram_frequency[Ngram::new(vec![_H, _E, _A])], 1 + 3);
	}
	

}