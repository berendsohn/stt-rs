//! A dynamic tree implementation based on Sleator and Tarjan's *link-cut trees*, specifically their
//! [self-adjusting variant](https://doi.org/10.1145/800076.802464).
//! 
//! A Link-cut-tree is a rooted tree where each node may have a designated *left* and/or *right*
//! child. All other children are called *middle* children. Edges between the parent and the
//! left/right child are called *solid* edges, all other edges are called *dashed* edges.
//! 
//! Each solid subtree is a binary tree representing a path in the underlying graph. The path is
//! obtained by simply reading the nodes in in-order, from left to right. Each middle edge from p to
//! c represents an edge from p to the leftmost node in the solid subtree containing c.
//! 
//! The operations [`link()`](DynamicForest::link()), [`cut()`](DynamicForest::cut()), and
//! [`compute_path_weight()`](DynamicForest::compute_path_weight()) are guaranteed to run in
//! O(log n) amortized time, where `n` is the number of nodes in the forest.


use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter::Map;
use std::ops::Range;

use crate::{DynamicForest, MonoidWeight, NodeData, NodeDataAccess, NodeIdx, PathWeightNodeData, RootedForest};
use crate::common::{EmptyNodeData, GroupWeight, WeightOrInfinity};
use crate::common::WeightOrInfinity::{Finite, Infinite};
use crate::rooted::RootedDynamicForest;

/// Enable or disable logging
const LOG_VERBOSE : bool = cfg!( feature = "verbose_lc" );

/// Enable or disable sanity testing
const VERIFY : bool = cfg!( feature = "verify_lc" );


/// Link-cut tree without edge weights
pub type EmptyLinkCutTree = LinkCutForest<EmptyNodeData, true>;

/// Link-cut tree with the given edge weights, which must form a group.
pub type GroupLinkCutTree<TWeight> = LinkCutForest<GroupPathWeightLCTNodeData<TWeight>, true>;

/// Link-cut tree with the given edge weights.
pub type MonoidLinkCutTree<TWeight> = LinkCutForest<MonoidPathWeightLCTNodeData<TWeight>, true>;

/// Link-cut tree that maintains a rooted forest without edge weights
pub type RootedLinkCutTree = LinkCutForest<EmptyNodeData, false>;


/// Node data for link-cut trees.
/// 
/// Must define associated functions to update node data after operations.
pub trait LCTNodeData<const IMPL_EVERT : bool> : PathWeightNodeData {
	/// Called before rotating v with its (solid) parent.
	fn before_rotation(f : &mut LinkCutForest<Self, IMPL_EVERT>, v : NodeIdx );
	
	/// Called before splicing v to its parent.
	fn before_splice(f : &mut LinkCutForest<Self, IMPL_EVERT>, v : NodeIdx );

	/// Called after adding an edge from v to another node (now its parent), with the given edge weight.
	fn after_attached(f : &mut LinkCutForest<Self, IMPL_EVERT>, v : NodeIdx, weight : Self::TWeight );

	/// Called before removing the edge between v and its parent.
	fn before_detached(f : &mut LinkCutForest<Self, IMPL_EVERT>, v : NodeIdx );
}


/// Standard implementation for empty node data.
impl<const IMPL_EVERT : bool> LCTNodeData<IMPL_EVERT> for EmptyNodeData {
	fn before_rotation(_ : &mut LinkCutForest<Self, IMPL_EVERT>, _ : NodeIdx ) {}
	
	fn before_splice(_ : &mut LinkCutForest<Self, IMPL_EVERT>, _ : NodeIdx ) {}
	
	fn after_attached(_ : &mut LinkCutForest<Self, IMPL_EVERT>, _ : NodeIdx, _ : Self::TWeight ) {}
	
	fn before_detached(_ : &mut LinkCutForest<Self, IMPL_EVERT>, _ : NodeIdx ) {}
}


/// Internal node of a link-cut tree.
#[derive(Clone)]
struct Node<TNodeData : NodeData> {
	parent : Option<NodeIdx>,
	left_child : Option<NodeIdx>,
	right_child : Option<NodeIdx>,
	reversed : bool,
	data : TNodeData
}

impl<TNodeData : NodeData> Node<TNodeData> {
	fn new( v : NodeIdx ) -> Node<TNodeData> {
		Node{ parent : None, left_child : None, right_child : None, reversed : false, data : TNodeData::new( v ) }
	}
}


/// A forest of link-cut-trees.
#[derive(Clone)]
pub struct LinkCutForest<TNodeData : LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> {
	nodes : Vec<Node<TNodeData>>
}

impl<TNodeData : LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> LinkCutForest<TNodeData, IMPL_EVERT> {
	fn node( &self, v : NodeIdx ) -> &Node<TNodeData> {
		&self.nodes[v.index()]
	}
	
	fn node_mut( &mut self, v : NodeIdx ) -> &mut Node<TNodeData> {
		&mut self.nodes[v.index()]
	}
	
	/// Reverses the solid subtree rooted at `v`.
	fn reverse( &mut self, v : NodeIdx ) {
		debug_assert!( IMPL_EVERT );
		self.node_mut( v ).reversed = !self.node( v ).reversed;
	}
	
	/// Sets the reversed bit of `v` to false, but possibly changes the reversed bit of children.
	fn push_reverse_bit( &mut self, v : NodeIdx ) {
		if !IMPL_EVERT {
			return
		}
		if self.node( v ).reversed {
			let v_mut = self.node_mut( v );
			v_mut.reversed = false;
			(v_mut.left_child, v_mut.right_child) = (v_mut.right_child, v_mut.left_child);
			
			if let Some( c ) = self.node( v ).left_child {
				self.reverse( c );
			}
			if let Some( c ) = self.node( v ).right_child {
				self.reverse( c );
			}
		}
	}
	
	/// Checks whether `v` is a left or right child of `p`, assuming `p` is its parent
	fn is_non_middle_child_hint( &self, v : NodeIdx, p : NodeIdx ) -> bool {
		self.node( p ).left_child == Some( v ) || self.node( p ).right_child == Some( v )
	}
	
	/// Checks whether `v` is a left or right child, i.e., whether `v` has a parent and a solid edge
	/// to that parent.
	fn is_non_middle_child( &self, v : NodeIdx ) -> bool {
		if let Some( p ) = self.node( v ).parent {
			self.is_non_middle_child_hint( v, p )
		}
		else {
			false
		}
	}
	
	/// Checks whether `v` is a left child.
	fn is_left_child( &self, v : NodeIdx ) -> bool {
		if let Some( p ) = self.node( v ).parent {
			debug_assert!( !self.node( p ).reversed );
			self.node( p ).left_child == Some( v )
		}
		else {
			false
		}
	}
	
	/// Checks whether `v` is a right child.
	fn is_right_child( &self, v : NodeIdx ) -> bool {
		if let Some( p ) = self.node( v ).parent {
			debug_assert!( !self.node( p ).reversed );
			self.node( p ).right_child == Some( v )
		}
		else {
			false
		}
	}
	
	/// Rotate `v` with its parent.
	fn rotate( &mut self, v : NodeIdx ) {
		TNodeData::before_rotation( self, v );
		
		let p = self.node( v ).parent.unwrap();
		if LOG_VERBOSE { println!( "rotate({v}) with parent {p}" ); }
		debug_assert!( [self.node( p ).left_child, self.node( p ).right_child].contains( &Some( v ) ),
			"{:?} does not contain {:?}", [self.node( p ).left_child, self.node( p ).right_child], &Some( v ) );
		
		// Update parent of v
		let g_opt = self.node( p ).parent;
		self.node_mut( v ).parent = g_opt;
		if let Some( g ) = g_opt {
			self.push_reverse_bit( g );
			if self.node( g ).left_child == Some( p ) {
				self.node_mut( g ).left_child = Some( v );
			}
			else if self.node( g ).right_child == Some( p ) {
				self.node_mut( g ).right_child = Some( v );
			}
		}
		
		self.push_reverse_bit( p );
		self.push_reverse_bit( v );
		
		// Update parents of p and (possibly) c
		self.node_mut( p ).parent = Some( v );
		if self.node( p ).left_child == Some( v ) {
			if let Some( c ) = self.node( v ).right_child {
				self.node_mut( c ).parent = Some( p );
				self.node_mut( p ).left_child = Some( c );
			}
			else {
				self.node_mut( p ).left_child = None;
			}
			self.node_mut( v ).right_child = Some( p );
		}
		else {
			debug_assert!( self.node( p ).right_child == Some( v ) );
			if let Some( c ) = self.node( v ).left_child {
				self.node_mut( c ).parent = Some( p );
				self.node_mut( p ).right_child = Some( c );
			}
			else {
				self.node_mut( p ).right_child = None;
			}
			self.node_mut( v ).left_child = Some( p );
		}
		
		if LOG_VERBOSE { println!( "{}", self.to_string() ); }
		if VERIFY { self._verify() }
	}
	
	fn get_parent_if_non_middle_child( &self, v : NodeIdx ) -> Option<NodeIdx> {
		if let Some( p ) = self.get_parent( v ) {
			if self.is_non_middle_child_hint( v, p ) {
				return Some( p )
			}
		}
		None
	}
	
	/// Splay `v` to the top of its solid subtree. Returns its parent afterwards, if it exists.
	fn splay_solid( &mut self, v : NodeIdx ) -> Option<NodeIdx> {
		if LOG_VERBOSE { println!( "splay_solid({v})" ); }
		loop {
			if let Some( p ) = self.get_parent( v ) {
				if self.is_non_middle_child_hint( v, p ) {
					if let Some( g ) = self.get_parent_if_non_middle_child( p ) {
						self.push_reverse_bit( g );
						self.push_reverse_bit( p );
						
						if self.node( p ).left_child == Some( v ) {
							if self.node( g ).left_child == Some( p ) {
								self.rotate( p );
								self.rotate( v );
							}
							else {
								debug_assert!( self.is_right_child( p ) );
								self.rotate( v );
								self.rotate( v );
							}
						}
						else if self.node( p ).right_child == Some( v ) {
							if self.node( g ).right_child == Some( p ) {
								self.rotate( p );
								self.rotate( v );
							}
							else {
								debug_assert!( self.is_left_child( p ) );
								self.rotate( v );
								self.rotate( v );
							}
						}
					}
					else {
						// p is root or middle child
						self.rotate( v );
					}
				}
				else {
					return Some( p )
				}
			}
			else {
				return None
			}
		}
		// Now v is a solid root
	}
	
	/// Make `v` the left child of its parent, if it has a parent, or does nothing otherwise.
	/// 
	/// Returns the parent of `v`
	fn try_splice( &mut self, v : NodeIdx ) -> Option<NodeIdx> {
		if let Some( p ) = self.node( v ).parent {
			if LOG_VERBOSE { println!( "splice({v})" ); }
			self.push_reverse_bit( p );
			TNodeData::before_splice( self, v );
			self.node_mut( p ).right_child = Some( v );
			if LOG_VERBOSE { println!( "{}", self.to_string() ); }
			if VERIFY { self._verify() }
			Some( p )
		}
		else {
			None
		}
	}
	
	/// Rotate the given node to the root, using a sequence of splays and splices.
	fn node_to_root( &mut self, v : NodeIdx ) {
		// println!( "node_to_root({v})" );
		// Splay in solid subtrees
		let mut x_opt = Some( v );
		while let Some( x ) = x_opt {
			x_opt = self.splay_solid( x );
		}
		
		// Splice
		let mut x_opt = Some( v );
		while let Some( x ) = x_opt {
			x_opt = self.try_splice( x );
		}
		
		// Splay a last time
		self.splay_solid( v );
		debug_assert!( self.node( v ).parent.is_none() );
	}
	
	/// Pushes reverse bit until the reverse bit is not set on any node in the solid subtree
	/// rooted at `v`.
	fn subtree_remove_reverse_bit( &mut self, v : NodeIdx ) {
		if !IMPL_EVERT {
			return
		}
		self.push_reverse_bit( v );
		if let Some( c ) = self.node( v ).left_child {
			self.subtree_remove_reverse_bit( c );
		}
		if let Some( c ) = self.node( v ).right_child {
			self.subtree_remove_reverse_bit( c );
		}
	}
	
	/// Transform the tree such that every solid subtree is a right spine.
	/// 
	/// In particular, after calling this method, the edges in the rooted trees (between parent and
	/// child) are exactly the edges of the represented tree.
	pub fn make_one_cut( &mut self ) {
		let mut seen_nodes : HashSet<NodeIdx> = HashSet::new();
		
		for v in self.nodes() {
			if self.node( v ).reversed {
				// Cleanup whole solid subtree
				let mut subtree_root = v;
				while let Some( p ) = self.get_parent( subtree_root ) {
					if [self.node( p ).left_child, self.node( p ).right_child].contains( &Some( subtree_root ) ) {
						subtree_root = p;
					}
					else {
						break;
					}
				}
				self.subtree_remove_reverse_bit( subtree_root );
			}
		}
		
		for v in self.nodes() {
			if !seen_nodes.contains( &v ) {
				// Find the solid root
				let mut r = v;
				while self.is_non_middle_child( r ) {
					r = self.node( r ).parent.unwrap();
				}
				
				// Rotate solid subtree into right spine
				let mut x_opt = Some( r );
				while let Some( x ) = x_opt {
					if let Some( c ) = self.node( x ).left_child {
						self.rotate( c );
						x_opt = Some( c );
					}
					else {
						seen_nodes.insert( x );
						x_opt = self.node( x ).right_child;
					}
				}
			}
		}
	}
	
	/// Make v the root of the tree and make sure it has no left child
	fn evert( &mut self, v : NodeIdx ) {
		debug_assert!( IMPL_EVERT );
		if LOG_VERBOSE { println!( "EVERT({v})" ); }
		self.node_to_root( v );
		if LOG_VERBOSE { println!( "{}", self.to_string() ); }
		
		// Get rid of the left child of v
		if let Some( c ) = self.node( v ).left_child {
			self.reverse( c );
			self.node_mut( v ).left_child = None
		}
		if LOG_VERBOSE { println!( "{}", self.to_string() ); }
	}
	
	/// Performs some sanity checks on the tree structure
	fn _verify( &self ) {
		for v in self.nodes() {
			if let Some( c ) = self.node( v ).left_child {
				assert_eq!( self.node(c).parent, Some( v ), "{c} has incorrect parent" );
			}
			if let Some( c ) = self.node( v ).right_child {
				assert_eq!( self.node(c).parent, Some( v ), "{c} has incorrect parent" );
			}
			if self.get_parent( v ).is_some() {
				self.data( v ).get_parent_path_weight(); // Shouldn't panic
			}
		}
	}
	
	/// Returns a textual representation of the rooted tree structure.
	pub fn to_string( &self ) -> String {
		let mut s = String::new();
		self._print( &mut s );
		s
	}

	fn _print( &self, out : &mut String ) {
		let mut children_map: HashMap<NodeIdx, Vec<NodeIdx>> = HashMap::new();
		for v in self.nodes() {
			if let Some( p ) = self.node( v ).parent {
				match children_map.get_mut( &p ) {
					Some( children ) => children.push( v ),
					None => { children_map.insert( p, vec![v] ); }
				}
			}
		}

		for v in self.nodes() {
			if self.node( v ).parent.is_none() {
				self._print_subtree( out, v, &children_map, "" );
			}
		}
	}

	fn _print_subtree( &self, out : &mut String, v : NodeIdx, children_map: &HashMap<NodeIdx, Vec<NodeIdx>>, indent : &str ) {
		out.push_str( indent );

		// Print node
		out.push_str( &format!( "{}", v.index()) );
		if let Some( p ) = self.node( v ).parent {
			if self.node( p ).left_child == Some( v ) {
				out.push_str( "L" );
			}
			else if self.node( p ).right_child == Some( v ) {
				out.push_str( "R" );
			}
		}
		if self.node( v ).reversed {
			out.push_str( "+" )
		}

		out.push_str( format!( "[{}]", self.data(v) ).as_str() );

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
		let mut child_it = children_map.get( &v).unwrap_or( &empty).iter().peekable();
		while let Some( c ) = child_it.next() {
			// Last symbol(s) before child depend on whether there is another child.
			let indent_symbol = if child_it.peek().is_none() { "└─" } else { "├─" };

			self._print_subtree(out, *c, children_map, format!( "{}{}", child_indent, indent_symbol ).as_str() );
		}
	}
	
	/// Returns the parent of `v` in the underlying (rooted) tree.
	/// 
	/// May return any neighbor of `v` if the underlying tree is not rooted.
	pub fn get_underlying_parent( &mut self, v : NodeIdx ) -> Option<NodeIdx> {
		self.node_to_root( v );
		self.node( v ).left_child
	}
}

impl<TNodeData : LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> RootedForest for LinkCutForest<TNodeData, IMPL_EVERT> {
	fn get_parent( &self, v: NodeIdx ) -> Option<NodeIdx> {
		self.node( v ).parent
	}
}

impl<TNodeData: LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> NodeDataAccess<TNodeData> for LinkCutForest<TNodeData, IMPL_EVERT> {
	fn data( &self, idx : NodeIdx ) -> &TNodeData {
		&self.node( idx ).data
	}
	
	fn data_mut( &mut self, idx : NodeIdx ) -> &mut TNodeData {
		&mut self.node_mut( idx ).data
	}
}

impl<TNodeData : LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> DynamicForest for LinkCutForest<TNodeData, IMPL_EVERT> {
	type TWeight = TNodeData::TWeight;
	
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_vertices : usize ) -> Self {
		LinkCutForest { nodes : (0..num_vertices).map( |i| Node::new( NodeIdx::new( i ) ) ).collect() }
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx, weight : TNodeData::TWeight ) {
		if LOG_VERBOSE { println!( "LINK({u}, {v}, {weight})" ); }
		self.node_to_root( u );
		self.node_to_root( v );
		
		self.evert( u );
		debug_assert!( self.node( v ).parent.is_none(), "Apparently attempting to link nodes in the same component" );
		self.node_mut( u ).parent = Some( v );
		
		TNodeData::after_attached( self, u, weight );
		
		if LOG_VERBOSE { println!( "{}", self.to_string() ) }
	}
	
	fn cut( &mut self, u : NodeIdx, v : NodeIdx ) {
		if LOG_VERBOSE { println!( "CUT({u}, {v})" ); }
		self.node_to_root( u );
		self.node_to_root( v );
		assert!( self.node( u ).parent.is_some(), "Apparently attempting to cut nodes in different components" );
		debug_assert!( self.node( u ).parent == Some( v ) );
		
		TNodeData::before_detached( self, u );
		
		self.node_mut( u ).parent = None;
		if self.node( v ).left_child == Some( u ) {
			self.node_mut( v ).left_child = None;
		}
		else if self.node( v ).right_child == Some( u ) {
			self.node_mut( v ).right_child = None;
		}
		
		if LOG_VERBOSE { println!( "{}", self.to_string() ) }
	}
	
	fn compute_path_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<TNodeData::TWeight> {
		if LOG_VERBOSE { println!( "COMPUTE_PATH_WEIGHT({u}, {v})" ); }
		self.node_to_root( u );
		self.node_to_root( v );
		
		// Compute path from u to root. Use the fact that u has depth at most 3, and is in the left or right spine of the root solid subtree
		let mut w = TNodeData::TWeight::identity();
		let mut x = u;
		while let Some( p ) = self.node( x ).parent {
			w = w + self.data( x ).get_parent_path_weight();
			x = p;
		}
		if x == v {
			Some( w )
		}
		else {
			None
		}
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.nodes.len()).map( |i| NodeIdx::new( i ) )
	}
	
	fn edges(&self ) -> Vec<(NodeIdx, NodeIdx)> {
		if LOG_VERBOSE { println!( "UNDERLYING_EDGES()" ); }
		// Make f one-cut
		let mut f = self.clone();
		f.make_one_cut();
		f.nodes().filter_map( |v| {
			if let Some( p ) = f.node( v ).parent {
				Some( (v, p) )
			}
			else {
				None
			}
		} ).collect()
	}
}


/// LCTNodeData capable of storing arbitrary Monoids as edge weights
#[derive(Clone)]
pub struct MonoidPathWeightLCTNodeData<TWeight : MonoidWeight> {
	/// Path weight to parent
	pdist : WeightOrInfinity<TWeight>,
	/// Path weight to lowest ancestor on the other side than parent. Dashed parents are considered left parents.
	adist : WeightOrInfinity<TWeight>
}

impl<TWeight: MonoidWeight> NodeData for MonoidPathWeightLCTNodeData<TWeight> {
	type TWeight = TWeight;
	
	fn new( _ : NodeIdx ) -> Self {
		MonoidPathWeightLCTNodeData { pdist : Infinite, adist : Infinite }
	}
}

impl<TWeight: MonoidWeight> Display for MonoidPathWeightLCTNodeData<TWeight> {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}/{}", self.pdist, self.adist )
	}
}

impl<TWeight: MonoidWeight> PathWeightNodeData for MonoidPathWeightLCTNodeData<TWeight> {
	fn get_parent_path_weight( &self ) -> Self::TWeight {
		self.pdist.unwrap()
	}
}

impl<TWeight : MonoidWeight> LCTNodeData<true> for MonoidPathWeightLCTNodeData<TWeight> {
	fn before_rotation(f: &mut LinkCutForest<Self, true>, v: NodeIdx ) {
		// Find parent and (possibly) grandparent
		let p = f.node( v ).parent.unwrap();
		
		if let Some( g ) = f.node( p ).parent {
			f.push_reverse_bit( g );
		}
		
		// Ensure left/right children are consistent for p, v, and v's children
		f.push_reverse_bit( p );
		f.push_reverse_bit( v );
		
		// Find and handle child that switches parent from v to p
		let c_opt = if f.is_left_child( v ) {
			f.node( v ).right_child
		} else {
			f.node( v ).left_child
		};
		
		if let Some( c ) = c_opt {
			// Swapping dist(c,p) and dist(c,v)
			let c_data = f.data_mut( c );
			(c_data.pdist, c_data.adist) = (c_data.adist, c_data.pdist);
		}

		// Remember old distances
		let old_v_data = f.data(v ).clone();
		let old_p_data = f.data( p ).clone();
		
		// v is now parent of p
		f.data_mut( p ).pdist = old_v_data.pdist; // dist(p,v)
		
		debug_assert!( f.is_non_middle_child( v ) );
		
		// Note: dashed children are considered right children here
		if ( f.node( p ).left_child == Some( v ) ) != f.is_left_child( p ) {
			// v is between p and g in the underlying tree, or p is the root of the whole link-cut tree
			
			f.data_mut( v ).pdist = old_v_data.adist; // dist(v,g) or infinity, if g doesn't exist
			
			// The relevant ancestors of v now and p previously are the same (possibly non-existent)
			f.data_mut( v ).adist = old_v_data.pdist + old_p_data.adist; // dist(v,p) + dist(p,a) = dist(v,a)
			
			// p.adist does not change
		}
		else {
			// p is between v and g in the underlying tree, or p is the root of the whole link-cut tree
			f.data_mut( v ).pdist = old_v_data.pdist + old_p_data.pdist; // dist(v,p) + dist(p,g) = dist(v,g)
			// v_data.adist does not change
			f.data_mut( p ).adist = old_p_data.pdist; // dist(p,gp)
		}
	}
	
	fn before_splice(f : &mut LinkCutForest<Self, true>, v : NodeIdx ) {
		debug_assert!( !f.is_non_middle_child( f.node( v ).parent.unwrap() ) );
		// Under the above assumption, there is nothing to do
	}
	
	fn after_attached(f : &mut LinkCutForest<Self, true>, v : NodeIdx, weight : Self::TWeight ) {
		f.data_mut( v ).pdist = Finite( weight );
	}
	
	fn before_detached(f: &mut LinkCutForest<Self, true>, v: NodeIdx) {
		f.data_mut( v ).pdist = Infinite
	}
}

/// LCTNodeData capable of storing arbitrary groups as edge weights
#[derive(Clone)]
pub struct GroupPathWeightLCTNodeData<TWeight : GroupWeight> {
	/// Path weight to parent
	pdist : WeightOrInfinity<TWeight>
}

impl<TWeight: GroupWeight> NodeData for GroupPathWeightLCTNodeData<TWeight> {
	type TWeight = TWeight;
	
	fn new( _ : NodeIdx ) -> Self {
		Self { pdist : Infinite }
	}
}

impl<TWeight: GroupWeight> Display for GroupPathWeightLCTNodeData<TWeight> {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.pdist )
	}
}

impl<TWeight: GroupWeight> PathWeightNodeData for GroupPathWeightLCTNodeData<TWeight> {
	fn get_parent_path_weight( &self ) -> Self::TWeight {
		self.pdist.unwrap()
	}
}

impl<TWeight : GroupWeight> LCTNodeData<true> for GroupPathWeightLCTNodeData<TWeight> {
	fn before_rotation( f : &mut LinkCutForest<Self, true>, v : NodeIdx ) {
		// Find parent and (possibly) grandparent
		let p = f.node( v ).parent.unwrap();
		
		if let Some( g ) = f.node( p ).parent {
			f.push_reverse_bit( g );
		}
		
		// Ensure left/right children are consistent for p, v, and v's children
		f.push_reverse_bit( p );
		f.push_reverse_bit( v );
		
		// Find and handle child that switches parent from v to p
		let c_opt = if f.is_left_child( v ) {
			f.node( v ).right_child
		} else {
			f.node( v ).left_child
		};

		// Remember old distances
		let dist_v_p = f.data(v ).pdist;
		let dist_p_g = f.data( p ).pdist;
		
		if let Some( c ) = c_opt {
			f.data_mut( c ).pdist = dist_v_p - f.data( c ).pdist.unwrap(); // dist(v,p) - dist(c,v)
		}
		
		// v is now parent of p
		f.data_mut( p ).pdist = dist_v_p;
		
		debug_assert!( f.is_non_middle_child( v ) );
		
		// Note: dashed children are considered right children here
		if ( f.node( p ).left_child == Some( v ) ) != f.is_left_child( p ) {
			// v is between p and g in the underlying tree, or p is the root of the whole link-cut tree
			
			f.data_mut( v ).pdist = dist_p_g - dist_v_p.unwrap();
		}
		else {
			f.data_mut( v ).pdist = dist_v_p + dist_p_g;
		}
	}
	
	fn before_splice( _ : &mut LinkCutForest<Self, true>, _ : NodeIdx ) {
		// Nothing to do
	}
	
	fn after_attached( f : &mut LinkCutForest<Self, true>, v : NodeIdx, weight : Self::TWeight ) {
		f.data_mut( v ).pdist = Finite( weight );
	}
	
	fn before_detached( f: &mut LinkCutForest<Self, true>, v: NodeIdx ) {
		f.data_mut( v ).pdist = Infinite
	}
}


impl RootedDynamicForest for LinkCutForest<EmptyNodeData, false> {
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_vertices : usize ) -> Self {
		LinkCutForest { nodes : (0..num_vertices).map( |i| Node::new( NodeIdx::new( i ) ) ).collect() }
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.nodes.len()).map( |i| NodeIdx::new( i ) )
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		if LOG_VERBOSE { println!( "LINK({u}, {v})" ); }
		self.node_to_root( u );
		self.node_to_root( v );
		assert!( self.node( u ).parent.is_none(), "Apparently attempting to link nodes in the same component" );
		
		self.node_mut( u ).parent = Some( v );
		
		if LOG_VERBOSE { println!( "{}", self.to_string() ) }
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		if LOG_VERBOSE { println!( "CUT({v})" ); }
		self.node_to_root( v );
		
		let underlying_parent = self.node( v ).left_child.unwrap(); // Error if v is the root
		self.node_mut( v ).left_child = None;
		self.node_mut( underlying_parent ).parent = None;
	}
	
	fn cut_edge( &mut self, u : NodeIdx, v : NodeIdx ) {
		if LOG_VERBOSE { println!( "CUT_EDGE({u}, {v})" ); }
		self.node_to_root( u );
		self.node_to_root( v );
		assert!( self.node( u ).parent.is_some(), "Apparently attempting to cut nodes in different components" );
		debug_assert!( self.node( u ).parent == Some( v ) );
		
		self.node_mut( u ).parent = None;
		if self.node( v ).left_child == Some( u ) {
			self.node_mut( v ).left_child = None;
		}
		else if self.node( v ).right_child == Some( u ) {
			self.node_mut( v ).right_child = None;
		}
		
		if LOG_VERBOSE { println!( "{}", self.to_string() ) }
	}
	
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx {
		self.node_to_root( v );
		let mut r = v;
		while let Some( x ) = self.node( r ).left_child {
			r = x;
		}
		self.node_to_root( r ); // Only for amortized analysis
		r
	}
	
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx> {
		self.node_to_root( u );
		
		let mut last_solid_leaf = v; // Will be the lowest ancestor of v on the root path
		let mut x = v; // Will be child of root (u) that is v or an ancestor of v
		while let Some( p ) = self.get_parent( x ) {
			if !self.is_non_middle_child( x ) {
				// x is a middle child
				last_solid_leaf = p
			}
			
			if p != u {
				x = p;
			}
			else {
				break;
			}
		}
		
		if self.node( x ).parent.is_none() {
			return None; // u and v are in different trees
		}
		
		let lca;
		if self.is_left_child( x ) {
			// last_solid_leaf is to the left of u on the root path, i.e., above u in the underlying tree
			lca = last_solid_leaf;
		}
		else {
			// last_solid_leaf is u or to the right of u on the root path, i.e., below u in the underlying tree
			lca = u;
		}
		
		self.node_to_root( v ); // Only for amortized analysis
		
		Some( lca )
	}
}

