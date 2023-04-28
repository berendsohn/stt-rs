//! Dynamic forest implementations based on the petgraph library.

use std::iter::Map;
use std::ops::Range;

use petgraph::algo;
use petgraph::graph::{NodeIndex, UnGraph};

use crate::{DynamicForest, MonoidWeight, NodeIdx};
use crate::common::EmptyGroupWeight;


/// A petgraph-based dynamic forest without edge weights.
pub type EmptyPetgraphDynamicForest = PetgraphDynamicForest<EmptyGroupWeight>;


fn conv_idx( v : NodeIdx ) -> NodeIndex {
	NodeIndex::new( v.index() )
}


/// A straight-forward implementation of dynamic forests using the petgraph library.
pub struct PetgraphDynamicForest<TWeight : MonoidWeight> {
	g : UnGraph<(), TWeight>
}

impl<TWeight : MonoidWeight> DynamicForest for PetgraphDynamicForest<TWeight> {
	type TWeight = TWeight;
	
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_nodes : usize ) -> Self {
		let mut g = UnGraph::new_undirected();
		for i in 0..num_nodes {
			let v = g.add_node( () );
			assert_eq!( v.index(), i, "Unexpected petgraph index {}, expected {i}", v.index() );
		}
		PetgraphDynamicForest{ g }
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx, weight : TWeight ) {
		assert!( self.g.find_edge( conv_idx( u ), conv_idx( v ) ).is_none() );
		self.g.add_edge( conv_idx( u ), conv_idx( v ), weight );
	}
	
	fn cut( &mut self, u : NodeIdx, v : NodeIdx ) {
		self.g.remove_edge( self.g.find_edge( conv_idx( u ), conv_idx( v ) )
			.expect( "No edge: {u}, {b}" ) );
	}
	
	fn compute_path_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<TWeight> {
		let path : Option<Vec<_>> = algo::all_simple_paths( &self.g, conv_idx( u ), conv_idx( v ), 0, None ).next();
		if let Some( path ) = path {
			let mut total = TWeight::identity();
			
			for i in 0..(path.len()-1) {
				let e = self.g.find_edge( path[i], path[i+1] ).unwrap();
				total = total + *self.g.edge_weight( e ).unwrap();
			}
			
			Some( total )
		}
		else {
			None
		}
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.g.node_count()).map( |i| NodeIdx::new( i ) )
	}
	
	fn edges( &self ) -> Vec<(NodeIdx, NodeIdx)> {
		self.g.edge_indices().map( |e| self.g.edge_endpoints( e ).unwrap() )
			.map( |(u,v)| ( NodeIdx::new( u.index() ), NodeIdx::new( v.index() ) ) )
			.collect()
	}
}