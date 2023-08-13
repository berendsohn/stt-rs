//! Fully-dynamic connectivity heuristic using dynamic forests.

use std::collections::BTreeSet;
use crate::common::EmptyGroupWeight;
use crate::{DynamicForest, MonoidWeight, NodeIdx};


type Edge = (NodeIdx, NodeIdx);


/// A data structure maintaining a spanning forest on a graph under edge insertions and deletions.
///
/// Simplification of the naive fully-dynamic MSF algorithm sketched in Cattaneo, Faruolo, Petrillo, and Italiano
/// ([2010](https://doi.org/10.1016/j.dam.2009.10.005)).
pub struct FullyDynamicConnectivity<TDynForest>
	where TDynForest : DynamicForest<TWeight = EmptyGroupWeight>
{
	df : TDynForest,
	unused_edges : BTreeSet<Edge>
}

impl<TDynForest> FullyDynamicConnectivity<TDynForest>
	where TDynForest : DynamicForest<TWeight = EmptyGroupWeight>
{
	/// Create a new instance
	pub fn new( num_vertices : usize ) -> Self {
		Self {
			df : TDynForest::new( num_vertices ),
			unused_edges : BTreeSet::new()
		}
	}
	
	/// Indicates whether the current graph contains a path between `u` and `v`.
	pub fn check_connected( &mut self, u : NodeIdx, v : NodeIdx ) -> bool {
		self.df.compute_path_weight( u, v ).is_some()
	}
	
	/// Insert the given (undirected) edge. Assumes the edge is not present currently.
	pub fn insert_edge( &mut self, u : NodeIdx, v : NodeIdx ) {
		if self.check_connected( u, v ) {
			self.unused_edges.insert( (u, v) );
		}
		else {
			self.df.link( u, v, EmptyGroupWeight::identity() );
		}
	}
	
	/// Deletes the given (undirected) edge. Assumes the edge is present currently.
	pub fn delete_edge( &mut self, u : NodeIdx, v : NodeIdx ) {
		if ! self.unused_edges.remove( &(u,v) ) {
			// (u,v) must be in forest
			self.df.cut( u, v );
			if let Some( (x, y) ) = self.find_usable_edge() {
				self.df.link( x, y, EmptyGroupWeight::identity() );
				self.unused_edges.remove( &(x,y) );
			}
		}
	}
	
	fn find_usable_edge( &mut self ) -> Option<Edge> {
		for &e in &self.unused_edges {
			let (u, v) = e;
			if self.df.compute_path_weight( u, v ).is_none() { // The borrow checker does not allow check_connected() here.
				return Some( e )
			}
		}
		None
	}
}
