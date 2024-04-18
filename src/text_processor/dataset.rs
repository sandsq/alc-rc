use std::fmt;
use std::fs::{metadata, read_dir};
use std::path::PathBuf;
use std::collections::HashMap;
use tqdm::tqdm;

use super::frequency_holder::{SingleGramFrequencies, Frequencies};
use super::keycode::Keycode::*;
use super::ngram::*;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FrequencyDatasetError {
	#[error("expected {0} to be a directory")]
	ExpectedDirectoryError(PathBuf)
}

// #[derive(Debug, PartialEq)]
// pub enum TextPresets {
// 	TheRustBook,
// }

type MultipleNgramFrequencies<T> = HashMap<usize, SingleGramFrequencies<T>>;
#[derive(Debug, PartialEq, Clone)]
pub struct FrequencyDataset<T> where T: Frequencies {
	name: String,
	ngram_frequencies: MultipleNgramFrequencies<T>,
}

impl FrequencyDataset<u32> {
	fn new(name: String, ngram_frequencies: MultipleNgramFrequencies<u32>) -> Self {
		Self {name, ngram_frequencies}
	}

	fn from_dir(dir: PathBuf, max_n: usize) -> Result<Self, FrequencyDatasetError> {
		let metadata = dir.metadata().unwrap();
		if !metadata.is_dir() {
			Err(FrequencyDatasetError::ExpectedDirectoryError(dir))
		} else {
			// be a bit lazy for now and don't check for directories recursively
			let mut ngram_frequencies: HashMap<usize, SingleGramFrequencies<u32>> = Default::default();
			for n in 1..=max_n {
				ngram_frequencies.insert(n, SingleGramFrequencies::new(n));
			}
			for n in tqdm(1..=max_n) {
				let files = read_dir(dir.clone()).unwrap();
				for file in tqdm(files) {
					let single_gram_frequencies = SingleGramFrequencies::<u32>::try_from_file(file.unwrap().path(), n).unwrap();
					ngram_frequencies.get_mut(&n).unwrap().combine_with(single_gram_frequencies);
				}
			}
			// do something about this
			let name = dir.file_name().unwrap().to_str().unwrap().to_string();
			Ok(FrequencyDataset::new(name, ngram_frequencies))
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_from_directory() {
		let frequency_dataset = FrequencyDataset::from_dir(PathBuf::try_from("./data/rust_book_test/").unwrap(), 4).unwrap();
		let twogram_frequency = frequency_dataset.ngram_frequencies.get(&(2 as usize)).unwrap();
		assert_eq!(twogram_frequency[Ngram::new(vec![_H, _E])], 145 + 201);
		assert_eq!(twogram_frequency[Ngram::new(vec![_B, _E])], 34 + 23);
		let threegram_frequency = frequency_dataset.ngram_frequencies.get(&(3 as usize)).unwrap();
		assert_eq!(threegram_frequency[Ngram::new(vec![_T, _H, _E])], 114 + 175);
		assert_eq!(threegram_frequency[Ngram::new(vec![_H, _E, _A])], 1 + 3);
	}
	

}