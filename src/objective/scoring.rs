


use crate::keyboard::key::{Hand::*, Finger::*, KeyValue, PhalanxKey};
use crate::keyboard::{LayoutPositionSequence, layer::Layer, layout::Layout};
use crate::optimizer::LayoutOptimizerConfig;



pub trait Score<const R: usize, const C: usize> {
	fn new() -> Self;
	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>,  layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64;
}

pub struct SimpleScoreFunction {}
impl SimpleScoreFunction {
	pub fn new() -> Self {
		SimpleScoreFunction{}
	}
}

impl<const R: usize, const C: usize> Score<R, C> for SimpleScoreFunction {
	fn new() -> Self {
		SimpleScoreFunction{}
	}
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
impl AdvancedScoreFunction {
	pub fn new() -> Self {
		AdvancedScoreFunction {}
	}
}


impl<const R: usize, const C: usize> Score<R, C> for AdvancedScoreFunction {
	fn new() -> Self {
		AdvancedScoreFunction {}
	}
	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64 {
		// during debug, check that the position preceeding a higher layer position is a layer switch
		// we can use the fact that layer switches always should occur before a higher layer position to eliminate the need to actually check the layout for layer switches, and simplify checking when layer switches can be canceled
		let debug_clone = layout_position_sequence.clone();
		let alt_raw_weight = config.hand_alternation_weight;
		let roll_raw_weight = config.finger_roll_weight;
		let alt_weight = alt_raw_weight / (alt_raw_weight + roll_raw_weight);
		let roll_weight = roll_raw_weight / (alt_raw_weight + roll_raw_weight);
		let alt_reduction = config.hand_alternation_reduction_factor;
		let roll_reduction = config.finger_roll_reduction_factor;
		let seq_len = layout_position_sequence.len();
		let mut score: f64 = 0.0;
		let mut previous_hand = Placeholder;
		// phalanx_layer[layout_position_sequence[0]].value().0;
		let mut alternating_hand_streak = 0; // streak of 1 means previous hand and current hand were different
		let mut alternating_hand_prev_streak = 0;
		let mut efforts: Vec<f64> = vec![];
		let mut alt_inds: Vec<usize> = vec![]; // index i is where a hand alternating streak starts, index i + 1 is where it ends (not inclusive)
		for (l_ind, layout_position) in layout_position_sequence.into_iter().enumerate() {
			alternating_hand_prev_streak = alternating_hand_streak;
			let (current_hand, current_finger) = phalanx_layer[layout_position].value();
			if l_ind > 0 {
				if current_hand != previous_hand {
					alternating_hand_streak += 1;
				} else {
					alternating_hand_streak = 0;
				}

				if alternating_hand_streak == 0 && alternating_hand_prev_streak != 0 {
					// streak just ended
					alt_inds.push(l_ind);
				} else if alternating_hand_prev_streak == 0 && alternating_hand_streak > 0 {
					// streak started in previous iteration
					alt_inds.push(l_ind - 1);
				}
			}

			
			previous_hand = current_hand;
			

			let effort_value = effort_layer[layout_position];
			efforts.push(effort_value);
			// score += effort_value;
		}
		if alternating_hand_streak > 0 {
			alt_inds.push(seq_len);
		}

		// println!("{}, {:?}", debug_clone, alt_inds);
		if alt_inds.len() > 1 {
			// println!("{}, {:?}", debug_clone, alt_inds);
			for i in (0..alt_inds.len()).step_by(2) {
				let alt_start = alt_inds[i];
				let alt_end = alt_inds[i + 1];
				// if alt_start == 0 && alt_end == 0 {
				// 	break;
				// }
				let streak_score: f64 = efforts[alt_start..alt_end].iter().sum();
				for j in alt_start..alt_end {
					efforts[j] = 0.0; // effort values within the streak will be used, so ignore them for the final sum of any non-streak positions
				}
				let reduction = calculate_final_reduction(alt_reduction, alt_end - alt_start - 1, alt_weight);
				score += streak_score * reduction;
			}
		}
		// println!("{}, {:?}", debug_clone, alt_inds);
		score += efforts.iter().sum::<f64>();
		score
	}
}

fn calculate_final_reduction(initial_reduction: f64, n: usize, weight: f64) -> f64 {
	// eg if initial reduction is 0.9 and the streak is 2, the total reduction is 0.81x. That corresponds to a 0.19x loss. If the weight is 0.4, then 0.19 * 0.4 = 0.076x loss, or (1 - 0.076) = 0.924x reduction
	1.0 - (1.0 - (initial_reduction).powf(n as f64)) * weight
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
	fn test_reduction() {
		let red = calculate_final_reduction(0.9, 2, 0.4);
		assert_eq!(red, 0.924);
		let red = calculate_final_reduction(0.9, 1, 0.5);
		assert_eq!(red, 0.95);
	}

	#[test]
	fn test_advanced_score() {

		let layout = Layout::<1, 4>::init_blank(1);
		let effort_layer = Layer::<1, 4, f64>::try_from("
			0.1 0.2 0.3 0.4
		").unwrap();
		let phalanx_layer = Layer::<1, 4, PhalanxKey>::try_from("
			l:r l:m r:m r:r
		").unwrap();
		let sf = AdvancedScoreFunction{};
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 3)]);
		let mut config = LayoutOptimizerConfig::default();
		config.hand_alternation_reduction_factor = 0.9;
		config.hand_alternation_weight = 3.0;
		config.finger_roll_weight = 2.0;
		let red = calculate_final_reduction(0.9, 3, 0.6);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, (0.1 + 0.3 + 0.2 + 0.4) * red);

		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 1, 0.6);
		assert_eq!(score, (0.1 + 0.3) * red + (0.3 + 0.2) * red + 0.1);

	}
}