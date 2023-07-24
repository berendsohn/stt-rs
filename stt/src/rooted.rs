//! Extension of dynamic trees to maintain unweighted rooted forests.

use std::collections::HashSet;
use std::iter::Map;
use std::ops::Range;
use crate::NodeIdx;

/// An unweighted rooted dynamic forest.
pub trait RootedDynamicForest {
	/// Iterator for nodes
	type NodeIdxIterator : Iterator<Item = NodeIdx>;
	
	/// Creates a new dynamic forest with the specified number of nodes and no edges.
	fn new( num_nodes : usize ) -> Self;

	/// Iterate over the nodes in this dynamic forest.
	fn nodes( &self ) -> Self::NodeIdxIterator;

	/// Adds `u` as a child to `v`. `u` must not have a parent, and must be in a different tree than
	/// `v`.
	fn link( &mut self, u : NodeIdx, v : NodeIdx );

	/// Removes `u` from its parent. `u` must have a parent.
	fn cut( &mut self, v : NodeIdx );

	/// Removes the edge between `u` and `v`. `u` must be a child of `v`.
	/// 
	/// Note: Supplying the parent (`u`) makes this function much easier for STT-based implementations
	fn cut_edge( &mut self, u : NodeIdx, v : NodeIdx );
	
	/// Return the root of the tree containing `v`.
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx;
	
	/// Returns the lowest common ancestor of `u` and `v`, or `None` if `u` and `v` are in different
	/// trees.
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx>;
}


struct SimpleRootedNode {
	parent : Option<NodeIdx>
}

impl SimpleRootedNode {
	fn new() -> Self {
		Self{ parent: None }
	}
}

/// A naive implementation of the [RootedDynamicForest] trait
pub struct SimpleRootedForest {
	nodes : Vec<SimpleRootedNode>
}

impl SimpleRootedForest {
	fn node( &self, v : NodeIdx ) -> &SimpleRootedNode {
		&self.nodes[v.index()]
	}
	
	fn node_mut( &mut self, v : NodeIdx ) -> &mut SimpleRootedNode {
		&mut self.nodes[v.index()]
	}
}

impl RootedDynamicForest for SimpleRootedForest {
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_nodes : usize ) -> Self {
		SimpleRootedForest{ nodes : (0..num_nodes).map( |_| SimpleRootedNode::new() ).collect() }
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.nodes.len()).map( NodeIdx::new )
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		debug_assert!( self.node( u ).parent == None );
		self.node_mut( u ).parent = Some( v );
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		self.node_mut( v ).parent = None;
	}
	
	fn cut_edge( &mut self, u: NodeIdx, v: NodeIdx ) {
		debug_assert!( self.node( u ).parent == Some( v ) );
		self.cut( u );
	}
	
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		let mut x = v;
		loop {
			if let Some( p ) = self.node( x ).parent {
				x = p;
			}
			else {
				return x;
			}
		}
	}
	
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		let mut u_ancs : HashSet<NodeIdx> = HashSet::new();
		let mut x = u;
		u_ancs.insert( x );
		while let Some( p ) = self.node( x ).parent {
			x = p;
			u_ancs.insert( x );
		}
		
		if u_ancs.contains( &v ) {
			return Some( v );
		}
		
		let mut x = v;
		while let Some( p ) = self.node( x ).parent {
			x = p;
		
			if u_ancs.contains( &x ) {
				return Some( x );
			}
		}
		
		return None;
	}
}