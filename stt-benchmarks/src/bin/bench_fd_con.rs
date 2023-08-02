use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::{BufRead, stdout, Write};
use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};

use clap::Parser;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use stt::{DynamicForest, NodeIdx};
use stt::common::EmptyGroupWeight;
use stt::connectivity::FullyDynamicConnectivity;
use stt::generate::generate_edge;
use stt::link_cut::*;
use stt::onecut::*;
use stt::pg::*;
use stt::twocut::mtrtt::*;
use stt::twocut::splaytt::*;

use stt_benchmarks::bench_util::{ImplDesc, ImplName, PrintType};
use stt_benchmarks::bench_util::PrintType::{Json, Print};
use stt_benchmarks::do_for_impl_empty;

use crate::Query::{DeleteEdge, InsertEdge};

type Edge = (NodeIdx, NodeIdx);

#[derive(Copy, Clone)]
enum Query {
	InsertEdge( NodeIdx, NodeIdx ),
	DeleteEdge( NodeIdx, NodeIdx )
}

fn generate_queries( num_vertices : usize, num_queries : usize, rng : &mut impl Rng ) -> impl Iterator<Item=Query> + '_ {
	let mut cur_edges : HashSet<Edge> = HashSet::new();
	(0..num_queries).map( move |_| {
		let (u,v) = generate_edge( num_vertices, rng );
		let u = NodeIdx::new( u );
		let v = NodeIdx::new( v );
		if cur_edges.remove( &(u,v) ) {
			DeleteEdge( u, v )
		}
		else {
			cur_edges.insert( (u,v) );
			InsertEdge( u, v )
		}
	} )
}

fn read_con_file( path : &PathBuf ) -> io::Result<(usize, Vec<Query>)> {
	let file = File::open( path )?;
	let mut num_vertices = 0;
	let mut queries : Vec<Query> = vec![];
	for line in io::BufReader::new( file ).lines() {
		let line = line?;
		let parts : Vec<_> = line.split( " " ).collect();
		if parts[0] == "fd_con" {
			// "fd_con <num_vertices> <num_edges>"
			if parts.len() == 3 {
				if let Ok( n ) = parts[1].parse() {
					// Ignore number of edges
					num_vertices = n;
					continue;
				}
			}
			return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
		}
		else if ["i", "d"].contains( &parts[0] ) {
			// "i <u> <v>" or "d <u> <v>"

			let parse_vertex = |s : &str| {
				match s.parse() {
					Err( _ ) => Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid vertex: '{s}'" ) ) ),
					Ok( idx ) => {
						if idx < num_vertices {
							Ok( NodeIdx::new( idx ) )
						}
						else {
							println!( "Note: out of bounds: {idx} > {num_vertices}" );
							Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid vertex: '{idx}'" ) ) )
						}
					}
				}
			};

			let parse_edge = |edge_parts : &Vec<&str>| -> Result<Edge, io::Error> {
				let u = parse_vertex( edge_parts[1] )?;
				let v = parse_vertex( edge_parts[2] )?;

				return Ok( (u,v) )
			};
			
			if parts.len() == 3 {
				if let Ok( e ) = parse_edge( &parts ) {
					let (u,v) = e;
					if parts[0] == "i" {
						queries.push( InsertEdge( u, v ) );
					}
					else {
						queries.push( DeleteEdge( u, v ) );
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
	
	fn benchmark_fd_con<TDynForest>( &mut self, impl_name : &str )
		where TDynForest: DynamicForest<TWeight = EmptyGroupWeight>
	{
		let start = Instant::now();
		let mut c : FullyDynamicConnectivity<TDynForest> = FullyDynamicConnectivity::new( self.num_vertices );
		for &query in &self.input {
			match query {
				InsertEdge( u, v ) => c.insert_edge( u, v ),
				DeleteEdge( u, v ) => c.delete_edge( u, v )
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

fn benchmark( imp : ImplDesc, helper : &mut Helper ) {
	do_for_impl_empty!( imp, do_benchmark, helper )
}


#[derive(Parser)]
#[command(name = "Fully dynamic connectivity Benchmark")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long, default_value_t = 1_000)]
	num_vertices : usize,
	
	/// Number of queries (insert/delete edge) to generate
	#[arg(short='q', long, default_value_t = 20_000)]
	num_queries : usize,
	
	/// Read input graph from the given file (ignore -n, -q, --seed)
	#[arg(short, long, group = "input")]
	input_file : Option<PathBuf>,
	
	/// Print the results in human-readable form
	#[arg(long, default_value_t = false)]
	print : bool,
	
	/// Output the results as json
	#[arg(long, default_value_t = false)]
	json : bool,
	
	/// Seed for the random query generator
	#[arg(short, long, group = "input")]
	seed : Option<u64>,
	
	/// Implementations to benchmark. Include all if omitted.
	impls : Vec<ImplDesc>
}



fn main() {
	let cli = CLI::parse();
	
	let print = PrintType::from_args( cli.print, cli.json );
	
	let impls : Vec<ImplDesc>;
	if !cli.impls.is_empty() {
		impls = cli.impls;
	}
	else {
		impls = ImplDesc::all()
	}

	let num_vertices : usize;
	let input : Vec<Query>;
	
	// Read edges
	if let Some( input_path ) = &cli.input_file {
		if cli.print {
			println!( "Reading edges from '{}'", input_path.display() );
		}
		match read_con_file( input_path ) {
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
		let seed = cli.seed.unwrap();
		let mut rng = StdRng::seed_from_u64( seed );
		num_vertices = cli.num_vertices;
		
		// Generate edges
		if cli.print {
			println!( "Generating {} queries on {num_vertices} vertices. Seed: {}.", cli.num_queries, seed );
			stdout().flush().expect( "Couldn't flush for some reason" );
		}
		input = generate_queries( num_vertices, cli.num_queries, &mut rng ).collect();
		
		if cli.print {
			println!( " Done." );
		}
	}
	
	if cli.print {
		let num_inserts = input.iter().filter( |q| matches!( q, InsertEdge( _, _ ) ) ).count();
		let num_deletes = input.len() - num_inserts;
		println!( "Benchmarking input with {num_inserts} inserts and {num_deletes} deletes." );
	}
	
	let mut helper = Helper::new( num_vertices, input, print );
	
	for imp in &impls {
		benchmark( *imp, &mut helper );
	}
}
