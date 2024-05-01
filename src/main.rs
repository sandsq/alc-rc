
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use alc::{objective::scoring::AdvancedScoreFunction, optimizer::LayoutOptimizer};




fn main() {
	// cargo flamegraph --bin=alc --palette=rust --output=performance/0_1_1.svg
	
	let mut lo = LayoutOptimizer::<4, 10, AdvancedScoreFunction>::try_from_optimizer_toml_file("./templates/ferris_sweep.toml").unwrap();
	
	let mut rng = ChaCha8Rng::seed_from_u64(1);
	println!("effort layer\n{}", lo.effort_layer);
	let _final_layout = lo.optimize(&mut rng).unwrap();

}
