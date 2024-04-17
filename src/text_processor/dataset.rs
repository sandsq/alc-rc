use std::fmt;
use std::fs::{metadata, read_dir};
use std::path::PathBuf;
use std::collections::HashMap;

use super::frequency_holder::SingleGramFrequencies;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FrequencyDatasetError {
	#[error("expected {0} to be a directory")]
	ExpectedDirectoryError(PathBuf)
}

// #[derive(Debug, PartialEq)]
// pub enum TextPresets {
// 	TheRustBook,
// }

#[derive(Debug, PartialEq, Clone)]
pub struct FrequencyDataset<T> {
	name: String,
	ngram_frequencies: HashMap<usize, SingleGramFrequencies<T>>,
}

impl<T> FrequencyDataset<T> {
	fn from_dir(dir: PathBuf) -> Result<Self, FrequencyDatasetError> {
		let metadata = dir.metadata().unwrap();
		if !metadata.is_dir() {
			Err(FrequencyDatasetError::ExpectedDirectoryError(dir))
		} else {
			let name = dir.file_name();
			let files = read_dir(dir).unwrap();
			Ok(())
		}
	}
}