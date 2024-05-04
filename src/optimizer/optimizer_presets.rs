

use crate::keyboard::key::PhalanxKey;
use crate::optimizer::LayoutOptimizerConfig;
use super::Layer;
use super::Layout;
use super::LayoutOptimizer;
use super::Score;

impl<S> Default for LayoutOptimizer<2, 4, S> where S: Score<2, 4> + Send + Sync {
	fn default() -> Self {
		let base_layout = Layout::<2, 4>::default();
		let effort_layer = Layer::<2, 4, f64>::default();
		let phalanx_layer = Layer::<2, 4, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, super::OperationCounter::new((0, 0, 0, 0)))
	}
}

impl<S> Default for LayoutOptimizer<4, 10, S> where S: Score<4, 10> + Send + Sync {
	fn default() -> Self {
		let base_layout = Layout::<4, 10>::default();
		let effort_layer = Layer::<4, 10, f64>::default();
		let phalanx_layer = Layer::<4, 10, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, super::OperationCounter::new((0, 0, 0, 0)))
	}
}

impl<S> Default for LayoutOptimizer<4, 12, S> where S: Score<4, 12> + Send + Sync {
	fn default() -> Self {
		let base_layout = Layout::<4, 12>::default();
		let effort_layer = Layer::<4, 12, f64>::default();
		let phalanx_layer = Layer::<4, 12, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, super::OperationCounter::new((0, 0, 0, 0)))
	}
}

impl<S> Default for LayoutOptimizer<5, 15, S> where S: Score<5, 15> + Send + Sync {
	fn default() -> Self {
		let base_layout = Layout::<5, 15>::default();
		let effort_layer = Layer::<5, 15, f64>::default();
		let phalanx_layer = Layer::<5, 15, PhalanxKey>::default();
		let score_function = S::new();
		let config = LayoutOptimizerConfig::default();	
		LayoutOptimizer::new(base_layout, effort_layer, phalanx_layer, score_function, config, super::OperationCounter::new((0, 0, 0, 0)))
	}
}