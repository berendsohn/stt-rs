use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use stt::link_cut::RootedLinkCutTree;
use stt::{generate, NodeIdx};
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
	test_basic_for::<SimpleRootedForest>();
	test_basic_for::<RootedLinkCutTree>();
	test_basic_for::<StandardRootedDynamicForest<GreedySplayStrategy>>();
	test_basic_for::<StandardRootedDynamicForest<TwoPassSplayStrategy>>();
	test_basic_for::<StandardRootedDynamicForest<LocalTwoPassSplayStrategy>>();
	test_basic_for::<StandardRootedDynamicForest<MoveToRootStrategy>>();

	test_against_simple::<RootedLinkCutTree>();
	test_against_simple::<StandardRootedDynamicForest<GreedySplayStrategy>>();
	test_against_simple::<StandardRootedDynamicForest<TwoPassSplayStrategy>>();
	test_against_simple::<StandardRootedDynamicForest<LocalTwoPassSplayStrategy>>();
	test_against_simple::<StandardRootedDynamicForest<MoveToRootStrategy>>();
}

fn test_basic_for<TRDynTree : RootedDynamicForest>() {
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


struct TestRootedDynamicForest<TRDynTree : RootedDynamicForest + Clone> {
	df : TRDynTree,
	df_ref : SimpleRootedForest
}

impl<TRDynTree : RootedDynamicForest + Clone> TestRootedDynamicForest<TRDynTree> {
	fn new( num_vertices : usize ) -> Self {
		Self{ df : TRDynTree::new( num_vertices ), df_ref : SimpleRootedForest::new( num_vertices ) }
	}

	fn test_find_root( &mut self, v : NodeIdx ) {
		assert_eq!( self.df.find_root( v ), self.df_ref.find_root( v ) );
	}

	fn test_cut( &mut self, v : NodeIdx ) {
		debug_assert!( v != self.df_ref.find_root( v ) );
		self.df.cut( v );
		self.df_ref.cut( v );

		assert_eq!( v, self.df.find_root( v ) );
	}

	fn test_link_or_lca( &mut self, u : NodeIdx, v : NodeIdx ) {
		// Check if in same tree (only in ref forest)
		if self.df_ref.find_root( u ) == self.df_ref.find_root( v ) {
			// LCA
			assert_eq!( self.df.lowest_common_ancestor( u, v ),
				self.df_ref.lowest_common_ancestor( u, v ) );
		}
		else {
			// Link
			let u_root = self.df_ref.find_root( u );
			self.df.link( u_root, v );
			self.df_ref.link( u_root, v );

			// Check
			let mut df_copy = self.df.clone();
			assert_eq!( df_copy.find_root( v ), self.df_ref.find_root( v ) );
		}
	}
}

fn test_against_simple<TRDynTree : RootedDynamicForest + Clone>() {
	let mut rng = StdRng::seed_from_u64( 0 );
	let num_vertices = 100;
	let num_queries = 1000;

	let mut tdf : TestRootedDynamicForest<TRDynTree> = TestRootedDynamicForest::new( num_vertices );

	for _ in 0..num_queries {
		if rng.gen_bool( 0.33 ) {
			// Just find root
			tdf.test_find_root( NodeIdx::new( rng.gen_range( 0..num_vertices ) ) );
		}
		else if rng.gen_bool( 0.5 ) {
			// Try cutting
			let v = NodeIdx::new( rng.gen_range( 0..num_vertices ) );
			if v != tdf.df_ref.find_root( v ) {
				tdf.test_cut( v );
			}
		}
		else {
			// Link or lca
			let (u,v) = generate::generate_edge( num_vertices, &mut rng );
			let u = NodeIdx::new( u );
			let v = NodeIdx::new( v );
			tdf.test_link_or_lca( u, v );
		}
	}
}