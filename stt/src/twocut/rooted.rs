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
	/// The lca (in the underlying tree) of the subtree (in the STT) rooted at this node
	st_lca : NodeIdx,
	/// Whether the edge from the subgraph (of the underlying tree) induced by this node and its
	/// descendants to the parent of this node is outgoing (the parent is a parent in the underlying
	/// tree).
	/// 
	/// If true, that edge is between `st_lca` and its parent in the underlying tree.
	/// If this node is a root, then `st_out` is false.
	st_out : bool
}

impl Display for RootedNodeData {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}, {}", self.st_lca, if self.st_out {"->"} else {"<-"} )
	}
}

impl NodeData for RootedNodeData {
	type TWeight = EmptyGroupWeight;
	
	fn new( v : NodeIdx ) -> Self {
		RootedNodeData{ st_lca : v, st_out : false }
	}
}

/// An STT-based rooted dynamic forest using an [ExtendedNTRStrategy].
pub struct StandardRootedDynamicForest<TNTRStrat : ExtendedNTRStrategy> {
	t : STT<RootedNodeData>,
	_m : PhantomData<TNTRStrat>
}

impl<TNTRStrat : ExtendedNTRStrategy> StandardRootedDynamicForest<TNTRStrat> {
	/// Finds `LCA(u,v)`, where `{u,v}` is the boundary of `T_x`.
	/// 
	/// Assumes `x` is a separator node, and `LCA(u,v)` is contains in `T_x`
	fn lca_in( &mut self, x : NodeIdx ) -> NodeIdx {
		// Denote by d the direct and by i the indirect separator of x
		if let Some( d ) = self.get_direct_separator_child( x ) {
			if !self.t.data( d ).st_out {
				// u -> d <- x <- v
				return self.lca_in( d )
			}
		}
		if let Some( i ) = self.get_indirect_separator_child( x ) {
			if !self.t.data( i ).st_out {
				// u -> x -> i <- v
				return self.lca_in( i )
			}
		}
		// Otherwise: u [-> d] -> x <- [i <-] v, so we found the LCA
		TNTRStrat::node_to_root( self, x ); // For amortized analysis
		x
	}
	
	/// Returns the parent of `v` in the underlying forest, if it has one.
	fn find_parent( &mut self, v : NodeIdx ) -> Option<NodeIdx> {
		// Do a partial find_root(), since we rotate v up later anyway.
		let r = { 
			let mut x = v;
			while let Some( p ) = self.get_parent( x ) {
				x = p;
			}
			self.t.data( x ).st_lca
		};
		if r == v {
			return None; // v itself is the root.
		}
		
		TNTRStrat::node_to_root( self, r );
		TNTRStrat::node_below_root( self, v );
		
		debug_assert!( self.t.data( v ).st_out ); // T_v -> r
		
		if let Some( c ) = self.get_direct_separator_child( v ) {
			debug_assert!( !self.t.data( c ).st_out ); // Otherwise: v <- T_c -> r
			// v -> T_c -> r
			// Find the node in T_c that is connected to v.
			// Follow the "left spine" of T_c
			if let Some( d ) = self.get_direct_separator_child( c ) {
				let mut x = d;
				while let Some( y ) = self.get_indirect_separator_child( x ) {
					x = y;
				}
				Some( x )
			}
			else {
				Some( c )
			}
		}
		else {
			// No nodes between v and r
			Some( r )
		}
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
		let p = self.get_parent( v ).unwrap();
		let old_p_out = self.t.data( p ).st_out;
		let old_p_lca = self.t.data( p ).st_lca;
		
		if let Some( c ) = self.get_direct_separator_child( v ) {
			self.t.data_mut( p ).st_out = self.t.data( c ).st_out;
			self.t.data_mut( c ).st_out = self.t.data( v ).st_out;
			
			if !self.t.data( v ).st_out {
				// [v -?- c <- p], so st_lca'(T_p) is in c
				self.t.data_mut( p ).st_lca = self.t.data( c ).st_lca;
			}
			// Otherwise, [v -?- c -> p], so st_lca'(T_p) = st_lca(T_p)
		}
		else {
			self.t.data_mut( p ).st_out = !self.t.data( v ).st_out;
			
			if !self.t.data( v ).st_out {
				// [v <- p]
				self.t.data_mut( p ).st_lca = p;
			}
			// Otherwise, [v -> p], so st_lca'(T_p) = st_lca(T_p)
		}
		
		self.t.data_mut( v ).st_out = old_p_out;
		self.t.data_mut( v ).st_lca = old_p_lca;
		
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
		self.t.attach( u, v );
		
		// No subtree root change
		self.t.data_mut( u ).st_out = true;
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		let p = self.find_parent( v  ).unwrap();
		return self.cut_edge( v, p );
	}
	
	fn cut_edge( &mut self, u : NodeIdx, v : NodeIdx) {
		TNTRStrat::node_to_root( self, v );
		TNTRStrat::node_below_root( self, u );
		debug_assert!( self.t.get_direct_separator_child( u ).is_none(), "It seems you're trying to cut a non-existing edge ({u}, {v})." );
		debug_assert_eq!( self.t.get_parent( u ), Some( v ), "It seems you're trying to cut a non-existing edge ({u}, {v}). The two nodes are not even in the same tree." );
		debug_assert!( self.t.data_mut( u ).st_out, "It seems you're trying to cut an edge in the wrong direction." );
		
		self.t.detach( u );
		
		self.t.data_mut( u ).st_out = false; // u is now a root
	}
	
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		TNTRStrat::node_to_root( self, v );
		self.t.data( v ).st_lca // The LCA of the whole tree is its root
	}
	
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		TNTRStrat::node_to_root( self, v );
		TNTRStrat::node_below_root( self, u );
		if self.get_parent( u ) != Some( v ) {
			return None // u, v are in different trees
		}
		
		if let Some( c ) = self.get_direct_separator_child( u ) {
			// T_c is between u and v
			let c_out = self.t.data( c ).st_out;
			let u_out = self.t.data( u ).st_out;
			if c_out && !u_out {
				// u <- T_c <- v
				Some( u )
			}
			else if !c_out && u_out {
				// u -> T_c -> v
				Some( v )
			}
			else {
				// u -> T_c <- v
				debug_assert!( !c_out && !u_out );
				Some( self.lca_in( c ) )
			}
		}
		else {
			if self.t.data( u ).st_out {
				// u -> v
				Some( v )
			}
			else {
				// u <- v
				Some( u )
			}
		}
	}
}