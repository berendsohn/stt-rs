//! Dynamic forest implementations based on splay trees.

use crate::common::EmptyNodeData;
use crate::NodeIdx;
use crate::twocut::{ExtendedNTRStrategy, NodesToTopPWImpl, StableNodesToTopPWImpl, StableNTRImplementation, StableNTRStrategy, StandardDynamicForest, ExtendedNTRImplementation};
use crate::twocut::basic::{STTRotate, STTStructureRead};
use crate::twocut::node_data::{GroupPathWeightNodeData, MonoidPathWeightNodeData};
use crate::twocut::rooted::StandardRootedDynamicForest;

/// A dynamic tree using [GreedySplayStrategy].
pub type GreedySplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, ExtendedNTRImplementation<GreedySplayStrategy>, NodesToTopPWImpl<GreedySplayStrategy>>;
/// A dynamic tree using [GreedySplayStrategy] with monoid edge weights.
pub type MonoidGreedySplayTT<TWeight> = GreedySplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [GreedySplayStrategy] with group edge weights.
pub type GroupGreedySplayTT<TWeight> = GreedySplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [GreedySplayStrategy] without edge weights.
pub type EmptyGreedySplayTT = GreedySplayTT<EmptyNodeData>;

/// A dynamic tree using [GreedySplayStrategy] as a stable strategy.
pub type StableGreedySplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, StableNTRImplementation<GreedySplayStrategy>, StableNodesToTopPWImpl<GreedySplayStrategy>>;
/// A dynamic tree using [GreedySplayStrategy] as a stable strategy with monoid edge weights.
pub type MonoidStableGreedySplayTT<TWeight> = StableGreedySplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [GreedySplayStrategy] as a stable strategy with group edge weights.
pub type GroupStableGreedySplayTT<TWeight> = StableGreedySplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [GreedySplayStrategy] as a stable strategy without edge weights.
pub type EmptyStableGreedySplayTT = StableGreedySplayTT<EmptyNodeData>;

/// A rooted dynamic forest using [GreedySplayStrategy].
pub type RootedGreedySplayTT = StandardRootedDynamicForest<GreedySplayStrategy>;

/// A dynamic tree using [TwoPassSplayStrategy].
pub type TwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, ExtendedNTRImplementation<TwoPassSplayStrategy>, NodesToTopPWImpl<TwoPassSplayStrategy>>;
/// A dynamic tree using [TwoPassSplayStrategy] with monoid edge weights.
pub type MonoidTwoPassSplayTT<TWeight> = TwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [TwoPassSplayStrategy] with group edge weights.
pub type GroupTwoPassSplayTT<TWeight> = TwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [TwoPassSplayStrategy] without edge weights.
pub type EmptyTwoPassSplayTT = TwoPassSplayTT<EmptyNodeData>;

/// A dynamic tree using [TwoPassSplayStrategy], stable variant.
pub type StableTwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, StableNTRImplementation<TwoPassSplayStrategy>, StableNodesToTopPWImpl<TwoPassSplayStrategy>>;
/// A dynamic tree using [TwoPassSplayStrategy] with monoid edge weights, stable variant.
pub type MonoidStableTwoPassSplayTT<TWeight> = StableTwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [TwoPassSplayStrategy] with group edge weights, stable variant.
pub type GroupStableTwoPassSplayTT<TWeight> = StableTwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [TwoPassSplayStrategy] without edge weights, stable variant.
pub type EmptyStableTwoPassSplayTT = StableTwoPassSplayTT<EmptyNodeData>;

/// A rooted dynamic forest using [TwoPassSplayStrategy].
pub type RootedTwoPassSplayTT = StandardRootedDynamicForest<TwoPassSplayStrategy>;

/// A dynamic tree using [LocalTwoPassSplayStrategy].
pub type LocalTwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, ExtendedNTRImplementation<LocalTwoPassSplayStrategy>, NodesToTopPWImpl<LocalTwoPassSplayStrategy>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] with monoid edge weights.
pub type MonoidLocalTwoPassSplayTT<TWeight> = LocalTwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] with group edge weights.
pub type GroupLocalTwoPassSplayTT<TWeight> = LocalTwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] without edge weights.
pub type EmptyLocalTwoPassSplayTT = LocalTwoPassSplayTT<EmptyNodeData>;

/// A dynamic tree using [LocalTwoPassSplayStrategy], stable variant.
pub type StableLocalTwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, StableNTRImplementation<LocalTwoPassSplayStrategy>, StableNodesToTopPWImpl<LocalTwoPassSplayStrategy>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] with monoid edge weights, stable variant.
pub type MonoidStableLocalTwoPassSplayTT<TWeight> = StableLocalTwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] with group edge weights, stable variant.
pub type GroupStableLocalTwoPassSplayTT<TWeight> = StableLocalTwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [LocalTwoPassSplayStrategy] without edge weights, stable variant.
pub type EmptyStableLocalTwoPassSplayTT = StableLocalTwoPassSplayTT<EmptyNodeData>;

/// A rooted dynamic forest using [LocalTwoPassSplayStrategy].
pub type RootedLocalTwoPassSplayTT = StandardRootedDynamicForest<LocalTwoPassSplayStrategy>;


// Common functions

/// Moves `x` up two steps. `x` must have depth at least 3.
/// 
/// `p` must be the parent of `x`.
fn splay_step(f : &mut (impl STTRotate + STTStructureRead), x : NodeIdx, p : NodeIdx ) {
	if f.is_direct_separator_hint( x, p ) { // T_x separates p and g
		f.rotate( x );
	}
	else { // p separates x and g
		f.rotate( p );
	}
	f.rotate( x );
}


/**
A greedy splay strategy.

Brings a node `v` to the top by repeatedly trying to do a splay step on `v`, or the parent of
`v`, or the grandparent of `v` (it can be shown that one of the three does work).
*/
#[derive(Clone)]
pub struct GreedySplayStrategy {}

impl StableNTRStrategy for GreedySplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		while let Some( p ) = f.get_parent( v ) {
			if let Some( g ) = f.get_parent( p ) {
				if let Some( gg ) = f.get_parent( g ) {
					let v_sep = f.is_separator_hint( v, p );
					let p_sep = f.is_separator_hint( p, g );
					let g_sep = f.is_separator_hint( g, gg );
					if ( v_sep && p_sep ) || !g_sep { // Can splay at v
						splay_step( f, v, p );
					}
					else { // Cannot splay at v
						if let Some( ggg ) = f.get_parent( gg ) {
							let gg_sep = f.is_separator_hint( gg, ggg );
							if ( p_sep && g_sep ) || !gg_sep { // Can splay at p
								splay_step( f, p, g );
							}
							else { // Cannot splay at p; splaying at g must be allowed
								splay_step( f, g, gg );
							}
						}
						else {
							// gg is root; splaying at p must be allowed
							splay_step( f, p, g );
						}
					}
				}
				else {
					// g is root, splaying at v must be allowed
					splay_step( f, v, p );
				}
			}
			else {
				// p is root
				f.rotate( v );
			}
		}
	}
}

impl ExtendedNTRStrategy for GreedySplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as StableNTRStrategy>::node_to_root( f, v );
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		loop {
			let p = f.get_parent( v ).unwrap();
			if let Some( g ) = f.get_parent( p ) {
				if let Some( gg ) = f.get_parent( g ) {
					let v_sep = f.is_separator_hint( v, p );
					let p_sep = f.is_separator_hint( p, g );
					let g_sep = f.is_separator_hint( g, gg );
					if ( v_sep && p_sep ) || !g_sep { // Can splay at v
						splay_step( f, v, p );
					}
					else { // Cannot splay at v
						if let Some( g3 ) = f.get_parent( gg ) {
							let gg_sep = f.is_separator_hint( gg, g3 );
							if ( p_sep && g_sep ) || !gg_sep { // Can splay at p
								splay_step( f, p, g );
							}
							else { // Cannot splay at p
								if f.get_parent( g3 ).is_some() {
									// g3 is not the root
									splay_step( f, g, gg );
								}
								else { // g3 is the root
									f.rotate( g );
								}
							}
						}
						else { // gg is the root
							f.rotate( p );
						}
					}
				}
				else { // g is the root
					f.rotate( v );
					return
				}
			}
			else { // p is the root
				return
			}
		}
	}
}


/**
A two-pass splay strategy.

To bring a node `v` to the top, first "clean" the root path of `v` by removing all so-called
"branching nodes", then splay `v` to the top.
*/
#[derive(Clone)]
pub struct TwoPassSplayStrategy {}

impl TwoPassSplayStrategy {
	/// Let v = v_1, v_2, v_3, ... be the root path of v. Find the node v_i such that v_{i-1} is
	/// not a separator and v_i is a separator (i.e., v_i is a branching node), and i is minimal
	/// (hence "next").
	fn find_next_branching_node( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) -> Option<NodeIdx> {
		let mut u = v;
		while let Some( p ) = f.get_parent( u ) {
			if f.can_rotate_hint( u, p ) {
				u = p;
			}
			else {
				return Some( p );
			}
		}
		return None;
	}
}


impl StableNTRStrategy for TwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		if let Some( b ) = Self::find_next_branching_node( f, v ) {
			let mut x = b;
			while let Some( p ) = f.get_parent( x ) {
				if let Some( g ) = f.get_parent( p ) {
					if f.is_separator_hint( p, g ) {
						// g is not a branching node
						if f.is_separator_hint( x, p ) {
							// p is not a branching node
							splay_step( f, x, p );
						}
						else {
							// p is a branching node
							x = p;
						}
					}
					else { 
						// p is not a branching node
						if f.is_separator( g ) {
							// g is a branching node
							f.rotate( x );
							x = g;
						}
						else {
							// g is not a branching node
							splay_step( f, x, p );
						}
					}
				}
				else { // p is root
					f.rotate( x );
					break
				}
			}
		}
		
		
		while let Some( p ) = f.get_parent( v ) {
			if f.get_parent( p ).is_some() {
				// v has a grandparent
				splay_step( f, v, p );
			}
			else {
				// v has a parent, but no grandparent
				f.rotate( v );
				return
			}
		}
	}
}

impl ExtendedNTRStrategy for TwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as StableNTRStrategy>::node_to_root( f, v );
	}
	
	fn node_below_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		if let Some( b ) = Self::find_next_branching_node( f, v ) {
			let mut x = b;
			while let Some( p ) = f.get_parent( x ) {
				if let Some( g ) = f.get_parent( p ) {
					if let Some( gg ) = f.get_parent( g ) {
						if f.is_separator_hint( p, g ) {
							// g is not a branching node
							if f.is_separator_hint( x, p ) {
								// p is not a branching node
								splay_step( f, x, p );
							}
							else {
								// p is a branching node
								x = p;
							}
						}
						else { 
							// p is not a branching node
							if f.is_separator_hint( g, gg ) {
								// g is a branching node
								f.rotate( x );
								x = g;
							}
							else {
								// g is not a branching node
								splay_step( f, x, p );
							}
						}
					}
					else {
						// g is the root
						f.rotate( x );
						break
					}
				}
				else { // p is the root
					break
				}
			}
		}
		
		loop {
			let p = f.get_parent( v ).unwrap();
			if let Some( g ) = f.get_parent( p ) {
				if f.get_parent( g ).is_some() {
					// g is not the root
					splay_step( f, v, p );
				}
				else {
					// g is the root
					f.rotate( v );
					return
				}
			}
			else {
				return
			}
		}
	}
}


/// A variant of [TwoPassSplayStrategy] executing the two passes in an interleaved manner.
#[derive(Clone)]
pub struct LocalTwoPassSplayStrategy {}

impl StableNTRStrategy for LocalTwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		while let Some( p ) = f.get_parent( v ) {
			if let Some( g ) = f.get_parent( p ) {
				let p_sep = f.is_separator_hint( p, g );
				// Try splaying at v without information about g's separator status.
				if f.is_separator_hint( v, p ) && p_sep {
					splay_step( f, v, p );
				}
				// Either v or p is not a separator
				else if let Some( gg ) = f.get_parent( g ) {
					if !f.is_separator_hint( g, gg ) {
						splay_step( f, v, p );
					}
					else if p_sep { // g_sep and p_sep => can splay at p
						splay_step( f, p, g );
					}
					else {
						// ggg (grandparent of g) must exist, since g is a separator
						let ggg = f.get_parent( gg ).unwrap();
						
						if f.is_separator_hint( gg, ggg ) || !f.is_separator( ggg ) {
							// g must be a separator, so we can splay at g
							splay_step( f, g, gg );
						}
						else {
							// ggg is a branching node
							f.rotate( g );
						}
					}
				}
				else {
					// g is root
					splay_step( f, v, p );
				}
			}
			else {
				// p is root
				f.rotate( v );
			}
		}
	}
}

impl ExtendedNTRStrategy for LocalTwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as StableNTRStrategy>::node_to_root( f, v );
	}

	fn node_below_root( f : &mut (impl STTRotate + STTStructureRead ), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );

		while let Some( p ) = f.get_parent( v ) {
			if let Some( g ) = f.get_parent( p ) {
				let p_sep = f.is_separator_hint( p, g );
				// Try splaying at v without information about g's separator status.
				if f.is_separator_hint( v, p ) && p_sep {
					// Below root: p_sep, so depth(p) >= 3, so we may splay at v
					splay_step( f, v, p );
				}
				// Either v or p is not a separator
				else if let Some( gg ) = f.get_parent( g ) {
					if !f.is_separator_hint( g, gg ) {
						// Below root: gg exists, so we may splay at v
						splay_step( f, v, p );
					}
					else if p_sep { // g_sep and p_sep => can splay at p
						// Below root: g_sep, so depth(g) >= 3, so we may splay at p
						splay_step( f, p, g );
					}
					else {
						// ggg (grandparent of g) must exist, since g is a separator
						let ggg = f.get_parent( gg ).unwrap();

						if f.is_separator_hint( gg, ggg ) || !f.is_separator( ggg ) {
							// g must be a separator, so we can splay at g
							if f.get_parent( ggg ).is_some() {
								splay_step( f, g, gg );
							}
							else {
								f.rotate( g );
							}
						}
						else {
							// ggg is a branching node
							// Below root: ggg is not the root, so we may splay at g
							f.rotate( g );
						}
					}
				}
				else {
					// g is root
					// Below root: only rotate at v
					f.rotate( v );
				}
			}
			else {
				// p is root
				// Below root: We're done.
				break
			}
		}
	}
}
