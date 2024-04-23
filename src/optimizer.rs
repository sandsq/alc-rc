
use std::cell::Cell;
use std::collections::HashSet;
use std::mem::discriminant;
use std::time::SystemTime;
use rand::prelude::*;
use rand::Rng;
use tqdm::tqdm;
use std::path::PathBuf;


use crate::alc_error::AlcError;
use crate::keyboard::key::KeyValue;
use crate::keyboard::LayoutPosition;
use crate::keyboard::LayoutPositionSequence;
use crate::keyboard::{layout::*, layer::*};
use crate::optimizer::ngram::Ngram;
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;
use self::frequency_holder::{SingleGramFrequencies, TopFrequenciesToTake::*};
use self::keycode::KeycodeOptions;
use self::keycode::{Keycode, generate_default_keycode_set};

#[derive(Debug, Clone)]
pub struct LayoutOptimizerConfig {
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
}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		let keycode_options = KeycodeOptions::default();
		let mut valid_keycodes = generate_default_keycode_set(&keycode_options).into_iter().collect::<Vec<Keycode>>();
		valid_keycodes.sort_unstable();
		println!("for dev: no swaps happening to test");
		LayoutOptimizerConfig { 
			population_size: 5, 
			generation_count: 1,
			fitness_cutoff: 0.1,
			swap_weight: 0.0,
			replace_weight: 1.0,
			dataset_weight: vec![1.0],
			dataset_paths: vec![String::from("./data/rust_book_test/")],
			keycode_options: keycode_options.clone(),
			valid_keycodes: valid_keycodes,
			max_ngram_size: 5,
			top_n_ngrams_to_take: 50, }
	}
}

pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	pub base_layout: Layout<R, C>,
	pub effort_layer: Layer<R, C, f64>,
	score_function: S,
	pub config: LayoutOptimizerConfig,
	operation_counter: Cell<(u32, u32, u32)>, // swaps, replacements, nothings
}
impl<const R: usize, const C: usize, S> LayoutOptimizer<R, C, S> where S: Score<R, C> {
	pub fn new(base_layout: Layout<R, C>, effort_layer: Layer<R, C, f64>, score_function: S, config: LayoutOptimizerConfig, operation_counter: Cell<(u32, u32, u32)>) -> Self {
		LayoutOptimizer { base_layout, effort_layer, score_function, config, operation_counter }
	}

	pub fn compute_datasets(&self) -> Vec<FrequencyDataset<u32>> {
		self.config.dataset_paths.iter()
			.map(|x| FrequencyDataset::<u32>::try_from_dir(PathBuf::from(x), self.config.max_ngram_size, Num(self.config.top_n_ngrams_to_take), &self.config.keycode_options).unwrap()).collect::<Vec<FrequencyDataset<u32>>>()
	}

	pub fn activate(&mut self) -> () {
		self.config.valid_keycodes = generate_default_keycode_set(&self.config.keycode_options).into_iter().collect();
		self.config.valid_keycodes.sort_unstable();
	}

	fn score_single_grams(&self, layout: &Layout<R, C>, frequencies: SingleGramFrequencies<u32>, save_positions: bool) -> (u32, HashSet<LayoutPosition>) {
		let mut score = 0;
		let total = frequencies.total;
		let mut visited_positions: HashSet<LayoutPosition> = HashSet::default();
		let effort_layer = &self.effort_layer;
		for (ngram, ngram_frequency) in frequencies {
			let sequences = match layout.ngram_to_sequences(&ngram) {
				Some(v) => v,
				None => panic!("unable to create sequence from {}", ngram),
				// return 0.0
			};
			let mut possible_scores: Vec<u32> = vec![];
			let mut possible_sequences: Vec<LayoutPositionSequence> = vec![];
			for sequence in sequences {
				if save_positions {
					possible_sequences.push(sequence.clone());
				}
				let sequence_score = self.score_function.score_layout_position_sequence(layout, effort_layer, sequence, &self.config);
				possible_scores.push(sequence_score);
			}
			

			// let min_index = match possible_scores.iter().enumerate().min_by(|(_, a), (_, b)| a.total_cmp(b)).map(|(idx, _)| idx) {
			// 	Some(v) => v,
			// 	None => 0,
			// };
			let min_index = arg_min2(&possible_scores);
			let min_score = possible_scores[min_index];
			
			if save_positions {
				let min_sequence = &possible_sequences[min_index];
				for pos in min_sequence.clone() {
					visited_positions.insert(pos);
				}
			}
			// let min_score = match possible_scores.iter().min_by(|a, b| a.total_cmp(b)) {
			// 	Some(v) => v,
			// 	None => &0.0,
			// };
			// println!("{} * {} / {}", min_score, ngram_frequency, total);
			
			
			score += min_score * (ngram_frequency); // / total; // should be slightly more efficient to precompute counts / total, but lazy for now
		}
		(score, visited_positions)
	}

	fn score_datasets(&self, layout: &Layout<R, C>, datasets: &Vec<FrequencyDataset<u32>>, save_positions: bool) -> (u32, HashSet<LayoutPosition>) {
		let mut score = 0;
		let mut visited_positions: HashSet<LayoutPosition> = HashSet::default();
		let mut d_ind: usize = 0;
		for dataset in datasets {
			let ngram_ratio = 1.0 / dataset.ngram_frequencies.len() as f64;
			let mut dataset_score = 0;
			for ngram_size in dataset.ngram_frequencies.keys() {
				let (calculated_score, calculated_positions) = self.score_single_grams(layout, dataset.ngram_frequencies[ngram_size].clone(), save_positions); // this clone might be expensive
				dataset_score += calculated_score; // * ngram_ratio; // each ngram length has equal weight to score, can change in the future
				if save_positions {
					visited_positions.extend(calculated_positions);
				}
			}
			dataset_score *= self.config.dataset_weight[d_ind] as u32;
			d_ind += 1;
			score += dataset_score;
		}
		(score, visited_positions)
	}


	fn score_population(&self, layouts: &Vec<Layout<R, C>>, datasets: &Vec<FrequencyDataset<u32>>) -> Vec<(Layout<R, C>, u32)> {
		let mut new_population: Vec<(Layout<R, C>, u32)> = Default::default();
		for layout in layouts {
			let (score, _) = self.score_datasets(layout, datasets, false);
			new_population.push((layout.clone(), score));
		}
		new_population
	}
	

	fn generate_and_score_initial_population(&self, rng: &mut impl Rng, datasets: &Vec<FrequencyDataset<u32>>) -> Vec<(Layout<R, C>, u32)> {
		let valid_keycodes = &self.config.valid_keycodes;
		let mut initial_population: Vec<(Layout<R, C>, u32)> = Default::default();
		for _i in 0..self.config.population_size {
			let mut initial_layout = self.base_layout.clone();
			initial_layout.randomize(rng, valid_keycodes).unwrap();
			let (initial_score, _) = self.score_datasets(&initial_layout, datasets, false);
			initial_population.push((initial_layout.clone(), initial_score));
		}
		initial_population
	}

	fn take_best_layouts(&self, mut population: Vec<(Layout<R, C>, u32)>) -> (Vec<Layout<R, C>>, Vec<u32>) {
    	population.sort_by(|a, b| a.1.cmp(&b.1));
		let num_to_take = (self.config.fitness_cutoff * (self.config.population_size as f64)).ceil() as usize;
		let _ = population.split_off(num_to_take); // the returned value is the low score ones
		let (left, right): (Vec<Layout<R, C>>, Vec<u32>) =  population.into_iter().unzip();
		println!("scores {:?}", right);
		(left, right)
	}

	fn generate_new_layouts(&self, rng: &mut impl Rng, mut layouts: Vec<Layout<R, C>>) -> Vec<Layout<R, C>> {
		let population_size = self.config.population_size;
		let swap_threshold = self.config.swap_weight / (self.config.swap_weight + self.config.replace_weight);
		let valid_keycodes = &self.config.valid_keycodes;
		// modify surviving layouts
		for layout in &mut layouts {
			let roll: f64 = rng.gen();
			if roll <= swap_threshold {
				let (p1, p2) = match layout.generate_random_valid_swap(rng) {
					Some((x, y)) => (x, y),
					// None => panic!("no swap found"),
					None => (LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 0)), // swapping the same position doesn't change the layout
				};
				// println!("swapping {} and {}", p1, p2);
				let swap_happened = layout.swap(p1, p2);
				let op_counter = self.operation_counter.get();
				if swap_happened {
					self.operation_counter.set((op_counter.0 + 1, op_counter.1, op_counter.2));
				} else {
					self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
				}
			} else {
				if let Some(p) = layout.generate_valid_replace_position(rng) {
					// println!("valid keycodes {:?}", valid_keycodes);
					let keycode = valid_keycodes.choose(rng).unwrap();
					let replace_happened = layout.replace(p, *keycode);
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
			let roll: f64 = rng.gen();
			// println!("roll {} vs swap threshold {}", roll, swap_threshold);
			if roll <= swap_threshold {
				let (p1, p2) = match new_layout.generate_random_valid_swap(rng) {
					Some((x, y)) => (x, y),
					None => panic!("no swap found"),
					//(LayoutPosition::for_layout(0, 0, 0), LayoutPosition::for_layout(0, 0, 0)), // swapping the same position doesn't change the layout
				};
				// println!("swapping {} and {}", p1, p2);
				let swap_happened = new_layout.swap(p1, p2);
				let op_counter = self.operation_counter.get();
				if swap_happened {
					self.operation_counter.set((op_counter.0 + 1, op_counter.1, op_counter.2));
				} else {
					self.operation_counter.set((op_counter.0, op_counter.1, op_counter.2 + 1));
				}
			} else {
				if let Some(p) = new_layout.generate_valid_replace_position(rng) {

					let keycode = valid_keycodes.choose(rng).unwrap();
					let replace_happened = new_layout.replace(p, *keycode);
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

	pub fn optimize(&mut self, rng: &mut impl Rng) -> Result<Layout<R, C>, AlcError> {
		let datasets = &self.compute_datasets();
		if datasets.len() != self.config.dataset_weight.len() {
			return Err(AlcError::DatasetWeightsMismatchError(datasets.len(), self.config.dataset_weight.len()));
		}

		self.activate();
		println!("base layout\n{}", self.base_layout);
		for dataset in datasets {
			let onegram = dataset.get(&1).unwrap().clone();
			let mut onegram_sorted = onegram.iter().collect::<Vec<(&Ngram, &u32)>>();
			onegram_sorted.sort_by(|a, b| b.1.cmp(a.1));
			println!("onegrams");
			for (gram, count) in onegram_sorted {
				println!("{}: {}", gram, count);
			}
		}

		let mut avg_score_time = 0.0;
		let mut avg_take_time = 0.0;
		let mut avg_gen_time = 0.0;

		let mut now = SystemTime::now();
		let mut layouts_and_scores = self.generate_and_score_initial_population(rng, datasets);
		let (mut best_layouts, mut best_scores) = self.take_best_layouts(layouts_and_scores);
		println!("initial best layout {}", best_layouts[0]);
		println!("initial, best score: {}, worst score {}", best_scores[0], best_scores[best_scores.len()-1]);
		let mut layouts = self.generate_new_layouts(rng, best_layouts);
		let initial_time = now.elapsed().unwrap().as_secs_f64();
	
		for i in tqdm(0..self.config.generation_count) {
			now = SystemTime::now();
			layouts_and_scores = self.score_population(&layouts, datasets);
			avg_score_time += now.elapsed().unwrap().as_secs_f64();
			
			now = SystemTime::now();
			(best_layouts, best_scores) = self.take_best_layouts(layouts_and_scores);
			avg_take_time +=  now.elapsed().unwrap().as_secs_f64();

			now = SystemTime::now();
			layouts = self.generate_new_layouts(rng, best_layouts);
			avg_gen_time +=  now.elapsed().unwrap().as_secs_f64();
			// println!("after {} generation(s), layout\n{}\n score: {}", i, printclone[0], best_scores[0]);
			println!("after {} generation(s), best score: {}, worst score {}", i, best_scores[0], best_scores[best_scores.len()-1]);
		}
		layouts_and_scores = self.score_population(&layouts, datasets);
		(best_layouts, best_scores) = self.take_best_layouts(layouts_and_scores);
		let mut final_layout = best_layouts[0].clone();
		println!("final layout pre removal\n{} score: {}", final_layout, best_scores[0]);

		let (score, visited) = self.score_datasets(&final_layout, datasets, true);
		assert_eq!(score, best_scores[0]);
		for layer_index in 0..final_layout.len() {
			for row_index in 0..R {
				for col_index in 0..C {
					let current_pos = LayoutPosition::new(layer_index, row_index, col_index);
					let k = final_layout[current_pos];
					if visited.contains(&current_pos) {
						continue;
					} else if !k.is_moveable() || k.is_symmetric() || discriminant(&k.value()) == discriminant(&Keycode::_LS(0)) || discriminant(&k.value()) == discriminant(&Keycode::_LST(0, 0,)) {
						continue;
					} else {
						final_layout.get_mut(layer_index, row_index, col_index).unwrap().set_value(Keycode::_NO);
					}
				}
			}
		}
		final_layout.generate_pathmap().unwrap();
		let (score, _) = self.score_datasets(&final_layout, datasets, false);
		println!("final layout post removal\n{:#} score: {}", final_layout, score);

		
		if score != best_scores[0] {
			println!("removing unused key positions gave a different score, something went wrong")
		} else {
			println!("verified that removing unused positions has the same score")
		}
		let (v1, v2) = final_layout.verify_layout_correctness();
		if v1.len() > 0 {
			println!("issue with layer switches {:?}", v1)
		} else {
			println!("layer switches checks passed")
		}
		if v2.len() > 0 {
			println!("issue with symmetric keys {:?}", v2)
		} else {
			println!("symmetric keys checks passed")
		}
		println!("operations: {:?}", self.operation_counter);
		println!("initial time: {}", initial_time);
		println!("avg score time: {}", avg_score_time / self.config.generation_count as f64);
		println!("avg take top time: {}", avg_take_time / self.config.generation_count as f64);
		println!("avg gen time: {}", avg_gen_time / self.config.generation_count as f64);
		Ok(final_layout)
		// symmetry check
		// layer reachability check
		// other sanity checks
		
	}
}
impl Default for LayoutOptimizer<2, 4, SimpleScoreFunction> {
	fn default() -> Self {
		let base_layout = Layout::<2, 4>::init_blank(2);
		let effort_layer = Layer::<2, 4, f64>::try_from("
			0.1 0.2 0.3 0.4
			0.5 0.6 0.7 0.8
		").unwrap();
		let score_function = SimpleScoreFunction{};
		let config = LayoutOptimizerConfig::default();
		LayoutOptimizer::new(base_layout, effort_layer, score_function, config, Cell::new((0, 0, 0)))
	}
}

impl Default for LayoutOptimizer<4, 12, SimpleScoreFunction> {
	fn default() -> Self {
		let base_layout = Layout::<4, 12>::default();
		let effort_layer = Layer::<4, 12, f64>::default();
		let score_function = SimpleScoreFunction{};
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, score_function, config, Cell::new((0, 0, 0)))
	}
}

fn arg_min(scores: &Vec<f64>) -> usize {
	let min_index = match scores.iter().enumerate().min_by(|(_, a), (_, b)| a.total_cmp(b)).map(|(idx, _)| idx) {
		Some(v) => v,
		None => panic!("Error for the developer, couldn't find a min score index"),
	};
	min_index
}
fn arg_min2(scores: &Vec<u32>) -> usize {
	let min_index = match scores.iter().enumerate().min_by(|(_, a), (_, b)| a.cmp(b)).map(|(idx, _)| idx) {
		Some(v) => v,
		None => panic!("Error for the developer, couldn't find a min score index"),
	};
	min_index
}



#[cfg(test)]
mod tests {

	use super::*;
	use rand_chacha::ChaCha8Rng;

	#[test]
	fn test_arg_min () {
		let v = vec![1.0, 5.5, 10.0, 0.5, 8.0];
		assert_eq!(arg_min(&v), 3);
	}

	#[test]
	fn test_ngram_scoring() {
		let base_layout = Layout::<1, 4>::init_blank(2);
		let effort_layer = Layer::<1, 4, f64>::try_from("
			0.1 0.4 0.3 0.2
		").unwrap();
		let test_layout = Layout::<1, 4>::try_from("
			___Layer 0___
			H_10 E_10 B_10 LS1_10
			___Layer 1___
			E_10 A_10 C_10 LST1_0_10
		
		").unwrap();
		let score_function = SimpleScoreFunction{};
		let mut config = LayoutOptimizerConfig::default();
		config.max_ngram_size = 2;
		config.dataset_paths = vec![String::from("./data/small_test/")];
		let layout_optimizer = LayoutOptimizer::new(base_layout, effort_layer, score_function, config, Cell::new((0, 0, 0)));
		let datasets = layout_optimizer.compute_datasets();
		let twogram_frequency = datasets[0].ngram_frequencies.get(&(2 as usize)).unwrap();
		println!("{:?}", twogram_frequency);
		let (s, poss) = layout_optimizer.score_single_grams(&test_layout, twogram_frequency.clone(), true);
		// 3 * he + 1 * be + 2 * eh + 1 + eb
		let expected_two_score = (3.0 * (0.1 + 0.2 + 0.1) + 1.0 * (0.3 + 0.2 + 0.1) + 2.0 * (0.2 + 0.1 + 0.1) + 1.0 * (0.2 + 0.1 + 0.3)) / 7.0;
		assert_eq!(format!("{s:.3}"), format!("{expected_two_score:.3}"));
		let expected_poss: HashSet<LayoutPosition> = HashSet::from_iter(vec![
			LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2),
			LayoutPosition::new(0, 0, 3), LayoutPosition::new(1, 0, 0),
		]);
		assert_eq!(poss, expected_poss);
		
		let (full_score, _) = layout_optimizer.score_datasets(&test_layout, &datasets, true);
		let expected_one_score = (3.0 * 0.1 + 4.0 * (0.2 + 0.1) + 1.0 * 0.3) / 8.0;
		// onegrams and twograms have equal weight
		let expected_score = 0.5 * expected_one_score + 0.5 * expected_two_score;
		assert_eq!(format!("{full_score:.3}"), format!("{expected_score:.3}"));
		// layout_optimizer.optimize(&mut rng, config);
	}

	#[test]
	#[ignore = "expensive"] // cargo test -- --ignored to run ignored, cargo test -- --include-ignored to run all
	fn test_optimize() {
		let mut lo = LayoutOptimizer::<4, 12, SimpleScoreFunction>::default();
		// lo.config.keycode_options.include_number_symbols = true;
		// lo.datasets = vec![FrequencyDataset::<u32>::try_from_dir(PathBuf::from("./data/rust_book_test/"), 4, Num(lo.config.top_n_ngrams_to_take), &lo.config.keycode_options).unwrap()];
		// lo.config.valid_keycodes = generate_default_keycode_set(&lo.config.keycode_options).into_iter().collect();
		lo.config.generation_count = 5;
		lo.config.population_size = 100;
		println!("initial valid keycodes {:?}", lo.config.valid_keycodes);
		let mut rng = ChaCha8Rng::seed_from_u64(1);
		// let test_layout = lo.base_layout.clone();
		// println!("initial layout\n{}", test_layout);
		// test_layout.randomize(&mut rng, &lo.config.valid_keycodes).unwrap();
		// println!("initial randomized layout\n{:#}", test_layout);
		println!("effort layer\n{}", lo.effort_layer);
		let _final_layout = lo.optimize(&mut rng).unwrap();
		// println!("final layout\n{:b}", final_layout);
	}
}
