use petgraph;
use petgraph::graph;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use stt::{DynamicForest, generate};
use stt::common::{EmptyGroupWeight, EmptyNodeData, IsizeAddGroupWeight, UsizeMaxMonoidWeight};
use stt::generate::GeneratableMonoidWeight;
use stt::link_cut::{EmptyLinkCutTree, GroupLinkCutTree, MonoidLinkCutTree};
use stt::pg::PetgraphDynamicForest;
use stt::twocut::mtrtt::MoveToRootTT;
use stt::twocut::node_data::{GroupPathWeightNodeData, MonoidPathWeightNodeData};
use stt::twocut::splaytt::{GreedySplayTT, LocalTwoPassSplayTT, StableGreedySplayTT, StableLocalTwoPassSplayTT, StableTwoPassSplayTT, TwoPassSplayTT};

use crate::util::DynamicTestForest;

#[test]
fn test() {
	test_queries_for::<PetgraphDynamicForest<EmptyGroupWeight>>();
	test_queries_for::<EmptyLinkCutTree>();
	test_queries_for::<GreedySplayTT<EmptyNodeData>>();
	test_queries_for::<StableGreedySplayTT<EmptyNodeData>>();
	test_queries_for::<TwoPassSplayTT<EmptyNodeData>>();
	test_queries_for::<LocalTwoPassSplayTT<EmptyNodeData>>();
	test_queries_for::<StableTwoPassSplayTT<EmptyNodeData>>();
	test_queries_for::<StableLocalTwoPassSplayTT<EmptyNodeData>>();
	test_queries_for::<MoveToRootTT<EmptyNodeData>>();
	
	test_queries_for::<PetgraphDynamicForest<IsizeAddGroupWeight>>();
	test_queries_for::<GroupLinkCutTree<IsizeAddGroupWeight>>();
	test_queries_for::<GreedySplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<StableGreedySplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<TwoPassSplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<LocalTwoPassSplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<StableTwoPassSplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<StableLocalTwoPassSplayTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	test_queries_for::<MoveToRootTT<GroupPathWeightNodeData<IsizeAddGroupWeight>>>();
	
	test_queries_for::<PetgraphDynamicForest<UsizeMaxMonoidWeight>>();
	test_queries_for::<MonoidLinkCutTree<IsizeAddGroupWeight>>();
	test_queries_for::<GreedySplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<StableGreedySplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<TwoPassSplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<LocalTwoPassSplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<StableTwoPassSplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<StableLocalTwoPassSplayTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
	test_queries_for::<MoveToRootTT<MonoidPathWeightNodeData<UsizeMaxMonoidWeight>>>();
}

fn test_queries_for<TDynForest : DynamicForest>()
	where TDynForest::TWeight : GeneratableMonoidWeight
{
	const NUM_NODES : usize = 50;
	const NUM_OPS : usize = 300;
	const VERBOSE : bool = false;
	const CHECK_EDGES : bool = true;

	let mut dtf : DynamicTestForest<TDynForest>
			= DynamicTestForest::new( NUM_NODES, VERBOSE );

	let mut rng = StdRng::seed_from_u64( 0 );
	for (u, v) in generate::generate_edges( NUM_NODES, NUM_OPS, &mut rng )
			.collect::<Vec<_>>().into_iter() {
		let (u_g, v_g) = ( dtf.g_node( u ), dtf.g_node( v ) );

		let g_edge = dtf.g.find_edge( u_g, v_g );
		
		if rng.gen_bool( 0.1 ) { // Don't always check this to avoid bias
			dtf.check_path_weight( u, v );
			if CHECK_EDGES { dtf.check_edges(); }
		}

		if g_edge.is_some() {
			if rng.gen_bool( 0.5 ) {
				dtf.remove_edge( u, v );
			}
			else {
				dtf.check_edge_weight( u, v );
			}
		}
		else if petgraph::algo::has_path_connecting( &dtf.g, u_g, v_g, None ) {
			// Find some path from u to v
			let path = petgraph::algo::all_simple_paths::<Vec<_>, _>(
				&dtf.g, u_g, v_g, 0, None ).next().unwrap();

			// Get the first edge on the path and remove it
			let x_g : graph::NodeIndex = path[1];
			if rng.gen_bool( 0.5 ) {
				dtf.remove_edge( u, x_g.index() );
			}
			else {
				dtf.check_edge_weight( u, x_g.index() );
			}
		}
		else {
			dtf.add_edge( u, v, TDynForest::TWeight::generate( &mut rng ) );
		}
		if CHECK_EDGES { dtf.check_edges(); }
	}
}
