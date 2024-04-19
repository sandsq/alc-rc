use std::cmp::max;
use std::fmt;
use rand::prelude::*;
use rand::Rng;
use tqdm::tqdm;
use std::path::PathBuf;
use rand::rngs::StdRng;


use crate::keyboard::{key::*, layout::*, layer::*};
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;
use self::frequency_holder::{SingleGramFrequencies, TopFrequenciesToTake::*};
use self::keycode::{Keycode::{self, *}, get_default_keycode_set};

pub struct LayoutOptimizerConfig {
	initial_population_size: u32,
	generation_count: u32,
	dataset_weight: Vec<u8>,
	valid_keycodes: Vec<Keycode>,
	top_n_ngrams_to_take: usize,
}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		LayoutOptimizerConfig { 
			initial_population_size: 5, 
			generation_count: 1, 
			dataset_weight: vec![1],
			valid_keycodes: get_default_keycode_set(),
			top_n_ngrams_to_take: 50, }
	}
}

pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	base_layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>,
	score_function: S,
	datasets: Vec<FrequencyDataset<u32>>,
}
impl<const R: usize, const C: usize, S> LayoutOptimizer<R, C, S> where S: Score<R, C> {
	pub fn new(base_layout: Layout<R, C>, effort_layer: Layer<R, C, f32>, score_function: S, datasets: Vec<FrequencyDataset<u32>>) -> Self {
		LayoutOptimizer { base_layout, effort_layer, score_function, datasets }
	}

	fn score_single_grams(&self, layout: &Layout<R, C>, frequencies: SingleGramFrequencies<u32>, config: LayoutOptimizerConfig) -> f32 {
		let mut score = 0.0;
		let effort_layer = &self.effort_layer;
		for (ngram, ngram_frequency) in frequencies {
			let sequences = layout.ngram_to_sequences(&ngram).unwrap().into_iter();		
			let min_score = sequences
				.map(|s| self.score_function.score_layout_position_sequence(layout, effort_layer, s.clone()))
				.min_by(|x, y| x.partial_cmp(y).unwrap())
				.unwrap();
			score += min_score * (ngram_frequency as f32);
		}
		score
	}

	fn score_dataset(&self, config: LayoutOptimizerConfig) -> f32 {
		let mut score = 0.0;
		for dataset in &self.datasets {

		}

		score
	}

	fn generate_and_score_initial_population(&self, rng: &mut impl Rng, config: LayoutOptimizerConfig) -> Vec<Layout<R, C>> {
		let initial_population_size = config.initial_population_size;
		let valid_keycodes = config.valid_keycodes;
		let mut initial_population: Vec<Layout<R, C>> = vec![];
		let mut scores: Vec<f32> = vec![];
		for i in tqdm(0..initial_population_size) {
			let mut initial_layout = self.base_layout.clone();
			initial_layout.randomize(rng, &valid_keycodes);
			println!("{}", initial_layout);
			initial_population.push(initial_layout);	
		}

		initial_population
	}

	pub fn optimize(&self, rng: &mut impl Rng, config: LayoutOptimizerConfig) -> Layout<R, C> {
		

		
		// symmetry check
		// layer reachability check
		// other sanity checks
		todo!()
	}
}
impl Default for LayoutOptimizer<2, 4, SimpleScoreFunction> {
	fn default() -> Self {
		let base_layout = Layout::<2, 4>::init_blank(2);
		let effort_layer = Layer::<2, 4, f32>::try_from("
			0.1 0.2 0.3 0.4
			0.5 0.6 0.7 0.8
		").unwrap();
		let score_function = SimpleScoreFunction{};
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/rust_book_test/"), 4, Num(50)).unwrap();
		LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset])
	}
}


#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_single_ngram_scoring() {
		let base_layout = Layout::<1, 4>::init_blank(2);
		let effort_layer = Layer::<1, 4, f32>::try_from("
			0.1 0.4 0.3 0.2
		").unwrap();
		let test_layout = Layout::<1, 4>::try_from("
			___Layer 0___
			H_10 E_10 B_10 LS1_10
			___Layer 1___
			E_10 A_10 C_10 LS1_10
		
		").unwrap();
		let score_function = SimpleScoreFunction{};
		let text = "hehehebe";
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/small_test/"), 4, All).unwrap();
		let layout_optimizer = LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset]);
		let config = LayoutOptimizerConfig::default();
		let twogram_frequency = layout_optimizer.datasets[0].ngram_frequencies.get(&(2 as usize)).unwrap();
		let s = layout_optimizer.score_single_grams(&test_layout, twogram_frequency.clone(), config);
		// 3 * he + 1 * be + 2 * eh + 1 + eb
		let expected_s = 3.0 * (0.1 + 0.2 + 0.1) + 1.0 * (0.3 + 0.2 + 0.1) + 2.0 * (0.2 + 0.1 + 0.1) + 1.0 * (0.2 + 0.1 + 0.3);
		assert_eq!(format!("{s:.3}"), format!("{expected_s:.3}"));
		
		// let mut rng = StdRng::seed_from_u64(0);
		// layout_optimizer.optimize(&mut rng, config);
	}
}
