use std::fmt;
use rand::prelude::*;
use rand::Rng;
use tqdm::tqdm;


use crate::keyboard::{key::*, layout::*, layer::*};
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;
use self::keycode::{Keycode::{self, *}, get_default_keycode_set};

pub struct LayoutOptimizerConfig {
	initial_population_size: u32,
	generation_count: u32,
	dataset_weight: Vec<u8>,
	valid_keycodes: Vec<Keycode>,
}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		LayoutOptimizerConfig { 
			initial_population_size: 5, 
			generation_count: 1, 
			dataset_weight: vec![1],
			valid_keycodes: get_default_keycode_set(), }
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

	pub fn optimize(&self, rng: &mut impl Rng, config: LayoutOptimizerConfig) -> Layout<R, C> {
		let initial_population_size = config.initial_population_size;
		let valid_keycodes = config.valid_keycodes;
		let mut population: Vec<Layout<R, C>> = vec![];
		let mut scores: Vec<f32> = vec![];
		for i in tqdm(0..initial_population_size) {
			let mut initial_layout = self.base_layout.clone();
			initial_layout.randomize(rng, &valid_keycodes);
			println!("{}", initial_layout);
			population.push(initial_layout);
			
		}

		
		// symmetry check
		// layer reachability check
		// other sanity checks
		todo!()
	}
}



#[cfg(test)]
mod tests {
	use std::path::PathBuf;

use rand::rngs::StdRng;

use super::*;

	// #[test]
	// fn test_initial_layout_generation() {
	// 	let base_layout = Layout::<2, 4>::init_blank(2);
	// 	let effort_layer = Layer::<2, 4, f32>::try_from("
	// 		0.1 0.2 0.3 0.4
	// 		0.5 0.6 0.7 0.8
	// 	").unwrap();
	// 	let score_function = SimpleScoreFunction{};
	// 	let dataset = FrequencyDataset::<u32>::from_dir(PathBuf::from("./data/rust_book_test/"), 4).unwrap();
	// 	let layout_optimizer = LayoutOptimizer::new(base_layout, effort_layer, score_function, vec![dataset]);
	// 	let config = LayoutOptimizerConfig::default();
	// 	let mut rng = StdRng::seed_from_u64(0);
	// 	layout_optimizer.optimize(&mut rng, config);
	// }
}
