use std::cell::Cell;
use std::cmp::max;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::iter::zip;
use rand::prelude::*;
use rand::Rng;
use tqdm::tqdm;
use std::path::PathBuf;
use rand::rngs::StdRng;


use crate::alc_error::AlcError;
use crate::keyboard::LayoutPosition;
use crate::keyboard::LayoutPositionSequence;
use crate::keyboard::{key::*, layout::*, layer::*};
use crate::optimizer::ngram::Ngram;
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;
use self::frequency_holder::{SingleGramFrequencies, TopFrequenciesToTake::*};
use self::keycode::KeycodeOptions;
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
			swap_weight: 9.0,
			replace_weight: 1.0,
			dataset_weight: vec![1.0],
			valid_keycodes: get_default_keycode_set(&KeycodeOptions::default()).into_iter().collect(),
			top_n_ngrams_to_take: 50, }
	}
}

pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	base_layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>,
	score_function: S,
	datasets: Vec<FrequencyDataset<u32>>,
	config: LayoutOptimizerConfig,
	operation_counter: Cell<(u32, u32, u32)>, // swaps, replacements, nothings
}
impl<const R: usize, const C: usize, S> LayoutOptimizer<R, C, S> where S: Score<R, C> {
	pub fn new(base_layout: Layout<R, C>, effort_layer: Layer<R, C, f32>, score_function: S, datasets: Vec<FrequencyDataset<u32>>, config: LayoutOptimizerConfig, operation_counter: Cell<(u32, u32, u32)>) -> Self {
		LayoutOptimizer { base_layout, effort_layer, score_function, datasets, config, operation_counter }
	}

	fn score_single_grams(&self, layout: &Layout<R, C>, frequencies: SingleGramFrequencies<u32>) -> f32 {
		let mut score = 0.0;
		let effort_layer = &self.effort_layer;
		for (ngram, ngram_frequency) in frequencies {
			let sequences = match layout.ngram_to_sequences(&ngram) {
				Some(v) => v.into_iter(),
				None => return 0.0, //panic!("unable to create sequence from {}", ngram),
			};
			let mut possible_scores: Vec<f32> = vec![];
			for sequence in sequences {
				// let s2 = sequence.clone();
				let sequence_score = self.score_function.score_layout_position_sequence(layout, effort_layer, sequence, &self.config);
				possible_scores.push(sequence_score);
				// if ngram == Ngram::new(vec![_T, _H, _E]) {
				// 	println!("ngram {} with sequence {} and score {}", ngram, s2, sequence_score);
				// }
			}
			
			// println!("possible scores {:?}", possible_scores);
			let min_score = match possible_scores.iter().min_by(|a, b| a.total_cmp(b)) {
				Some(v) => v,
				None => &0.0,
			};
				
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


	fn score_population(&self, layouts: &Vec<Layout<R, C>>) -> Vec<(Layout<R, C>, f32)> {
		let mut new_population: Vec<(Layout<R, C>, f32)> = Default::default();
		for layout in layouts {
			let score = self.score_dataset(&layout);
			new_population.push((layout.clone(), score));
		}
		new_population
	}
	

	fn generate_and_score_initial_population(&self, rng: &mut impl Rng) -> Vec<(Layout<R, C>, f32)> {
		let initial_population_size = self.config.population_size;
		let valid_keycodes = &self.config.valid_keycodes;
		let mut initial_population: Vec<(Layout<R, C>, f32)> = Default::default();
		for i in 0..self.config.population_size {
			let mut initial_layout = self.base_layout.clone();
			initial_layout.randomize(rng, valid_keycodes);
			let initial_score = self.score_dataset(&initial_layout);
			initial_population.push((initial_layout.clone(), initial_score));
		}
		initial_population
	}

	fn take_best_layouts(&self, mut population: Vec<(Layout<R, C>, f32)>) -> (Vec<Layout<R, C>>, Vec<f32>) {
    	population.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
		let num_to_take = (self.config.fitness_cutoff * (self.config.population_size as f32)).ceil() as usize;
		population.split_off(num_to_take);
		let (left, right): (Vec<Layout<R, C>>, Vec<f32>) =  population.into_iter().unzip();
		// println!("scores {:?}", right);
		(left, right)
	}

	fn generate_new_layouts(&self, rng: &mut impl Rng, mut layouts: Vec<Layout<R, C>>) -> Vec<Layout<R, C>> {
		let population_size = self.config.population_size;
		let swap_threshold = self.config.swap_weight / (self.config.swap_weight + self.config.replace_weight);
		let valid_keycodes = &self.config.valid_keycodes;
		// modify surviving layouts
		for mut layout in &mut layouts {
			let roll: f32 = rng.gen();
			if roll <= swap_threshold {
				let (p1, p2) = match layout.gen_valid_swap(rng) {
					Some((x, y)) => (x, y),
					// None => panic!("no swap found"),
					None => (LayoutPosition::for_layout(0, 0, 0), LayoutPosition::for_layout(0, 0, 0)), // swapping the same position doesn't change the layout
				};
				// println!("swapping {} and {}", p1, p2);
				let swap_happened = layout.swap(&p1, &p2);
				let op_counter = self.operation_counter.get();
				if swap_happened {
					self.operation_counter.set((op_counter.0 + 1, op_counter.1, op_counter.2));
				} else {
					self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
				}
			} else {
				if let Some(p) = layout.gen_valid_replace(rng) {
					// println!("valid keycodes {:?}", valid_keycodes);
					let keycode = valid_keycodes.choose(rng).unwrap();
					let replace_happened = layout.replace(&p, *keycode);
					let op_counter = self.operation_counter.get();
					if replace_happened {
						self.operation_counter.set((op_counter.0, op_counter.1 + 1, op_counter.2));
					} else {
						self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
					}
				}
			}
		}
		// fill up to population size
		while layouts.len() < (population_size as usize) {
			let mut new_layout = layouts.choose(rng).unwrap().clone();
			let roll: f32 = rng.gen();
			// println!("roll {} vs swap threshold {}", roll, swap_threshold);
			if roll <= swap_threshold {
				let (p1, p2) = match new_layout.gen_valid_swap(rng) {
					Some((x, y)) => (x, y),
					None => panic!("no swap found"),
					//(LayoutPosition::for_layout(0, 0, 0), LayoutPosition::for_layout(0, 0, 0)), // swapping the same position doesn't change the layout
				};
				// println!("swapping {} and {}", p1, p2);
				let swap_happened = new_layout.swap(&p1, &p2);
				let op_counter = self.operation_counter.get();
				if swap_happened {
					self.operation_counter.set((op_counter.0 + 1, op_counter.1, op_counter.2));
				} else {
					self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
				}
			} else {
				if let Some(p) = new_layout.gen_valid_replace(rng) {
					let keycode = valid_keycodes.choose(rng).unwrap();
					let replace_happened = new_layout.replace(&p, *keycode);
					let op_counter = self.operation_counter.get();
					if replace_happened {
						self.operation_counter.set((op_counter.0, op_counter.1 + 1, op_counter.2));
					} else {
						self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
					}
				}
			}
			layouts.push(new_layout);
		}
		layouts
	}

	pub fn optimize(&self, rng: &mut impl Rng) -> Result<Layout<R, C>, AlcError> {
		if self.datasets.len() != self.config.dataset_weight.len() {
			return Err(AlcError::DatasetWeightsMismatchError(self.datasets.len(), self.config.dataset_weight.len()));
		}
		println!("base layout\n{:b}", self.base_layout);
		let mut layouts_and_scores = self.generate_and_score_initial_population(rng);
		let (mut best_layouts, mut best_scores) = self.take_best_layouts(layouts_and_scores);
		let mut layouts = self.generate_new_layouts(rng, best_layouts);
		for i in 0..self.config.generation_count {
			layouts_and_scores = self.score_population(&layouts);
			(best_layouts, best_scores) = self.take_best_layouts(layouts_and_scores);
			let printclone = best_layouts.clone();
			layouts = self.generate_new_layouts(rng, best_layouts);
			// println!("after {} generation(s), layout\n{}\n score: {}", i, printclone[0], best_scores[0]);
			println!("after {} generation(s), best score: {}, worst score {}", i, best_scores[0], best_scores[best_scores.len()-1]);
		}
		layouts_and_scores = self.score_population(&layouts);
		(best_layouts, best_scores) = self.take_best_layouts(layouts_and_scores);
		println!("final layout\n{:#}\nscore: {}", best_layouts[0], best_scores[0]);
		let final_layout = best_layouts[0].clone();
		let (v1, v2) = final_layout.verify_layout_correctness();
		if v1.len() > 0 {
			println!("issue with layer switches")
		}
		if v2.len() > 0 {
			println!("issue with symmetric keys")
		}
		println!("operations: {:?}", self.operation_counter);
		Ok(final_layout)
		// symmetry check
		// layer reachability check
		// other sanity checks
		
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
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/rust_book_test/"), 4, Num(50), &KeycodeOptions::default()).unwrap();
		let config = LayoutOptimizerConfig::default();
		LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset], config, Cell::new((0, 0, 0)))
	}
}

impl Default for LayoutOptimizer<4, 12, SimpleScoreFunction> {
	fn default() -> Self {
		let base_layout = Layout::<4, 12>::default();
		let effort_layer = Layer::<4, 12, f32>::default();
		let score_function = SimpleScoreFunction{};
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/rust_book_test/"), 4, Num(50), &KeycodeOptions::default()).unwrap();
		let config = LayoutOptimizerConfig::default();
		LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset], config, Cell::new((0, 0, 0)))
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
			E_10 A_10 C_10 LST1_0_10
		
		").unwrap();
		let score_function = SimpleScoreFunction{};
		let text = "hehehebe";
		let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/small_test/"), 2, All, &KeycodeOptions::default()).unwrap();
		let config = LayoutOptimizerConfig::default();
		let layout_optimizer = LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset], config, Cell::new((0, 0, 0)));
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

	#[test]
	fn test_optimize() {
		let mut lo = LayoutOptimizer::<4, 12, SimpleScoreFunction>::default();
		// let mut config = LayoutOptimizerConfig::default();
		lo.config.generation_count = 100;
		lo.config.population_size = 100;
		println!("initial valid keycodes {:?}", lo.config.valid_keycodes);
		let mut rng = StdRng::seed_from_u64(0);
		let mut test_layout = lo.base_layout.clone();
		println!("initial layout\n{}", test_layout);
		test_layout.randomize(&mut rng, &lo.config.valid_keycodes).unwrap();
		// println!("initial randomized layout\n{:#}", test_layout);
		println!("effort layer\n{}", lo.effort_layer);
		lo.optimize(&mut rng);
	}
}
