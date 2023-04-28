//! NodeData implementations specific to two-cut trees

use std::fmt::{Display, Formatter};

use crate::{MonoidWeight, NodeData, NodeDataAccess, NodeIdx, PathWeightNodeData};
use crate::common::WeightOrInfinity;
use crate::common::GroupWeight;
use crate::common::WeightOrInfinity::{Finite, Infinite};
use crate::twocut::UpdatingNodeData;
use crate::twocut::basic::STTStructureRead;


/// Node data that stores distance (path weight) to parent and adjacent ancestor
#[derive(Clone, Debug)]
pub struct MonoidPathWeightNodeData<TWeight : MonoidWeight> {
	/// Distance to parent, or None if it doesn't exist
	pdist : WeightOrInfinity<TWeight>,

	/// Distance to ancestor a, where a is not the parent but is adjacent
	///  to some node in this node's subtree; or None if a doesn't exist
	adist : WeightOrInfinity<TWeight>
}

impl<TWeight : MonoidWeight> Display for MonoidPathWeightNodeData<TWeight> {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}/{}", self.pdist, self.adist )
	}
}

impl<TWeight : MonoidWeight> NodeData for MonoidPathWeightNodeData<TWeight> {
	type TWeight = TWeight;
	
	fn new() -> Self {
		MonoidPathWeightNodeData { pdist : Infinite, adist : Infinite }
	}
}

impl<TWeight : MonoidWeight> PathWeightNodeData for MonoidPathWeightNodeData<TWeight> {
	fn get_parent_path_weight( &self ) -> TWeight {
		self.pdist.unwrap()
	}
}

impl<TWeight : MonoidWeight> UpdatingNodeData for MonoidPathWeightNodeData<TWeight> {
	fn before_rotation(t: &mut (impl NodeDataAccess<Self> + STTStructureRead), v: NodeIdx ) {
		let p = t.get_parent( v ).unwrap();

		if let Some( c ) = t.get_direct_separator_child(v) {
			// c is between v and p; c will switch parent from v to p.
			debug_assert!( t.is_direct_separator( c ) );
			// Swapping dist(c,p) and dist(c,v)
			let c_data = t.data_mut( c );
			(c_data.pdist, c_data.adist) = (c_data.adist, c_data.pdist);
		}

		let old_v_data = t.data(v).clone();
		let old_p_data = t.data( p ).clone();

		t.data_mut( p ).pdist = old_v_data.pdist; // dist(p,v)

		// if t.is_direct_separator( v ) {
		if t.get_direct_separator_child( p ) == Some( v ) {
			// v is between p and gp
			t.data_mut(v).pdist = old_v_data.adist; // dist(v, gp)
			
			// If p is a separator, then p has an ancestor a such that p is between v and a. Thus,
			// v.adist = dist(v,p) + dist(p,a) = dist(v,a).
			// Otherwise, v.adist = infinity, but then also old_p.adist is infinity. In both cases,
			// the following works.
			t.data_mut( v ).adist = old_v_data.pdist + old_p_data.adist;
		}
		else {
			// p is between v and gp, or gp doesn't exist
			t.data_mut(v).pdist = old_v_data.pdist + old_p_data.pdist; // dist(v,p) + dist(p,gp) = dist(v,gp)
			// v_data.adist does not change
			t.data_mut( p ).adist = old_p_data.pdist; // dist(p,gp)
		}
	}

	fn after_attached( t: &mut (impl NodeDataAccess<Self> + STTStructureRead), v: NodeIdx, weight: TWeight ) {
		t.data_mut( v ).pdist = Finite( weight );
	}

	fn before_detached( t: &mut (impl NodeDataAccess<Self> + STTStructureRead), v: NodeIdx ) {
		t.data_mut( v ).pdist = Infinite;
	}
}


/// Node data that stores distance (path weight) to parent. Weight must be a group.
#[derive(Clone, Debug)]
pub struct GroupPathWeightNodeData<TWeight : GroupWeight> {
	/// Distance to parent, or None if it doesn't exist
	pdist : WeightOrInfinity<TWeight>
}

impl<TWeight : GroupWeight> Display for GroupPathWeightNodeData<TWeight> {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.pdist )
	}
}

impl<TWeight : GroupWeight> NodeData for GroupPathWeightNodeData<TWeight> {
	type TWeight = TWeight;
	
	fn new() -> Self {
		GroupPathWeightNodeData { pdist : Infinite }
	}
}

impl<TWeight : GroupWeight> PathWeightNodeData for GroupPathWeightNodeData<TWeight> {
	fn get_parent_path_weight( &self ) -> TWeight {
		self.pdist.unwrap()
	}
}

impl<TWeight : GroupWeight> UpdatingNodeData for GroupPathWeightNodeData<TWeight> {
	fn before_rotation( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx ) {
		let p = t.get_parent( v ).unwrap();
		let v_pdist_old = t.data( v ).pdist.unwrap();
		let p_pdist_old_p = t.data( p ).pdist;

		// println!( "Before rotation: {}, {}", t.data( v ).pdist, t.data( p ).pdist );

		if let Some( c ) = t.get_direct_separator_child( v ) {
			// c is between v and p; c will switch parent from v to p.
			debug_assert!( t.is_direct_separator( c ) );
			let c_pdist_old = t.data( c ).pdist.unwrap();
			t.data_mut( c ).pdist = Finite( v_pdist_old - c_pdist_old ); // d(v,p) - d(v,c)
		}

		t.data_mut( p ).pdist = Finite( v_pdist_old ); // d(v,p)

		if let Finite( p_pdist_old ) = p_pdist_old_p {
			// p is not the root, has a parent g
			t.data_mut( v ).pdist = Finite(
				// if t.is_direct_separator( v ) {
				if t.get_direct_separator_child( p ) == Some( v ) {
					p_pdist_old - v_pdist_old // d(g,p) - d(v,p) = d(v,g)
				}
				else {
					p_pdist_old + v_pdist_old // d(g,p) + d(v,p) = d(v,g)
				}
			)
		}
		else {
			t.data_mut( v ).pdist = Infinite;
		}
		// println!( "After rotation: {}, {}", t.data( v ).pdist, t.data( p ).pdist );
	}

	fn after_attached( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx, weight : TWeight ) {
		t.data_mut( v ).pdist = Finite( weight );
	}

	fn before_detached( t : &mut (impl NodeDataAccess<Self> + STTStructureRead), v : NodeIdx ) {
		t.data_mut( v ).pdist = Infinite;
	}
}
