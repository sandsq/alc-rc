use std::fmt;

use crate::keyboard::{key::*, layout::*, layer::*};
use crate::text_processor::*;
use crate::objective::scoring::*;

use self::dataset::FrequencyDataset;

pub struct LayoutOptimizerConfig {
	initial_population: u32,
	generations: u32,
	dataset_weight: Vec<u8>,
}
impl Default for LayoutOptimizerConfig {
	fn default() -> Self {
		LayoutOptimizerConfig { initial_population: 1000, generations: 100, dataset_weight: vec![1] }
	}
}

pub struct LayoutOptimizer<const R: usize, const C: usize, S> where S: Score<R, C> {
	base_layout: Layout<R, C>,
	effort_layer: Layer<R, C, f32>,
	score_function: S,
	datasets: Vec<FrequencyDataset<u32>>,
}
impl<const R: usize, const C: usize, S> LayoutOptimizer<R, C, S> where S: Score<R, C> {
	fn optimize(&self, config: LayoutOptimizerConfig) -> Layout<R, C> {
		
		
		// symmetry check
		// layer reachability check
		// other sanity checks
		todo!()
	}
}