use alc::{keyboard::{key::PhalanxKey, layer::Layer, layout::Layout, LayoutPosition, LayoutPositionSequence}, objective::scoring::{calculate_final_reduction, AdvancedScoreFunction, Score}, optimizer::LayoutOptimizerConfig};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

// #[divan::bench(args = [1_000_000])]
// fn loop_test(n: u64) -> (u64, u64) {
// 	let lookup = [4; 1_000_000];
// 	let mut score1 = 0;
// 	let mut score2 = 0;
// 	for i in 1..n {
// 		let mut xfactor = 1;
// 		if i % 3 == 0 {
// 			xfactor = 2;
// 		}
// 		score1 += i * xfactor;
// 		score2 += i * lookup[i as usize];
// 	}
// 	(score1, score2)
// }

// #[divan::bench(args = [1_000_000])]
// fn loop_test2(n: u64) -> (u64, u64) {
// 	let lookup = [4; 1_000_000];
// 	let mut score1 = 0;
// 	let mut score2 = 0;
// 	for i in 1..n {
// 		let mut xfactor = 1;
// 		if i % 3 == 0 {
// 			xfactor = 2;
// 		}
// 		score1 += i * xfactor;
// 	}
// 	for i in 1..n {
// 		score2 += i * lookup[i as usize];
// 	}
// 	(score1, score2)
// }

#[divan::bench(args = [100_000])]
fn score(n: u64) -> f64 {
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
	let mut final_score = 0.0;

	for _ in 0..n {
		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 1, 1), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 4), LayoutPosition::new(0, 1, 0), LayoutPosition::new(0, 1, 4)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		final_score += score;

		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 0, 0), LayoutPosition::new(0, 0, 2), LayoutPosition::new(0, 0, 0)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		final_score += score;

		let layout_position_sequence = LayoutPositionSequence::from_vector(vec![LayoutPosition::new(0, 2, 0), LayoutPosition::new(0, 1, 1), LayoutPosition::new(0, 0, 2)]);
		let score = sf.score_layout_position_sequence(&layout, &effort_layer, &phalanx_layer, layout_position_sequence, &config);
		final_score += score;
	}
	
	final_score
}