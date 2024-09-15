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
	"1-cut",
	"Petgraph",
	"Simple",
	"./stt-cpp/bin/greedy_stt",
	"./stt-cpp/bin/ltp_stt",
	"./stt-cpp/bin/mtr_stt",
	"./dtree/dtree_queries",
	"./tarjan-werneck/connectivity_st_v",
	"./tarjan-werneck/connectivity_st_e"
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


def int_to_nice_str( i : int ) -> str :
	s = str( i )
	if len( s ) <= 4 :
		return s
	else :
		return "".join( c + r"\," if i % 3 == ( len( s ) - 1 ) % 3 and i < len( s ) - 1 else c for i, c in enumerate ( s ) )


def main() :
	parser = argparse.ArgumentParser( description = "Convert stt benchmark results into a table." )
	parser.add_argument( "--input-file", required = True )
	
	data_group = parser.add_argument_group( "Data retrieval")
	data_group.add_argument( "--key", required = True )
	data_group.add_argument( "--value", choices = ["time_ns", "rotation_count"], default="time_ns" )
	data_group.add_argument( "--value-unit", choices = sorted( UNIT_DIVISORS.keys() ), required = True )
	data_group.add_argument( "--value-per", choices = ["edge", "query", "vertex"], default = None )
	
	parser.add_argument( "--format", choices = ["latex", "plain", "pgfdata"], default="plain" )
	
	parser.add_argument( "--decimal-places", type = int, default = 2)
	parser.add_argument( "--latex-key-template", default = None )
	# parser.add_argument( "--algorithm", choices = sorted( ALGORITHMS ), help = "Only use the specified algorithm" )
	parser.add_argument( "--exclude", nargs="*", choices = sorted( ALGORITHMS ), help = "Exclude the specified algorithm(s)" )
	parser.add_argument( "--rename", nargs="*", help = "Rename algorithms; format: 'key:val'" )
	parser.add_argument( "--exclude-key", nargs="*", help = "Exclude the specified key(s)" )
	parser.add_argument( "--stdev", action = "store_true", help = "Include standard deviation" )
	parser.add_argument( "--range", type = float, default = None,
		help = "Instead of '±σ', display a range between x-rσ and x+rσ, where x is the mean and r is the provided value" )
	args = parser.parse_args()
	
	# Basic validation
	if ( args.value == "time_ns" ) != ( args.value_unit in ["ms", "us", "ns"] ) :
		parser.error( f"Value '{args.value}' and unit '{args.value_unit}' incompatible" )
	if args.range and not args.stdev :
		parser.error( "--range requires --stdev" )
	if args.latex_key_template is not None and args.format != "latex" :
		parser.error( "--latex-key-template requires --format latex" )
	# if args.algorithm is not None and args.exclude :
	# 	parser.error( "Cannot have both --algorithm and --exclude" )
	# if args.format == "pgfdata" and args.algorithm is None :
	# 	parser.error( "--format pgfdata requires --algorithm" )
	
	# Try to derive latex key template, if not given
	if args.format == "latex" and args.latex_key_template is None :
		try :
			args.latex_key_template = DEFAULT_KEY_TEMPLATES[args.key]
		except KeyError :
			parser.error( f"Cannot derive key template from key '{args.key}', please specify --latex-key-template" )
	
	# Load benchmarks
	try :
		with open( args.input_file, "r" ) as fp :
			benchmarks = list( load_benchmarks( fp, args.exclude ) )
	except FileNotFoundError :
		sys.exit( f"Error: File not found: '{args.input_file}'" )
	
	# Retrieve keys
	key_func = lambda b : b[args.key]
	keys = sorted( set( key_func( b ) for b in benchmarks if str( key_func( b ) ) not in ( args.exclude_key or () ) ) )
	
	# Value retrieval and formatting
	if args.format == "plain" :
		value_unit_map = {"us" : r"µs", "rots" : "rotations"}
	else :
		value_unit_map = {"us" : r"\textmu{}s", "rots" : "rotations"}
	
	if args.value_per is not None :
		value_func = lambda b : b[args.value] / UNIT_DIVISORS[args.value_unit] / b[PER_VAL_KEYS[args.value_per]]
	else :
		value_func = lambda b : b[args.value] / UNIT_DIVISORS[args.value_unit]
	value_unit_str = value_unit_map.get( args.value_unit, args.value_unit )
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
	algorithms = list( a for a in ALGORITHMS if a in used_algorithms ) \
			+ list( a for a in used_algorithms if a not in ALGORITHMS) # Preserve algorithm sorting
	
	# For plain display
	def display_mean( values : List[float] ) -> Optional[str] :
		if len( values ) == 0 :
			assert args.format != "pgfdata"
			if args.format == "plain" :
				return None
			else :
				assert args.format == "latex"
				return "--"
		
		mean = statistics.mean( values )
		if args.format == "plain" :
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
		elif args.format == "latex" :
			result = "$" + value_format.format( mean )
			if args.stdev and len( values ) >= 2 :
				stdev = statistics.stdev( values )
				result += r"\pm" + value_format.format( stdev )
			result += "$"
			return result
		else :
			assert args.format == "pgfdata"
			return value_format.format( mean )
	
	# Values by key, algo
	value_dict = {
		key : { algo : display_mean( list( map( value_func, benchmark_map[algo].get( key, [] ) ) ) ) for algo in algorithms }
		for key in keys
	}
	
	algo_names = { k : v for k, v in map( lambda s : s.split( ":" ), args.rename or () ) }		
	
	if args.format == "plain" :
		algo_len = max( len( algo ) for algo in algorithms )
		algo_fmt = "{:%d}" % (algo_len+2)
		for key in keys :
			print( f"{args.key}: {key}" )
			
			val_post_fmt = "{:>%d}" % max( len( val ) for val in value_dict[key].values() if val is not None )
			
			for algo in algorithms :
				val = value_dict[key][algo]
				if val is not None :
					algo_str = algo_fmt.format( algo + ":" )
					val_str = val_post_fmt.format( val )
					sys.stdout.write( f"  {algo_str} {val_str} {value_unit_str}\n" )	
	elif args.format == "latex" :
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
		for idx, key in enumerate( keys ) :
			key = int_to_nice_str( key )
			if idx == 0 :
				sys.stdout.write( " & " + args.latex_key_template.format( key ) )
			else :
				sys.stdout.write( f" & ${key}$" )
		sys.stdout.write( r"\\" + "\n" )
		sys.stdout.write( "\t\t" + r"\midrule" + "\n" )
		for algo in algorithms :
			algo_name = algo_names.get( algo, algo )
			if algo_name.startswith( "Kruskal" ) : # TODO: Hack
				sys.stdout.write( f"\t\t{algo_name}" )
			else :
				sys.stdout.write( f"\t\t\\dsimpl{{{algo_name}}}" )
			for key in keys :
				sys.stdout.write( " & " )
				sys.stdout.write( value_dict[key][algo] )
			sys.stdout.write( r"\\" + "\n" )
		
		sys.stdout.write( "\t\t" + r"\bottomrule" + "\n" )
		sys.stdout.write( "\t" + r"\end{tabular}" + "\n" )
	else :
		assert args.format == "pgfdata"
		print( "value " + " ".join( algo.replace( " ", "_" ) for algo in algorithms ) )
		for key in keys :
			print( str( key ) + " " + " ".join( value_dict[key][algo] for algo in algorithms ) )
		sys.stdout.write( "\n" )

if __name__ == "__main__" :
	main()
