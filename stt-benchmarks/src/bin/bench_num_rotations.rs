use std::cmp::max;
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};

use clap::{Parser, ValueEnum};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use stt::{NodeIdx, RootedForest};
use stt::common::EmptyNodeData;
use stt::twocut::basic::{STT, STTRotate, STTStructureRead};
use stt::twocut::mtrtt::*;
use stt::twocut::splaytt::*;
use stt::twocut::StableNTRStrategy;

use stt_benchmarks::bench_util::PrintType;
use stt_benchmarks::bench_util::PrintType::*;

/// An STT that also counts rotations.
#[derive( Clone )]
struct RotationCountingSTT {
	f : STT<EmptyNodeData>,
	rot_count : usize
}

impl RotationCountingSTT {
	fn new( num_nodes : usize ) -> Self {
		Self { f : STT::new( num_nodes ), rot_count : 0 }
	}
	
	fn attach( &mut self, c : NodeIdx, p : NodeIdx ) {
		self.f.attach( c, p );
	}
}

impl STTStructureRead for RotationCountingSTT {
	fn get_direct_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.f.get_direct_separator_child( v )
	}
	
	fn get_indirect_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.f.get_indirect_separator_child( v )
	}
}

impl RootedForest for RotationCountingSTT {
	fn get_parent( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.f.get_parent( v )
	}
}

impl STTRotate for RotationCountingSTT {
	fn rotate( &mut self, v : NodeIdx ) {
		self.f.rotate( v );
		self.rot_count += 1;
	}
}


/// Helper struct
struct ComputeHeight {
	depths : Vec<Option<usize>>
}

impl ComputeHeight {
	fn compute_depth( &mut self, f : &RotationCountingSTT, v : NodeIdx ) -> usize {
		match self.depths[v.index()] {
			None => {
				match f.get_parent( v ) {
					None => 1,
					Some( p ) => {
						let p_depth = self.compute_depth( f, p );
						self.depths[v.index()] = Some( p_depth + 1 );
						p_depth + 1
					}
				}
			},
			Some( d ) => d
		}
	}
	
	fn compute_height( f : &RotationCountingSTT ) -> usize {
		let mut ch = ComputeHeight{ depths : vec![None; f.f.num_nodes()] };
		
		let mut max_depth = 0;
		for v in f.f.nodes() {
			let v_depth = ch.compute_depth( f, v );
			max_depth = max( max_depth, v_depth );
		}
		max_depth
	}
}


/// Shape of the underlying tree
#[derive( Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum )]
enum TreeShape {
	/// Attach each node to a random earlier node, to form a relatively shallow tree
	Shallow,
	
	/// Attach each node to the previous node, to form a path
	Path
}

impl Display for TreeShape {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", match self {
			TreeShape::Shallow => "shallow",
			TreeShape::Path => "path"
		} )
	}
}


struct Helper
{
	num_nodes : usize,
	initial_forest: RotationCountingSTT,
	queries : Vec<NodeIdx>,
	seed : u64,
	print : PrintType
}

impl Helper
{
	fn new( num_nodes: usize, num_queries : usize, seed : u64, print : PrintType, shape : TreeShape ) -> Helper
	{
		let mut rng = StdRng::seed_from_u64( seed );
		
		if print == Print {
			print!( "Generating initial STT..." );
			stdout().flush().expect( "Flushing failed!" );
		}
		
		let mut initial_forest = RotationCountingSTT::new( num_nodes );
		for idx in 1..num_nodes {
			let p = match shape {
				TreeShape::Shallow => NodeIdx::new( rng.gen_range( 0..idx) ),
				TreeShape::Path => NodeIdx::new( idx-1 )
			};
			initial_forest.attach( NodeIdx::new( idx ), p );
		}
		
		if print == Print {
			println!( " Done (h={}).", ComputeHeight::compute_height( &initial_forest ) );
		}
		
		if print == Print {
			// print!( "Generating queries with {node_dist} distribution..." );
			print!( "Generating queries..." );
			stdout().flush().expect( "Flushing failed!" );
		}
		
		let queries = (0..num_queries)
				.map( |_| NodeIdx::new( rng.gen_range( 0..num_nodes ) ) )
				.collect();
		
		if print == Print {
			println!( " Done." );
		}
		
		Helper{ num_nodes, initial_forest, queries, seed, print }
	}
	
	fn benchmark<TNTRStrategy>( &self, impl_name : &str )
		where TNTRStrategy : StableNTRStrategy
	{
		let mut f = self.initial_forest.clone();
		for &v in &self.queries {
			TNTRStrategy::node_to_root( &mut f, v );
		}
		if self.print == Print {
			let per_query_str = format!( "({:.2}rots/query)", f.rot_count as f64 / ( self.queries.len() as f64 ) );
			println!( "{impl_name:<20} {:8}rots {per_query_str:>17}", f.rot_count )
		}
		else if self.print == Json {
			println!( "{}", json::stringify( json::object!{
				name : impl_name,
				num_vertices : self.num_nodes,
				num_queries : self.queries.len(),
				seed : self.seed,
				rotation_count : f.rot_count
			} ) )
		}
	}
}


#[derive(Parser)]
#[command(name = "Random query Benchmark")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long, default_value_t = 1000)]
	num_vertices : usize,
	
	/// Number of queries
	#[arg(short='q', long, default_value_t = 100_000)]
	num_queries : usize,
	
	/// Print the results in human-readable form
	#[arg(short, long, default_value_t = false)]
	print : bool,
	
	/// Output the results as json
	#[arg(short, long, default_value_t = false)]
	json : bool,
	
	/// Seed for the tree and query generator
	#[arg(short, long)]
	seed : u64,
	
	/// Shape of the generated underlying tree
	#[arg(long, default_value_t = TreeShape::Shallow)]
	shape : TreeShape
}


fn main() {
	let cli = CLI::parse();
	
	let print = PrintType::from_args( cli.print, cli.json );
	
	let helper = Helper::new( cli.num_vertices, cli.num_queries, cli.seed, print,
			cli.shape );
	
	helper.benchmark::<GreedySplayStrategy>( "Greedy Splay" );
	helper.benchmark::<TwoPassSplayStrategy>( "2P Splay" );
	helper.benchmark::<LocalTwoPassSplayStrategy>( "L2P Splay" );
	helper.benchmark::<MoveToRootStrategy>( "MTR" );
}
