use stt::connectivity::FullyDynamicConnectivity;
use stt::link_cut::EmptyLinkCutTree;
use stt::{DynamicForest, NodeIdx};
use stt::common::EmptyGroupWeight;
use stt::pg::EmptyPetgraphDynamicForest;
use stt::twocut::mtrtt::EmptyMoveToRootTT;
use stt::twocut::splaytt::{EmptyGreedySplayTT, EmptyLocalTwoPassSplayTT, EmptyStableGreedySplayTT, EmptyStableLocalTwoPassSplayTT, EmptyStableTwoPassSplayTT, EmptyTwoPassSplayTT};


#[test]
fn test() {
	test_for::<EmptyPetgraphDynamicForest>();
	test_for::<EmptyLinkCutTree>();
	test_for::<EmptyGreedySplayTT>();
	test_for::<EmptyStableGreedySplayTT>();
	test_for::<EmptyTwoPassSplayTT>();
	test_for::<EmptyLocalTwoPassSplayTT>();
	test_for::<EmptyStableTwoPassSplayTT>();
	test_for::<EmptyStableLocalTwoPassSplayTT>();
	test_for::<EmptyMoveToRootTT>();
}

fn test_for<TDynForest : DynamicForest<TWeight=EmptyGroupWeight>>()
{
	let mut c : FullyDynamicConnectivity<TDynForest> = FullyDynamicConnectivity::new( 5 );
	c.insert_edge( NodeIdx::new( 0 ), NodeIdx::new( 1 ) );
	c.insert_edge( NodeIdx::new( 2 ), NodeIdx::new( 4 ) );
	c.insert_edge( NodeIdx::new( 3 ), NodeIdx::new( 1 ) );
	c.insert_edge( NodeIdx::new( 3 ), NodeIdx::new( 2 ) );
	assert!( c.check_connected( NodeIdx::new( 0 ), NodeIdx::new( 4 ) ) );
	c.delete_edge( NodeIdx::new( 3 ), NodeIdx::new(2 ) );
	assert!( !c.check_connected( NodeIdx::new( 0 ), NodeIdx::new( 4 ) ) );
	c.delete_edge( NodeIdx::new( 3 ), NodeIdx::new( 1 ) );
	c.insert_edge( NodeIdx::new( 2 ), NodeIdx::new( 1 ) );
	assert!( c.check_connected( NodeIdx::new( 0 ), NodeIdx::new( 4 ) ) );
}
