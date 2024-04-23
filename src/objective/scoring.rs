
use crate::keyboard::{LayoutPositionSequence, layer::Layer, layout::Layout};
use crate::optimizer::LayoutOptimizerConfig;



pub trait Score<const R: usize, const C: usize> {
	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64;
}

pub struct SimpleScoreFunction {}

impl<const R: usize, const C: usize> Score<R, C> for SimpleScoreFunction {
	fn score_layout_position_sequence(&self, _layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, layout_position_sequence: LayoutPositionSequence, _config: &LayoutOptimizerConfig) -> f64 {
		let mut score = 0.0;
		for layout_position in layout_position_sequence {
			let effort_value = effort_layer[layout_position];
			score += effort_value;
		}
		score
	}
}

#[cfg(test)]
mod tests {
	use crate::keyboard::LayoutPosition;

use super::*;

	#[test]
	fn test_simple_score() {
		let layout = Layout::<2, 3>::init_blank(2);
		let effort_layer = Layer::<2, 3, f64>::try_from("
			0.1 0.2 0.3
			0.4 0.5 0.6
		").unwrap();
		let sf = SimpleScoreFunction{};
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(1, 1, 1)]); 
		let config = LayoutOptimizerConfig::default();
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1 + 0.3 + 0.5);
	}
}