//! Extension of dynamic trees to maintain unweighted rooted forests.

use std::collections::{HashMap, HashSet};
use std::iter::Map;
use std::ops::Range;
use crate::NodeIdx;

/// An unweighted rooted dynamic forest.
pub trait RootedDynamicForest {
	/// Iterator for nodes
	type NodeIdxIterator : Iterator<Item = NodeIdx>;
	
	/// Creates a new dynamic forest with the specified number of nodes and no edges.
	fn new( num_vertices : usize ) -> Self;

	/// Iterate over the nodes in this dynamic forest.
	fn nodes( &self ) -> Self::NodeIdxIterator;

	/// Adds `u` as a child to `v`. `u` must not have a parent, and must be in a different tree than
	/// `v`.
	fn link( &mut self, u : NodeIdx, v : NodeIdx );

	/// Removes `u` from its parent. `u` must have a parent.
	fn cut( &mut self, v : NodeIdx );
	
	/// Return the root of the tree containing `v`.
	fn find_root( &mut self, v : NodeIdx ) -> NodeIdx;
	
	/// Returns the lowest common ancestor of `u` and `v`, or `None` if `u` and `v` are in different
	/// trees.
	fn lowest_common_ancestor( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<NodeIdx>;
}

/// An unweighted rooted dynamic forest which allows changing the root.
pub trait EversibleRootedDynamicForest : RootedDynamicForest {
	/// Makes `u` the root of the underlying forest
	fn make_root( &mut self, v : NodeIdx );
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

	/// Returns the parent of the given node in the underlying tree
	pub fn get_parent( &self, v : NodeIdx ) -> Option<NodeIdx> {
		self.node( v ).parent
	}

	/// Produces a multi-line string representation of the forest
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
			if self.node( v ).parent.is_none() {
				self._print_subtree( out, v, &child_map, "" );
			}
		}
	}

	fn _print_subtree( &self, out : &mut String, root : NodeIdx, child_map: &HashMap<NodeIdx, Vec<NodeIdx>>, indent : &str ) {
		out.push_str( indent );

		// Print node
		out.push_str( &format!( "{}\n", root.index()) );

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
}

impl RootedDynamicForest for SimpleRootedForest {
	type NodeIdxIterator = Map<Range<usize>, fn(usize) -> NodeIdx>;
	
	fn new( num_vertices : usize ) -> Self {
		SimpleRootedForest{ nodes : (0..num_vertices).map( |_| SimpleRootedNode::new() ).collect() }
	}
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		(0..self.nodes.len()).map( NodeIdx::new )
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx ) {
		debug_assert!( self.node( u ).parent == None );
		self.node_mut( u ).parent = Some( v );
	}
	
	fn cut( &mut self, v : NodeIdx ) {
		debug_assert!( self.node( v ).parent.is_some() );
		self.node_mut( v ).parent = None;
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

impl EversibleRootedDynamicForest for SimpleRootedForest {
	fn make_root( &mut self, v : NodeIdx ) {
		if let Some( p ) = self.node( v ).parent {
			self.node_mut( v ).parent = None;
			let mut x = v; // First node of remaining path
			let mut y = p; // Second node of remaining path
			while let Some( p ) = self.node( y ).parent {
				self.node_mut( y ).parent = Some( x );
				x = y;
				y = p;
			}
			self.node_mut( y ).parent = Some( x );
		}
	}
}
