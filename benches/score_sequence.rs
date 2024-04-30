use alc::{keyboard::{key::PhalanxKey, layer::Layer, layout::Layout, LayoutPosition, LayoutPositionSequence}, objective::scoring::{ AdvancedScoreFunction, Score}, optimizer::config::LayoutOptimizerConfig};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

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
	config.score_options.hand_alternation_reduction_factor = 0.9;
	config.score_options.finger_roll_reduction_factor = 0.9;
	config.score_options.hand_alternation_weight = 3.0;
	config.score_options.finger_roll_weight = 2.0;
	config.score_options.same_finger_penalty_factor = 3.0;
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