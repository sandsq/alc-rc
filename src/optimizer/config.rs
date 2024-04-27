use std::fs;
use serde_derive::{Deserialize, Serialize};
use toml;

use super::keycode::{generate_default_keycode_set, Keycode, KeycodeOptions};

#[derive(Debug, Deserialize)]
struct Data {
	keycode_options: KeycodeOptions,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOptimizerConfig {
	// make sure constructor puts limits on fields
	pub population_size: u32,
	pub generation_count: u32,
	pub fitness_cutoff: f64, // keep top x% for the next generation
	pub swap_weight: f64,
	pub replace_weight: f64,
	pub dataset_paths: Vec<String>,
	pub dataset_weight: Vec<f64>,
	pub keycode_options: KeycodeOptions,
	pub valid_keycodes: Vec<Keycode>,
	pub max_ngram_size: usize,
	pub top_n_ngrams_to_take: usize,
	pub hand_alternation_weight: f64, // determines the relative weight of hand alternation bonus vs finger roll bonus. 
	pub hand_alternation_reduction_factor: f64, // say this is 0.9. Then a hand alternation of left-right-left would reduce the effort of that sequence by 0.9 * 0.9x. Min length 3.
	pub finger_roll_weight: f64,
	pub finger_roll_reduction_factor: f64, // say this is 0.9. Then a roll of length 3 would reduce the effort by 0.9 * 0.9x. Min length 3.
	pub finger_roll_same_row_reduction_factor: f64,
	pub same_finger_penalty_factor: f64,
	pub extra_length_penalty: f64,

}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		let keycode_options = KeycodeOptions::default();
		let mut valid_keycodes = generate_default_keycode_set(&keycode_options).into_iter().collect::<Vec<Keycode>>();
		valid_keycodes.sort_unstable();
		LayoutOptimizerConfig { 
			population_size: 5, 
			generation_count: 1,
			fitness_cutoff: 0.1,
			swap_weight: 4.0,
			replace_weight: 1.0,
			dataset_weight: vec![1.0],
			dataset_paths: vec![String::from("./data/rust_book_test/")],
			keycode_options: keycode_options.clone(),
			valid_keycodes: valid_keycodes,
			max_ngram_size: 4,
			top_n_ngrams_to_take: 100,
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


#[cfg(test)]
pub mod tests {
	
	use super::*;

	#[test]
	fn test_read() {
		let test_file = "./templates/ferris_sweep.toml";
		let contents = match fs::read_to_string(test_file) {
			Ok(c) => c,
			Err(_) => {
				panic!("could not read file {}", test_file)
			}
		};
		let data: Data = match toml::from_str(&contents) {
			Ok(d) => d,
			Err(e) => {
				panic!("{}", e)
			}
		};
	
		println!("{:?}", data);
		println!("{:?}", data.keycode_options.explicit_inclusion)
	}
}