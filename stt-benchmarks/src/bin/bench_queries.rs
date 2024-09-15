use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, stdout, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use num_traits::pow::Pow;
use rand::{distributions, SeedableRng};
use rand::rngs::StdRng;
use stt::common::{EmptyGroupWeight, IsizeAddGroupWeight, UsizeMaxMonoidWeight};
use stt::{DynamicForest, NodeIdx};
use stt::generate::GeneratableMonoidWeight;
use stt::link_cut::*;
use stt::onecut::*;
use stt::pg::*;
use stt::twocut::mtrtt::*;
use stt::twocut::splaytt::*;

use stt_benchmarks::{bench_util, do_for_impl_empty, do_for_impl_group, do_for_impl_monoid};
use stt_benchmarks::bench_util::{ImplDesc, ImplName, PrintType, Query};
use stt_benchmarks::bench_util::PrintType::*;

const GEOM_P : f64 = 0.01;

/// A distribution to choose nodes in a dynamic tree
#[derive( Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum )]
enum NodeDistribution {
	Uniform,
	
	/// Geometric distribution with p=[GEOM_P]
	Geometric
}

impl Display for NodeDistribution {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", match self {
			Self::Uniform => "uniform",
			Self::Geometric => "geometric"
		} )
	}
}


struct Helper<TWeight>
	where TWeight : GeneratableMonoidWeight
{
	num_vertices : usize,
	queries : Vec<Query<TWeight>>,
	path_query_prob : f64,
	print : PrintType,
	repeat : usize
}

impl<TWeight> Helper<TWeight>
	where TWeight : GeneratableMonoidWeight
{
	fn new( cli : &CLI ) -> Helper<TWeight>
	{
		let print = PrintType::from_args( cli.print, cli.json );
	
		let num_vertices;
		let queries;
		
		if let Some( path ) = &cli.input_file {
			if print == Print {
				print!( "Reading queries from {}...", path.display() );
				stdout().flush().expect( "Flushing failed!" );
			}
			(num_vertices, queries) = Self::read_con_queries( path ).expect( "Couldn't read queries." );
		}
		else {
			if print == Print {
				print!( "Generating queries with {} distribution...", cli.dist );
				stdout().flush().expect( "Flushing failed!" );
			}
			
			num_vertices = cli.num_vertices;
			
			let num_queries = cli.num_queries.unwrap_or( match cli.dist {
				NodeDistribution::Uniform => 20*num_vertices,
				NodeDistribution::Geometric => ( 10. * (1. / (1. - GEOM_P )).pow( num_vertices as f64 ) ) as usize
			} );
			
			let mut rng = StdRng::seed_from_u64( cli.seed.unwrap() );
			queries =  match cli.dist {
				NodeDistribution::Uniform => bench_util::generate_queries_with_node_dist(
					num_vertices, num_queries, &mut rng, TWeight::generate, cli.path_query_prob, 
					&distributions::Uniform::new( 0, num_vertices ) ),
				NodeDistribution::Geometric => bench_util::generate_queries_with_node_dist(
					num_vertices, num_queries, &mut rng, TWeight::generate, cli.path_query_prob,
					&distributions::WeightedIndex::new( 
						(0..num_vertices).map( |i| (1.-GEOM_P).pow( i as f64 ) ) )
						.expect( "Couldn't create distribution" ) )
			};
		}
		
		if print == Print {
			println!( " Done." );
		}
		
		Helper{ num_vertices, queries, path_query_prob : cli.path_query_prob, print, repeat : cli.repeat }
	}
	
	fn read_con_queries( path : &PathBuf ) -> io::Result<(usize, Vec<Query<TWeight>>)> {
		let file = File::open( path )?;
		let mut num_vertices = 0;
		let mut queries : Vec<Query<TWeight>> = vec![];
		for line in io::BufReader::new( file ).lines() {
			let line = line?;
			let parts : Vec<_> = line.split( " " ).collect();
			if parts[0] == "con" {
				// "con <num_vertices> <num_edges>"
				if parts.len() == 3 {
					if let Ok( n ) = parts[1].parse() {
						// Ignore number of edges
						num_vertices = n;
						continue;
					}
				}
				return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
			}
			else if ["i", "d", "p"].contains( &parts[0] ) {
				// "i/d/p <u> <v>
	
				fn parse_edge( edge_parts : &Vec<&str>) -> Result<(NodeIdx, NodeIdx), ParseIntError> {
					let u = NodeIdx::new( edge_parts[1].parse()? );
					let v = NodeIdx::new( edge_parts[2].parse()? );
					return Ok( (u,v) )
				}
				
				if parts.len() == 3 {
					if let Ok( e ) = parse_edge( &parts ) {
						let (u, v) = e;
						queries.push( match parts[0] {
							"i" => Query::InsertEdge( u, v, TWeight::identity() ),
							"d" => Query::DeleteEdge( u, v ),
							"p" => Query::PathWeight( u, v ),
							_ => panic!()
						} );
						continue
					}
				}
				return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
			}
			else if parts[0] == "c" {} // Ignore comment
			else {
				return Err( io::Error::new( io::ErrorKind::Other, format!( "Invalid line: '{line}'" ) ) );
			}
		}
		Ok( (num_vertices, queries) )
	}
	
	fn benchmark<TDynForest>( &self, impl_name : &str )
		where TDynForest : DynamicForest<TWeight=TWeight>
	{
		let mut duration = Duration::ZERO;
		for _ in 0..self.repeat {
			duration += bench_util::benchmark_queries::<TDynForest>( self.num_vertices, &self.queries )
		}
		if self.print == Print {
			let per_query_str = format!( "({:.3}µs/query)",
				duration.as_micros() as f64 / ( self.queries.len() as f64 ) / ( self.repeat as f64 ) );
			println!( "{impl_name:<20} {:8.3}ms {per_query_str:>17}", duration.as_micros() as f64 / 1000. )
		}
		else if self.print == Json {
			println!( "{}", json::stringify( json::object!{
				name : impl_name,
				num_vertices : self.num_vertices,
				num_queries : self.queries.len(),
				path_query_prob : self.path_query_prob,
				time_ns : usize::try_from( duration.as_nanos() / ( self.repeat as u128 ) )
					.expect( format!( "Duration too long: {}", duration.as_nanos() ).as_str() )
			} ) )
		}
	}
	
	fn print_query_type_dist( &self ) {
		let mut inserts = 0;
		let mut deletes = 0;
		let mut path_weights = 0;
		
		for query in &self.queries {
			match query {
				Query::InsertEdge( _, _, _ ) => inserts += 1,
				Query::DeleteEdge( _, _ ) => deletes += 1,
				Query::PathWeight( _, _ ) => path_weights +=1
			}
		}
		
		println!( "Generated {inserts}x Link, {deletes}x Cut, {path_weights}x PathWeight" );
	}
}





fn benchmark_empty( helper : &Helper<EmptyGroupWeight>, impls: &Vec<ImplDesc> ) {
	if helper.print == Print {
		println!( "Benchmarking {} connectivity queries on {} vertices",
			helper.queries.len(), helper.num_vertices );
		helper.print_query_type_dist();
	}
	
	macro_rules! do_benchmark_empty {
		( $obj : ident, $impl_tpl : ident ) => {
			$obj.benchmark::<$impl_tpl>( <$impl_tpl as ImplName>::name() )
		}
	}
	
	for imp in impls {
		do_for_impl_empty!( imp, do_benchmark_empty, helper );
	}
}


fn benchmark_group( helper : &Helper<IsizeAddGroupWeight>, impls: &Vec<ImplDesc> ) {
	if helper.print == Print {
		println!( "Benchmarking {} signed-sum queries on {} vertices",
			helper.queries.len(), helper.num_vertices );
		helper.print_query_type_dist();
	}
	
	macro_rules! do_benchmark_group {
		( $obj : ident, $impl_tpl : ident ) => {
			$obj.benchmark::<$impl_tpl<IsizeAddGroupWeight>>( <$impl_tpl<IsizeAddGroupWeight> as ImplName>::name() )
		}
	}
	
	for imp in impls {
		do_for_impl_group!( imp, do_benchmark_group, helper );
	}
}


fn benchmark_monoid( helper : &Helper<UsizeMaxMonoidWeight>, impls: &Vec<ImplDesc> ) {
	if helper.print == Print {
		println!( "Benchmarking {} unsigned-max queries on {} vertices",
			helper.queries.len(), helper.num_vertices );
		helper.print_query_type_dist();
	}
	
	macro_rules! do_benchmark_monoid {
		( $obj : ident, $impl_tpl : ident ) => {
			$obj.benchmark::<$impl_tpl<UsizeMaxMonoidWeight>>( <$impl_tpl<UsizeMaxMonoidWeight> as ImplName>::name() )
		}
	}
	
	for imp in impls {
		do_for_impl_monoid!( imp, do_benchmark_monoid, helper );
	}
}


/// Enum listing possible weight types.
#[derive( Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum )]
enum WeightType {
	/// No weights and thus no additional data stored per node
	Empty,
	
	/// Signed-add group weights, some strage and update overhead
	Group,
	
	/// Unsigned-max monoid weights, more strage and update overhead
	Monoid
}

impl Display for WeightType {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", match self {
			Self::Empty => "empty",
			Self::Group => "group",
			Self::Monoid => "monoid"
		} )
	}
}


#[derive(Parser)]
#[command(name = "Random query Benchmark")]
struct CLI {
	/// Number of vertices in the underlying graph
	#[arg(short, long, default_value_t = 100)]
	num_vertices : usize,
	
	/// Number of queries
	/// 
	/// [default: 20*NUM_VERTICES for uniform distribution,
	/// 10*(1/0.99)^NUM_VERTICES for geometric distribution]
	#[arg(short='q', long)]
	num_queries : Option<usize>,
	
	/// Probability of generating a path_weight query (instead of a cut) when querying two nodes in
	/// the same tree.
	#[arg(short='p', long, default_value_t = 0.5)]
	path_query_prob : f64,
	
	/// Node distribution to generate queries
	#[arg(short, long, default_value_t = NodeDistribution::Uniform)]
	dist : NodeDistribution,
	
	/// Input file to read queries from. Currently only supports connectivity queries. Ignores -p, -s, --dist.
	#[arg(short, long, group = "input")]
	input_file : Option<PathBuf>,
	
	/// Print the results in human-readable form
	#[arg(short, long, default_value_t = false)]
	print : bool,
	
	/// Output the results as json
	#[arg(short, long, default_value_t = false)]
	json : bool,
	
	/// Seed for the random query generator
	#[arg(short, long, group = "input")]
	seed : Option<u64>,
	
	/// What weights to use in the benchmark.
	#[arg(short, long, default_value_t = WeightType::Empty)]
	weight : WeightType,
	
	/// How often to repeat the benchmark
	#[arg(short, long, default_value_t = 1)]
	repeat: usize,
	
	/// Implementations to benchmark. Include all but petgraph if omitted.
	impls : Vec<ImplDesc>
}


fn main() {
	let cli = CLI::parse();
	
	let impls : Vec<ImplDesc>;
	if !cli.impls.is_empty() {
		impls = cli.impls.clone();
	}
	else {
		impls = ImplDesc::all_efficient()
	}
	
	match cli.weight {
		WeightType::Empty => benchmark_empty( &Helper::new( &cli ), &impls ),
		WeightType::Group => benchmark_group( &Helper::new( &cli ), &impls ),
		WeightType::Monoid => benchmark_monoid( &Helper::new( &cli ), &impls )
	}
}
