//! Dynamic tree implementation based on a simple move-to-root strategy in 2-cut STTs.


use crate::common::EmptyNodeData;
use crate::twocut::{NodesToTopPWImpl, StableNodesToTopPWImpl, StableNTRImplementation, StableNTRStrategy, StandardDynamicForest, ExtendedNTRImplementation, ExtendedNTRStrategy};
use crate::NodeIdx;
use crate::twocut::basic::{STTRotate, STTStructureRead};
use crate::twocut::node_data::{GroupPathWeightNodeData, MonoidPathWeightNodeData};
use crate::twocut::rooted::StandardRootedDynamicForest;

/// A dynamic forest using a simple move-to-root strategy.
pub type MoveToRootTT<TNodeData>
	= StandardDynamicForest<TNodeData,
		ExtendedNTRImplementation<MoveToRootStrategy>,
		NodesToTopPWImpl<MoveToRootStrategy>>;

/// A dynamic forest using a simple move-to-root strategy without edge weights.
pub type EmptyMoveToRootTT = MoveToRootTT<EmptyNodeData>;

/// A dynamic forest using a simple move-to-root strategy with monoid edge weights.
pub type MonoidMoveToRootTT<TWeight> = MoveToRootTT<MonoidPathWeightNodeData<TWeight>>;

/// A dynamic forest using a simple move-to-root strategy with group edge weights.
pub type GroupMoveToRootTT<TWeight> = MoveToRootTT<GroupPathWeightNodeData<TWeight>>;


/// A dynamic forest using a simple move-to-root strategy.
pub type StableMoveToRootTT<TNodeData>
	= StandardDynamicForest<TNodeData,
		StableNTRImplementation<MoveToRootStrategy>,
		StableNodesToTopPWImpl<MoveToRootStrategy>>;

/// A dynamic forest using a simple move-to-root strategy without edge weights.
pub type EmptyStableMoveToRootTT = StableMoveToRootTT<EmptyNodeData>;

/// A dynamic forest using a simple move-to-root strategy with monoid edge weights.
pub type MonoidStableMoveToRootTT<TWeight> = StableMoveToRootTT<MonoidPathWeightNodeData<TWeight>>;

/// A dynamic forest using a simple move-to-root strategy with group edge weights.
pub type GroupStableMoveToRootTT<TWeight> = StableMoveToRootTT<GroupPathWeightNodeData<TWeight>>;

/// A rooted dynamic forest using a simple move-to-root strategy.
pub type RootedMoveToRootTT = StandardRootedDynamicForest<MoveToRootStrategy>;


/// A dynamic tree implementation using the simple move-to-root strategy
#[derive(Clone)]
pub struct MoveToRootStrategy {}

impl MoveToRootStrategy
{
	/// Rotates `v` with its parent `p`, if possible. Otherwise, first rotate `p` as often as
	/// necessary.
	fn move_step( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx, p : NodeIdx ) {
		if ! f.is_separator_hint( v, p ) {
			// Rotate at p as long as p is a separator
			loop {
				if let Some( g ) = f.get_parent( p ) {
					if f.is_separator_hint( p, g ) {
						f.rotate( p );
						continue;
					}
				}
				break;
			}
		}
		// Now either v is a separator, or p is not, meaning we are allowed to rotate at v.
		f.rotate( v );
	}
}

impl ExtendedNTRStrategy for MoveToRootStrategy {
	fn node_to_root(f: &mut (impl STTRotate + STTStructureRead), v: NodeIdx ) {
		while let Some( p ) = f.get_parent( v ) {
			Self::move_step( f, v, p );
		}
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root(f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		loop {
			let p = f.get_parent( v ).unwrap();
			if f.get_parent( p ).is_some() {
				Self::move_step( f, v, p ); // Will only rotate at v if grandparent is the root
			}
			else {
				break
			}
		}
		debug_assert!( f.get_parent( v ).is_some() && f.get_parent( f.get_parent( v ).unwrap() ).is_none() );
	}
}

impl StableNTRStrategy for MoveToRootStrategy {
	fn node_to_root( f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as ExtendedNTRStrategy>::node_to_root( f, v );
	}
}
