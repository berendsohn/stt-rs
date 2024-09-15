//! Dynamic tree implementations based on 2-cut STTs.

use std::marker::PhantomData;

use crate::{RootedForest, DynamicForest, MonoidWeight, NodeData, NodeDataAccess, NodeIdx, PathWeightNodeData};
use crate::common::{EmptyGroupWeight, EmptyNodeData};
use crate::twocut::basic::{MakeOneCutSTT, STT, STTRotate, STTStructureRead};

pub mod basic;
pub mod node_data;
pub mod splaytt;
pub mod mtrtt;
pub mod rooted;


/// Node data that includes associated methods to update itself when the tree is modified.
pub trait UpdatingNodeData : NodeData {
	/// Called before rotating v.
	fn before_rotation( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx );

	/// Called after adding an edge from v to another node (now its parent), with the given edge weight.
	fn after_attached( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx, weight : Self::TWeight );

	/// Called before removing the edge between v and its parent.
	fn before_detached( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx );
}


/// Trivial implementation for EmptyNodeData.
impl UpdatingNodeData for EmptyNodeData {
	fn before_rotation( _ : &mut (impl NodeDataAccess<Self> + STTStructureRead), _ : NodeIdx ) {}

	fn after_attached( _ : &mut (impl NodeDataAccess<Self> + STTStructureRead), _ : NodeIdx, _ : EmptyGroupWeight) {}

	fn before_detached( _ : &mut (impl NodeDataAccess<Self> + STTStructureRead), _ : NodeIdx ) {}
}


/// An implementation that allows rotating nodes and edges to the root in a 2-cut STT.
pub trait NTRImplementation : Clone 
{
	/// Rotates v to the root. Only affects nodes on the search path of v.
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx );

	/// Transforms the tree such that u is the root and v is a child of u.
	/// Assumes that the underlying tree has an edge between u and v.
	/// Only affects nodes on the search path of v.
	fn edge_to_top( f : &mut (impl STTRotate + STTStructureRead), u : NodeIdx, v : NodeIdx );
}


/// An implementation of `compute_path_weight` for 2-cut STTs.
pub trait CPWImplementation<TNodeData> : Clone
	where TNodeData : PathWeightNodeData
{
	/// Computes the weight of the path between u and v, if it exists, or returns None otherwise.
	fn compute_path_weight( f: &mut (impl NodeDataAccess<TNodeData> + STTRotate + STTStructureRead), u : NodeIdx, v : NodeIdx ) -> Option<TNodeData::TWeight>;
}


/** A standard implementation of DynamicForest using a 2-cut STT.

The types `TNTRImpl` and `TCPWImpl` define behavior. The struct itself implements some traits by
delegating to a contained [STT], and handles the callback functions of [UpdatingNodeData].
*/
#[derive(Clone)]
pub struct StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	t : STT<TNodeData>,
	_m1 : PhantomData<TNTRImpl>,
	_m2 : PhantomData<TCPWImpl>
}

impl<TNodeData, TNTRImpl, TCPWImpl> StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	/// Returns a human-readable string representation of this dynamic tree.
	pub fn to_string( &self ) -> String {
		self.t.to_string()
	}
}

impl<TNodeData, TNTRImpl, TCPWImpl> RootedForest for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	fn get_parent(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_parent( v )
	}
}

impl<TNodeData, TNTRImpl, TCPWImpl> STTStructureRead for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	fn get_direct_separator_child(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_direct_separator_child( v )
	}

	fn get_indirect_separator_child(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_indirect_separator_child( v )
	}
}

impl<TNodeData, TNTRImpl, TCPWImpl> MakeOneCutSTT for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	type NodeIdxIterator = <Self as DynamicForest>::NodeIdxIterator;
	
	fn nodes( &self ) -> Self::NodeIdxIterator {
		<Self as DynamicForest>::nodes( self )
	}
}


impl<TNodeData, TNTRImpl, TCPWImpl> STTRotate for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	fn rotate( &mut self, v : NodeIdx ) {
		TNodeData::before_rotation( &mut self.t, v );
		self.t.rotate( v );
	}
}

impl<TNodeData, TNTRImpl, TCPWImpl> NodeDataAccess<TNodeData> for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	fn data( &self, idx: NodeIdx ) -> &TNodeData {
		self.t.data( idx )
	}

	fn data_mut( &mut self, idx: NodeIdx ) -> &mut TNodeData {
		self.t.data_mut( idx )
	}
}

impl<TNodeData, TNTRImpl, TCPWImpl> DynamicForest for StandardDynamicForest<TNodeData, TNTRImpl, TCPWImpl>
	where TNodeData : PathWeightNodeData + UpdatingNodeData,
		TNTRImpl : NTRImplementation, TCPWImpl : CPWImplementation<TNodeData>
{
	type TWeight = TNodeData::TWeight;
	
	type NodeIdxIterator = <STT<TNodeData> as MakeOneCutSTT>::NodeIdxIterator;
	
	fn new( num_nodes : usize ) -> Self {
		StandardDynamicForest{ t : STT::new( num_nodes ), _m1 : PhantomData::default(),
			_m2 : PhantomData::default() }
	}
	
	fn link( &mut self, u : NodeIdx, v : NodeIdx, weight : TNodeData::TWeight ) {
		// println!( "Linking #{u} to #{v}" );
		TNTRImpl::node_to_root( self, u );
		TNTRImpl::node_to_root( self, v );
		assert!( self.t.get_parent( u ).is_none(), "It seems you're trying to link two nodes {u}, {v} in the same tree." );
		self.t.attach( u, v );
		TNodeData::after_attached( &mut self.t, u, weight );
	}
	
	fn cut( &mut self, u: NodeIdx, v: NodeIdx ) {
		// println!( "Cutting #{u} – #{v}" );
		TNTRImpl::edge_to_top( self, v, u );
		assert!( self.t.get_direct_separator_child( u ).is_none(), "It seems you're trying to cut a non-existing edge ({u}, {v})." );
		assert_eq!( self.t.get_parent( u ), Some( v ), "It seems you're trying to cut a non-existing edge ({u}, {v}). The two nodes are not even in the same tree." );
		TNodeData::before_detached( &mut self.t, u );
		self.t.detach( u );
	}
	
	fn compute_path_weight( &mut self, u : NodeIdx, v : NodeIdx ) -> Option<TNodeData::TWeight> {
		TCPWImpl::compute_path_weight( self, u, v )
	}

	fn get_edge_weight( &mut self, u : NodeIdx, v: NodeIdx) -> Option<Self::TWeight> {
		TNTRImpl::edge_to_top( self, v, u ); // Make u child of v
		if self.t.get_parent( u ) == Some( v )
				&& self.t.get_direct_separator_child( u ).is_none() {
			Some( self.t.data( u ).get_parent_path_weight() )
		}
		else {
			None
		}
	}

	fn nodes( &self ) -> Self::NodeIdxIterator {
		self.t.nodes()
	}

	fn edges( &self ) -> Vec<(NodeIdx, NodeIdx)> {
		let mut tmp = self.clone();
		basic::make_1_cut( &mut tmp );
		tmp.t.edges().collect()
	}
}


/// A strategy to move nodes to the root.
pub trait NTRStrategy : Clone {
	/// Move the given node to the root.
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx );
}


/// A strategy to move nodes to the root and below the root.
pub trait ExtendedNTRStrategy : NTRStrategy {
	/// Move the given node up such that it becomes the child of the current root. The root is not changed.
	fn node_below_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx );
}

/// A strategy to move nodes to the root which ensures the previous root does not move too much.
///
/// More precisely, calling `node_to_root(v)` moves `v` to the root. If the previous root was `u != v`,
/// then afterwards `u` and all ancestors of `u` are 1-cut.
///
/// Essentially, the stability requirement allows discarding the `node_below_root` function.
pub trait StableNTRStrategy : NTRStrategy {}


/// A [NTRImplementation] based on an [ExtendedNTRStrategy].
#[derive( Clone )]
pub struct ExtendedNTRImplementation<TStrategy>
	where TStrategy : ExtendedNTRStrategy
{
	_m : PhantomData<TStrategy>
}


impl<TStrategy> NTRImplementation for ExtendedNTRImplementation<TStrategy>
	where TStrategy : ExtendedNTRStrategy
{
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		TStrategy::node_to_root( f, v );
	}
	
	fn edge_to_top( f : &mut (impl STTRotate + STTStructureRead), u: NodeIdx, v: NodeIdx) {
		TStrategy::node_to_root( f, u );
		TStrategy::node_below_root( f, v );
	}
}


/// A [NTRImplementation] based on a [StableNTRStrategy].
#[derive( Clone )]
pub struct StableNTRImplementation<TStrategy>
	where TStrategy : StableNTRStrategy
{
	_m : PhantomData<TStrategy>
}


impl<TStrategy> NTRImplementation for StableNTRImplementation<TStrategy>
	where TStrategy : StableNTRStrategy
{
	fn node_to_root( f : &mut (impl STTRotate + STTStructureRead), v : NodeIdx ) {
		TStrategy::node_to_root( f, v );
	}
	
	fn edge_to_top( f : &mut (impl STTRotate + STTStructureRead), u: NodeIdx, v: NodeIdx ) {
		TStrategy::node_to_root( f, v );
		TStrategy::node_to_root( f, u );
	}
}


/// Compute path weight by moving two nodes to top first, then read parent path weight.
/// 
/// More specifically, first move `v` to the root, then move `u` below the `v`, then use
/// `[PathWeightNodeData::get_parent_path_weight()]` to read the result.
#[derive(Clone)]
pub struct NodesToTopPWImpl<TStrategy : ExtendedNTRStrategy> {
	_m : PhantomData<TStrategy>
}

impl<TNodeData, TStrategy> CPWImplementation<TNodeData> for NodesToTopPWImpl<TStrategy>
	where TNodeData : PathWeightNodeData, TStrategy : ExtendedNTRStrategy
{
	fn compute_path_weight( f: &mut (impl NodeDataAccess<TNodeData> + STTRotate + STTStructureRead),
			u : NodeIdx, v : NodeIdx ) -> Option<TNodeData::TWeight>
	{
		TStrategy::node_to_root( f, v );
		if f.get_parent( u ).is_none() {
			return None; // u is still root, and thus cannot be in same tree as v.
		}
		TStrategy::node_below_root( f, u );
		if f.get_parent( u ) == Some( v ) {
			Some( f.data( u ).get_parent_path_weight() )
		}
		else {
			None
		}
	}
}


/// Compute path weight by moving two nodes to top first, then read parent path weight.
/// 
/// Uses a stable NTR strategy.
/// 
/// More specifically, first move `v` to the root, then move `u` to the root, then add the weights
/// of edges from `v` to the new root `u`. [StableNTRStrategy] guarantees that the root path of `u`
/// is one cut, so this yields the correct result.
#[derive(Clone)]
pub struct StableNodesToTopPWImpl<TStrategy : StableNTRStrategy> {
	_m : PhantomData<TStrategy>
}

impl<TNodeData, TStrategy> CPWImplementation<TNodeData> for StableNodesToTopPWImpl<TStrategy>
	where TNodeData : PathWeightNodeData, TStrategy : StableNTRStrategy
{
	fn compute_path_weight( f : &mut (impl NodeDataAccess<TNodeData> + STTRotate + STTStructureRead),
			u : NodeIdx, v : NodeIdx ) -> Option<TNodeData::TWeight>
	{
		TStrategy::node_to_root( f, u );
		TStrategy::node_to_root( f, v );
		
		// Compute path from u to root. Use the fact that u has depth at most 3 and u and all its
		// ancestors are 1-cut
		let mut w = TNodeData::TWeight::identity();
		let mut x = u;
		while let Some( p ) = f.get_parent( x ) {
			debug_assert!( !f.is_separator( p ) );
			w = w + f.data( x ).get_parent_path_weight();
			x = p;
		}
		if x == v {
			Some( w )
		}
		else {
			None
		}
	}
}
