use std::fs;
use serde_derive::{Deserialize, Serialize};
use toml;
use struct_iterable::Iterable;

use crate::{alc_error::AlcError, keyboard::layout};
use super::{keycode::{generate_default_keycode_set, Keycode, KeycodeOptions}, Score};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct GeneticOptions {
	pub population_size: u32,
	pub generation_count: u32,
	pub fitness_cutoff: f64, // keep top x% for the next generation
	pub swap_weight: f64,
	pub replace_weight: f64,
}
impl Default for GeneticOptions {
	fn default() -> Self {
		GeneticOptions {
			population_size: 5, 
			generation_count: 1,
			fitness_cutoff: 0.1,
			swap_weight: 4.0,
			replace_weight: 1.0,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetOptions {
	pub dataset_paths: Vec<String>,
	pub dataset_weights: Vec<f64>,
	pub max_ngram_size: usize,
	pub top_n_ngrams_to_take: usize,
}
impl Default for DatasetOptions {
	fn default() -> Self {
		DatasetOptions {
			dataset_weights: vec![1.0],
			dataset_paths: vec![String::from("./data/rust_book_test/")],
			max_ngram_size: 4,
			top_n_ngrams_to_take: 100,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreOptions {
	pub hand_alternation_weight: f64, // determines the relative weight of hand alternation bonus vs finger roll bonus. 
	pub hand_alternation_reduction_factor: f64, // say this is 0.9. Then a hand alternation of left-right-left would reduce the effort of that sequence by 0.9 * 0.9x. Min length 3.
	pub finger_roll_weight: f64,
	pub finger_roll_reduction_factor: f64, // say this is 0.9. Then a roll of length 3 would reduce the effort by 0.9 * 0.9x. Min length 3.
	pub finger_roll_same_row_reduction_factor: f64,
	pub same_finger_penalty_factor: f64,
	pub extra_length_penalty: f64,
}
impl Default for ScoreOptions {
	fn default() -> Self {
		ScoreOptions {
			hand_alternation_weight: 3.0,
			hand_alternation_reduction_factor: 0.9,
			finger_roll_weight: 2.0,
			finger_roll_reduction_factor: 0.9,
			finger_roll_same_row_reduction_factor: 0.9,
			same_finger_penalty_factor: 3.0,
			extra_length_penalty: 1.1,
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize, struct_iterable::Iterable)]
#[serde(default)]
pub struct LayoutOptimizerConfig {
	// make sure constructor puts limits on fields
	pub genetic_options: GeneticOptions,
	pub keycode_options: KeycodeOptions,
	pub dataset_options: DatasetOptions,
	pub valid_keycodes: Vec<Keycode>,
	pub score_options: ScoreOptions,

}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		let keycode_options = KeycodeOptions::default();
		// let mut valid_keycodes = generate_default_keycode_set(&keycode_options).into_iter().collect::<Vec<Keycode>>();
		// valid_keycodes.sort_unstable();
		LayoutOptimizerConfig { 
			genetic_options: GeneticOptions::default(),
			keycode_options: keycode_options.clone(),
			dataset_options: DatasetOptions::default(),
			valid_keycodes: vec![],
			score_options: ScoreOptions::default(),
		 }
	}
}
impl LayoutOptimizerConfig {
	pub fn try_from_file(filename: String) -> Result<Self, AlcError> {
		let contents = match fs::read_to_string(filename.clone()) {
			Ok(c) => c,
			Err(_) => {
				panic!("could not read file {}", filename)
			}
		};
		let layout_optimizer_config: LayoutOptimizerConfig = toml::from_str(&contents)?;
		Ok(layout_optimizer_config)
	}

	pub fn try_to_string(&self) -> Result<String, AlcError> {
		for (field, value) in self.iter() {
			println!("{}: {:?}", field, value);
		}

		let toml = toml::to_string(self).unwrap();
		println!("{}", toml);
		Ok(toml)
	}
}


#[cfg(test)]
pub mod tests {
	
	use super::*;

	#[test]
	fn test_write() {
		let genetic_options = GeneticOptions::default();
		let toml = toml::to_string(&genetic_options).unwrap();
		println!("{}", toml);

		let layout_optimizer_config = LayoutOptimizerConfig::default();
		layout_optimizer_config.try_to_string().unwrap();
	}

	#[test]
	fn test_read() {
		let test_file = "./templates/ferris_sweep.toml";
		let contents = match fs::read_to_string(test_file) {
			Ok(c) => c,
			Err(_) => {
				panic!("could not read file {}", test_file)
			}
		};
		let data: LayoutOptimizerConfig = match toml::from_str(&contents) {
			Ok(d) => d,
			Err(e) => {
				panic!("{}", e)
			}
		};
	
		println!("{:?}", data);
		println!();
		// println!("{:?}", data.keycode_options.explicit_inclusion);

		let toml = toml::to_string(&KeycodeOptions::default()).unwrap();
		println!("{}", toml);
	}
}