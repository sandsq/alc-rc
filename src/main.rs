
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use alc::{objective::scoring::AdvancedScoreFunction, optimizer::LayoutOptimizer};




fn main() {
	// cargo flamegraph --bin=alc --palette=rust --output=performance/0_1_1.svg
	let mut lo = LayoutOptimizer::<4, 12, AdvancedScoreFunction>::default();
	// let mut config = LayoutOptimizerConfig::default();
	lo.config.genetic_options.generation_count = 50;
	lo.config.genetic_options.population_size = 100;
	lo.config.keycode_options.include_numbers = true;
	println!("initial valid keycodes {:?}", lo.config.valid_keycodes);
	let mut rng = ChaCha8Rng::seed_from_u64(1);
	println!("initial layout\n{}", lo.base_layout);
	println!("effort layer\n{}", lo.effort_layer);
	lo.optimize(&mut rng).unwrap();

}
