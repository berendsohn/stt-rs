//! Test different implementations of the make_1_cut function for [STT]s.
//! 
//! The simplest implementation seems to be the best by some margin, hence it is included in the
//! library.

use std::time::{Duration, Instant};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use stt::{NodeIdx};
use stt::common::EmptyNodeData;
use stt::generate::generate_stt;
use stt::twocut::basic::{MakeOneCutSTT, STT, STTRotate, STTStructureRead};

type EmptySTT = STT<EmptyNodeData>;


/// Perform rotations to make t a 1-cut tree. Returns the number of rotations performed.
/// 
/// Simple and fast implementation.
pub fn make_1_cut<STT : MakeOneCutSTT>( t : &mut STT ) -> usize {
	let mut rot_count = 0;
	for v in t.nodes() {
		while t.is_separator( v ) {
			t.rotate( v );
			rot_count += 1;
		}
	}
	rot_count
}

/// Perform rotations to make t a 1-cut tree. Returns the number of rotations performed.
/// 
/// Simple variant trying to minimize number of rotations.
pub fn make_1_cut_skip<STT : MakeOneCutSTT>( t : &mut STT ) -> usize {
	let mut rot_count = 0;
	let mut cont = true;
	while cont {
		cont = false;
		for v in t.nodes() {
			while t.is_separator( v ) && !t.is_separator( t.get_parent( v ).unwrap() ) {
				t.rotate( v );
				rot_count += 1;
			}

			if t.is_separator( v ) {
				cont = true; // Need to look at this node again later
			}
			else {
				// Not a separator, check children while we're at it
				if let Some( c ) = t.get_direct_separator_child( v ) {
					t.rotate( c );
				}
				if let Some( c ) = t.get_indirect_separator_child( v ) {
					t.rotate( c );
				}
			}
		}
	}
	rot_count
}

/// Perform rotations to make t a 1-cut tree. Returns the number of rotations performed.
/// 
/// More complicated variant trying to minimize number of rotations.
pub fn make_1_cut_reduce<STT : MakeOneCutSTT>( t : &mut STT ) -> usize {
	let mut rot_count = 0;

	let mut remaining : Vec<NodeIdx> = t.nodes().collect();

	while !remaining.is_empty() {
		let mut idx = 0;
		while idx < remaining.len() {
			let v = remaining[idx];
			let mut remove = false;
			if t.is_separator( v ) && !t.is_separator( t.get_parent( v ).unwrap() ) {
				t.rotate( v );
				rot_count += 1;
				remove = true;
			}
			else if !t.is_separator( v ) {
				remove = true;
			}

			if remove {
				if idx == remaining.len() - 1 {
					remaining.pop();
					break
				}
				else {
					remaining[idx] = remaining.pop().unwrap();
				}
			}
			else {
				idx += 1;
			}
		}
	}
	rot_count
}


fn main() {
	const NUM_NODES : usize = 200_000;
	const NUM_PRE_ROTATIONS : usize = 10*NUM_NODES;
	const NUM_TESTS : usize = 10;

	const FUNCS : [(&str, for<'a> fn(&'a mut EmptySTT) -> usize); 3] = [
			("simple", make_1_cut),
			("skip", make_1_cut_skip),
			("reduce", make_1_cut_reduce)
	];

	println!( "Starting {NUM_TESTS} tests with trees of size {NUM_NODES} and {NUM_PRE_ROTATIONS} pre-rotations" );

	let mut rng = StdRng::seed_from_u64( 0 );
	let mut durations : Vec<Vec<Duration>> = vec![ vec![], vec![], vec![] ];
	let mut rot_counts: Vec<Vec<usize>> = vec![ vec![], vec![], vec![] ];
	for test_idx in 0..NUM_TESTS {
		let mut t = generate_stt( NUM_NODES, &mut rng );

		for _ in 0..NUM_PRE_ROTATIONS {
			let v = NodeIdx::new( rng.gen_range( 0..NUM_NODES ) );
			if t.can_rotate( v ) {
				t.rotate( v );
			}
		}

		for (func_idx, (func_name, func)) in FUNCS.into_iter().enumerate() {
			let mut t_tmp = t.clone();

			let start = Instant::now();
			let rot_count = func( &mut t_tmp );
			let dur = start.elapsed();

			debug_assert!( t_tmp.nodes().all( |v| !t_tmp.is_separator( v ) ) );
			
			durations[func_idx].push( dur );
			rot_counts[func_idx].push( rot_count );

			let dur_ms = dur.as_micros() as f64 / 1000.0;
			println!( "{:<11} {dur_ms:6.3}ms/{rot_count}", format!( "{func_name} #{test_idx}:" ) );
		}
		println!();
	}
	
	println!( "Averages: " );
	for (func_idx, (func_name, _)) in FUNCS.into_iter().enumerate() {
		let mean_dur = durations[func_idx].iter()
				.map( |dur| dur.as_micros() as f64 / 1000.0 ).sum::<f64>() / NUM_TESTS as f64;
		let mean_rot_count = rot_counts[func_idx].iter().sum::<usize>() as f64 / NUM_TESTS as f64;
		println!( "{:<11} {mean_dur:6.3}ms/{mean_rot_count}", format!( "{func_name}:" ) );
	}
}