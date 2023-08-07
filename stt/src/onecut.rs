//! A very simple dynamic forest implementation.
//! 
//! Maintains each tree as a one-cut STT, i.e., a rooting of that tree.
//! The root of a tree can be changed by repeatedly rotating the root with one of its children.
//! 
//! The operations [`link()`](DynamicForest::link()), [`cut()`](DynamicForest::cut()), and
//! [`compute_path_weight()`](DynamicForest::compute_path_weight()) are guaranteed to run in
//! O(n) amortized time, where `n` is the number of nodes in the forest.

use std::fmt::{Display, Formatter};
use std::iter::Map;
use std::ops::Range;
use crate::common::{EmptyGroupWeight, WeightOrInfinity};
use crate::common::WeightOrInfinity::{Finite, Infinite};
use crate::{DynamicForest, MonoidWeight, NodeData, NodeIdx};
use crate::NodeDataAccess;


/// A [SimpleDynamicTree] with no edge weights.
pub type EmptySimpleDynamicTree = SimpleDynamicTree<EmptyGroupWeight>;


/// Node data for [SimpleDynamicTree].
#[derive(Clone)]
pub struct SimpleParentWeightNodeData<TWeight : MonoidWeight> {
	pdist : WeightOrInfinity<TWeight>
}

impl<TWeight : MonoidWeight> SimpleParentWeightNodeData<TWeight> {
	fn new() -> SimpleParentWeightNodeData<TWeight> {
		SimpleParentWeightNodeData{ pdist : Infinite }
	}
}

impl<TWeight: MonoidWeight> Display for SimpleParentWeightNodeData<TWeight> {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.pdist )
	}
}

impl<TWeight : MonoidWeight> NodeData for SimpleParentWeightNodeData<TWeight> {
	type TWeight = TWeight;
	
	fn new( _ : NodeIdx ) -> Self {
		SimpleParentWeightNodeData{ pdist : Infinite }
	}
}

/// An (internal) node
struct Node<TWeight : MonoidWeight> {
	parent : Option<NodeIdx>,
	data : SimpleParentWeightNodeData<TWeight>
}

impl<TWeight : MonoidWeight> Node<TWeight> {
	fn new() -> Node<TWeight> {
		Node{ parent : None, data : SimpleParentWeightNodeData::new() }
	}
}


/// A dynamic forest implementation that explicitly maintains each tree in the forest.
/// 
/// Each tree is represented by a rooting of itself (i.e., a 1-cut STT on itself). The root is
/// changed by rotating at the root, which maintains the 1-cut property.
pub struct SimpleDynamicTree<TWeight : MonoidWeight> {
	nodes : Vec<Node<TWeight>>
}

impl<TWeight : MonoidWeight> SimpleDynamicTree<TWeight> {
	fn node( &self, idx : NodeIdx ) -> &Node<TWeight> {
		&self.nodes[idx.index()]
	}

	fn node_mut( &mut self, idx : NodeIdx ) -> &mut Node<TWeight> {
		&mut self.nodes[idx.index()]
	}

	fn rotate(&mut self, v: NodeIdx ) {
		let p = self.node( v ).parent.unwrap();
		debug_assert!( self.node( p ).parent.is_none() );
		self.node_mut( p ).parent = Some(v);
		self.data_mut( p ).pdist = self.data( v ).pdist;
		self.node_mut(v ).parent = None;
		self.data_mut( v ).pdist = Infinite;
	}

	fn move_to_root( &mut self, v : NodeIdx ) {
		if let Some( p ) = self.node(v ).parent {
			self.move_to_root( p );
			self.rotate( v );
		}
	}
}

impl<TWeight: MonoidWeight> NodeDataAccess<SimpleParentWeightNodeData<TWeight>> for SimpleDynamicTree<TWeight> {
	fn data(&self, idx: NodeIdx) -> &SimpleParentWeightNodeData<TWeight> {
		&self.nodes[idx.index()].data
	}

	fn data_mut(&mut self, idx: NodeIdx) -> &mut SimpleParentWeightNodeData<TWeight> {
		&mut self.nodes[idx.index()].data
	}
}

impl<TWeight : MonoidWeight> DynamicForest for SimpleDynamicTree<TWeight> {
	type TWeight = TWeight;
	
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_vertices: usize ) -> Self {
		Self{ nodes : (0..num_vertices).map( |_| Node::new() ).collect() }
	}

	fn link( &mut self, u : NodeIdx, v : NodeIdx, weight : TWeight ) {
		self.move_to_root( u );
		self.node_mut( u ).parent = Some( v );
		self.data_mut( u ).pdist = Finite( weight );
	}

	fn cut( &mut self, u : NodeIdx, v : NodeIdx ) {
		if self.node( u ).parent == Some( v ) {
			self.node_mut( u ).parent = None;
			self.data_mut( u ).pdist = Infinite;
		}
		else {
			assert_eq!( self.node( v ).parent, Some( u ) );
			self.node_mut( v ).parent = None;
			self.data_mut( v ).pdist = Infinite;
		}
	}

	fn compute_path_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<TWeight> {
		self.move_to_root( u );
		let mut total_weight: TWeight = TWeight::identity();
		let mut x = v;
		while let Some( p ) = self.node( x ).parent {
			if let Finite( w ) = self.data( x ).pdist {
				total_weight = total_weight + w;
				x = p;
			}
			else {
				panic!()
			}
		}
		if x == u {
			Some( total_weight )
		}
		else {
			None
		}
	}

	fn get_edge_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<Self::TWeight> {
		if self.node( u ).parent == Some( v ) {
			Some( self.data( u ).pdist.unwrap() )
		}
		else if self.node( v ).parent == Some( u ){
			Some( self.data( v ).pdist.unwrap() )
		}
		else {
			None
		}
	}

	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.nodes.len()).map( |i| NodeIdx::new( i ) )
	}
	
	fn edges( &self ) -> Vec<(NodeIdx, NodeIdx)> {
		self.nodes().filter_map( |v| {
			match self.node( v ).parent {
				Some( p ) => Some( (v, p) ),
				None => None
			}
		} ).collect()
	}
}
