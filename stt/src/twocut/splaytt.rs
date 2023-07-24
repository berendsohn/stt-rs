//! Dynamic forest implementations based on splay trees.

use crate::common::EmptyNodeData;
use crate::NodeIdx;
use crate::twocut::{ExtendedNTRStrategy, NodesToTopPWImpl, StableNodesToTopPWImpl, StableNTRImplementation, StableNTRStrategy, StandardDynamicForest, ExtendedNTRImplementation};
use crate::twocut::basic::{STTRotate, STTStructureRead};
use crate::twocut::node_data::{GroupPathWeightNodeData, MonoidPathWeightNodeData};
use crate::twocut::rooted::StandardRootedDynamicForest;
use crate::twocut::splaytt::SplayTarget::*;

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

/// A dynamic tree using [StableTwoPassSplayStrategy].
pub type StableTwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, StableNTRImplementation<StableTwoPassSplayStrategy>, StableNodesToTopPWImpl<StableTwoPassSplayStrategy>>;
/// A dynamic tree using [StableTwoPassSplayStrategy] with monoid edge weights.
pub type MonoidStableTwoPassSplayTT<TWeight> = StableTwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [StableTwoPassSplayStrategy] with group edge weights.
pub type GroupStableTwoPassSplayTT<TWeight> = StableTwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [StableTwoPassSplayStrategy] without edge weights.
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

/// A dynamic tree using [StableLocalTwoPassSplayStrategy].
pub type StableLocalTwoPassSplayTT<TNodeData>
	= StandardDynamicForest<TNodeData, StableNTRImplementation<StableLocalTwoPassSplayStrategy>, StableNodesToTopPWImpl<StableLocalTwoPassSplayStrategy>>;
/// A dynamic tree using [StableLocalTwoPassSplayStrategy] with monoid edge weights.
pub type MonoidStableLocalTwoPassSplayTT<TWeight> = StableLocalTwoPassSplayTT<MonoidPathWeightNodeData<TWeight>>;
/// A dynamic tree using [StableLocalTwoPassSplayStrategy] with group edge weights.
pub type GroupStableLocalTwoPassSplayTT<TWeight> = StableLocalTwoPassSplayTT<GroupPathWeightNodeData<TWeight>>;
/// A dynamic tree using [StableLocalTwoPassSplayStrategy] without edge weights.
pub type EmptyStableLocalTwoPassSplayTT = StableLocalTwoPassSplayTT<EmptyNodeData>;

/// A rooted dynamic forest using [LocalTwoPassSplayStrategy].
pub type RootedLocalTwoPassSplayTT = StandardRootedDynamicForest<LocalTwoPassSplayStrategy>;


// Common functions

fn can_splay_step( f : &mut (impl STTRotate + STTStructureRead), x : NodeIdx, p : NodeIdx, g : NodeIdx ) -> bool {
	!f.is_separator( g ) || ( f.is_separator( x ) && f.is_separator( p ) )
}


/// Moves x up two steps. x must have depth at least 3.
fn splay_step( f : &mut (impl STTRotate + STTStructureRead), x : NodeIdx ) {
	let p = f.get_parent( x ).unwrap();
	if f.is_direct_separator( x ) { // T_x separates p and g
		f.rotate( x );
	}
	else { // p separates x and g
		f.rotate( p );
	}
	f.rotate( x );
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SplayTarget {
	BelowRoot,
	Root
}


/**
A greedy splay strategy.

Brings a node `v` to the top by repeatedly trying to do a splay step on `v`, or the parent of
`v`, or the grandparent of `v` (it can be shown that one of the three does work).
*/
#[derive(Clone)]
pub struct GreedySplayStrategy {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SplayResult {
	Success, // Successfully splayed
	Failed, // Could not splay, but we're not done yet
	Done
}

impl GreedySplayStrategy {
	// Tries to Splay at v.
	fn try_splay( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx, target : SplayTarget ) -> SplayResult {
		if let Some( p ) = f.get_parent( v ) {
			if let Some( g ) = f.get_parent( p ) {
				if target == Root || f.get_parent( g ).is_some() {
					if can_splay_step(f, v, p, g) {
						splay_step(f, v);
						SplayResult::Success
					}
					else {
						SplayResult::Failed
					}
				}
				else {
					// g is root and we want to splay v below g
					f.rotate(v);
					SplayResult::Done
				}
			}
			else {
				// p is root
				if target == Root {
					f.rotate( v );
				}
				SplayResult::Done
			}
		}
		else {
			// v is root
			SplayResult::Done
		}
	}

	fn move_to( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx, target : SplayTarget ) {
		loop {
			match Self::try_splay( f, v, target ) {
				SplayResult::Success => {},
				SplayResult::Failed => {
					let p = f.get_parent( v ).unwrap();
					if Self::try_splay( f, p, target ) == SplayResult::Failed {
						let g = f.get_parent( p ).unwrap();
						splay_step( f, g ); // Must work
					}
					// Else: successfully splayed
				},
				SplayResult::Done => { return }
			}
		}
	}
}

impl ExtendedNTRStrategy for GreedySplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		Self::move_to( f, v, Root );
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		Self::move_to( f, v, BelowRoot );
		debug_assert!( f.get_parent( v ).is_some() && f.get_parent( f.get_parent( v ).unwrap() ).is_none() );
	}
}

impl StableNTRStrategy for GreedySplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as ExtendedNTRStrategy>::node_to_root( f, v );
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
	fn find_next_branching_node( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) -> Option<NodeIdx> {
		let mut u = v;
		while f.can_rotate( u ) {
			u = f.get_parent( u ).unwrap();
		}
		f.get_parent( u ) // Might be None, then there is no branching node on the path
	}

	// Do a splay_step or rotation on v
	fn branching_step( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx,
			target : SplayTarget )
	{
		debug_assert!( f.is_separator( v ) );
		// v is a separator, and thus has depth at least 3
		let p = f.get_parent( v ).unwrap();
		let g = f.get_parent( p ).unwrap();

		if !f.is_separator( p ) && f.is_separator( g ) {
			// g is branching node
			f.rotate( v );
			return
		}
		else if f.get_parent( g ).is_none() {
			// g is the root
			match target {
				BelowRoot => f.rotate( v ),
				Root => splay_step( f, v )
			};
			return
		}
		else {
			splay_step( f, v );
			return
		}
	}

	fn move_to( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx,
			target : SplayTarget )
	{
		let mut b_opt = Self::find_next_branching_node( f, v );
		while let Some( b ) = b_opt {
			// Found a branching node on the path
			Self::branching_step( f, b, target );
			if !f.is_separator( b ) {
				b_opt = Self::find_next_branching_node( f, b );
			}
		}

		// Now splay v to the target without worrying about branching nodes
		loop {
			if let Some( p ) = f.get_parent( v ) {
				if let Some( g ) = f.get_parent( p ) {
					// v has a grandparent
					if target == Root || f.get_parent( g ).is_some() {
						splay_step( f, v );
					}
					else {
						f.rotate( v );
						return
					}
				}
				else {
					// v has a parent, but no grandparent
					if target == Root {
						f.rotate( v );
					}
					return
				}
			}
			else {
				// v has no parent
				return;
			}
		}
	}
}

impl ExtendedNTRStrategy for TwoPassSplayStrategy {
	fn node_to_root(f: &mut (impl STTRotate + STTStructureRead), v: NodeIdx ) {
		Self::move_to( f, v, Root );
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root(f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		Self::move_to( f, v, BelowRoot );
		debug_assert!( f.get_parent( v ).is_some() && f.get_parent( f.get_parent( v ).unwrap() ).is_none() );
	}
}


/// A simplified variant of [TwoPassSplayStrategy].
#[derive(Clone)]
pub struct StableTwoPassSplayStrategy {}

impl StableTwoPassSplayStrategy {
	fn find_next_branching_node( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) -> Option<NodeIdx> {
		let mut u = v;
		while f.can_rotate( u ) {
			u = f.get_parent( u ).unwrap();
		}
		f.get_parent( u ) // Might be None, then there is no branching node on the path
	}

	// Do a splay_step or rotation on v
	fn branching_step( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx )
	{
		debug_assert!( f.is_separator( v ) );
		// v is a separator, and thus has depth at least 3
		let p = f.get_parent( v ).unwrap();
		let g = f.get_parent( p ).unwrap();

		if !f.is_separator( p ) && f.is_separator( g ) {
			// g is branching node
			f.rotate( v );
		}
		else {
			splay_step( f, v );
		}
	}
}

impl StableNTRStrategy for StableTwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		let mut b_opt = Self::find_next_branching_node( f, v );
		while let Some( b ) = b_opt {
			// Found a branching node on the path
			Self::branching_step( f, b );
			if !f.is_separator( b ) {
				b_opt = Self::find_next_branching_node( f, b );
			}
		}

		// Now splay v to the target without worrying about branching nodes
		loop {
			if let Some( p ) = f.get_parent( v ) {
				if f.get_parent( p ).is_some() {
					// v has a grandparent
					splay_step( f, v );
				}
				else {
					// v has a parent, but no grandparent
					f.rotate( v );
					return
				}
			}
			else {
				// v has no parent
				return;
			}
		}
	}
}


/// A variant of [TwoPassSplayStrategy] executing the two passes in an interleaved manner.
#[derive(Clone)]
pub struct LocalTwoPassSplayStrategy {}

impl LocalTwoPassSplayStrategy {
	fn move_to( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx,
			target : SplayTarget )
	{
		loop {
			if let Some( p ) = f.get_parent( v ) {
				if let Some( g ) = f.get_parent( p ) {
					// v has a grandparent
					if target == Root || f.get_parent( g ).is_some() {
						if can_splay_step( f, v, p, g ) {
							splay_step( f, v );
						}
						else {
							debug_assert!( f.is_separator( g ) ); // Otherwise we could splay v
							// Must move a branching node
							if f.is_separator( p ) {
								// p is a branching node
								// Since g is a separator, we can definitely splay p
								splay_step( f, p );
							}
							else {
								// g is a branching node, but we may not be able to splay
								TwoPassSplayStrategy::branching_step( f, g, target );
							}
						}
					}
					else {
						f.rotate( v );
						return
					}
				}
				else {
					// v has a parent, but no grandparent
					if target == Root {
						f.rotate( v );
					}
					return
				}
			}
			else {
				// v has no parent
				return;
			}
		}
	}
}

impl ExtendedNTRStrategy for LocalTwoPassSplayStrategy {
	fn node_to_root(f: &mut (impl STTRotate + STTStructureRead), v: NodeIdx ) {
		Self::move_to( f, v, Root );
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root(f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		Self::move_to( f, v, BelowRoot );
		debug_assert!( f.get_parent( v ).is_some() && f.get_parent( f.get_parent( v ).unwrap() ).is_none() );
	}
}


/// A simplified variant of [LocalTwoPassSplayStrategy].
#[derive(Clone)]
pub struct StableLocalTwoPassSplayStrategy {}

impl StableLocalTwoPassSplayStrategy {
	// Do a splay_step or rotation on v
	fn branching_step( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx )
	{
		debug_assert!( f.is_separator( v ) );
		// v is a separator, and thus has depth at least 3
		let p = f.get_parent( v ).unwrap();
		let g = f.get_parent( p ).unwrap();

		if !f.is_separator( p ) && f.is_separator( g ) {
			// g is branching node
			f.rotate( v );
			return
		}
		else {
			splay_step( f, v );
			return
		}
	}
}

impl StableNTRStrategy for StableLocalTwoPassSplayStrategy {
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		loop {
			if let Some( p ) = f.get_parent( v ) {
				if let Some( g ) = f.get_parent( p ) {
					// v has a grandparent
					if can_splay_step( f, v, p, g ) {
						splay_step( f, v );
					}
					else {
						debug_assert!( f.is_separator( g ) ); // Otherwise we could splay v
						// Must move a branching node
						if f.is_separator( p ) {
							// p is a branching node
							// Since g is a separator, we can definitely splay p
							splay_step( f, p );
						}
						else {
							// g is a branching node, but we may not be able to splay
							Self::branching_step( f, g );
						}
					}
				}
				else {
					// v has a parent, but no grandparent
					f.rotate( v );
					return
				}
			}
			else {
				// v has no parent
				return;
			}
		}
	}
}
