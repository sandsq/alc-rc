use std::{collections::HashMap, fs};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use toml;

use crate::alc_error::AlcError;
use super::{keycode::{Keycode, KeycodeOptions}, LayoutOptimizer, Score};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LayoutOptimizerConfig {
	// make sure constructor puts limits on fields
	pub genetic_options: GeneticOptions,
	pub keycode_options: KeycodeOptions,
	pub valid_keycodes: Vec<Keycode>,
	pub dataset_options: DatasetOptions,
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
			valid_keycodes: vec![],
			dataset_options: DatasetOptions::default(),
			score_options: ScoreOptions::default(),
		 }
	}
}


/// SerDe isn't implemented for Layout / Layer, so adapting those structs from strings for now
/// don't create this directly, as it only serves to translate the actual layout / layer stucts to toml.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutInfoTomlAdapter {
	pub num_rows: usize,
	pub num_cols: usize,
	pub layout: String,
	pub effort_layer: String,
	pub phalanx_layer: String,
}


/// SerDe isn't implemented for Layout / Layer, so adapting those structs from strings for now
/// don't create this directly, as it only serves to translate the actual layout / layer stucts to toml. Instead, only create it from LayoutOptimizer
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutOptimizerTomlAdapter {
	pub layout_info: LayoutInfoTomlAdapter,
	pub layout_optimizer_config: LayoutOptimizerConfig,
}
impl LayoutOptimizerTomlAdapter {
	pub fn try_from_toml_file(filename: &str) -> Result<Self, AlcError> {
		let contents = match fs::read_to_string(filename) {
			Ok(c) => c,
			Err(_) => {
				panic!("could not read file {}", filename)
			}
		};
		let optimizer_object: LayoutOptimizerTomlAdapter = toml::from_str(&contents)?;
		Ok(optimizer_object)
	}

	pub fn try_to_toml_string(&self) -> Result<String, AlcError> {
		let option_to_description = option_descriptions();
		let toml_string = toml::to_string(&self).unwrap();
		
		// let mut available_layouts = String::from("");
		// for (i, preset_name) in LayoutPreset::iter().enumerate() {
		// 	available_layouts.push_str(&preset_name.to_string());
		// 	if i < LayoutPreset::iter().len() - 1 {
		// 		available_layouts.push_str(", ")
		// 	}
		// }

		let mut comments_string = String::from("");
		
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
							comments_string.push_str("# ");
							comments_string.push_str(option_name);
							comments_string.push_str(": ");
							comments_string.push_str(option_description);
							comments_string.push_str("\n");
						},
						None => {
							if line.starts_with("[") {
								comments_string.push_str("\n");
								if line.trim().len() > 0 {
									comments_string.push_str("# ");
								}
								comments_string.push_str(line);
								comments_string.push_str("\n");
							}
						},
					}
					
				},
				Err(e) => panic!("{}", e),
			}
		}
		let output_string = format!("# See ending comments for field information.\n{}\n# [Autogenerated]\n# Option info (note: some descriptions may not be totally accurate due to complexity, but the general idea should be present.)\n{}", toml_string, comments_string);
		Ok(output_string)
	}

	pub fn write_to_file(&self, filename: &str) -> Result<(), AlcError> {
		fs::write(filename, self.try_to_toml_string()?).expect(format!("Unable to write file {}", filename).as_str());
		Ok(())
	}

	pub fn try_from_layout_optimizer<const R: usize, const C: usize, S>(lo: &LayoutOptimizer<R, C, S>) -> Self where S: Score<R, C> + Send + Sync {
		let base_layout_string = format!("{:b}", lo.base_layout);
		let effort_layer_string = format!("{}", lo.effort_layer);
		let phalanx_layer_string = format!("{}", lo.phalanx_layer);

		let layout_info = LayoutInfoTomlAdapter {
			num_rows: R,
			num_cols: C,
			layout: base_layout_string,
			effort_layer: effort_layer_string,
			phalanx_layer: phalanx_layer_string,
		};

		LayoutOptimizerTomlAdapter {
			layout_optimizer_config: lo.config.clone(),
			layout_info,
		}
	}

}



pub fn prettify_layer_string(s: String) -> String {
	let mut output_str = String::from("");
	for line in s.lines() {
		if line.trim().len() == 0 {
			continue;
		}
		let newline = &line.replace("\\t", "").replace("\t", "");
		output_str.push_str(newline);
		output_str.push_str("\n");
	}
	output_str
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
	options_map.insert(String::from("explicit_inclusions"), String::from("Keycodes to explicitly include. If no combination of options covers exactly what you want, add them here."));
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
	options_map.insert(String::from("num_rows"), String::from("Number of rows in the layout. Note that some row x column combinations may not exist, in which case use the next size up and block key positions as necessary. Available sizes should be listed here at some point: "));
	options_map.insert(String::from("num_cols"), String::from("Number of columns in the layout."));
	options_map.insert(String::from("layout"), String::from("Collection of layers. Each key is of the format {{keycode}}_{{moveability flag}}{{symmetry flag}}. Keycode reference should be available here: . Moveability of 1 means the optimizer can change the key in the given position; otherwise, the key will be fixed. Symmetry of 1 means it and its corresponding symmetric key will be locked in symmetry -- if one moves, the other will be moved to the corresponding symmetric location."));
	options_map.insert(String::from("effort_layer"), String::from("Specify the relative effort required to reach each key position. Smaller number means lower effort. Recommended to make the most accessible keys a weight of 1 and scale other keys accordingly. Does require some tinkering to create a grid that works for you."));
	options_map.insert(String::from("phalanx_layer"), String::from("Specify which hand and finger you want to use for each key. Used in calculating hand alternation bonuses, roll bonuses, and same finger penalties. Format is {{hand}}:{{finger}}, with hand options (L)eft and (R)ight and finger options (T)humb, (I)ndex, (M)iddle, (R)ing, (P)inkie, and (J)oint. Joint refers to where your pinkie meets your palm; some users use this part of their hand to hit the bottom left- / bottom right-most keys."));

	options_map
}


#[cfg(test)]
pub mod tests {
	
	use crate::optimizer::AdvancedScoreFunction;

use super::*;

	#[test]
	fn test_read_write() {
		let mut lo: LayoutOptimizer<4, 12, AdvancedScoreFunction> = LayoutOptimizer::default();
		lo.config.genetic_options.generation_count = 100;
		lo.config.genetic_options.population_size = 200;
		let optimizer_toml_object = LayoutOptimizerTomlAdapter::try_from_layout_optimizer(&lo);
		// let optimizer_toml_string = optimizer_toml_object.try_to_toml_string().unwrap();
		// println!("{}", optimizer_toml_string);
		optimizer_toml_object.write_to_file("./templates/test.toml").unwrap();

		let optimizer_toml_object_from_file = LayoutOptimizerTomlAdapter::try_from_toml_file("./templates/test.toml").unwrap();
		assert_eq!(optimizer_toml_object, optimizer_toml_object_from_file);
		assert_eq!(optimizer_toml_object_from_file.layout_optimizer_config.genetic_options.generation_count, 100);
		assert_eq!(optimizer_toml_object_from_file.layout_optimizer_config.genetic_options.population_size, 200);

		let lo_from_toml: LayoutOptimizer<4, 12, AdvancedScoreFunction> = LayoutOptimizer::try_from_optimizer_toml_file("./templates/test.toml").unwrap();
		assert_eq!(lo, lo_from_toml);

		// let optimizer_toml_object_from_file = LayoutOptimizerTomlAdapter::try_from_toml_file("./templates/ferris_sweep.toml").unwrap();
		let optimizer: LayoutOptimizer<4, 10, AdvancedScoreFunction> = LayoutOptimizer::try_from_optimizer_toml_file("./templates/ferris_sweep.toml").unwrap();
		optimizer.write_to_toml("./templates/ferris_sweep.toml").unwrap();
		
	}
	
}