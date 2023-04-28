//! Incremental online MST algorithm using dynamic forests.

use crate::common::{UnsignedMaxMonoidWeight, UsizeMaxMonoidWeightWithMaxEdge};
use crate::DynamicForest;
use crate::NodeIdx;

const LOG_VERBOSE : bool = cfg!( feature = "verbose_mst" );


/// Tuple of the form `(u, v, weight)`
pub type EdgeWithWeight = (usize, usize, usize);


// Helper function
fn link_with_weight(f : &mut impl DynamicForest<TWeight = UsizeMaxMonoidWeightWithMaxEdge>, u : NodeIdx, v : NodeIdx, weight : usize ) {
	f.link( u, v, UsizeMaxMonoidWeightWithMaxEdge::new(UnsignedMaxMonoidWeight::new( weight ), (u, v) ) );
}


/// Compute the minimum spanning tree of the given edges, online.
/// 
/// `f` may already contain edges, which are assumed to be a starting MST.
pub fn compute_mst<TDynForest>( f : &mut TDynForest, edges : impl Iterator<Item=EdgeWithWeight> )
		-> Vec<(NodeIdx, NodeIdx)>
	where TDynForest : DynamicForest<TWeight = UsizeMaxMonoidWeightWithMaxEdge>
{
	for (u, v, uv_weight) in edges {
		let u = NodeIdx::new( u );
		let v = NodeIdx::new( v );
		let path_weight = f.compute_path_weight( u, v );
		if LOG_VERBOSE { println!( "Processing edge ({u}, {v}) with weight {uv_weight}")}
		if let Some( path_weight ) = path_weight {
			if uv_weight < path_weight.weight().value() {
				// If there is a heavier edge on the path, swap it with {u,v}
				let (x, y) = path_weight.unwrap_edge();
				f.cut( x, y );
				link_with_weight( f, u, v, uv_weight );
			}
			// Otherwise, {u,v} is to heavy to be of any use.
		}
		else {
			link_with_weight( f, u, v, uv_weight );
		}
	}
	f.edges()
}
