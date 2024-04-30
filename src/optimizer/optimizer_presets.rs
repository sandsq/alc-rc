use std::cell::Cell;

use crate::keyboard::key::PhalanxKey;
use crate::optimizer::LayoutOptimizerConfig;
use super::Layer;
use super::Layout;
use super::LayoutOptimizer;
use super::Score;

impl<S> Default for LayoutOptimizer<4, 12, S> where S: Score<4, 12> + Send {
	fn default() -> Self {
		let base_layout = Layout::<4, 12>::default();
		let effort_layer = Layer::<4, 12, f64>::default();
		let phalanx_layer = Layer::<4, 12, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, Cell::new((0, 0, 0, 0)))
	}
}

impl<S> Default for LayoutOptimizer<4, 10, S> where S: Score<4, 10> + Send {
	fn default() -> Self {
		let base_layout = Layout::<4, 10>::default();
		let effort_layer = Layer::<4, 10, f64>::default();
		let phalanx_layer = Layer::<4, 10, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, Cell::new((0, 0, 0, 0)))
	}
}