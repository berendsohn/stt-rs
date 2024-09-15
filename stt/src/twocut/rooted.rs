//! Rooted dynamic tree STT-based implementation template

use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::*;
use crate::common::EmptyGroupWeight;
use crate::rooted::{EversibleRootedDynamicForest, RootedDynamicForest};
use crate::twocut::{ExtendedNTRStrategy, NTRStrategy, StableNTRStrategy};
use crate::twocut::basic::*;

/// NodeData allowing maintanance of the (underlying) root of each tree.
#[derive( Clone )]
pub struct RootedNodeData {
	/// The root `r` of the underlying tree containing this node, if `r` is a descendant of this
	/// node. Otherwise, None.
	desc_root : Option<NodeIdx>
}

impl Display for RootedNodeData {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		if let Some( r ) = self.desc_root {
			write!( f, "{}", r )
		}
		else {
			write!( f, "-" )
		}
	}
}

impl NodeData for RootedNodeData {
	type TWeight = EmptyGroupWeight;
	
	fn new( v : NodeIdx ) -> Self {
		RootedNodeData{ desc_root : Some( v ) }
	}
}


/// An STT with correctly handled `RootedNodeData` and some basic functions common to the stable and
/// non-stable implementations.
#[derive(Clone)]
struct BasicRootedDynamicForest<TNTRStrat: NTRStrategy> {
	t : STT<RootedNodeData>,
	_m : PhantomData<TNTRStrat>
}

#[portrait::fill(portrait::delegate(STT<RootedNodeData>; self.t))]
impl<TNTRStrat: NTRStrategy> RootedForest for BasicRootedDynamicForest<TNTRStrat> {}

#[portrait::fill(portrait::delegate(STT<RootedNodeData>; self.t))]
impl<TNTRStrat: NTRStrategy> STTStructureRead for BasicRootedDynamicForest<TNTRStrat> {}

impl<TNTRStrat: NTRStrategy> STTRotate for BasicRootedDynamicForest<TNTRStrat> {
	fn rotate( &mut self, v : NodeIdx ) {
		let p = self.get_parent(v).unwrap();

		let old_v_desc_root = self.t.data( v ).desc_root;
		self.t.data_mut( v ).desc_root = self.t.data( p ).desc_root;

		if old_v_desc_root.is_some() {
			if let Some( c ) = self.t.get_direct_separator_child( v ) {
				if self.t.data( c ).desc_root.is_none() {
					// Root is below v, but not below c
					self.t.data_mut( p ).desc_root = None;
				}
			} else {
				// Root is below v, but not below (non-existing) c
				self.t.data_mut( p ).desc_root = None;
			}
		}

		self.t.rotate( v );
	}
}

impl<TNTRStrat: NTRStrategy> BasicRootedDynamicForest<TNTRStrat> {
	fn new( num_vertices : usize ) -> Self {
		Self{ t : STT::new( num_vertices ), _m : PhantomData::default() }
	}

	/// Finds `LCA(u,v)`, where `{u,v}` is the boundary of `T_x`.
	///
	/// Assumes `x` is a separator node, and the underlying tree root, as well as `LCA(u,v)` is
	/// contained in `T_x`.
	fn lca_in( &mut self, x : NodeIdx ) -> NodeIdx {
		if let Some( d ) = self.get_direct_separator_child( x ) {
			if self.t.data( d ).desc_root.is_some() {
				return self.lca_in( d )
			}
		}
		if let Some( i ) = self.get_indirect_separator_child( x ) {
			if self.t.data( i ).desc_root.is_some() {
				return self.lca_in( i )
			}
		}

		// Root is below x, but not below d or i.
		TNTRStrat::node_to_root( self, x ); // For amortized analysis
		x
	}

	/// Implementation of `RootedDynamicForest::find_root`
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		TNTRStrat::node_to_root( self, v );
		self.t.data( v ).desc_root.unwrap()
	}

	/// Implementation of `RootedDynamicForest::link`
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, u );
		TNTRStrat::node_to_root( self, v );
		debug_assert!( self.t.get_parent( u ).is_none(), "It seems you're trying to link two nodes {u}, {v} in the same tree." );
		debug_assert!( self.t.data( u ).desc_root == Some( u ), "It seems you're trying to link a non-root node {u}" );
		self.t.attach( u, v );
		self.t.data_mut( u ).desc_root = None;
	}

	/// Implementation of `EversibleRootedDynamicForest::make_root`
	fn make_root( &mut self, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, v );
		let r = self.t.data( v ).desc_root.unwrap(); // Old root of the underlying tree
		let mut x = r;
		while let Some( p ) = self.get_parent( x ) {
			self.t.data_mut( x ).desc_root = None; // Delete references to old root
			x = p;
		}
		debug_assert!( x == v );
		self.t.data_mut( v ).desc_root = Some( v );
		TNTRStrat::node_to_root( self, r ); // Performance
	}
}


/// An STT-based rooted dynamic forest using an [ExtendedNTRStrategy].
#[derive(Clone)]
pub struct StandardRootedDynamicForest<TNTRStrat : ExtendedNTRStrategy> {
	t : BasicRootedDynamicForest<TNTRStrat>
}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat: ExtendedNTRStrategy> RootedForest for StandardRootedDynamicForest<TNTRStrat> {}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat: ExtendedNTRStrategy> STTStructureRead for StandardRootedDynamicForest<TNTRStrat> {}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat : ExtendedNTRStrategy> STTRotate for StandardRootedDynamicForest<TNTRStrat>{}

impl<TNTRStrat : ExtendedNTRStrategy> RootedDynamicForest for StandardRootedDynamicForest<TNTRStrat> {
	type NodeIdxIterator = <STT<RootedNodeData> as MakeOneCutSTT>::NodeIdxIterator;
	
	fn new( num_vertices : usize ) -> Self {
		Self{ t : BasicRootedDynamicForest::new( num_vertices ) }
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		self.t.t.nodes()
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		self.t.link( u, v );
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, v );
		let r = self.t.t.data( v ).desc_root.unwrap();

		debug_assert!( r != v, "It seems you're trying to cut at the root." );
		
		TNTRStrat::node_below_root( self, r );
		
		let mut x = r;
		// Find parent of v in the underlying tree
		if let Some( d ) = self.get_direct_separator_child( x ) {
			x = d;
			while let Some( i ) = self.get_indirect_separator_child( x ) {
				x = i;
			}
		}
		TNTRStrat::node_below_root( self, x );
		debug_assert!( self.get_direct_separator_child( x ).is_none() );

		self.t.t.detach( x );
		self.t.t.data_mut( v ).desc_root = Some( v );
		return;
	}
	
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		self.t.find_root( v )
	}
	
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		TNTRStrat::node_to_root( self, v );
		TNTRStrat::node_below_root( self, u );
		if self.get_parent( u ) != Some( v ) {
			return None // u, v are in different trees
		}

		// u is now a child of v
		if self.t.t.data( u ).desc_root.is_none() {
			// Root is in T_v, but not T_u
			Some( v )
		}
		else if let Some( c ) = self.get_direct_separator_child( u ) {
			if self.t.t.data( c ).desc_root.is_none() {
				// Root is in T_u, but not T_v
				Some( u )
			}
			else {
				// Root is in T_c, so lca(u,v) is also in T_c
				Some( self.t.lca_in( c ) )
			}
		}
		else {
			// Root is below u, nothing between u and v
			Some( u )
		}
	}
}

impl<TNTRStrat : ExtendedNTRStrategy> EversibleRootedDynamicForest for StandardRootedDynamicForest<TNTRStrat> {
	fn make_root( &mut self, v : NodeIdx ) {
		self.t.make_root( v );
	}
}


/// An STT-based rooted dynamic forest using an [StableNTRStrategy].
#[derive(Clone)]
pub struct StableRootedDynamicForest<TNTRStrat : StableNTRStrategy> {
	t : BasicRootedDynamicForest<TNTRStrat>
}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat: StableNTRStrategy> RootedForest for StableRootedDynamicForest<TNTRStrat> {}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat: StableNTRStrategy> STTStructureRead for StableRootedDynamicForest<TNTRStrat> {}

#[portrait::fill(portrait::delegate(BasicRootedDynamicForest<TNTRStrat>; self.t))]
impl<TNTRStrat : StableNTRStrategy> STTRotate for StableRootedDynamicForest<TNTRStrat> {}


impl<TNTRStrat : StableNTRStrategy> RootedDynamicForest for StableRootedDynamicForest<TNTRStrat> {
	type NodeIdxIterator = <STT<RootedNodeData> as MakeOneCutSTT>::NodeIdxIterator;

	fn new( num_vertices : usize ) -> Self {
		Self{ t : BasicRootedDynamicForest::new( num_vertices ) }
	}

	fn nodes( &self ) -> Self::NodeIdxIterator {
		self.t.t.nodes()
	}

	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		self.t.link( u, v )
	}

	fn cut( &mut self, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, v );
		let r = self.t.t.data( v ).desc_root.unwrap();

		debug_assert!( r != v, "It seems you're trying to cut at the root." );

		TNTRStrat::node_to_root( self, r );
		TNTRStrat::node_to_root( self, v );

		debug_assert!( self.get_parent( r ).is_some() );

		// Now v is the root and r and all its ancestors are 1-cut.
		// Find child x of v that is (non-strict) ancestor of r
		let mut p = self.get_parent( r ).unwrap();
		let mut x = r;
		loop {
			if p == v {
				break
			}
			x = p;
			p = self.get_parent( p ).unwrap();
		}

		// Find parent of v in the underlying tree
		if let Some( d ) = self.get_direct_separator_child( x ) {
			x = d;
			while let Some( i ) = self.get_indirect_separator_child( x ) {
				x = i;
			}
		}
		TNTRStrat::node_to_root( self, x );
		debug_assert!( self.get_direct_separator_child( x ).is_none() );

		// Since there is an edge between x and v, now x must be parent of v in the search tree.
		debug_assert!( self.get_parent( v ) == Some( x ) );

		self.t.t.detach( v );
		self.t.t.data_mut( v ).desc_root = Some( v );
	}

	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		self.t.find_root( v )
	}

	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		TNTRStrat::node_to_root( self, u );
		TNTRStrat::node_to_root( self, v );

		if self.get_parent( u ).is_none() {
			return None // u, v are in different trees
		}

		// Now v is the root and all ancestors of u are 1-cut

		// Find lowest ancestor of u where the underlying tree root is a descendant.
		let mut x = u;
		while let Some( p ) = self.get_parent( x ) {
			if self.t.t.data( x ).desc_root.is_none() {
				x = p;
			}
			else { break }
		}

		if x == v {
			// v separates root from u.
			Some( v )
		}
		else if let Some( c ) = self.get_direct_separator_child( x ) {
			if self.t.t.data( c ).desc_root.is_none() {
				// x separates root from v
				Some( x )
			}
			else {
				// Root is between x and its parent, which form the boundary of T_c
				Some( self.t.lca_in( c ) )
			}
		}
		else {
			// x separates root from v
			Some( x )
		}
	}
}

impl<TNTRStrat : StableNTRStrategy> EversibleRootedDynamicForest for StableRootedDynamicForest<TNTRStrat> {
	fn make_root( &mut self, v : NodeIdx ) {
		self.t.make_root( v );
	}
}
