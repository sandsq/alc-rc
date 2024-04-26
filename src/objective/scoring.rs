

use crate::keyboard::key::{Finger::{self, *}, Hand::{self, *}, KeyValue, PhalanxKey};
use crate::keyboard::{LayoutPositionSequence, layer::Layer, layout::Layout};
use crate::optimizer::LayoutOptimizerConfig;



pub trait Score<const R: usize, const C: usize> {
	fn new() -> Self;

	fn score_small(&self, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> Option<f64>;

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

	fn score_small(&self, _effort_layer: &Layer<R, C, f64>, _phalanx_layer: &Layer<R, C, PhalanxKey>, _layout_position_sequence: LayoutPositionSequence, _config: &LayoutOptimizerConfig) -> Option<f64> {
		None
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RollDirection {
	Inner,
	Outer,
	PlaceholderDirection,
}
use RollDirection::*;



impl<const R: usize, const C: usize> Score<R, C> for AdvancedScoreFunction {
	fn new() -> Self {
		AdvancedScoreFunction {}
	}

	fn score_small(&self, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> Option<f64> {
		let alt_raw_weight = config.hand_alternation_weight;
		let roll_raw_weight = config.finger_roll_weight;
		let (alt_weight, roll_weight) = if alt_raw_weight == 0.0 && roll_raw_weight == 0.0 {
			(0.0, 0.0)
		} else {
			(alt_raw_weight / (alt_raw_weight + roll_raw_weight), roll_raw_weight / (alt_raw_weight + roll_raw_weight))
		};
		let alt_reduction = config.hand_alternation_reduction_factor;
		let roll_reduction = config.finger_roll_reduction_factor;

		if layout_position_sequence.len() == 1 {
			let lp = layout_position_sequence[0];
			return Some(effort_layer[lp]);
		} else if layout_position_sequence.len() == 2 {
			let lp1 = layout_position_sequence[0];
			let lp2 = layout_position_sequence[1];
			let (hand1, finger1) = phalanx_layer[lp1].value();
			let (hand2, finger2) = phalanx_layer[lp2].value();
			if hand1 == hand2 && finger1 == finger2 {
				return Some(effort_layer[lp1] + effort_layer[lp2] * config.same_finger_penalty_factor);
			}
		} else if layout_position_sequence.len() == 3 {
			let lp1 = layout_position_sequence[0];
			let lp2 = layout_position_sequence[1];
			let lp3 = layout_position_sequence[2];
			let (hand1, finger1) = phalanx_layer[lp1].value();
			let (hand2, finger2) = phalanx_layer[lp2].value();
			let (hand3, finger3) = phalanx_layer[lp3].value();
			if hand1 == hand2 && finger1 == finger2 {
				return Some(effort_layer[lp1] + effort_layer[lp2] * config.same_finger_penalty_factor + effort_layer[lp3]);
			}
			if hand2 == hand3 && finger2 == finger3 {
				return Some(effort_layer[lp1] + effort_layer[lp2] + effort_layer[lp3] * config.same_finger_penalty_factor);
			}
			if hand1 != hand2 && hand2 != hand3 {
				let red = calculate_final_reduction(alt_reduction, 2, alt_weight);
				return Some((effort_layer[lp1] + effort_layer[lp2] + effort_layer[lp3]) * red);
			}
			if hand1 == hand2 && hand2 == hand3 {
				// doesn't span 2 rows
				if (lp1.row_index as i8 - lp2.row_index as i8).abs() <= 1 && (lp2.row_index as i8 - lp3.row_index as i8).abs() <= 1 {
					// inner roll
					if finger1 < finger2 && finger2 < finger3 {
						let red = calculate_final_reduction(roll_reduction, 2, roll_weight);
						return Some((effort_layer[lp1] + effort_layer[lp2] + effort_layer[lp3]) * red);
					}
					// outer roll
					if finger1 > finger2 && finger2 > finger3 {
						let red = calculate_final_reduction(roll_reduction, 2, roll_weight);
						return Some((effort_layer[lp1] + effort_layer[lp2] + effort_layer[lp3]) * red);
					}
				}
			}
			return Some(effort_layer[lp1] + effort_layer[lp2] + effort_layer[lp3]);
		}
		return None;
	}

	fn score_layout_position_sequence(&self, layout: &Layout<R, C>, effort_layer: &Layer<R, C, f64>, phalanx_layer: &Layer<R, C, PhalanxKey>, layout_position_sequence: LayoutPositionSequence, config: &LayoutOptimizerConfig) -> f64 {
		// during debug, check that the position preceeding a higher layer position is a layer switch
		// we can use the fact that layer switches always should occur before a higher layer position to eliminate the need to actually check the layout for layer switches, and simplify checking when layer switches can be canceled
		let debug_clone = layout_position_sequence.clone();
		let alt_raw_weight = config.hand_alternation_weight;
		let roll_raw_weight = config.finger_roll_weight;
		let (alt_weight, roll_weight) = if alt_raw_weight == 0.0 && roll_raw_weight == 0.0 {
			(0.0, 0.0)
		} else {
			(alt_raw_weight / (alt_raw_weight + roll_raw_weight), roll_raw_weight / (alt_raw_weight + roll_raw_weight))
		};
		let alt_reduction = config.hand_alternation_reduction_factor;
		let roll_reduction = config.finger_roll_reduction_factor;

		match self.score_small(effort_layer, phalanx_layer, layout_position_sequence.clone(), config) {
			Some(v) => return v,
			None => (),
		}


		let seq_len = layout_position_sequence.len();
		let mut score: f64 = 0.0;
		let mut previous_hand = PlaceholderHand;
		let mut previous_finger = PlaceholderFinger;
		// phalanx_layer[layout_position_sequence[0]].value().0;
		let mut alternating_hand_streak = 0; // streak of 1 means previous hand and current hand were different. ie, it's 1 less than the length of the sequence satisfying the alternating / rolling criteria
		let mut previous_alternating_hand_streak;
		let mut efforts: Vec<f64> = vec![];
		let mut alt_inds: Vec<usize> = vec![]; // index i is where a hand alternating streak starts, index i + 1 is where it ends (not inclusive)
		let mut roll_inds: Vec<usize> = vec![]; // rolls can go in or out, but they should not span more than two rows
		let mut roll_direction = PlaceholderDirection;
		let mut previous_roll_direction = PlaceholderDirection;
		let mut roll_streak = 0;
		let mut previous_roll_streak;
		let mut current_row = 0;
		let mut previous_row;
		for (l_ind, layout_position) in layout_position_sequence.into_iter().enumerate() {
			let base_effort_value = effort_layer[layout_position];
			let effort_value = base_effort_value;
			previous_alternating_hand_streak = alternating_hand_streak;
			previous_roll_streak = roll_streak;
			previous_row = current_row;

			let (current_hand, current_finger) = phalanx_layer[layout_position].value();
			current_row = layout_position.row_index as i8;
			if l_ind > 0 {
				if current_hand != previous_hand {
					alternating_hand_streak += 1;

					roll_streak = 0;
				} else {
					alternating_hand_streak = 0;

					if current_finger > previous_finger {
						roll_direction = Inner;
						if roll_streak == 0 {
							previous_roll_direction = Inner;
							// roll_streak += 1;
						}
					} else if current_finger < previous_finger {
						roll_direction = Outer;
						if roll_streak == 0 {
							previous_roll_direction = Outer;
							// roll_streak += 1;
						}
					} else {
						roll_direction = PlaceholderDirection;
					}
					if (current_row - previous_row).abs() > 1 {
						roll_streak = 0;
					} else if roll_direction != PlaceholderDirection && previous_roll_direction == roll_direction {
						roll_streak += 1;
					} else {
						roll_streak = 0;
					}
					
				}
				// a left-right-left sequence of length 3 will have a streak of 2; 3 is the minimum sequence length eligible for an alternation or roll streak. We do not consider streaks of 1 (sequence of length 2, eg left-right) as otherwise anything that is not a roll would automatically be an alternation, and vice versa.
				if alternating_hand_streak == 0 && previous_alternating_hand_streak > 1 {
					// streak just ended
					alt_inds.push(l_ind);
				} else if previous_alternating_hand_streak == 1 && alternating_hand_streak > 1 {
					// streak started previously
					alt_inds.push(l_ind - alternating_hand_streak);
				}

				if roll_streak == 0 && previous_roll_streak > 1 {
					roll_inds.push(l_ind);
				} else if previous_roll_streak == 1 && roll_streak > 1 {
					roll_inds.push(l_ind - roll_streak);
				}

			}

			// penalize same finger
			if same_hand_and_finger(current_hand, previous_hand, current_finger, previous_finger) {
				score += (config.same_finger_penalty_factor - 1.0) * effort_value;
				// effort_value *= config.same_finger_penalty_factor;
			}

			// prepare next iteration
			previous_hand = current_hand;
			previous_finger = current_finger;

			previous_roll_direction = roll_direction;

			efforts.push(effort_value);
		}

		

		// if ending on a hand alternation, add the last index
		if alternating_hand_streak > 1 {
			alt_inds.push(seq_len);
		}
		if roll_streak > 1 {
			roll_inds.push(seq_len);
		}
		// println!("{}\n\t{:?}\n\t{:?}", debug_clone, alt_inds, roll_inds);

		let mut reductions: Vec<f64> = vec![];

		if alt_inds.len() > 1 {
			// println!("{}, {:?}", debug_clone, alt_inds);
			for i in (0..alt_inds.len()).step_by(2) {
				let alt_start = alt_inds[i];
				let alt_end = *match alt_inds.get(i + 1){
					Some(v) => v,
					None => panic!("{}\n\t{:?}\n\t{:?}", debug_clone, alt_inds, roll_inds),
				};
				// if alt_start == 0 && alt_end == 0 {
				// 	break;
				// }
				let streak_score: f64 = efforts[alt_start..alt_end].iter().sum();
				// for j in alt_start..alt_end {
				// 	efforts[j] = 0.0; // effort values within the streak will be used, so ignore them for the final sum of any non-streak positions
				// }
				let reduction = calculate_final_reduction(alt_reduction, alt_end - alt_start - 1, alt_weight);
				reductions.push(-(1.0 - reduction) * streak_score);
				// score += streak_score * reduction;
			}
		}

		if roll_inds.len() > 1 {
			for i in (0..roll_inds.len()).step_by(2) {
				let roll_start = roll_inds[i];
				let roll_end = *match roll_inds.get(i + 1) {
					Some(v) => v,
					None => panic!("{}\n\t{:?}\n\t{:?}", debug_clone, alt_inds, roll_inds),
				};
				let streak_score: f64 = efforts[roll_start..roll_end].iter().sum();
				let reduction = calculate_final_reduction(roll_reduction, roll_end - roll_start - 1, roll_weight);
				reductions.push(-(1.0 - reduction) * streak_score);
			}
		}
		
		score += efforts.iter().sum::<f64>() + reductions.iter().sum::<f64>();
		score
	}
}

pub fn calculate_final_reduction(initial_reduction: f64, n: usize, weight: f64) -> f64 {
	// eg if initial reduction is 0.9 and the streak is 2, the total reduction is 0.81x. That corresponds to a 0.19x loss. If the weight is 0.4, then 0.19 * 0.4 = 0.076x loss, or (1 - 0.076) = 0.924x reduction
	1.0 - (1.0 - (initial_reduction).powf(n as f64)) * weight
}

fn same_hand_and_finger(current_hand: Hand, previous_hand: Hand, current_finger: Finger, previous_finger: Finger) -> bool {
	if current_hand == previous_hand && current_finger == previous_finger {
		true
	} else {
		false
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
	fn test_reduction() {
		let red = calculate_final_reduction(0.9, 2, 0.4);
		assert_eq!(red, 0.924);
		let red = calculate_final_reduction(0.9, 1, 0.5);
		assert_eq!(red, 0.95);
	}

	#[test]
	fn test_alternating() {
		let layout = Layout::<1, 4>::init_blank(1);
		let effort_layer = Layer::<1, 4, f64>::try_from("
			0.1 0.2 0.3 0.4
		").unwrap();
		let phalanx_layer = Layer::<1, 4, PhalanxKey>::try_from("
			l:r l:m r:m r:r
		").unwrap();
		let sf = AdvancedScoreFunction{};

		// long alternating sequence
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 3)]);
		let mut config = LayoutOptimizerConfig::default();
		config.hand_alternation_reduction_factor = 0.9;
		config.hand_alternation_weight = 3.0;
		config.finger_roll_weight = 2.0;
		config.same_finger_penalty_factor = 3.0;
		let red = calculate_final_reduction(0.9, 3, 0.6);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, (0.1 + 0.3 + 0.2 + 0.4) * red);

		// two shorter alternating sequences
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 1)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 2, 0.6);
		assert_eq!(score, (0.1 + 0.3 + 0.2) * red + (0.1 + 0.4 + 0.1) * red + 0.2);

		// shorter alternating sequences, same finger in the middle
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 1)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 2, 0.6);
		assert_eq!(format!("{:.3}", score), format!("{:.3}", (0.1 + 0.4 + 0.2) * red + 0.2 * 2.0 + (0.2 + 0.4 + 0.2) * red));
		// 0, 0, 1, with effort 0.2, is repeated. So it incurs an extra 2x cost, with the original cost being part of an alternating sequence, for a "total" of 3.0x as set by the config option.

		// right left left right left
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 3), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 2, 0.6);
		assert_eq!(format!("{:.3}", score), format!("{:.3}", (0.4 + 0.1) + (0.2 + 0.4 + 0.1) * red));

		// same finger
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1 + 0.1 * 3.0);

		// length 1
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1);
		
		//length 2
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 1)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1 + 0.2);

		// length 3, nothing special
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, 0.1 + 0.2 + 0.1);
		
		// length 3, alternating
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 2, 0.6);
		assert_eq!(score, (0.1 + 0.3 + 0.1) * red);

		// length 3, repeat
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		assert_eq!(score, (0.1 + 0.1 * 3.0 + 0.3));
	}
	#[test]
	fn test_roll() {
		// roll
		let layout = Layout::<3, 5>::init_blank(1);
		let effort_layer = Layer::<3, 5, f64>::try_from("
			0.1 0.2 0.3 0.4 0.45
			0.5 0.6 0.7 0.8 0.85
			0.9 1.0 1.1 1.2 1.25
		").unwrap();
		let phalanx_layer = Layer::<3, 5, PhalanxKey>::try_from("
			l:p l:r l:m l:i r:i
			l:p l:r l:m l:i r:i
			l:p l:r l:m l:i r:i
		").unwrap();

		let sf = AdvancedScoreFunction{};
		let mut config = LayoutOptimizerConfig::default();
		config.hand_alternation_reduction_factor = 0.9;
		config.finger_roll_reduction_factor = 0.9;
		config.hand_alternation_weight = 3.0;
		config.finger_roll_weight = 2.0;
		config.same_finger_penalty_factor = 3.0;

		// crossing two columns
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 0, 1), LayoutPosition::new(0, 0, 2)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		// let red = calculate_final_reduction(0.9, 2, 0.6);
		assert_eq!(score, 0.9 + 0.2 + 0.3);

		// length 3
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 1, 1), LayoutPosition::new(0, 0, 2)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red = calculate_final_reduction(0.9, 2, 0.4);
		assert_eq!(format!("{:.5}", score), format!("{:.5}", (0.9 + 0.6 + 0.3) * red));

		// roll into alternate
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 1, 1), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 4), LayoutPosition::new(0, 1, 0), LayoutPosition::new(0, 1, 4)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red_roll = calculate_final_reduction(0.9, 2, 0.4);
		let red_alt = calculate_final_reduction(0.9, 3, 0.6);
		assert_eq!(format!("{:.5}", score), format!("{:.5}", (0.9 + 0.6 + 0.3 + 0.45 + 0.5 + 0.85) - (0.9 + 0.6 + 0.3) * (1.0 - red_roll) - (0.3 + 0.45 + 0.5 + 0.85) * (1.0 - red_alt)));

		// length 3, roll
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 1, 1), LayoutPosition::new(0, 0, 2)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		let red_roll = calculate_final_reduction(0.9, 2, 0.4);
		assert_eq!(format!("{:.5}", score), format!("{:.5}", (0.9 + 0.6 + 0.3) * red_roll));

	}



}