from typing import *

from dataclasses import dataclass

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


def load_benchmarks( line : str ) -> Iterator[JsonObj] :
	obj = json.loads( line )
	assert isinstance( obj, dict )
	if "results" in obj :
		results = obj.pop( "results" )
		for result in results :
			yield {**obj, **result}
	else :
		yield obj


### Benchmark helper functions

def time_ms( benchmark : JsonObj ) -> float :
	return benchmark["time_ns"] / 1_000_000

def time_us( benchmark : JsonObj ) -> float :
	return benchmark["time_ns"] / 1_000

def edge_factor( benchmark : JsonObj ) -> int :
	return benchmark["num_edges"] // benchmark["num_vertices"]


def validate_queries_quadratic( benchmarks : List[JsonObj] ) :
	for benchmark in benchmarks :
		if benchmark["num_queries"] != benchmark["num_vertices"] **2 :
			print( f"WARNING: benchmark with m = {benchmark['num_queries']} and n = {benchmark['num_vertices']}. m is not the square of n." )

def validate_vertices_constant( benchmarks : List[JsonObj] ) :
	ns = {benchmark["num_vertices"] for benchmark in benchmarks}
	if len( ns ) > 1 :
		print( f"WARNING: Multiple vertex counts: {', '.join( sorted( ns ) )}")

def validate_edge_factor_constant( benchmarks : List[JsonObj] ) :
	fs = {edge_factor( benchmark ) for benchmark in benchmarks}
	if len( fs ) > 1 :
		print( f"WARNING: Multiple edge factors: {', '.join( sorted( fs ) )}")

def validate_edge_factor_int( benchmarks : List[JsonObj] ) :
	for benchmark in benchmarks :
		if benchmark["num_edges"] % benchmark["num_vertices"] != 0 :
			print( f"WARNING: benchmark with m = {benchmark['num_edges']} and n = {benchmark['num_verices']}. m not a multiple of n!" )
	

class XEdgeFactor :
	label = "m/n"
	log_scale = False
	
	@staticmethod
	def index( benchmark : JsonObj ) -> float :
		return benchmark["num_edges"] // benchmark["num_vertices"]

@dataclass
class XSimple :
	label : str
	index_key : str
	log_scale : bool = False
	
	def index( self, benchmark : JsonObj ) -> float :
		return benchmark[self.index_key]

XStdDev = XSimple( "σ", "std_dev" )
XNumGroups = XSimple( "#groups", "num_groups" )
XPathProb = XSimple( "p", "path_query_prob")

def XNumVerts( log_scale : bool = False ) -> XSimple :
	return XSimple( "n", "num_vertices", log_scale )


class YMicrosPerQuery :
	label = "µs/query"
	
	@staticmethod
	def value( benchmark : JsonObj ) -> float :
		return benchmark["time_ns"] / 1_000 / benchmark["num_queries"]

class YMicrosPerEdge :
	label = "µs/edge"
	
	@staticmethod
	def value( benchmark : JsonObj ) -> float :
		return benchmark["time_ns"] / 1_000 / benchmark["num_edges"]

class YMicrosPerVertex :
	label = "µs/vertex"
	
	@staticmethod
	def value( benchmark : JsonObj ) -> float :
		return benchmark["time_ns"] / 1_000 / benchmark["num_vertices"]

class YMillis :
	label = "ms"
	
	@staticmethod
	def value( benchmark : JsonObj ) -> float :
		return benchmark["time_ns"] / 1_000_000

class YRotationsPerQuery :
	label = "rots/query"

	@staticmethod
	def value( benchmark : JsonObj ) -> float :
		return benchmark["rotation_count"] / benchmark["num_queries"]

class TitleFixedVal :
	def __init__( self, tpl : str, val_func : Callable[[JsonObj], Any], val_name_plural : str ) :
		self._tpl = tpl
		self._val_func = val_func
		self._val_name_plural = val_name_plural
		
	def title( self, benchmarks : List[JsonObj] ) :
		assert len( benchmarks ) > 0
		vals = {self._val_func( benchmark ) for benchmark in benchmarks}
		if len( vals ) > 1 :
			print( f"WARNING: Multiple {self._val_name_plural}: {', '.join( sorted( map( str, vals ) ) )}")
			v = next( iter( vals ) )
			if not isinstance( v, (tuple, list) ) :
				v = (v,)
			return self._tpl.format( *(("?",) * len( v )) )
		else :
			v = next( iter( vals ) )
			if not isinstance( v, (tuple, list) ) :
				v = (v,)
			return self._tpl.format( *v )

def TitleFixedVertices( tpl : str ) -> TitleFixedVal :
	return TitleFixedVal( tpl, lambda b : b["num_vertices"], "vertex counts" )

def TitleFixedEdgeFactor( tpl : str ) -> TitleFixedVal :
	return TitleFixedVal( tpl, lambda b : b["num_edges"] // b["num_vertices"], "edge factors" )

def TitleFixedQueryFactor( tpl : str ) -> TitleFixedVal :
	return TitleFixedVal( tpl, lambda b : b["num_queries"] // b["num_vertices"], "query factors" )

def TitleFixedQueryFactorSquared( tpl : str ) -> TitleFixedVal :
	return TitleFixedVal( tpl, lambda b : b["num_queries"] // b["num_vertices"] ** 2, "q/n² factors" )

def TitleFixedGroupSizesAndQueries( tpl : str ) -> TitleFixedVal :
	return TitleFixedVal( tpl, lambda b : ( b["group_size"], b["queries_per_group"] ), "group sizes/queries" )
	

def plot_data( name : str, benchmark : Dict[int, List[int]], verbose : bool ) -> Tuple[List[float], List[float], List[float]] :
	xs = []
	ys = []
	stdevs = []
	for x in sorted( benchmark ) :
		results = benchmark[x]
		mean_us = statistics.mean( results )
		if len( results ) >= 2 :
			stdev_us = statistics.stdev( results )
		else :
			stdev_us = None
		xs.append( x )
		ys.append( mean_us )
		stdevs.append( stdev_us )
		if verbose :
			print( f"{name:>16}, {x:7}: {mean_us:5.3}±{stdev_us:4.3}ms")
	return xs, ys, stdevs

PROFILES = {
	"mst-edge-factor" : ( XEdgeFactor, YMicrosPerEdge,
			TitleFixedVertices( "Minimum Spanning forest (n = {})" ), lambda _ : True,
			validate_vertices_constant, validate_edge_factor_int ),
	"mst-vertices" : ( XNumVerts( log_scale = True ), YMicrosPerEdge,
			TitleFixedEdgeFactor( "Minimum Spanning forest (m/n = {})" ), lambda _ : True,
			validate_edge_factor_constant, validate_edge_factor_int ),
	"fd-con" : ( XNumVerts(), YMicrosPerQuery,
			TitleFixedQueryFactorSquared( "Fully-dynamic connectivity (q/n² = {})" ), lambda _ : True ),
	"degenerate" : ( XNumVerts(), YMicrosPerVertex, "Degenerate queries", lambda _ : True ),
	"degenerate-noisy" : ( XStdDev, YMillis, TitleFixedVertices( "Noisy degenerate queries (n = {})" ), lambda _ : True),
	"queries-uniform" : ( XNumVerts( log_scale = False ),
			YMicrosPerQuery, TitleFixedQueryFactor( "Uniform random queries (q/n = {})" ), lambda _ : True ),
	"queries-path-prob" : ( XPathProb, YMicrosPerQuery,
			TitleFixedVal( "Random queries (n = {}, q = {})", lambda b : ( b["num_vertices"], b["num_queries"] ), "vertices/queries" ),
			lambda _ : True ),
	"cache" : ( XNumGroups, YMicrosPerQuery,
			TitleFixedGroupSizesAndQueries( "Cache (n/group = {}, q/group = {})" ), lambda _ : True ),
	"lca" : ( XNumVerts( log_scale = False ),
			YMicrosPerQuery, TitleFixedQueryFactor( "Uniform LCA queries (q/n = {})" ), lambda _ : True ),
	"num_rotations" : ( XNumVerts(), YRotationsPerQuery,
			"Rotation count (q=n²)", lambda _ : True, validate_queries_quadratic )
}

ALGORITHM_COLORS = {
	"Petgraph" : "black",
	"Kruskal (petgraph)" : "black",
	"Link-cut" : "tab:brown",
	"Greedy Splay" : "tab:blue",
	"Stable Greedy Splay" : "tab:cyan",
	"2P Splay" : "tab:red",
	"Stable 2P Splay" : "tab:orange",
	"L2P Splay" : "tab:purple",
	"Stable L2P Splay" : "tab:pink",
	"MTR" : "tab:green",
	"Stable MTR" : "tab:olive",
	"1-cut" : "tab:gray",
	"Simple" : "tab:gray"
}

def main() :
	parser = argparse.ArgumentParser( description = "Parse stt benchmark results." )
	parser.add_argument( "--input-file", required = True )
	parser.add_argument( "--profile", choices = sorted( PROFILES.keys() ), required = True )
	parser.add_argument( "--output-file", help = "Where to write the resulting image. If omitted, shows the image instead", default = None )
	parser.add_argument( "--exclude", nargs="*", choices = sorted( ALGORITHM_COLORS.keys() ), help = "Exclude the specified algorithm(s)" )
	parser.add_argument( "--stdev", action = "store_true", help = "Show standard deviation error bars" )
	parser.add_argument( "-v", "--verbose", help = "Print results to stdout" )
	args = parser.parse_args()
	
	OUTPUT_FOR_PAPER = False # Whether to produce plots for the paper, or larger plots to be read separately
	
	print( f"Drawing plot from {args.input_file} with profile {args.profile}..." )
	
	try :
		x_profile, y_profile, title_profile, include_func, *validators = PROFILES[args.profile]
	except KeyError :
		print( f"ERROR: Unknown profile '{args.profile}'" )
		sys.exit( -1 )
	
	try :
		with open( args.input_file, "r" ) as fp :
			benchmarks = [
				b for line in fp for b in load_benchmarks( line )
				if include_func( b["name"] ) and b["name"] not in (args.exclude or ())
			]
	except OSError as e :
		sys.stderr.write( f"Could not open file '{args.input_file}': {e}\n" )
		sys.exit( 1 )
	
	if len( benchmarks ) == 0 :
		print( "No valid benchmarks found" )
		return
	
	for validator in validators :
		validator( benchmarks )
	
	benchmark_map = collections.defaultdict( lambda : collections.defaultdict( lambda : [] ) )
	for benchmark in benchmarks :
		x = x_profile.index( benchmark )
		benchmark_map[benchmark["name"]][x].append( y_profile.value( benchmark ) )
	benchmark_map = {k : dict( val ) for k, val in benchmark_map.items()}
	
	impls_with_plots = [(name, *plot_data( name, b, args.verbose ) ) for name, b in benchmark_map.items()]
	impls_with_plots.sort( key = lambda t : t[2][-1], reverse = True ) # Sort by last value
	
	linewidth = 1.5
	if args.output_file :
		if OUTPUT_FOR_PAPER :
			plt.figure( figsize = (4.5, 4) )
			linewidth = 1
		else :
			plt.figure( figsize = (11.69, 8.27) )  # A4
	
	max_y = 0
	for impl, xs, ys, stdevs in impls_with_plots :
		max_y = max( max_y, max( ys ) )
		print( impl, ys )
		if args.stdev and None not in stdevs :
			plt.errorbar( xs, ys, yerr = [stdevs, stdevs], capsize = 2, label = impl, color = ALGORITHM_COLORS[impl], linewidth = linewidth )
		else :
			plt.plot( xs, ys, label = impl, linewidth = linewidth )

	plt.xlabel( x_profile.label )
	if isinstance( title_profile, str ) :
		plt.title( title_profile )
	else :
		plt.title( title_profile.title( benchmarks ) )
	
	if x_profile.log_scale :
		plt.xscale( "log" )
	plt.ylabel( y_profile.label )
	if 50 <= max_y <= 100 :
		plt.yticks( range( 0, int( max_y+1 ), 5 ) )
	# Otherwise: default
	
	# Create legend
	if not OUTPUT_FOR_PAPER or args.output_file is None :
		plt.legend()

	plt.axis( ymin = 0 )
	
	if args.output_file is None :
		print( "Showing plot..." )
		plt.show()
	else :
		print( f"Saving plot to {args.output_file}..." )
		plt.savefig( args.output_file )
	print( "Done." )

if __name__ == "__main__" :
	main()
