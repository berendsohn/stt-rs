//! Rooted dynamic tree STT-based implementation template

use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::{NodeData, NodeDataAccess, NodeIdx, RootedForest};
use crate::common::EmptyGroupWeight;
use crate::rooted::RootedDynamicForest;
use crate::twocut::ExtendedNTRStrategy;
use crate::twocut::basic::{MakeOneCutSTT, STT, STTRotate, STTStructureRead};

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

/// An STT-based rooted dynamic forest using an [ExtendedNTRStrategy].
#[derive(Clone)]
pub struct StandardRootedDynamicForest<TNTRStrat : ExtendedNTRStrategy> {
	t : STT<RootedNodeData>,
	_m : PhantomData<TNTRStrat>
}

impl<TNTRStrat : ExtendedNTRStrategy> StandardRootedDynamicForest<TNTRStrat> {
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
}

impl<TNTRStrat: ExtendedNTRStrategy> STTStructureRead for StandardRootedDynamicForest<TNTRStrat> {
	fn get_direct_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.t.get_direct_separator_child( v )
	}
	
	fn get_indirect_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.t.get_indirect_separator_child( v )
	}
}

impl<TNTRStrat: ExtendedNTRStrategy> RootedForest for StandardRootedDynamicForest<TNTRStrat> {
	fn get_parent( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.t.get_parent( v )
	}
}

impl<TNTRStrat : ExtendedNTRStrategy> STTRotate for StandardRootedDynamicForest<TNTRStrat>{
	fn rotate( &mut self, v : NodeIdx) {
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

impl<TNTRStrat : ExtendedNTRStrategy> RootedDynamicForest for StandardRootedDynamicForest<TNTRStrat> {
	type NodeIdxIterator = <STT<RootedNodeData> as MakeOneCutSTT>::NodeIdxIterator;
	
	fn new( num_nodes : usize ) -> Self {
		Self{ t : STT::new( num_nodes ), _m : PhantomData::default() }
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		self.t.nodes()
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, u );
		TNTRStrat::node_to_root( self, v );
		debug_assert!( self.t.get_parent( u ).is_none(), "It seems you're trying to link two nodes {u}, {v} in the same tree." );
		debug_assert!( self.t.data( u ).desc_root == Some( u ), "It seems you're trying to link a non-root node {u}" );
		self.t.attach( u, v );
		self.t.data_mut( u ).desc_root = None;
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		TNTRStrat::node_to_root( self, v );
		let r = self.t.data( v ).desc_root.unwrap();

		debug_assert!( r != v, "It seems you're trying to cut at the root." );

		// TODO: Simple checks here?

		// Find child of x with r in its subtree.
		let mut x = r;
		while let Some( p ) = self.get_parent( x ) {
			if p == v {
				break;
			}
			x = p;
		}

		// Find parent of v in the underlying tree
		if let Some( d ) = self.get_direct_separator_child( x ) {
			x = d;
			while let Some( i ) = self.get_indirect_separator_child( x ) {
				x = i;
			}
		}
		TNTRStrat::node_below_root( self, x );
		debug_assert!( self.get_direct_separator_child( x ).is_none() );

		self.t.detach( x );
		self.t.data_mut( v ).desc_root = Some( v );
		return;
	}
	
	fn cut_edge( &mut self, u : NodeIdx, v : NodeIdx) {
		TNTRStrat::node_to_root( self, v );
		TNTRStrat::node_below_root( self, u );
		debug_assert!( self.t.get_direct_separator_child( u ).is_none(), "It seems you're trying to cut a non-existing edge ({u}, {v})." );
		debug_assert_eq!( self.t.get_parent( u ), Some( v ), "It seems you're trying to cut a non-existing edge ({u}, {v}). The two nodes are not even in the same tree." );

		self.t.detach( u );
	}
	
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		TNTRStrat::node_to_root( self, v );
		self.t.data( v ).desc_root.unwrap()
	}
	
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		TNTRStrat::node_to_root( self, v );
		TNTRStrat::node_below_root( self, u );
		if self.get_parent( u ) != Some( v ) {
			return None // u, v are in different trees
		}

		// u is now a child of v
		if self.t.data( u ).desc_root.is_none() {
			// Root is in T_v, but not T_u
			Some( v )
		}
		else if let Some( c ) = self.get_direct_separator_child( u ) {
			if self.t.data( c ).desc_root.is_none() {
				// Root is in T_u, but not T_v
				Some( u )
			}
			else {
				// Root is in T_c, so lca(u,v) is also in T_c
				Some( self.lca_in( c ) )
			}
		}
		else {
			// Root is below u, nothing between u and v
			Some( u )
		}
	}
}