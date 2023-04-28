use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use stt::NodeIdx;
use stt::generate::generate_stt;
use stt::twocut::basic::{make_1_cut, STTRotate, STTStructureRead};

use crate::util::{is_same_underlying_tree};

#[test]
pub fn test_random_rotations() {
	const SIZE : usize = 100;
	const NUM_TESTS : usize = 5;
	const NUM_ROTATIONS : usize = 100;

	println!( "Starting {NUM_TESTS} tests with tree size {SIZE} and {NUM_ROTATIONS} rotations" );

	let mut rng = StdRng::seed_from_u64( 0 );
	for test_idx in 0..NUM_TESTS {
		let mut rot_counter = 0;
		let mut t = generate_stt( SIZE, &mut rng );
		let t0 = t.clone();

		let mut last_tree = t.clone();
		for rot_idx in 0..NUM_ROTATIONS {
			let v = NodeIdx::new( rng.gen_range( 0..SIZE ) );
			if t.can_rotate( v ) {
				t.rotate( v );
				rot_counter += 1;
			}

			assert!( t._is_valid(),
				"Rotation #{rot_idx} produces invalid tree:\n{}\n{}", last_tree.to_string(), t.to_string() );

			let mut t_1cut = t.clone();
			make_1_cut( &mut t_1cut );
			assert!( t_1cut.nodes().all( |v| ! t_1cut.is_separator( v ) ),
				"Not 1-cut:\n{}", t_1cut.to_string() );

			assert!( is_same_underlying_tree( &t0, &t ),
				"Not the same underlying tree:\n{}\n{}\n{}\n{}", t0.to_string(), last_tree.to_string(), t.to_string(), t_1cut.to_string() );
			last_tree = t.clone();
		}

		println!( "Finished test {test_idx} with {rot_counter} successful rotations" );
	}
}