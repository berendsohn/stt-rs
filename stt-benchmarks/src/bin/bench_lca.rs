use std::fs::File;
use std::io;
use std::io::{BufRead, stdout, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};

use clap::Parser;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use stt::generate::generate_edge;
use stt::link_cut::RootedLinkCutTree;
use stt::NodeIdx;
use stt::rooted::{RootedDynamicForest, SimpleRootedForest};
use stt::twocut::mtrtt::*;
use stt::twocut::splaytt::*;

use stt_benchmarks::bench_util::{ImplName, PrintType, RootedImplDesc};
use stt_benchmarks::bench_util::PrintType::{Json, Print};
use stt_benchmarks::do_for_impl_rooted;

use crate::Query::*;


type DiEdge = (NodeIdx, NodeIdx);

#[derive(Copy, Clone)]
enum Query {
	Link( NodeIdx, NodeIdx ),
	Cut( NodeIdx ),
	CutEdge( NodeIdx, NodeIdx ),
	LCA( NodeIdx, NodeIdx )
}

fn generate_queries( num_vertices : usize, num_queries : usize, rng : &mut impl Rng, use_cut_edge : bool ) -> impl Iterator<Item=Query> + '_ {
	let mut drf = RootedLinkCutTree::new( num_vertices );
	let mut edges : Vec<DiEdge> = vec![];
	(0..num_queries).map( move |_| {
		if edges.len() > 0 && rng.gen_bool( 0.5 * ( edges.len() as f64 ) / ( (num_vertices - 1) as f64 ) ) {
			// Delete some edge
			let idx = rng.gen::<usize>() % edges.len();
			let (u,v) = edges.swap_remove( idx );
			// drf.cut_edge( u, v );
			drf.cut( u );
			if use_cut_edge {
				CutEdge( u, v )
			}
			else {
				Cut( u )
			}
		}
		else {
			// Insert or query
			let (u,v) = generate_edge( num_vertices, rng );
			let u = NodeIdx::new( u );
			let v = NodeIdx::new( v );
			let ur = drf.find_root( u );
			let vr = drf.find_root( v );

			if ur == vr {
				// u and v in same tree
				LCA( u, v )
			}
			else {
				drf.link( ur, v );
				edges.push( (ur, v) );
				Link( ur, v )
			}
		}
	} )
}

fn read_lca_file( path : &PathBuf ) -> io::Result<(usize, Vec<Query>)> {
	let file = File::open( path )?;
	let mut num_vertices = 0;
	let mut queries : Vec<Query> = vec![];
	for line in io::BufReader::new( file ).lines() {
		let line = line?;
		let parts : Vec<_> = line.split( " " ).collect();
		if parts[0] == "lca" {
			// "lca <num_vertices> <num_edges>"
			if parts.len() == 3 {
				if let Ok( n ) = parts[1].parse() {
					// Ignore number of edges
					num_vertices = n;
					continue;
				}
			}
			return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
		}
		else if ["l", "c", "a"].contains( &parts[0] ) {
			// "l <u> <v>" or "c <u> <v>" or "c <u> <v>"

			fn parse_edge( edge_parts : &Vec<&str>) -> Result<DiEdge, ParseIntError> {
				let u = NodeIdx::new( edge_parts[1].parse()? );
				let v = NodeIdx::new( edge_parts[2].parse()? );
				return Ok( (u,v) )
			}

			if parts.len() == 4 {
				if let Ok( e ) = parse_edge( &parts ) {
					let (u,v) = e;
					match parts[0] {
						"l" => queries.push( Link( u, v ) ),
						"c" => queries.push( CutEdge( u, v ) ),
						_ => queries.push( LCA( u, v ) )
					}
					continue
				}
			}
			return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
		}
		else if parts[0] == "c" {}
		else {
			return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
		}
	}
	Ok( (num_vertices, queries) )
}


/// Helper struct to store configuration, aggregate results, etc.
struct Helper {
	num_vertices : usize,
	input : Vec<Query>,
	print : PrintType
}

impl Helper {
	fn new( num_vertices : usize, input : Vec<Query>, print : PrintType ) -> Helper {
		Helper{ num_vertices, input, print }
	}

	fn report_test_result( &mut self, impl_name : &str, dur : Duration ) {
		if self.print == Print {
			let millis = dur.as_micros() as f64 / 1000.;
			let micros_per_edge = dur.as_micros() as f64 / self.input.len() as f64;
			println!( "{:<20} {millis:10.3}ms ({micros_per_edge:7.3}µs/query)", impl_name.to_owned() + ":" )
		}
		else if self.print == Json {
			println!( "{}", json::stringify( json::object!{
				num_vertices : self.num_vertices,
				num_queries : self.input.len(),
				name : impl_name,
				time_ns : dur.as_nanos() as usize
			} ) )
		}
	}

	fn benchmark_fd_con<TRDynForest : RootedDynamicForest>( &mut self, impl_name : &str ) {
		let start = Instant::now();
		let mut drf = TRDynForest::new( self.num_vertices );
		for &query in &self.input {
			match query {
				Link( u, v ) => drf.link( u, v ),
				Cut( u ) => drf.cut( u ),
				CutEdge( u, v ) => drf.cut_edge( u, v ),
				LCA( u, v ) => { drf.lowest_common_ancestor( u, v ).expect( "LCA query failed" ); }
			}
		}
		let dur = start.elapsed();
		self.report_test_result( impl_name, dur );
	}
}


macro_rules! do_benchmark {
	( $obj : ident, $impl_tpl : ident ) => {
		$obj.benchmark_fd_con::<$impl_tpl>( <$impl_tpl as ImplName>::name() )
	}
}

fn benchmark( imp : RootedImplDesc, helper : &mut Helper ) {
	do_for_impl_rooted!( imp, do_benchmark, helper )
}


#[derive(Parser)]
#[command(name = "Rooted dynamic trees Benchmark")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long, default_value_t = 1_000)]
	num_vertices : usize,

	/// Number of queries (link/cut/lca) to generate
	#[arg(short='q', long, default_value_t = 20_000)]
	num_queries : usize,

	/// Read input graph from the given file (ignore -n, -q, --seed)
	#[arg(short, long, required = false)]
	input : Option<PathBuf>,

	/// Use the cut_edge() method instead of cut() (puts STT implementation at an (unfair) advantage)
	#[arg(long)]
	cut_edge : bool,

	/// Print the results in human-readable form
	#[arg(long, default_value_t = false)]
	print : bool,

	/// Output the results as json
	#[arg(long, default_value_t = false)]
	json : bool,

	/// Seed for the random query generator
	#[arg(short, long)]
	seed : u64,

	/// Implementations to benchmark. Include all if omitted.
	impls : Vec<RootedImplDesc>
}


fn main() {
	let cli = CLI::parse();

	let print = PrintType::from_args( cli.print, cli.json );

	let impls : Vec<RootedImplDesc>;
	if !cli.impls.is_empty() {
		impls = cli.impls;
	}
	else {
		impls = RootedImplDesc::all()
	}

	let num_vertices : usize;
	let input : Vec<Query>;

	// Read edges // TODO: Remove if unused!
	if let Some( input_path ) = &cli.input {
		if cli.print {
			println!( "Reading edges from '{}'", input_path.display() );
		}
		match read_lca_file( input_path ) {
			Ok( ( n, i ) ) => { num_vertices = n; input = i },
			Err( e ) => {
				println!( "Could not read file '{}': {}", input_path.display(), e );
				exit( 1 );
			}
		}

		if cli.print {
			println!( " Done reading {} edges on {num_vertices} vertices.", input.len() );
		}
	}
	else {
		let mut rng = StdRng::seed_from_u64( cli.seed );
		num_vertices = cli.num_vertices;

		// Generate edges
		if cli.print {
			println!( "Generating {} queries on {num_vertices} vertices. Seed: {}.", cli.num_queries, cli.seed );
			if cli.cut_edge {
				println!( "Using cut_edge() method instead of cut()" );
			}
			stdout().flush().expect( "Couldn't flush for some reason" );
		}
		input = generate_queries( num_vertices, cli.num_queries, &mut rng, cli.cut_edge ).collect();

		if cli.print {
			println!( " Done." );
		}
	}

	if cli.print {
		let num_links = input.iter().filter( |q| matches!( q, Link( _, _ ) ) ).count();
		let num_cuts = input.iter().filter( |q| matches!( q, CutEdge( _, _ ) ) || matches!( q, Cut( _ ) ) ).count();
		let num_lcas = input.len() - num_links - num_cuts;
		println!( "Benchmarking input with {num_links} links, {num_cuts} cuts and {num_lcas} LCA queries." );
	}

	let mut helper = Helper::new( num_vertices, input, print );

	for imp in &impls {
		benchmark( *imp, &mut helper );
	}
}
