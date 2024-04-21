use rand::{rngs::StdRng, SeedableRng};


use alc::{objective::scoring::SimpleScoreFunction, optimizer::LayoutOptimizer};




fn main() {

	let mut lo = LayoutOptimizer::<4, 12, SimpleScoreFunction>::default();
	// let mut config = LayoutOptimizerConfig::default();
	lo.config.generation_count = 20;
	lo.config.population_size = 200;
	println!("initial valid keycodes {:?}", lo.config.valid_keycodes);
	let mut rng = StdRng::seed_from_u64(0);
	println!("initial layout\n{}", lo.base_layout);
	println!("effort layer\n{}", lo.effort_layer);
	lo.optimize(&mut rng).unwrap();

}
