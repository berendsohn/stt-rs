use std::cmp::{max, min};
use std::time::Instant;
use clap::error::ErrorKind;

use clap::{CommandFactory, Parser};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::Normal;
use stt::{DynamicForest, MonoidWeight, NodeIdx};
use stt::common::EmptyGroupWeight;
use stt::link_cut::*;
use stt::onecut::*;
use stt::pg::*;
use stt::twocut::mtrtt::*;
use stt::twocut::splaytt::*;

use stt_benchmarks::bench_util::{ImplDesc, ImplName, PrintType};
use stt_benchmarks::bench_util::PrintType::{Json, Print};
use stt_benchmarks::do_for_impl_empty;


struct Helper {
	num_nodes : usize,
	print : PrintType,
	std_dev : f64,
	rng : StdRng
}

impl Helper {
	fn new( num_nodes : usize, print : PrintType, std_dev : f64, seed : u64 ) -> Self {
		Helper { num_nodes, print, std_dev, rng : StdRng::seed_from_u64( seed ) }
	}
	
	fn generate_index( &mut self, i : usize ) -> NodeIdx {
		if self.std_dev == 0. {
			NodeIdx::new( i )
		}
		else {
			let j = self.rng.sample( Normal::new( i as f64, self.std_dev )
				.expect( "Invalid distribution.") ) as usize;
			NodeIdx::new( max( 0, min( self.num_nodes, j ) ) )
		}
	}
	
	fn benchmark_degenerate<TDynTree>( &mut self, impl_name : &str )
		where TDynTree : DynamicForest<TWeight=EmptyGroupWeight>
	{
		let start = Instant::now();
		let mut f = TDynTree::new( self.num_nodes + 1 );
		for i in 0..(self.num_nodes-1) {
			f.link( NodeIdx::new( i ), NodeIdx::new( i+1 ), EmptyGroupWeight::identity() );
		}
	
		let last_node = NodeIdx::new( self.num_nodes-1 );
		for i in 0..(self.num_nodes-1) {
			f.compute_path_weight( self.generate_index( i ), last_node );
		}
		let dur = start.elapsed();
		
		if self.print == Print {
			let millis = dur.as_micros() as f64 / 1000.;
			println!( "{:<20} {millis:10.3}ms", impl_name.to_owned() + ":" )
		}
		else if self.print == Json {
			println!( "{}", json::stringify( json::object!{
				"type" : "degenerate",
				num_vertices : self.num_nodes,
				name : impl_name,
				std_dev : self.std_dev,
				time_ns : dur.as_nanos() as usize
			} ) )
		}
	}
}


macro_rules! do_benchmark {
	( $obj : ident, $impl_tpl : ident ) => {
		$obj.benchmark_degenerate::<$impl_tpl>( <$impl_tpl as ImplName>::name() )
	}
}

fn benchmark( imp : ImplDesc, helper : &mut Helper ) {
	do_for_impl_empty!( imp, do_benchmark, helper )
}


#[derive(Parser)]
#[command(name = "Degenerate tree benchmark")]
#[command(long_about = "Perform compute_path_weight(u,v) queries on a path, where u goes through the nodes in order, and v is the last node.")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long, default_value_t = 1_000)]
	num_nodes : usize,
	
	/// Standard deviation of node queries.
	#[arg(short='d', long, default_value_t = 0_f64)]
	std_dev : f64,

	/// Random seed, if --std-dev is not 0.
	#[arg(short, long)]
	seed : Option<u64>,
	
	/// Print the results in human-readable form
	#[arg(long, default_value_t = false)]
	print : bool,
	
	/// Output the results as json
	#[arg(long, default_value_t = false)]
	json : bool,
	
	/// Implementations to benchmark. Include all but petgraph if omitted.
	impls : Vec<ImplDesc>
}

fn main() {
	let cli = CLI::parse();
	
	let impls : Vec<ImplDesc>;
	if !cli.impls.is_empty() {
		impls = cli.impls;
	}
	else {
		impls = ImplDesc::all_efficient()
	}

	let seed : u64;
	if let Some( s ) = cli.seed {
		if cli.std_dev == 0. {
			CLI::command().error( ErrorKind::ArgumentConflict, "stdev is 0, no seed allowed" ).exit();
		}
		else {
			seed = s;
		}
	}
	else {
		if cli.std_dev != 0. {
			CLI::command().error( ErrorKind::ArgumentConflict, "stdev is not 0, need a seed" ).exit();
		}
		else {
			seed = 0
		}
	}
	
	for imp in impls {
		benchmark( imp, &mut Helper::new(
			cli.num_nodes,
			PrintType::from_args( cli.print, cli.json ),
			cli.std_dev,
			seed
		) );
	}
}