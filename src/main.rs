
// use rand::SeedableRng;
// use rand_chacha::ChaCha8Rng;
// use alc::{objective::scoring::AdvancedScoreFunction, optimizer::{optimize_from_toml, LayoutOptimizer}};
use alc::optimizer::optimize_from_toml;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
	#[arg(short, long)]
	config: String,
}

// fn main() {
//     let args = Args::parse();

//     for _ in 0..args.count {
//         println!("Hello {}!", args.name)
//     }
// }


fn main() {
	// cargo flamegraph --bin=alc --palette=rust --output=performance/0_1_1.svg
	
	let args = Args::parse();
	match optimize_from_toml(args.config) {
		Ok(_) => (),
		Err(e) => println!("{}", e),
	}
	
	// let mut lo = LayoutOptimizer::<4, 10, AdvancedScoreFunction>::try_from_optimizer_toml_file("./templates/ferris_sweep.toml").unwrap();
	
	// let mut rng = ChaCha8Rng::seed_from_u64(1);
	// println!("effort layer\n{}", lo.effort_layer);
	// let _final_layout = lo.optimize(&mut rng).unwrap();

}
