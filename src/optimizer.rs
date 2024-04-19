use std::cmp::max;
use std::fmt;
use std::iter::zip;
use rand::prelude::*;
use rand::Rng;
use tqdm::tqdm;
use std::path::PathBuf;
use rand::rngs::StdRng;


use crate::alc_error::AlcError;
use crate::keyboard::LayoutPosition;
use crate::keyboard::{key::*, layout::*, layer::*};
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;
use self::frequency_holder::{SingleGramFrequencies, TopFrequenciesToTake::*};
use self::keycode::{Keycode::{self, *}, get_default_keycode_set};

pub struct LayoutOptimizerConfig {
	population_size: u32,
	generation_count: u32,
	fitness_cutoff: f32, // keep top x% for the next generation
	swap_weight: f32,
	replace_weight: f32,
	dataset_weight: Vec<f32>, 
	valid_keycodes: Vec<Keycode>,
	top_n_ngrams_to_take: usize,
}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		LayoutOptimizerConfig { 
			population_size: 5, 
			generation_count: 1,
			fitness_cutoff: 0.1,
			swap_weight: 2.0,
			replace_weight: 1.0,
			dataset_weight: vec![1.0],
			valid_keycodes: get_default_keycode_set(),
			top_n_ngrams_to_take: 50, }
	}
}

pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	base_layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>,
	score_function: S,
	datasets: Vec<FrequencyDataset<u32>>,
	config: LayoutOptimizerConfig,
}
impl<const R: usize, const C: usize, S> LayoutOptimizer<R, C, S> where S: Score<R, C> {
	pub fn new(base_layout: Layout<R, C>, effort_layer: Layer<R, C, f32>, score_function: S, datasets: Vec<FrequencyDataset<u32>>, config: LayoutOptimizerConfig) -> Self {
		LayoutOptimizer { base_layout, effort_layer, score_function, datasets, config }
	}

	fn score_single_grams(&self, layout: &Layout<R, C>, frequencies: SingleGramFrequencies<u32>) -> f32 {
		let mut score = 0.0;
		let effort_layer = &self.effort_layer;
		for (ngram, ngram_frequency) in frequencies {
			let sequences = layout.ngram_to_sequences(&ngram).unwrap().into_iter();		
			let min_score = sequences
				.map(|s| self.score_function.score_layout_position_sequence(layout, effort_layer, s.clone(), &self.config))
				.min_by(|x, y| x.partial_cmp(y).unwrap())
				.unwrap();
			score += min_score * (ngram_frequency as f32);
		}
		score
	}

	fn score_dataset(&self, layout: &Layout<R, C>) -> f32 {
		let mut score = 0.0;
		let mut d_ind: usize = 0;
		for dataset in &self.datasets {
			let mut dataset_score = 0.0;
			for ngram_size in dataset.ngram_frequencies.keys() {
				dataset_score += self.score_single_grams(layout, dataset.ngram_frequencies.get(ngram_size).unwrap().clone());
			}
			dataset_score *= self.config.dataset_weight[d_ind];
			d_ind += 1;
			score += dataset_score;
		}
		score
	}

	fn generate_and_score_initial_population(&self, rng: &mut impl Rng) -> Vec<(Layout<R, C>, f32)> {
		let initial_population_size = self.config.population_size;
		let valid_keycodes = &self.config.valid_keycodes;
		// let mut initial_population: Vec<Layout<R, C>> = vec![];
		// let mut initial_scores: Vec<f32> = vec![];
		let mut initial_population: Vec<(Layout<R, C>, f32)> = Default::default();
		for i in tqdm(0..initial_population_size) {
			let mut initial_layout = self.base_layout.clone();
			initial_layout.randomize(rng, &valid_keycodes);
			let initial_score = self.score_dataset(&initial_layout);
			initial_population.push((initial_layout, initial_score));
		}
		initial_population
	}

	fn take_best_layouts(&self, mut population: Vec<(Layout<R, C>, f32)>) -> Vec<Layout<R, C>> {
    	population.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
		let num_to_take = (self.config.fitness_cutoff * (self.config.population_size as f32)).round() as usize;
		let (left, right): (Vec<Layout<R, C>>, Vec<f32>) = population.split_off(num_to_take).into_iter().unzip();
		left
	}

	fn generate_new_layouts(&self, rng: &mut impl Rng, mut layouts: Vec<Layout<R, C>>) -> Vec<Layout<R, C>> {
		let population_size = self.config.population_size;
		let swap_threshold = self.config.swap_weight / (self.config.swap_weight + self.config.replace_weight);
		let valid_keycodes = &self.config.valid_keycodes;
		while layouts.len() < (population_size as usize) {
			let mut new_layout = layouts.choose(rng).unwrap().clone();
			let roll: f32 = rng.gen();
			if roll <= swap_threshold {
				let (p1, p2) = match new_layout.gen_valid_swap(rng) {
					Some((x, y)) => (x, y),
					None => (LayoutPosition::for_layout(0, 0, 0), LayoutPosition::for_layout(0, 0, 0)), // swapping the same position doesn't change the layout
				};
				new_layout.swap(&p1, &p2);
			} else {
				// replace
			}
			layouts.push(new_layout);
		}

		layouts
	}

	pub fn optimize(&self, rng: &mut impl Rng, config: LayoutOptimizerConfig) -> Result<Layout<R, C>, AlcError> {
		if self.datasets.len() != config.dataset_weight.len() {
			return Err(AlcError::DatasetWeightsMismatchError(self.datasets.len(), config.dataset_weight.len()));
		}
		let initial_layouts = self.generate_and_score_initial_population(rng);

		
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
		let config = LayoutOptimizerConfig::default();
		LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset], config)
	}
}


#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_ngram_scoring() {
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
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/small_test/"), 2, All).unwrap();
		let config = LayoutOptimizerConfig::default();
		let layout_optimizer = LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset], config);
		let twogram_frequency = layout_optimizer.datasets[0].ngram_frequencies.get(&(2 as usize)).unwrap();
		let s = layout_optimizer.score_single_grams(&test_layout, twogram_frequency.clone());
		// 3 * he + 1 * be + 2 * eh + 1 + eb
		let expected_two_score = 3.0 * (0.1 + 0.2 + 0.1) + 1.0 * (0.3 + 0.2 + 0.1) + 2.0 * (0.2 + 0.1 + 0.1) + 1.0 * (0.2 + 0.1 + 0.3);
		assert_eq!(format!("{s:.3}"), format!("{expected_two_score:.3}"));
		
		let full_score = layout_optimizer.score_dataset(&test_layout);
		let expected_one_score = 3.0 * 0.1 + 4.0 * (0.2 + 0.1) + 1.0 * 0.3;
		let expected_score = expected_one_score + expected_two_score;
		assert_eq!(format!("{full_score:.3}"), format!("{expected_score:.3}"));
		// let mut rng = StdRng::seed_from_u64(0);
		// layout_optimizer.optimize(&mut rng, config);
	}
}
