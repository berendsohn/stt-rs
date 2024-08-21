use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;

use clap::Parser;
use rand::distributions;
use rand::SeedableRng;
use rand::rngs::StdRng;
use stt::common::EmptyGroupWeight;
use stt::generate::GeneratableMonoidWeight;

use stt_benchmarks::bench_util;
use stt_benchmarks::bench_util::{Query, Query::*};


#[derive(Parser)]
#[command(name = "Generated random connectivity query Benchmark")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long)]
	num_vertices : usize,
	
	/// Number of queries
	#[arg(short='q', long)]
	num_queries : usize,
	
	/// Seed for the random query generator
	#[arg(short, long)]
	seed : u64,
	
	/// Probability of generating a connectivity query (instead of a cut) when querying two nodes in
	/// the same tree.
	#[arg(short='p', long, default_value_t = 0.5)]
	path_query_prob : f64,
	
	/// Write the generated queries to the given file
	#[arg(short, long)]
	output_file : Option<PathBuf>
}

fn write_queries( fp : &mut impl Write, num_vertices : usize, queries : &Vec<Query<EmptyGroupWeight>> ) {
	writeln!( fp, "queries {} {}", num_vertices, queries.len() ).expect( "write error" );
	for query in queries {
		match query {
			InsertEdge( u, v, weight ) => writeln!( fp, "i {u} {v} {weight}" ),
			DeleteEdge( u, v ) => writeln!( fp, "d {u} {v}" ),
			PathWeight( u, v ) => writeln!( fp, "p {u} {v}" )
		}.expect( "write error" );
	}
}

fn print_query_type_dist( queries : &Vec<Query<EmptyGroupWeight>> ) {
	let mut inserts = 0;
	let mut deletes = 0;
	let mut path_weights = 0;
	
	for query in queries {
		match query {
			Query::InsertEdge( _, _, _ ) => inserts += 1,
			Query::DeleteEdge( _, _ ) => deletes += 1,
			Query::PathWeight( _, _ ) => path_weights +=1
		}
	}
	
	println!( "Generated {inserts}x Link, {deletes}x Cut, {path_weights}x PathWeight" );
}

fn main() {
	let cli = CLI::parse();
	
	let num_vertices : usize = cli.num_vertices;
	let num_queries = cli.num_queries;
	
	let verbose = cli.output_file.is_some();
	
	if verbose {
		print!( "Generating {num_queries} queries on {num_vertices} vertices..." );
		stdout().flush().expect( "Flushing failed!" );
	}
	
	let mut rng = StdRng::seed_from_u64( cli.seed );
	let queries = bench_util::generate_queries_with_node_dist(
			num_vertices, num_queries, &mut rng, EmptyGroupWeight::generate, cli.path_query_prob, 
			&distributions::Uniform::new( 0, num_vertices ) );
	
	if verbose { print_query_type_dist( &queries ); }
	
	if let Some( output_path ) = cli.output_file {
		let mut fp = File::create( output_path ).unwrap();
		write_queries( &mut fp, num_vertices, &queries );
	}
	else {
		write_queries( &mut stdout(), num_vertices, &queries );
	}
	
	if verbose { println!( " Done." ); }
}
