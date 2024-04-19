use std::fmt::{self, Arguments};

use crate::keyboard::{LayoutPosition, LayoutPositionSequence, layer::Layer, layout::Layout};
use crate:: text_processor::frequency_holder::SingleGramFrequencies;



pub trait Score<const R: usize, const C: usize> {
	fn score_layout_position_sequence(&self, layout: Layout<R, C>, effort_layer: Layer<R, C, f32>, layout_position_sequence: LayoutPositionSequence) -> f32;
}

pub struct SimpleScoreFunction {}

impl<const R: usize, const C: usize> Score<R, C> for SimpleScoreFunction {
	fn score_layout_position_sequence(&self, _layout: Layout<R, C>, effort_layer: Layer<R, C, f32>, layout_position_sequence: LayoutPositionSequence) -> f32 {
		let mut score = 0.0;
		for layout_position in layout_position_sequence {
			let effort_value = effort_layer.get_from_layout_position(&layout_position).unwrap(); // might need to deal with accessing invalid location
			score += effort_value;
		}
		score
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple_score() {
		let layout = Layout::<2, 3>::init_blank(2);
		let effort_layer = Layer::<2, 3, f32>::try_from("
			0.1 0.2 0.3
			0.4 0.5 0.6
		").unwrap();
		let sf = SimpleScoreFunction{};
		let layout_position_sequence = LayoutPositionSequence::from_layout_positions(vec![LayoutPosition::for_layout(0, 0, 0), LayoutPosition::for_layout(0, 0, 2), LayoutPosition::for_layout(1, 1, 1)]); 
		let score = sf.score_layout_position_sequence(layout, effort_layer, layout_position_sequence);
		assert_eq!(score, 0.1 + 0.3 + 0.5);
	}
}