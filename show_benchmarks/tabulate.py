from typing import *
from typing import TextIO

import collections
import json
import statistics
import sys

try :
	import argparse
except ImportError :
	sys.stderr.write( "argparse not installed!\n" )
	sys.exit( 2 )

try :
	import matplotlib.pyplot as plt
except ImportError :
	sys.stderr.write( "matplotlib not installed!\n" )
	sys.exit( 3 )


JsonObj = Dict[str, Any]

ALGORITHMS = [
	"Petgraph",
	"Kruskal (petgraph)",
	"Link-cut",
	"Greedy Splay",
	"Stable Greedy Splay",
	"2P Splay",
	"Stable 2P Splay",
	"L2P Splay",
	"Stable L2P Splay",
	"MTR",
	"Stable MTR",
	"1-cut"
]


### Benchmark helper functions

def benchmarks_from_line( line : str ) -> Iterator[JsonObj] :
	obj = json.loads( line )
	assert isinstance( obj, dict )
	if "results" in obj :
		results = obj.pop( "results" )
		for result in results :
			yield {**obj, **result}
	else :
		yield obj

def load_benchmarks( fp : TextIO, excluded_algorithms : Set[str] ) -> Iterator[JsonObj] :
	for line in fp :
		for b in benchmarks_from_line( line ) :
			if b["name"] not in ( excluded_algorithms or () ) :
				yield b

UNIT_DIVISORS = {
	"ms" : 1_000_000,
	"us" : 1_000,
	"ns" : 1,
	"rots" : 1
}

PER_VAL_KEYS = {
	"edge" : "num_edges",
	"query" : "num_queries",
	"vertex" : "num_vertices"
}

DEFAULT_KEY_TEMPLATES = {
	"num_edges" : "$m = {}$",
	"num_queries" : "$m = {}$",
	"num_vertices" : "$n = {}$"
}



def main() :
	parser = argparse.ArgumentParser( description = "Convert stt benchmark results into a table." )
	parser.add_argument( "--input-file", required = True )
	parser.add_argument( "--key", required = True )
	parser.add_argument( "--key-template", default = None )
	parser.add_argument( "--value", choices = ["time_ns", "rotation_count"], default="time_ns" )
	parser.add_argument( "--value-unit", choices = sorted( UNIT_DIVISORS.keys() ), required = True )
	parser.add_argument( "--value-per", choices = [None, "edge", "query", "vertex"], default = None )
	parser.add_argument( "--decimal-places", type = int, required = True )
	parser.add_argument( "--exclude", nargs="*", choices = sorted( ALGORITHMS ), help ="Exclude the specified algorithm(s)" )
	parser.add_argument( "--exclude-key", nargs="*", help ="Exclude the specified key(s)" )
	# ~ parser.add_argument( "-v", "--verbose", help = "Print results to stdout" )
	parser.add_argument( "--plain", action = "store_true", help = "Output simple non-latex text" )
	parser.add_argument( "--stdev", action = "store_true", help = "Include standard deviation" )
	parser.add_argument( "--range", type = float, default = None,
		help = "Instead of '±σ', display a range between x-rσ and x+rσ, where x is the mean and r is the provided value" )
	args = parser.parse_args()
	
	# ~ print( f"Creating table from {args.input_file} with key {args.key}..." )
	
	if ( args.value == "time_ns" ) != ( args.value_unit in ["ms", "us", "ns"] ) :
		parser.error( f"Value '{args.value}' and unit '{args.value_unit}' incompatible" )
	if args.range and not args.stdev :
		parser.error( "--range requires --stdev" )
	if args.key_template is None :
		try :
			args.key_template = DEFAULT_KEY_TEMPLATES[args.key]
		except KeyError :
			parser.error( "Cannot derive key template from key '{args.key}', please specify --key-template" )
	
	with open( args.input_file, "r" ) as fp :
		benchmarks = list( load_benchmarks( fp, args.exclude ) )
	
	key_func = lambda b : b[args.key]
	keys = sorted( set( key_func( b ) for b in benchmarks if str( key_func( b ) ) not in ( args.exclude_key or () ) ) )
	
	if args.value_per is not None :
		value_func = lambda b : b[args.value] / UNIT_DIVISORS[args.value_unit] / b[PER_VAL_KEYS[args.value_per]]
	else :
		value_func = lambda b : b[args.value] / UNIT_DIVISORS[args.value_unit]
	value_unit_str = {"us" : r"\textmu{}s", "rots" : "rotations"}.get( args.value_unit, args.value_unit )
	if args.value_per is not None :
		value_unit_str += "/" + args.value_per
	value_format = "{:.%sf}" % args.decimal_places
	
	# Map by algorithm/key
	benchmark_map = collections.defaultdict( lambda : collections.defaultdict( lambda : [] ) )
	for benchmark in benchmarks :
		key = key_func( benchmark )
		benchmark_map[benchmark["name"]][key].append( benchmark )
	benchmark_map = {k : dict( val ) for k, val in benchmark_map.items()}
	
	used_algorithms = set( benchmark_map.keys() )
	algorithms = list( a for a in ALGORITHMS if a in used_algorithms ) # Preserve algorithm sorting
	
	# For plain display
	def display_values( values : List[float] ) -> str :
		mean = statistics.mean( values )
		if args.stdev and len( values ) >= 2 :
			stdev = statistics.stdev( values )
			if args.range is not None :
				assert args.range > 0
				minval = mean - args.range * stdev
				maxval = mean + args.range * stdev
				return value_format.format( minval ) + "-" + value_format.format( maxval )
			else :
				return value_format.format( mean ) + "±" + value_format.format( stdev )
		else :
			return value_format.format( mean )
				
	
	if args.plain :
		algo_len = max( len( algo ) for algo in algorithms )
		algo_fmt = "{:%d}" % (algo_len+2)
		for key in keys :
			print( f"{args.key}: {key}" )
			for algo in algorithms :
				values = list( map( value_func, benchmark_map[algo][key] ) )
				sys.stdout.write( "  " + algo_fmt.format( algo + ":" ) + display_values( values ) + "\n" )
				
	else :
		sys.stdout.write( "\t" + r"\begin{tabular}{l%s}" % ("c" * len( keys ) ) + "\n" )
		sys.stdout.write( "\t\t" + r"\toprule" + "\n" )
		sys.stdout.write( "\t\t" + r"& \multicolumn{%d}{c}{" % len( keys ) + "\n" )
		if args.value_unit == "rots" :
			sys.stdout.write( "Number of %s" % value_unit_str )
		else :
			sys.stdout.write( "Running time (%s)" % value_unit_str )
		if args.stdev :
			sys.stdout.write( " with standard deviation" )
		sys.stdout.write( r"}\\" )
		sys.stdout.write( "\t\t" + r"\cmidrule{2-%d}Algorithm" % ( len( keys ) + 1 ) )
		for key in keys :
			sys.stdout.write( " & " + args.key_template.format( key ) )
		sys.stdout.write( r"\\" + "\n" )
		sys.stdout.write( "\t\t" + r"\midrule" + "\n" )
		for algo in algorithms :
			sys.stdout.write( "\t\t" + algo )
			for key in keys :
				sys.stdout.write( " & " )
				values = list( map( value_func, benchmark_map[algo][key] ) )
				mean = statistics.mean( values )
				sys.stdout.write( "$" + value_format.format( mean ) )
				if args.stdev and len( values ) >= 2 :
					stdev = statistics.stdev( values )
					sys.stdout.write( r"\pm" + value_format.format( stdev ) )
				sys.stdout.write( "$" )
			sys.stdout.write( r"\\" + "\n" )
		
		sys.stdout.write( "\t\t" + r"\bottomrule" + "\n" )
		sys.stdout.write( "\t" + r"\end{tabular}" + "\n" )

if __name__ == "__main__" :
	main()
