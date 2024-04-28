use std::{collections::HashMap, fs, option};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use toml;

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
			same_finger_penalty_factor: 5.0,
			extra_length_penalty: 1.1,
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
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
		let option_to_description = option_descriptions();
		let toml_string = toml::to_string(self).unwrap();
		let mut info_string = String::from("");
		for line in toml_string.lines() {
			match Regex::new(r"(?<option_name>.*) = (.*)") {
				Ok(v) => {
					match v.captures(line) {
						Some(v2) => {
							let option_name = &v2["option_name"];
							let option_description = match option_to_description.get(option_name) {
								Some(v) => v,
								None => {
									println!("Developer error: couldn't get description for {}", option_name);
									""
								},
							};
							info_string.push_str("# ");
							info_string.push_str(option_name);
							info_string.push_str(": ");
							info_string.push_str(option_description);
							info_string.push_str("\n");
						},
						None => {
							info_string.push_str("# ");
							info_string.push_str(line);
							info_string.push_str("\n");
						},
					}
					
				},
				Err(e) => panic!("{}", e),
			}
		}
		let toml_string_output = format!("{}\n# Option info (note: some descriptions may not be 100% accurate due to laziness / complexity, will update as necessary)\n{}", toml_string, info_string);
		Ok(toml_string_output)
	}
}

pub fn option_descriptions() -> HashMap<String, String> {
	let mut options_map: HashMap<String, String> = HashMap::default();
	options_map.insert(String::from("population_size"), String::from("Number of layouts per generation."));
	options_map.insert(String::from("generation_count"), String::from("Number of generations."));
	options_map.insert(String::from("fitness_cutoff"), String::from("Keep this proportion of best layouts per generation."));
	options_map.insert(String::from("swap_weight"), String::from("swap_weight:replace_weight represents the ratio of swap mutations (i.e., swapping two keys) to replace mutations (i.e., replacing one key with another). For example, 2:1 means 2/(2 + 1) of the mutations will be swaps and the remaining 1/(2 + 1) will be replaces."));
	options_map.insert(String::from("replace_weight"), String::from("See swap_weight."));
	options_map.insert(String::from("include_alphas"), String::from("Whether to include alphabet keycodes. Should generally be set to true."));
	options_map.insert(String::from("include_numbers"), String::from("Whether to include number keycodes. Recommended to set this to false and manually place numbers yourself since optimized layouts cannot currently guarantee numbers to be arranged in order."));
	options_map.insert(String::from("include_number_symbols"), String::from("Whether to include shifted numbers (!@#$ etc.). Recommended to set this to false for similar reasons as numbers. For specific symbols, such as ones common to programming languages, include them in `explicit_inclusionss`."));
	options_map.insert(String::from("include_brackets"), String::from("Whether to include ()[]{}<>. Recommended to set to false and manually place brackets yourself, as optimized layouts cannot guarantee corresponding brackets will appear next to each other."));
	options_map.insert(String::from("include_misc_symbols"), String::from("Whether to include -=\\;'`/[]. Recommended to set to true, as these are generally needed for typing."));
	options_map.insert(String::from("include_misc_symbols_shifted"), String::from("Whether to include shifted versions of misc. symbols, i.e., _+|:\"~?{}. Recommended to set to false and access through shift."));
	options_map.insert(String::from("explicit_inclusions"), String::from("Keycodes to explicitly include. If no combination of options cover exactly what you want, add them here."));
	options_map.insert(String::from("dataset_paths"), String::from("Path to directories containing files of text data. Currently only looks in the immediate directory and does not look recursively. Eventually will have presets."));
	options_map.insert(String::from("dataset_weights"), String::from("Ratio of datasets' importance. For example, with two datasets at a 2:1 ratio, the first dataset will constitute 2/(2 + 1) of the score and the second will constitute 1/(2 + 1)."));
	options_map.insert(String::from("max_ngram_size"), String::from("Maximum length of ngrams to extract from text."));
	options_map.insert(String::from("top_n_ngrams_to_take"), String::from("Number of most frequent ngrams to include. Some ngrams barely occur, thus having very little impact on overall score, so excluding them can decrease runtime. Applies to all ngrams. For example, if this value is 50, then we take the top 50 characters, top 50 bigrams, top 50 trigrams, etc."));
	options_map.insert(String::from("hand_alternation_weight"), String::from("hand_alteration_weight:finger_roll_weight represents the importance of hand alternating vs. finger rolls."));
	options_map.insert(String::from("hand_alternation_reduction_factor"), String::from("When a sequence of at least 3 keys alternates hands, multiply the effort of that sequence by this factor."));
	options_map.insert(String::from("finger_roll_weight"), String::from("See `hand_alteration_weight`."));
	options_map.insert(String::from("finger_roll_reduction_factor"), String::from("When a sequence of at least 3 keys is a finger roll, multiply the effort of that sequence by this factor. Sequential keys that cross two or more rows are not eligible for rolls. Inner and outer rolls are weighed the same (for now)."));
	options_map.insert(String::from("finger_roll_same_row_reduction_factor"), String::from("If a roll occurs where all fingers are in the same row, multiply the effort of that sequence by this factor, on top of the standard roll reduction factor. In other words, rolls where all keys are in the same row are extra favorable."));
	options_map.insert(String::from("same_finger_penalty_factor"), String::from("If the same finger (on the same hand, of course) is used twice in a row, multiply the effort by this factor. In other words, repeating the same finger is unfavorable."));
	options_map.insert(String::from("extra_length_penalty"), String::from("If the keycode sequence is longer than the ngram (e.g., from layer switches or shifting), apply this penalty for each additional key, exponentially."));
	options_map.insert(String::from("valid_keycodes"), String::from("Recommended to leave empty, as these will be generated from keycode options. If keycodes are supplied here, they will override keycode options; however, you can simply use the options + `explicit_inclusions` to fine tune the set you want, rather than having to list everything out here."));
	
	options_map
}


#[cfg(test)]
pub mod tests {
	
	use super::*;

	#[test]
	fn test_write() {
		let layout_optimizer_config = LayoutOptimizerConfig::default();
		let config_str = layout_optimizer_config.try_to_string().unwrap();
		println!("{}", config_str);
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
		// println!("{:?}", data.keycode_options.explicit_inclusions);

		let toml = toml::to_string(&KeycodeOptions::default()).unwrap();
		println!("{}", toml);
	}
}