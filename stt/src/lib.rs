/*!
Data structures to maintain dynamic trees, based on *search trees on trees*.

This crate provides multiple implementations of *dynamic trees*, or rather collections of
dynamic trees, a.k.a. *dynamic forests*. Dynamic forests allow adding and removing edges (as
long as no cycle is created). This crate additionally allows maintaining edge weights and
querying the total weight of a path. This create does not support adding or removing nodes.
More details are found in the documentation of the trait [DynamicForest].

Most dynamic tree implementations in this crate are based on *search trees on trees* as defined
in [\[BK22\]](https://doi.org/10.1137/1.9781611977073.75).


# Examples

```
use stt::{DynamicForest, MonoidWeight, NodeData};
use stt::common::EmptyGroupWeight;
use stt::twocut::splaytt::EmptyLocalTwoPassSplayTT;

// Create a new dynamic tree with no edge weights
let mut t = EmptyLocalTwoPassSplayTT::new( 4 );

// Get the four nodes
if let [u,v,w,x] = t.nodes().collect::<Vec<_>>()[..] {
	// Add two edges (with no weights)
	t.link( u, v, EmptyGroupWeight::identity() );
	t.link( v, w, EmptyGroupWeight::identity() );

	// Check connectivity
	assert!( t.compute_path_weight( u, v ).is_some() );
	assert!( t.compute_path_weight( u, w ).is_some() );
	assert!( t.compute_path_weight( v, x ).is_none() );
}
else { panic!(); }
```

```
use stt::{DynamicForest, MonoidWeight, NodeData};
use stt::common::IsizeAddGroupWeight;
use stt::twocut::mtrtt::GroupMoveToRootTT;

// Create a new dynamic tree, now with integer edge weights and using the Move To Root heuristic
let mut t = GroupMoveToRootTT::new( 4 );
// Get the four nodes
if let [u,v,w,x] = t.nodes().collect::<Vec<_>>()[..] {
	// Add three edges to form a star
	t.link( u, v, IsizeAddGroupWeight::new( 0 ) );
	t.link( u, w, IsizeAddGroupWeight::new( 1 ) );
	t.link( u, x, IsizeAddGroupWeight::new( 2 ) );

	// Check path sums
	assert_eq!( t.compute_path_weight( u, v ), Some( IsizeAddGroupWeight::new( 0 ) ) );
	assert_eq!( t.compute_path_weight( v, w ), Some( IsizeAddGroupWeight::new( 1 ) ) );
	assert_eq!( t.compute_path_weight( w, x ), Some( IsizeAddGroupWeight::new( 3 ) ) );
}
else { panic!(); }
```

# Crate feature flags

The following crate feature flags are available. They are configured in your `Cargo.toml`.

* `space_efficient_nodes`
	* Optional, requires the `nomax` crate.
	* Improve node space usage. Disallows the maximum node index 2^64-1 and incurs a small runtime
		cost to check that this node index is not used.
* `petgraph`
	* Optional, requires the `petgraph` crate.
	* Enable a petgraph-based dynamic forest implementation. This implementation is very slow and
		only intended to be used for comparison or verification.
* `generate`
	* Optional, requires the `rand` crate.
	* Enables functionality to randomly generate stuff. Used for tests.
* `verbose_mst`
	* Optional. WARNING: slow.
	* Print out extra information when computing minimum spanning trees.
* `verbose_lc`
	* Optional. WARNING: very slow.
	* Print out detailed information about Link-cut tree operations.
* `verify_lc`
	* Optional. WARNING: very slow.
	* Verify link-cut tree implementation while running.

# Literature

\[BK22\] Benjamin Aram Berendsohn and László Kozma. Splay trees on trees.
Proceedings of the 2022 ACM-SIAM Symposium on Discrete Algorithms, SODA 2022, 1875–1900, 2022.
doi:[10.1137/1.9781611977073.75](https://doi.org/10.1137/1.9781611977073.75)

\[ST83\] Daniel D. Sleator and Robert Endre Tarjan. A Data Structure for Dynamic Trees.
Journal of Computer and System Sciences, 26(3):362–391, 1983.
doi:[10.1145/800076.802464](https://doi.org/10.1145/800076.802464)

\[DT85\] Daniel Dominic Sleator and Robert Endre Tarjan. Self-adjusting binary search trees.
Journal of the ACM, 32(3):652–686, 1985.
doi:[10.1145/3828.3835](https://doi.org/10.1145/3828.3835)
*/

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::private_doc_tests)]


use std::fmt::{Debug, Display, Formatter};
use std::ops;

#[cfg( feature = "space_efficient_nodes" )]
use nonmax::NonMaxUsize;

pub mod common;
pub mod connectivity;
pub mod link_cut;
pub mod mst;
pub mod onecut;
pub mod rooted;
pub mod twocut;

#[cfg( feature = "generate" )]
pub mod generate;

#[cfg( feature = "petgraph" )]
pub mod pg;


/// Represents a node in a dynamic tree to the outside world.
#[cfg( not( feature = "space_efficient_nodes" ) )]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeIdx {
	raw_idx: usize
}

#[cfg( not( feature = "space_efficient_nodes" ) )]
impl NodeIdx {
	/// Convert `usize` into `NodeIdx`.
	/// 
	/// Use with care, as this can circumvent bounds checking.
	pub fn new( idx : usize ) -> NodeIdx {
		NodeIdx { raw_idx: idx }
	}
	
	/// Convert this into `usize`.
	#[inline]
	pub fn index( &self ) -> usize {
		self.raw_idx
	}
}


/// Represents a node in a dynamic tree to the outside world.
#[cfg( feature = "space_efficient_nodes" )]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeIdx {
	raw_idx : NonMaxUsize
}

#[cfg( feature = "space_efficient_nodes" )]
impl NodeIdx {
	/// Convert `usize` into `NodeIdx`.
	/// 
	/// Use with care, as this can circumvent bounds checking.
	pub fn new( idx : usize ) -> NodeIdx {
		NodeIdx { raw_idx : NonMaxUsize::new( idx ).unwrap() }
	}
	
	/// Convert this into `usize`.
	#[inline]
	pub fn index( &self ) -> usize {
		self.raw_idx.get()
	}
}

impl Display for NodeIdx {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.index() )
	}
}


/// Base trait for edge weights.
/// 
/// Edge weights must form a [commutative monoid](https://en.wikipedia.org/wiki/Monoid#Commutative_monoid).
/// The identity element is constructed by the function [MonoidWeight::identity()]. The monoid
/// operation is addition via the [ops::Add] trait.
pub trait MonoidWeight : Copy + Eq + ops::Add<Self, Output=Self> + Debug + Display {
	/// Returns the identity of this monoid.
	fn identity() -> Self;
}


/// Data associated to a node in a dynamic tree.
/// 
/// Each node in the dynamic forest holds an NodeData object. The actual contents are tied to the
/// dynamic forest implementation.
pub trait NodeData : Clone + Display {
	/// The edge weight of the dynamic forest.
	type TWeight : MonoidWeight;
	
	/// Create a default instance, to be used on newly created nodes.
	fn new( v : NodeIdx ) -> Self;
}

/// Node data that knows the weight of the path to the associated node's parent.
pub trait PathWeightNodeData : NodeData {
	/// Returns the weight of the path in the underlying graph of the associated node to its parent.
	/// 
	/// This node must have a parent.
	fn get_parent_path_weight( &self ) -> Self::TWeight;
}


/// A trait allowing directly accessing node data.
pub trait NodeDataAccess<TData : NodeData> {
	/// Returns a reference to the data associated to the given node.
	fn data( &self, idx : NodeIdx ) -> &TData;

	/// Returns a mutable reference to the data associated to the given node.
	fn data_mut( &mut self, idx : NodeIdx ) -> &mut TData;
}


/// A dynamic forest with edge weights.
pub trait DynamicForest {
	/// The edge weight type
	type TWeight : MonoidWeight;
	
	/// Iterator for nodes
	type NodeIdxIterator : Iterator<Item = NodeIdx>;
	
	/// Creates a new dynamic forest with the specified number of nodes and no edges.
	fn new( num_nodes : usize ) -> Self;

	/// Adds an edge between u and v with the given weight. The edge must not yet exist.
	fn link( &mut self, u : NodeIdx, v : NodeIdx, weight : Self::TWeight );

	/// Removes the edge between u and v. The edge must exist.
	fn cut( &mut self, u : NodeIdx, v : NodeIdx );

	/// Computes the weight of the path between u and v, or returns None if no such path exists.
	fn compute_path_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<Self::TWeight>;

	/// Returns the weight of the edges between u and v, or returns None if that edge doesn't exist.
	fn get_edge_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<Self::TWeight>;

	/// Iterate over the nodes in this dynamic forest.
	fn nodes( &self ) -> Self::NodeIdxIterator;

	/// Returns a vector containing all edges in this dynamic forest.
	/// 
	/// Note that this refers to the edges added with the [link](Self::link()) operation.
	/// Implementations may use rooted trees with a different edge set.
	/// Since such implementations may only represent the actual edges implicitly, this function
	/// might be quite costly.
	fn edges( &self ) -> Vec<(NodeIdx, NodeIdx)>;
}

/// A collection of rooted trees.
/// 
/// Most [DynamicForest] implementations use a collection of rooted trees internally. This trait
/// allows accessing the internal structure of these implementations.
/// 
/// Note that these rooted trees may have different edge sets from the represented dynamic tree.
#[portrait::make]
pub trait RootedForest {
	/// The parent of the given node.
	/// 
	/// If this is a [DynamicForest] implementation, the represented dynamic tree does not
	/// necessarily have an edge between `v` and `get_parent(v)`.
	fn get_parent( &self, v : NodeIdx ) -> Option<NodeIdx>;
}

#[cfg(test)]
mod tests {
	use crate::NodeIdx;
	
	#[cfg( not( feature = "space_efficient_nodes" ) )]
	#[test]
	fn test_node_idx_valid() {
		assert_eq!( NodeIdx::new( 0 ).index(), 0 );
		assert_eq!( NodeIdx::new( usize::MAX ).index(), usize::MAX );
	}
	
	#[cfg( feature = "space_efficient_nodes" )]
	#[test]
	fn test_node_idx_valid() {
		assert_eq!( NodeIdx::new( 0 ).index(), 0 );
		assert_eq!( NodeIdx::new( usize::MAX - 1 ).index(), usize::MAX - 1 );
	}
	
	#[cfg( feature = "space_efficient_nodes" )]
	#[test]
	#[should_panic]
	fn test_node_idx_invalid() {
		NodeIdx::new( usize::MAX );
	}
}
