use std::time::Instant;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use stt::NodeIdx;
use stt::generate::generate_stt;
use stt::twocut::basic::*;

fn perf_test() {
	const SIZE : usize = 1000000;
	const NUM_TESTS : usize = 10;
	const NUM_ROTATIONS : usize = 1000000;

	println!( "Starting {NUM_TESTS} tests with tree size {SIZE} and {NUM_ROTATIONS} rotations" );

	let mut total_dur_per_rot : f64 = 0.0;

	let mut rng = StdRng::seed_from_u64( 0 );
	for test_idx in 0..NUM_TESTS {
		let mut rot_counter = 0;
		let mut t = generate_stt( SIZE, &mut rng );

		let start = Instant::now();
		for _ in 0..NUM_ROTATIONS {
			let v = NodeIdx::new( rng.gen_range( 0..SIZE ) );
			if t.can_rotate( v ) {
				t.rotate( v );
				rot_counter += 1;
			}
		}
		let dur = start.elapsed().as_nanos();
		let dur_per_rot = dur as f64 / NUM_ROTATIONS as f64;
		total_dur_per_rot += dur_per_rot;
		println!( "Finished test {test_idx} with {rot_counter} successful rotations in {dur}ns" );
	}

	println!( "Average time per rotation: {:.3}ns", total_dur_per_rot / NUM_TESTS as f64 );
}

fn main() {
	perf_test();
}