//! Basic implementation of a forest of 2-cut STTs, including rotations.
//! 
//! Does not implement the dynamic forest interface. Instead, this is meant to be used as a building
//! block for 2-cut-STT-based dynamic forest implementations.

use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::Map;
use std::ops::Range;

use crate::{RootedForest, NodeData, NodeIdx};
use crate::common::EmptyNodeData;
use crate::NodeDataAccess;

/// An STT with no edge weights.
pub type EmptySTT = STT<EmptyNodeData>;

/// Internal node
#[derive(Clone, Debug)]
struct Node<TData : NodeData> {
	/// The parent of node in the STT
	parent : Option<NodeIdx>,
	
	/// The unique child of this node that has parent of node in its boundary
	dsep_child : Option<NodeIdx>,
	
	/// The unique child with boundary size two that does not have parent in boundary
	isep_child : Option<NodeIdx>,
	
	/// The data associated to this node
	data : TData
}

impl<TData : NodeData> Node<TData> {
	fn new( v : NodeIdx ) -> Node<TData> {
		Node { parent : None, dsep_child : None, isep_child : None, data : TData::new( v ) }
	}

	fn swap_sep_children( &mut self ) {
		(self.dsep_child, self.isep_child) = (self.isep_child, self.dsep_child);
	}
}


/// Helper function
fn opt_is_different<T : Copy + Eq>( opt : Option<T>, expected : T ) -> bool {
	match opt {
		Some( x ) => x != expected,
		None => false
	}
}

/// Trait to read the two-cut STT structure.
pub trait STTStructureRead : RootedForest {
	/// Get the 2-cut child of `v` that also has the parent of `v` in its boundary, if any.
	fn get_direct_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx>;

	/// Get the 2-cut child of `v` that does not have the parent of `v` in its boundary, if any.
	fn get_indirect_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx>;

	/// Indicate whether `v` is a 2-cut node.
	fn is_separator( &self, v : NodeIdx ) -> bool {
		return self.is_direct_separator( v ) || self.is_indirect_separator( v );
	}

	/// Indicate whether `v` is a 2-cut node and the grandparent of `v` is in its boundary.
	fn is_direct_separator( &self, v : NodeIdx ) -> bool {
		match self.get_parent( v ) {
			Some( p ) => self.get_direct_separator_child( p ) == Some( v ),
			None => false
		}
	}

	/// Indicate whether `v` is a 2-cut node and the grandparent of `v` is not in its boundary.
	fn is_indirect_separator( &self, v : NodeIdx ) -> bool {
		match self.get_parent( v ) {
			Some( p ) => self.get_indirect_separator_child( p ) == Some( v ),
			None => false
		}
	}
}

/// Trait to execute rotations in 2-cut STTs.
pub trait STTRotate : STTStructureRead {
	/// Rotate v with its parent.
	/// Requires that v is not the root, i.e., v has a parent.
	/// Also requires that v is a separator node, or the parent of v is not a separator node (to
	/// maintain the 2-cut property).
	fn rotate( &mut self, v : NodeIdx );

	/// Whether a rotation is allowed for this node
	fn can_rotate( &self, v : NodeIdx ) -> bool {
		if let Some( p ) = self.get_parent( v ) {
			self.is_separator( v ) || ! self.is_separator( p )
		}
		else {
			false
		}
	}
}


/// A 2-cut search tree on a tree.
#[derive(Clone)]
pub struct STT<TData : NodeData> {
	nodes : Vec<Node<TData>>
}

impl<TData : NodeData> NodeDataAccess<TData> for STT<TData> {
	fn data( &self, idx : NodeIdx ) -> &TData {
		&self.node( idx ).data
	}

	fn data_mut( &mut self, idx : NodeIdx ) -> &mut TData {
		&mut self.node_mut( idx ).data
	}
}

impl<TData : NodeData> RootedForest for STT<TData> {
	fn get_parent( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.node( v ).parent
	}
}

impl<TData : NodeData> STTStructureRead for STT<TData> {
	fn get_direct_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.node( v ).dsep_child
	}

	fn get_indirect_separator_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.node( v ).isep_child
	}
}

impl<TData : NodeData> STTRotate for STT<TData> {
	fn rotate( &mut self, v : NodeIdx ) {
		// println!( "Rotating {v}" );

		// Get parent
		let p= self.node( v ).parent.unwrap();

		debug_assert!( self.is_separator( v ) || ! self.is_separator( p ) );
		let p_was_sep = self.is_separator( p );

		// Get (possibly) other relevant nodes
		let gp_p = self.node( p ).parent;
		let c_p = self.node( v ).dsep_child;

		// Change parents
		self.node_mut( p ).parent = Some( v );
		self.node_mut( v ).parent = gp_p; // May be None
		if let Some( c ) = c_p {
			self.node_mut( c ).parent = Some( p );
		}

		// Change separator information for children of gp
		if let Some( gp ) = gp_p {
			// v is now the root of the tree formerly rooted at p
			if self.node( gp ).dsep_child == Some( p ) {
				self.node_mut( gp ).dsep_child = Some( v );
			}
			else if self.node( gp ).isep_child == Some( p ) {
				self.node_mut( gp ).isep_child = Some( v );
			}
		}

		// Change separator information for children of p
		let old_p_dsep_child = self.node( p ).dsep_child;
		self.node_mut( p ).dsep_child = c_p;
		if opt_is_different( old_p_dsep_child, v ) {
			self.node_mut( p ).isep_child = old_p_dsep_child;
		}
		else if self.node( p ).isep_child == Some( v ) {
			self.node_mut( p ).isep_child = None;
		}

		// Change separator information for children of v
		if gp_p.is_some() { // p was not root
			if old_p_dsep_child != Some( v ) {
				// p separates v and gp
				self.node_mut( v ).dsep_child = Some( p );
			}
			else {
				// v separates p and gp
				self.node_mut( v ).dsep_child = self.node( v ).isep_child; // gp is now parent of v
				if p_was_sep {
					// p separates v and some ancestor above gp
					self.node_mut( v ).isep_child = Some( p );
				}
				else {
					// v separates all ancestors from p
					self.node_mut( v ).isep_child = None;
				}
			}
		}
		else { // p was root
			self.node_mut( v ).dsep_child = None;
			debug_assert!( self.node_mut( v ).isep_child == None ); // Since this had only one ancestor
		}

		// Change separator information for children of c (not affected by the rotation otherwise)
		if let Some( c ) = c_p {
			self.node_mut( c ).swap_sep_children();
		}
	}
}

impl<TData : NodeData> STT<TData> {
	/// Creates a new STT on `n` nodes.
	pub fn new( n : usize ) -> STT<TData> {
		STT { nodes : (0..n).map( |i| Node::new( NodeIdx::new( i ) ) ).collect() }
	}

	/// Makes `parent` the parent of `child`.
	/// 
	/// `child` must not yet have a parent.
	pub fn attach( &mut self, child : NodeIdx, parent : NodeIdx ) {
		debug_assert!( self.node( child ).parent.is_none() );
		self.node_mut( child ).parent = Some( parent );
	}

	/// Removes `v` as a child from its parent.
	/// 
	/// `child` must have a parent.
	pub fn detach( &mut self, v : NodeIdx ) {
		debug_assert!( self.node( v ).parent.is_some() );
		debug_assert!( !self.is_separator( v ) );
		self.node_mut( v ).parent = None;
	}

	/// Returns a human-readable string representation of this STT.
	pub fn to_string( &self ) -> String {
		let mut s = String::new();
		self._print( &mut s );
		s
	}

	fn _print( &self, out : &mut String ) {
		let mut child_map: HashMap<NodeIdx, Vec<NodeIdx>> = HashMap::new();
		for v in self.nodes() {
			if let Some( p ) = self.node( v ).parent {
				match child_map.get_mut( &p ) {
					Some( children ) => children.push( v ),
					None => {
						child_map.insert( p, vec![ v ] );
					}
				}
			}
		}

		for v in self.nodes() {
			if self.get_parent( v ).is_none() {
				self._print_subtree( out, v, &child_map, "" );
			}
		}
	}

	fn _print_subtree( &self, out : &mut String, root : NodeIdx, child_map: &HashMap<NodeIdx, Vec<NodeIdx>>, indent : &str ) {
		out.push_str( indent );

		// Print node
		out.push_str( &format!( "{}", root.index()) );
		if self.is_direct_separator( root ) {
			out.push_str( "d" );
		}
		else if self.is_indirect_separator( root ) {
			out.push_str( "i" );
		}

		out.push_str( format!( "[{}]", self.data( root ) ).as_str() );

		out.push_str( "\n" );

		// Print children
		fn indent_map( c : char ) -> char {
			match c {
				'├' => '│',
				'└' => ' ',
				'─' => ' ',
				x => x
			}
		}
		let child_indent : String = String::from( indent ).chars().map( indent_map ).collect();
		let empty = vec![];
		let mut child_it = child_map.get( &root ).unwrap_or( &empty).iter().peekable();
		while let Some( c ) = child_it.next() {
			// Last symbol(s) before child depend on whether there is another child.
			let indent_symbol = if child_it.peek().is_none() { "└─" } else { "├─" };

			self._print_subtree(out, *c, child_map, format!( "{}{}", child_indent, indent_symbol ).as_str() );
		}
	}

	/// Performs some sanity checks and returns `true` if they succeed.
	pub fn _is_valid( &self ) -> bool {
		self.nodes().all(
			|v|
			match self.node( v ).dsep_child {
				Some( c ) => self.node( c ).parent == Some( v ),
				None => true
			} &&
			match self.node( v ).isep_child {
				Some( c ) => self.node( c ).parent == Some( v ),
				None => true
			}
		)
	}
	
	/// Iterates over the indices of nodes in this STT.
	pub fn nodes( &self ) -> Map<Range<usize>, fn(usize) -> NodeIdx> {
		( 0..self.nodes.len() ).map( NodeIdx::new )
	}

	/// Iterates over each child-parent edge.
	pub fn edges( &self ) -> impl Iterator<Item = (NodeIdx, NodeIdx)> + '_ {
		self.nodes().filter_map(
			|v|
			match self.get_parent( v ) {
				Some( p ) => Some( (p,v) ),
				None => None
			}
		)
	}

	fn node( &self, idx : NodeIdx ) -> &Node<TData> {
		&self.nodes[idx.index()]
	}

	fn node_mut( &mut self, idx : NodeIdx ) -> &mut Node<TData> {
		&mut self.nodes[idx.index()]
	}
}


/// A trait for STTs that can be transformed into 1-cut trees.
/// 
/// Apart from accessing the structure and allowing rotations, transforming into 1-cut trees also
/// requires us to iterate over all nodes.
pub trait MakeOneCutSTT : STTRotate + STTStructureRead {
	/// Iterator for nodes
	type NodeIdxIterator : Iterator<Item = NodeIdx>;
	
	/// Iterate over the nodes in this STT.
	fn nodes( &self ) -> Self::NodeIdxIterator;
}

impl<TData : NodeData> MakeOneCutSTT for STT<TData> {
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		( 0..self.nodes.len() ).map( NodeIdx::new )
	}
	
}


/// Perform rotations to make t a 1-cut tree.
pub fn make_1_cut<STT : MakeOneCutSTT>( t : &mut STT ) {
	for v in t.nodes() {
		while t.is_separator( v ) {
			t.rotate( v );
		}
	}
}
