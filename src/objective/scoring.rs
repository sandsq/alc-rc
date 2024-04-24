


use crate::keyboard::key::{Hand::*, Finger::*, KeyValue, PhalanxKey};
use crate::keyboard::{LayoutPositionSequence, layer::Layer, layout::Layout};
use crate::optimizer::LayoutOptimizerConfig;



pub trait Score<const R: usize, const C: usize> {
	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>,  layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64;
}

pub struct SimpleScoreFunction {}

impl<const R: usize, const C: usize> Score<R, C> for SimpleScoreFunction {
	fn score_layout_position_sequence(&self, _layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, _phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, _config: &LayoutOptimizerConfig) -> f64 {
		let mut score = 0.0;
		for (_i, layout_position) in layout_position_sequence.into_iter().enumerate() {
			let effort_value = effort_layer[layout_position];
			score += effort_value;
		}
		score
	}
}

pub struct AdvancedScoreFunction {}

impl<const R: usize, const C: usize> Score<R, C> for AdvancedScoreFunction {
	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64 {
		// during debug, check that the position preceeding a higher layer position is a layer switch
		// we can use the fact that layer switches always should occur before a higher layer position to eliminate the need to actually check the layout for layer switches, and simplify checking when layer switches can be canceled
		let mut score: f64 = 0.0;
		let mut previous_hand = Left;
		let mut alternating_hand_streak = 0; // streak of 1 means previous hand and current hand were different
		for (l_ind, layout_position) in layout_position_sequence.into_iter().enumerate() {
			let (current_hand, current_finger) = phalanx_layer[layout_position].value();
			if current_hand != previous_hand {
				if l_ind > 0 {
					alternating_hand_streak += 1;
				}
			}
			

			previous_hand = current_hand;

			let effort_value = effort_layer[layout_position];
			score += effort_value;
		}
		if alternating_hand_streak > 0 {
			
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
		let phalanx_layer = Layer::<2, 3, PhalanxKey>::try_from("
			l:r l:m l:i
			l:r l:m l:i
		").unwrap();
		let sf = SimpleScoreFunction{};
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(1, 1, 1)]); 
		let config = LayoutOptimizerConfig::default();
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1 + 0.3 + 0.5);
	}
	#[test]
	fn test_advanced_score() {

	}
}