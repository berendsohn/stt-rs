use stt::link_cut::RootedLinkCutTree;
use stt::NodeIdx;
use stt::rooted::{RootedDynamicForest, SimpleRootedForest};
use stt::twocut::mtrtt::MoveToRootStrategy;
use stt::twocut::rooted::StandardRootedDynamicForest;
use stt::twocut::splaytt::{GreedySplayStrategy, LocalTwoPassSplayStrategy, TwoPassSplayStrategy};

// Helper functions
fn link( df : &mut impl RootedDynamicForest, u : usize, v : usize ) {
	df.link( NodeIdx::new( u ), NodeIdx::new( v ) );
}

fn cut( df : &mut impl RootedDynamicForest, u : usize ) {
	df.cut( NodeIdx::new( u ) );
}

fn find_root( df : &mut impl RootedDynamicForest, v : usize ) -> usize {
	df.find_root( NodeIdx::new( v  ) ).index()
}

fn lca( df : &mut impl RootedDynamicForest, u : usize, v : usize ) -> Option<usize> {
	df.lowest_common_ancestor( NodeIdx::new( u ), NodeIdx::new( v ) ).map( |x| x.index() )
}


#[test]
fn test() {
	test_for::<SimpleRootedForest>();
	test_for::<RootedLinkCutTree>();
	test_for::<StandardRootedDynamicForest<GreedySplayStrategy>>();
	test_for::<StandardRootedDynamicForest<TwoPassSplayStrategy>>();
	test_for::<StandardRootedDynamicForest<LocalTwoPassSplayStrategy>>();
	test_for::<StandardRootedDynamicForest<MoveToRootStrategy>>();
}

fn test_for<TRDynTree : RootedDynamicForest>() {
	
	let mut df = TRDynTree::new( 6 );
	
	link( &mut df, 0, 1 );
	link( &mut df, 1, 2 );
	link( &mut df, 2, 3 );
	link( &mut df, 4, 2 );
	
	for i in 0..5 {
		assert_eq!( find_root( &mut df, i ), 3 );
	}
	
	assert_eq!( lca( &mut df, 0, 4 ), Some( 2 ) );
	assert_eq!( lca( &mut df, 3, 5 ), None );
	
	cut( &mut df, 1 );
	
	for i in 0..1 {
		assert_eq!( find_root( &mut df, i ), 1 );
	}
	
	for i in 2..5 {
		assert_eq!( find_root( &mut df, i ), 3 );
	}
	
	assert_eq!( lca( &mut df, 2, 4 ), Some( 2 ) );
	assert_eq!( lca( &mut df, 1, 2 ), None );
}