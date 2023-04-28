use std::cmp::{max, min};
use std::iter::zip;
use std::ops::Add;
use itertools::Itertools;
use petgraph;
use petgraph::graph;

use stt::{NodeDataAccess, PathWeightNodeData};
use stt::{DynamicForest, MonoidWeight, NodeData, NodeIdx, RootedForest};
use stt::twocut::basic::{make_1_cut, STT};

/// Check if the two given trees have the same underlying tree
#[allow(dead_code)]
pub fn is_same_underlying_tree<TData : NodeData>(t1 : &STT<TData>, t2 : &STT<TData> ) -> bool {
	let mut t1 = t1.clone();
	let mut t2 = t2.clone();
	make_1_cut( &mut t1 );
	make_1_cut( &mut t2 );

	fn sort_edge( e : (NodeIdx, NodeIdx) ) -> (NodeIdx, NodeIdx ) {
		let (u,v) = e;
		if u <= v {
			(u,v)
		}
		else {
			(v,u)
		}
	}

	let edges1 : Vec<(NodeIdx,NodeIdx)> = t1.edges().map( sort_edge ).sorted().collect();
	let edges2 : Vec<(NodeIdx,NodeIdx)> = t2.edges().map( sort_edge ).sorted().collect();

	edges1 == edges2
}

#[allow(dead_code)]
pub struct DynamicTestForest<TDynForest : DynamicForest> {
	pub df : TDynForest,
	pub g : graph::UnGraph<(), TDynForest::TWeight>,
	verbose : bool,
	g_nodes : Vec<graph::NodeIndex>
}

impl<TDynForest : DynamicForest> DynamicTestForest<TDynForest> {
	#[allow(dead_code)]
	pub fn new( num_vertices : usize, verbose : bool ) -> Self {
		let df = TDynForest::new( num_vertices );
		let mut g = graph::UnGraph::new_undirected();
		let g_nodes= (0..num_vertices).map( |_| g.add_node( () ) ).collect();

		DynamicTestForest { df, g, g_nodes, verbose }
	}

	#[allow(dead_code)]
	pub fn df_node( &self, v : usize ) -> NodeIdx {
		NodeIdx::new( v )
	}

	#[allow(dead_code)]
	pub fn g_node( &self, v : usize ) -> graph::NodeIndex {
		self.g_nodes[v]
	}

	#[allow(dead_code)]
	pub fn g_edge( &self, u : usize, v : usize ) -> graph::EdgeIndex {
		self.g.find_edge( self.g_node( u ), self.g_node( v ) ).unwrap()
	}

	#[allow(dead_code)]
	pub fn g_edge_from_df_edge( &self, e : (NodeIdx, NodeIdx) ) -> graph::EdgeIndex {
		self.g_edge( e.0.index(), e.1.index() )
	}

	#[allow(dead_code)]
	pub fn add_edge( &mut self, u : usize, v : usize, weight : TDynForest::TWeight ) {
		if self.verbose { println!( "Adding edge {u},{v} with weight {weight}" ) }
		self.df.link( self.df_node( u ), self.df_node( v ), weight );
		self.g.add_edge( self.g_node( u ), self.g_node( v ), weight );
	}

	#[allow(dead_code)]
	pub fn remove_edge( &mut self, u : usize, v : usize ) {
		if self.verbose { println!( "Removing edge {u},{v}" ) }
		self.df.cut( self.df_node( u ), self.df_node( v ) );
		self.g.remove_edge( self.g.find_edge( self.g_node( u ), self.g_node( v ) ).unwrap() );
	}

	#[allow(dead_code)]
	pub fn df_compute_path_weight( &mut self, u : usize, v : usize ) -> Option<TDynForest::TWeight> {
		self.df.compute_path_weight( self.df_node( u ), self.df_node( v ) )
	}

	#[allow(dead_code)]
	pub fn g_get_path_edges( &self, u : usize, v : usize ) -> Option<Vec<graph::EdgeIndex>> {
		let (u_g, v_g) = ( self.g_node( u ), self.g_node( v ) );

		// There is one or no path between u and v
		let path : Option<Vec<graph::NodeIndex>> = petgraph::algo::all_simple_paths(
				&self.g, u_g, v_g, 0, None ).next();
		if let Some( path ) = path {
			Some( zip( &path, &path[1..] )
				.map( |(x,y)| self.g.find_edge( *x, *y ).unwrap() )
				.collect()
			)
		}
		else {
			None
		}
	}

	#[allow(dead_code)]
	pub fn g_compute_path_weight( &self, u : usize, v : usize ) -> Option<TDynForest::TWeight> {
		if let Some( edges ) = self.g_get_path_edges( u, v ) {
			Some( edges.iter()
				.map( |e| *self.g.edge_weight( *e ).unwrap() )
				.fold( TDynForest::TWeight::identity(), TDynForest::TWeight::add )
			)
		}
		else {
			None
		}
	}
	
	#[allow(dead_code)]
	pub fn check_path_weight( &mut self, u : usize, v : usize ) {
		assert_eq!( self.df_compute_path_weight( u, v ), self.g_compute_path_weight( u, v ) );
	}
	
	#[allow(dead_code)]
	pub fn check_edges( &self ) {
		let g_edges : Vec<(usize,usize)> = self.g.edge_indices()
			.map( |e| self.g.edge_endpoints( e ).unwrap() )
			.map( |(u,v)| (u.index(), v.index()) )
			.map( |(u,v,)| ( min(u,v), max(u,v) ) ).sorted().collect();
		
		let df_edges : Vec<(usize, usize)> = self.df.edges().into_iter()
			.map( |(u,v)| (u.index(), v.index()) )
			.map( |(u,v,)| ( min(u,v), max(u,v) ) ).sorted().collect();
		
		if self.verbose {
			println!( "Graph edges: {g_edges:?}" );
			println!( "DF edges:    {df_edges:?}" );
		}
		assert_eq!( g_edges, df_edges );
	}
}

#[allow(dead_code)]
pub fn check_all_parent_paths<TDynForest, TNodeData>( dtf : &DynamicTestForest<TDynForest> )
	where TDynForest : DynamicForest + RootedForest + NodeDataAccess<TNodeData>,
		TNodeData : PathWeightNodeData<TWeight=TDynForest::TWeight>
{
	for v in dtf.df.nodes() {
		if let Some( p ) = dtf.df.get_parent( v ) {
			let pdist = dtf.df.data( v ).get_parent_path_weight();
			assert_eq!( pdist, dtf.g_compute_path_weight( v.index(), p.index() ).unwrap() );
		}
	}
}
