///! Utilities for benchmarking

use std::time::{Duration, Instant};

use rand::Rng;
use stt::{DynamicForest, MonoidWeight, NodeData, NodeIdx, PathWeightNodeData, RootedForest};
use stt::common::{EmptyGroupWeight, MonoidWeightWithMaxEdge};
use stt::generate::{GeneratableMonoidWeight, generate_edge, generate_edge_with_dist};
use stt::twocut::basic::{MakeOneCutSTT, STT, STTRotate, STTStructureRead};

use clap::clap_derive::ValueEnum;
use rand::distributions::Distribution;
use stt::link_cut::{LCTNodeData, LinkCutForest};
use stt::onecut::SimpleDynamicTree;
use stt::pg::PetgraphDynamicForest;
use stt::rooted::SimpleRootedForest;
use stt::twocut::mtrtt::{MoveToRootTT, RootedMoveToRootTT, StableMoveToRootTT};
use stt::twocut::splaytt::{GreedySplayTT, LocalTwoPassSplayTT, MonoidTwoPassSplayTT, RootedGreedySplayTT, RootedLocalTwoPassSplayTT, RootedTwoPassSplayTT, StableGreedySplayTT, StableLocalTwoPassSplayTT, StableTwoPassSplayTT, TwoPassSplayTT};
use stt::twocut::UpdatingNodeData;
use Query::*;

/// A query to a dynamic forest
pub enum Query<TWeight : MonoidWeight> {
	InsertEdge( NodeIdx, NodeIdx, TWeight ),
	DeleteEdge( NodeIdx, NodeIdx ),
	PathWeight( NodeIdx, NodeIdx )
}

impl<TWeight : MonoidWeight> Query<TWeight> {
	pub fn execute(&self, f : &mut impl DynamicForest<TWeight=TWeight> )
	{
		match *self {
			InsertEdge( u, v, weight ) => f.link( u, v, weight ),
			DeleteEdge( u, v ) => f.cut( u, v ),
			PathWeight( u, v ) => assert!( f.compute_path_weight( u, v ).is_some() )
		}
	}
}

/// Transforms a list of node pairs into queries
/// 
/// Uses a MonoidTwoPassSplayTT to determine valid queries.
pub fn transform_into_queries<TWeight : MonoidWeight, TRng : Rng>(
	num_nodes : usize,
	node_pairs : impl Iterator<Item=(NodeIdx, NodeIdx)>,
	rng : &mut TRng,
	weight_gen : impl Fn( &mut TRng ) -> TWeight,
	weight_query_prob : f64 )
	-> Vec<Query<TWeight>>
{
	let mut f = MonoidTwoPassSplayTT::<MonoidWeightWithMaxEdge<EmptyGroupWeight>>::new( num_nodes );
	
	node_pairs.map( |(u, v)| {
		if let Some( w ) = f.compute_path_weight(u, v) {
			let (x, y) = w.unwrap_edge();
			if rng.gen_bool( weight_query_prob ) {
				PathWeight(u, v)
			}
			else {
				f.cut( x, y );
				DeleteEdge( x, y )
			}
		}
		else {
			f.link(u, v, MonoidWeightWithMaxEdge::new( EmptyGroupWeight{}, (u, v) ) );
			InsertEdge(u, v, weight_gen( rng ) )
		}
	} ).collect()
}


/// Generates a sequence of queries that can be handled when starting with an empty tree.
pub fn generate_queries<TWeight : MonoidWeight, TRng : Rng>( num_vertices : usize, num_queries : usize,
	rng : &mut TRng, weight_gen : impl Fn( &mut TRng ) -> TWeight, weight_query_prob : f64 ) -> Vec<Query<TWeight>>
{
	let node_pairs : Vec<_> = (0..num_queries)
		.map( |_| generate_edge( num_vertices, rng ) )
		.map( |(u, v)| ( NodeIdx::new( u ), NodeIdx::new( v ) ) )
		.collect();
	transform_into_queries( num_vertices, node_pairs.into_iter(), rng,
			weight_gen, weight_query_prob )
}


/// Generate queries using [GeneratableMonoidWeight::generate]
pub fn generate_queries_default<TWeight : GeneratableMonoidWeight>( num_vertices : usize,
		num_queries : usize, rng : &mut impl Rng, weight_query_prob : f64 ) -> Vec<Query<TWeight>>
{
	generate_queries( num_vertices, num_queries, rng, TWeight::generate, weight_query_prob )
}


/// Generates a sequence of queries that can be handled when starting with an empty tree.
pub fn generate_queries_with_node_dist<TWeight : MonoidWeight, TRng : Rng>(
	num_nodes: usize,
	num_queries : usize,
	rng : &mut TRng,
	weight_gen : impl Fn( &mut TRng ) -> TWeight,
	weight_query_prob : f64,
	node_dist : &impl Distribution<usize> ) -> Vec<Query<TWeight>>
{
	let node_pairs : Vec<_> = (0..num_queries)
		.map( |_| generate_edge_with_dist( node_dist, rng ) )
		.map( |(u, v)| ( NodeIdx::new( u ), NodeIdx::new( v ) ) )
		.collect();
	transform_into_queries( num_nodes, node_pairs.into_iter(), rng, weight_gen,
			weight_query_prob )
}


/// Execute the given queries on the given dynamic forest and measure the elapsed time.
pub fn benchmark_queries_on<TDynForest>( f : &mut TDynForest, queries : &Vec<Query<TDynForest::TWeight>> ) -> Duration
	where TDynForest : DynamicForest
{
	let start = Instant::now();
	for q in queries {
		q.execute( f );
	}
	start.elapsed()
}


/// Create a new dynamic forest of the given type, then execute the given queries on the given
/// dynamic forest and measure the total elapsed time (including creation of the dynamic forest).
pub fn benchmark_queries<TDynForest>( num_vertices : usize, queries : &Vec<Query<TDynForest::TWeight>> ) -> Duration
	where TDynForest : DynamicForest
{
	let start = Instant::now();
	let mut f = TDynForest::new( num_vertices );
	for q in queries {
		q.execute( &mut f );
	}
	start.elapsed()
}


/// An STT wrapper that counts rotations
pub struct RotationCountSTT<TData : NodeData> {
	t : STT<TData>,
	pub count : usize
}

impl<TData : NodeData> RotationCountSTT<TData> {
	pub fn from( t : STT<TData> ) -> RotationCountSTT<TData> {
		RotationCountSTT{ t, count : 0 }
	}
}

impl<TData: NodeData> RootedForest for RotationCountSTT<TData> {
	fn get_parent(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_parent( v )
	}
}

impl<TData: NodeData> STTStructureRead for RotationCountSTT<TData> {
	fn get_direct_separator_child(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_direct_separator_child( v )
	}

	fn get_indirect_separator_child(&self, v: NodeIdx) -> Option<NodeIdx> {
		self.t.get_indirect_separator_child( v )
	}
}

impl<TData : NodeData> STTRotate for RotationCountSTT<TData> {
	fn rotate( &mut self, v : NodeIdx ) {
		self.count += 1;
		self.t.rotate( v );
	}
}

impl<TData : NodeData> MakeOneCutSTT for RotationCountSTT<TData> {
	type NodeIdxIterator = <STT<TData> as MakeOneCutSTT>::NodeIdxIterator;
	
	fn nodes(&self) -> Self::NodeIdxIterator {
		self.t.nodes()
	}
}

/// How to print benchmark results
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PrintType {
	Silent,
	Print,
	Json
}

impl PrintType {
	pub fn from_args( print : bool, json : bool ) -> Self {
		if print {
			if json {
				eprintln!( "Cannot both print and print json" )
			}
			Self::Print
		}
		else if json {
			Self::Json
		}
		else {
			Self::Silent
		}
	}
}


/// Enum listing possible dynamic tree implementations, usable by CLAP.
#[derive( Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum )]
pub enum ImplDesc {
	PetgraphDynamic,
	LinkCut,
	GreedySplay,
	StableGreedySplay,
	TwoPassSplay,
	StableTwoPassSplay,
	LocalTwoPassSplay,
	LocalStableTwoPassSplay,
	MoveToRoot,
	StableMoveToRoot,
	OneCut
}

impl ImplDesc {
	pub fn all() -> Vec<ImplDesc> {
		vec![ImplDesc::PetgraphDynamic, ImplDesc::LinkCut, ImplDesc::GreedySplay,
			ImplDesc::StableGreedySplay, ImplDesc::TwoPassSplay, ImplDesc::StableTwoPassSplay,
			ImplDesc::LocalTwoPassSplay, ImplDesc::LocalStableTwoPassSplay, ImplDesc::MoveToRoot,
			ImplDesc::StableMoveToRoot, ImplDesc::OneCut]
	}
}


/// Enum listing possible rooted dynamic tree implementations, usable by CLAP.
#[derive( Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum )]
pub enum RootedImplDesc {
	LinkCut,
	GreedySplay,
	TwoPassSplay,
	LocalTwoPassSplay,
	MoveToRoot,
	Simple
}

impl RootedImplDesc {
	pub fn all() -> Vec<RootedImplDesc> {
		vec![RootedImplDesc::LinkCut, RootedImplDesc::GreedySplay, RootedImplDesc::TwoPassSplay,
			RootedImplDesc::LocalTwoPassSplay, RootedImplDesc::MoveToRoot, RootedImplDesc::Simple]
	}
}


pub trait ImplName {
	fn name() -> &'static str;
}

impl<TWeight : MonoidWeight> ImplName for PetgraphDynamicForest<TWeight> {
	fn name() -> &'static str {
		"Petgraph"
	}
}

impl<TNodeData : LCTNodeData<IMPL_EVERT>, const IMPL_EVERT : bool> ImplName for LinkCutForest<TNodeData, IMPL_EVERT> {
	fn name() -> &'static str {
		"Link-cut"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for GreedySplayTT<TNodeData> {
	fn name() -> &'static str {
		"Greedy Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for StableGreedySplayTT<TNodeData> {
	fn name() -> &'static str {
		"Stable Greedy Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for TwoPassSplayTT<TNodeData> {
	fn name() -> &'static str {
		"2P Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for StableTwoPassSplayTT<TNodeData> {
	fn name() -> &'static str {
		"Stable 2P Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for LocalTwoPassSplayTT<TNodeData> {
	fn name() -> &'static str {
		"L2P Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for StableLocalTwoPassSplayTT<TNodeData> {
	fn name() -> &'static str {
		"Stable L2P Splay"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for MoveToRootTT<TNodeData> {
	fn name() -> &'static str {
		"MTR"
	}
}

impl<TNodeData : PathWeightNodeData + UpdatingNodeData> ImplName for StableMoveToRootTT<TNodeData> {
	fn name() -> &'static str {
		"Stable MTR"
	}
}

impl<TWeight : MonoidWeight> ImplName for SimpleDynamicTree<TWeight> {
	fn name() -> &'static str {
		"1-cut"
	}
}

impl ImplName for RootedGreedySplayTT {
	fn name() -> &'static str {
		"Greedy Splay"
	}
}

impl ImplName for RootedTwoPassSplayTT {
	fn name() -> &'static str {
		"2P Splay"
	}
}

impl ImplName for RootedLocalTwoPassSplayTT {
	fn name() -> &'static str {
		"L2P Splay"
	}
}

impl ImplName for RootedMoveToRootTT {
	fn name() -> &'static str {
		"MTR"
	}
}

impl ImplName for SimpleRootedForest {
	fn name() -> &'static str {
		"Simple"
	}
}


/// Call `$do_mac!( $obj, <type>, <name> )`, where `<name>` is the name of the implementation
/// `$imp_enum`, and `<type>` is the monoid dynamic tree type associated to `$imp_enum`. `<type>` is
/// parametrized with a monoid weight type.
#[macro_export]
macro_rules! do_for_impl_monoid {
	( $imp_enum : ident, $do_mac : ident, $obj : ident ) => {
		{
			match $imp_enum {
				stt_benchmarks::bench_util::ImplDesc::PetgraphDynamic => $do_mac!( $obj, PetgraphDynamicForest ),
				stt_benchmarks::bench_util::ImplDesc::LinkCut => $do_mac!( $obj, MonoidLinkCutTree ),
				stt_benchmarks::bench_util::ImplDesc::GreedySplay => $do_mac!( $obj, MonoidGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableGreedySplay => $do_mac!( $obj, MonoidStableGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::TwoPassSplay => $do_mac!( $obj, MonoidTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableTwoPassSplay => $do_mac!( $obj, MonoidStableTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalTwoPassSplay => $do_mac!( $obj, MonoidLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalStableTwoPassSplay => $do_mac!( $obj, MonoidStableLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::MoveToRoot => $do_mac!( $obj, MonoidMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::StableMoveToRoot => $do_mac!( $obj, MonoidStableMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::OneCut => $do_mac!( $obj, SimpleDynamicTree )
			}
		}
	}
}

/// Call `$do_mac!( $obj, <type>, <name> )`, where `<name>` is the name of the implementation
/// `$imp_enum`, and `<type>` is the group dynamic tree type associated to `$imp_enum`. `<type>` is
/// parametrized with a group weight type.
#[macro_export]
macro_rules! do_for_impl_group {
	( $imp_enum : ident, $do_mac : ident, $obj : ident ) => {
		{
			match $imp_enum {
				stt_benchmarks::bench_util::ImplDesc::PetgraphDynamic => $do_mac!( $obj, PetgraphDynamicForest ),
				stt_benchmarks::bench_util::ImplDesc::LinkCut => $do_mac!( $obj, GroupLinkCutTree ),
				stt_benchmarks::bench_util::ImplDesc::GreedySplay => $do_mac!( $obj, GroupGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableGreedySplay => $do_mac!( $obj, GroupStableGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::TwoPassSplay => $do_mac!( $obj, GroupTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableTwoPassSplay => $do_mac!( $obj, GroupStableTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalTwoPassSplay => $do_mac!( $obj, GroupLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalStableTwoPassSplay => $do_mac!( $obj, GroupStableLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::MoveToRoot => $do_mac!( $obj, GroupMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::StableMoveToRoot => $do_mac!( $obj, GroupStableMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::OneCut => $do_mac!( $obj, SimpleDynamicTree )
			}
		}
	}
}

/// Call `$do_mac!( $obj, <type>, <name> )`, where `<name>` is the name of the implementation
/// `$imp_enum`, and `<type>` is the empty dynamic tree type associated to `$imp_enum`.
#[macro_export]
macro_rules! do_for_impl_empty {
	( $imp_enum : ident, $do_mac : ident, $obj : ident ) => {
		{
			match $imp_enum {
				stt_benchmarks::bench_util::ImplDesc::PetgraphDynamic => $do_mac!( $obj, EmptyPetgraphDynamicForest ),
				stt_benchmarks::bench_util::ImplDesc::LinkCut => $do_mac!( $obj, EmptyLinkCutTree ),
				stt_benchmarks::bench_util::ImplDesc::GreedySplay => $do_mac!( $obj, EmptyGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableGreedySplay => $do_mac!( $obj, EmptyStableGreedySplayTT ),
				stt_benchmarks::bench_util::ImplDesc::TwoPassSplay => $do_mac!( $obj, EmptyTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::StableTwoPassSplay => $do_mac!( $obj, EmptyStableTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalTwoPassSplay => $do_mac!( $obj, EmptyLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::LocalStableTwoPassSplay => $do_mac!( $obj, EmptyStableLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::ImplDesc::MoveToRoot => $do_mac!( $obj, EmptyMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::StableMoveToRoot => $do_mac!( $obj, EmptyStableMoveToRootTT ),
				stt_benchmarks::bench_util::ImplDesc::OneCut => $do_mac!( $obj, EmptySimpleDynamicTree )
			}
		}
	}
}


/// Call `$do_mac!( $obj, <type>, <name> )`, where `<name>` is the name of the implementation
/// `$imp_enum`, and `<type>` is the empty dynamic tree type associated to `$imp_enum`.
#[macro_export]
macro_rules! do_for_impl_rooted {
	( $imp_enum : ident, $do_mac : ident, $obj : ident ) => {
		{
			match $imp_enum {
				stt_benchmarks::bench_util::RootedImplDesc::LinkCut => $do_mac!( $obj, RootedLinkCutTree ),
				stt_benchmarks::bench_util::RootedImplDesc::GreedySplay => $do_mac!( $obj, RootedGreedySplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::TwoPassSplay => $do_mac!( $obj, RootedTwoPassSplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::LocalTwoPassSplay => $do_mac!( $obj, RootedLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::MoveToRoot => $do_mac!( $obj, RootedMoveToRootTT ),
				stt_benchmarks::bench_util::RootedImplDesc::Simple => $do_mac!( $obj, SimpleRootedForest )
			}
		}
	}
}

/// Call `$do_mac!( $obj, <type>, <name> )`, where `<name>` is the name of the implementation
/// `$imp_enum`, and `<type>` is the empty dynamic tree type associated to `$imp_enum`.
#[macro_export]
macro_rules! do_for_impl_eversible_rooted {
	( $imp_enum : ident, $do_mac : ident, $obj : ident ) => {
		{
			match $imp_enum {
				stt_benchmarks::bench_util::RootedImplDesc::LinkCut => $do_mac!( $obj, RootedLinkCutTreeWithEvert ),
				stt_benchmarks::bench_util::RootedImplDesc::GreedySplay => $do_mac!( $obj, RootedGreedySplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::TwoPassSplay => $do_mac!( $obj, RootedTwoPassSplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::LocalTwoPassSplay => $do_mac!( $obj, RootedLocalTwoPassSplayTT ),
				stt_benchmarks::bench_util::RootedImplDesc::MoveToRoot => $do_mac!( $obj, RootedMoveToRootTT ),
				stt_benchmarks::bench_util::RootedImplDesc::Simple => $do_mac!( $obj, SimpleRootedForest )
			}
		}
	}
}
