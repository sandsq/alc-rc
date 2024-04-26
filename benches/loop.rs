fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench(args = [1_000_000])]
fn loop_test(n: u64) -> (u64, u64) {
	let lookup = [4; 1_000_000];
	let mut score1 = 0;
	let mut score2 = 0;
	for i in 1..n {
		let mut xfactor = 1;
		if i % 3 == 0 {
			xfactor = 2;
		}
		score1 += i * xfactor;
		score2 += i * lookup[i as usize];
	}
	(score1, score2)
}

#[divan::bench(args = [1_000_000])]
fn loop_test2(n: u64) -> (u64, u64) {
	let lookup = [4; 1_000_000];
	let mut score1 = 0;
	let mut score2 = 0;
	for i in 1..n {
		let mut xfactor = 1;
		if i % 3 == 0 {
			xfactor = 2;
		}
		score1 += i * xfactor;
	}
	for i in 1..n {
		score2 += i * lookup[i as usize];
	}
	(score1, score2)
}