//! Dynamic tree implementation based on a simple move-to-root strategy in 2-cut STTs.


use crate::common::EmptyNodeData;
use crate::twocut::{NodesToTopPWImpl, StableNodesToTopPWImpl, StableNTRImplementation, StableNTRStrategy, StandardDynamicForest, ExtendedNTRImplementation, ExtendedNTRStrategy};
use crate::NodeIdx;
use crate::twocut::basic::{STTRotate, STTStructureRead};
use crate::twocut::node_data::{GroupPathWeightNodeData, MonoidPathWeightNodeData};

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


/// A dynamic tree implementation using the simple move-to-root strategy
#[derive(Clone)]
pub struct MoveToRootStrategy {}

impl MoveToRootStrategy
{
	fn move_step( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		if ! f.is_separator( v ) {
			while f.is_separator( f.get_parent( v  ).unwrap() ) {
				f.rotate( f.get_parent( v ).unwrap() );
			}
		}
		f.rotate( v );
	}
}

impl ExtendedNTRStrategy for MoveToRootStrategy {
	fn node_to_root(f: &mut (impl STTRotate + STTStructureRead), v: NodeIdx ) {
		while f.get_parent( v ).is_some() {
			Self::move_step( f, v );
		}
		debug_assert!( f.get_parent( v ).is_none() );
	}

	fn node_below_root(f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		debug_assert!( f.get_parent( v ).is_some() );
		while f.get_parent( f.get_parent( v ).unwrap() ).is_some() {
			Self::move_step( f, v ); // Will never rotate with grandparent if grandparent does not exist.
		}
		debug_assert!( f.get_parent( v ).is_some() && f.get_parent( f.get_parent( v ).unwrap() ).is_none() );
	}
}

impl StableNTRStrategy for MoveToRootStrategy {
	fn node_to_root( f: &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		<Self as ExtendedNTRStrategy>::node_to_root( f, v );
	}
}