//! Various helpers to randomly generate stuff.

use rand::prelude::Distribution;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::common::{EmptyGroupWeight, EmptyNodeData, IsizeAddGroupWeight, SignedAddGroupWeight, UnsignedMaxMonoidWeight, UsizeMaxMonoidWeight};
use crate::{MonoidWeight, NodeIdx};
use crate::twocut::basic::{STT};

/// Return the edges of a random rooted tree with given number of nodes.
/// 
/// The edges have the form (parent, child), i.e., are oriented away from the root.
pub fn generate_rooted_tree_edges<'a>( num_vertices : usize, rng : &'a mut impl Rng )
		-> impl Iterator<Item=(usize, usize)> + 'a
{
	let mut nodes : Vec<_> = (0..num_vertices).collect();
	nodes.shuffle( rng );
	
	(1..num_vertices).map( move |v| {
		( nodes[rng.gen_range( 0..v )], nodes[v] )
	} )
}


/// Generate a random one-cut STT.
pub fn generate_stt( num_vertices: usize, rng : &mut impl Rng ) -> STT<EmptyNodeData> {
	let mut tree = STT::new( num_vertices );
	
	for (p, c) in generate_rooted_tree_edges( num_vertices, rng ) {
		tree.attach( NodeIdx::new( c ), NodeIdx::new( p ) );
	}
	tree
}


/// Generate a uniformly random edge `(u,v)`, where `u` and `v` are distinct and in `0..num_nodes`.
pub fn generate_edge( num_nodes : usize, rng : &mut impl Rng ) -> (usize, usize) {
	let u = rng.gen_range( 0..num_nodes );
	let mut v = rng.gen_range( 0..num_nodes-1 );
	if v >= u {
		v += 1;
	}
	( u, v )
}


/// Generate a random edge using the given node distribution.
/// 
/// WARNING: Repeats if a node is sampled twice.
pub fn generate_edge_with_dist( dist : &impl Distribution<usize>, rng : &mut impl Rng ) -> (usize, usize)
{
	let u = rng.sample( dist );
	for v in rng.sample_iter( dist ) {
		if u != v {
			return (u,v);
		}
	}
	panic!( "sample_iter stopped!" );
}


/// Generate `num_edges` random edges, possibly with repetition.
pub fn generate_edges<'a>( num_nodes : usize, num_edges : usize, rng : &'a mut impl Rng ) -> impl Iterator<Item=(usize, usize)> + 'a {
	(0..num_edges).map( move |_| generate_edge( num_nodes, rng ) )
}


/// A weight type that has a default way of being randomly generated.
pub trait GeneratableMonoidWeight : MonoidWeight {
	/// Generate a weight in the default way.
	fn generate( rng : &mut impl Rng ) -> Self;
}

impl GeneratableMonoidWeight for EmptyGroupWeight {
	fn generate( _ : &mut impl Rng ) -> EmptyGroupWeight {
		EmptyGroupWeight::identity()
	}
}

impl GeneratableMonoidWeight for IsizeAddGroupWeight {
	fn generate( rng : &mut impl Rng ) -> IsizeAddGroupWeight {
		SignedAddGroupWeight::new( rng.gen_range( (-1000)..1000 ) )
	}
}

impl GeneratableMonoidWeight for UsizeMaxMonoidWeight {
	fn generate( rng : &mut impl Rng ) -> UsizeMaxMonoidWeight {
		UnsignedMaxMonoidWeight::new( rng.gen_range( 0..1000 ) )
	}
}