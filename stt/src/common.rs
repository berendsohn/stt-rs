//! Some simple trait extensions and implementations.

use core::fmt::{Display, Formatter};
use core::option::Option;
use core::option::Option::{None, Some};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops;
use num_traits::{PrimInt, Signed, Unsigned};

use WeightOrInfinity::*;

use crate::{MonoidWeight, NodeData, NodeIdx, PathWeightNodeData};


/// Group (Z,+), implemented using [isize].
pub type IsizeAddGroupWeight = SignedAddGroupWeight<isize>;

/// Monoid (N,max), implemented using [usize].
pub type UsizeMaxMonoidWeight = UnsignedMaxMonoidWeight<usize>;

/// Monoid (N,max), implemented using [usize], that additionally stores a maximum edge.
pub type UsizeMaxMonoidWeightWithMaxEdge = MonoidWeightWithMaxEdge<UsizeMaxMonoidWeight>;


/// A weight type that also forms a group by allowing negation.
pub trait GroupWeight : MonoidWeight + ops::Neg + ops::Sub<Self, Output=Self> {}


/// Wrapper around a weight that adds an infinity element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightOrInfinity<TWeight : MonoidWeight> {
	/// Infinity
	Infinite,
	
	/// A finite value
	Finite( TWeight )
}

impl<TWeight : MonoidWeight> WeightOrInfinity<TWeight> {
	/// Returns this as `TWeight`, if finite. Panics otherwise.
	pub fn unwrap( &self ) -> TWeight {
		match self {
			Infinite => panic!( "Cannot unwrap infinite weight." ),
			Finite( w ) => *w
		}
	}
}

impl<TWeight : MonoidWeight> Display for WeightOrInfinity<TWeight> {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		match self {
			Infinite => write!(f, "∞" ),
			Finite( weight ) => write!( f, "{}", weight )
		}

	}
}

impl<TWeight : MonoidWeight + Ord> Ord for WeightOrInfinity<TWeight> {
	fn cmp( &self, other : &Self ) -> Ordering {
		if let Finite( s ) = self {
			if let Finite( o ) = other {
				s.cmp( o )
			}
			else {
				Ordering::Less
			}
		}
		else {
			if let Finite( _ ) = other {
				Ordering::Greater
			}
			else {
				Ordering::Equal
			}
		}
	}
}

impl<TWeight : MonoidWeight + Ord> PartialOrd for WeightOrInfinity<TWeight> {
	fn partial_cmp( &self, other : &Self ) -> Option<Ordering> {
		Some( self.cmp( other ) )
	}
}

impl<TWeight : MonoidWeight> ops::Add<Self> for WeightOrInfinity<TWeight> {
	type Output = Self;

	fn add( self, rhs : Self ) -> Self {
		if let Finite( lv) = self {
			if let Finite( rv ) = rhs {
				return Finite( lv + rv );
			}
		}
		return Infinite;
	}
}

impl<TWeight : MonoidWeight> ops::Add<TWeight> for WeightOrInfinity<TWeight> {
	type Output = Self;
	
	fn add( self, rhs : TWeight ) -> Self {
		match self {
			Infinite => Infinite,
			Finite( w ) => Finite( w + rhs )
		}
	}
}

impl<TWeight : GroupWeight> ops::Sub<TWeight> for WeightOrInfinity<TWeight> {
	type Output = Self;
	
	fn sub( self, rhs : TWeight ) -> Self {
		match self {
			Infinite => Infinite,
			Finite( w ) => Finite( w - rhs )
		}
	}
}


/// A weight type with only one element, the identity.
/// 
/// This is useful for connectivity testing in unweighted forests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmptyGroupWeight {}

impl ops::Add<Self> for EmptyGroupWeight {
	type Output = EmptyGroupWeight;

	fn add( self, _ : Self ) -> Self { self }
}

impl Display for EmptyGroupWeight {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "" )
	}
}

impl MonoidWeight for EmptyGroupWeight {
	fn identity() -> Self {
		Self{}
	}
}

impl ops::Neg for EmptyGroupWeight {
	type Output = Self;
	
	fn neg(self) -> Self { self }
}

impl ops::Sub<Self> for EmptyGroupWeight {
	type Output = Self;
	
	fn sub( self, _ : Self ) -> Self { self }
}

impl GroupWeight for EmptyGroupWeight {}


/// Node data that doesn't store anything. Like [EmptyGroupWeight], this is useful for connectivity
/// testing in unweighted forests.
#[derive(Clone)]
pub struct EmptyNodeData {}

impl Display for EmptyNodeData {
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "" )
	}
}

impl NodeData for EmptyNodeData {
	type TWeight = EmptyGroupWeight;
	
	fn new( _ : NodeIdx ) -> Self { EmptyNodeData{} }
}

impl PathWeightNodeData for EmptyNodeData {
	fn get_parent_path_weight( &self ) -> EmptyGroupWeight {
		EmptyGroupWeight {}
	}
}


/// Weight with unsigned integer values, where the monoid operation is the maximum.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UnsignedMaxMonoidWeight<TNum>
	where TNum : PrimInt + Unsigned + Debug + Display
{
	value : TNum
}

impl<TNum> UnsignedMaxMonoidWeight<TNum>
	where TNum : PrimInt + Unsigned + Debug + Display
{
	/// Constructs a new weight with the given value.
	pub fn new( value : TNum ) -> Self {
		UnsignedMaxMonoidWeight { value }
	}
	
	/// This as `TNum`.
	pub fn value( &self ) -> TNum {
		self.value
	}
}

impl<TNum> ops::Add<UnsignedMaxMonoidWeight<TNum>> for UnsignedMaxMonoidWeight<TNum>
	where TNum : PrimInt + Unsigned + Debug + Display
{
	type Output = Self;

	fn add( self, rhs: Self ) -> Self {
		if self.value > rhs.value {
			self
		}
		else {
			rhs
		}
	}
}

impl<TNum> Display for UnsignedMaxMonoidWeight<TNum>
	where TNum : PrimInt + Unsigned + Debug + Display
{
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.value )
	}
}

impl<TNum> MonoidWeight for UnsignedMaxMonoidWeight<TNum>
	where TNum : PrimInt + Unsigned + Debug + Display
{
	fn identity() -> Self {
		UnsignedMaxMonoidWeight { value: TNum::zero() }
	}
}


/// Weight with signed integer values, where the group operation is addition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	value : TNum
}

impl<TNum> SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	/// Creates a new weight with the given value.
	pub fn new( value : TNum ) -> Self {
		SignedAddGroupWeight { value }
	}
	
	/// This as `TNum`.
	pub fn value( &self ) -> TNum {
		self.value
	}
}

impl<TNum> ops::Add<Self> for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	type Output = Self;

	fn add( self, rhs : Self ) -> Self {
		Self::new( self.value + rhs.value )
	}
}

impl<TNum> ops::Neg for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	type Output = Self;

	fn neg(self) -> Self {
		Self::new( - self.value )
	}
}

impl<TNum> ops::Sub<Self> for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	type Output = Self;

	fn sub( self, rhs : Self ) -> Self {
		self + (-rhs)
	}
}

impl<TNum> Display for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	fn fmt( &self, f : &mut Formatter<'_> ) -> std::fmt::Result {
		write!( f, "{}", self.value )
	}
}

impl<TNum> MonoidWeight for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{
	fn identity() -> Self {
		SignedAddGroupWeight::new( TNum::zero() )
	}
}

impl<TNum> GroupWeight for SignedAddGroupWeight<TNum>
	where TNum : PrimInt + Signed + Debug + Display
{}


/// Monoid that additionally stores a (max-weight) edge.
/// 
/// Stores a weight from the underlying monoid `TSubMonoid` and, possibly, an edge. The underlying
/// monoid must be ordered. The monoid operation of this monoid is the same as the one from the
/// underlying monoid, except that the edge from the larger value is retained.
/// 
/// The identity of this monoid is the identity of the underlying monoid with no edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MonoidWeightWithMaxEdge<TSubMonoid : MonoidWeight + Ord> {
	value : TSubMonoid,
	edge : Option<(NodeIdx, NodeIdx)>
}

impl<TSubMonoid : MonoidWeight + Ord> MonoidWeightWithMaxEdge<TSubMonoid> {
	/// Construct a new (non-zero) weight with the given value and edge.
	pub fn new( value : TSubMonoid, edge : (NodeIdx, NodeIdx) ) -> Self {
		MonoidWeightWithMaxEdge { value, edge : Some( edge ) }
	}
	
	/// Returns the underlying weight.
	pub fn weight( &self ) -> TSubMonoid {
		self.value
	}
	
	/// Returns the maximum edge associated with this weight.
	/// 
	/// Panics if this is the identity weight.
	pub fn unwrap_edge( &self ) -> (NodeIdx, NodeIdx) {
		self.edge.unwrap()
	}
}

impl<TSubMonoid : MonoidWeight + Ord> ops::Add<MonoidWeightWithMaxEdge<TSubMonoid>> for MonoidWeightWithMaxEdge<TSubMonoid> {
	type Output = MonoidWeightWithMaxEdge<TSubMonoid>;

	fn add( self, rhs : Self ) -> Self {
		let edge;
		if self.value > rhs.value {
			edge = self.edge
		}
		else {
			edge = rhs.edge
		};
		Self{ value : self.value + rhs.value, edge }
	}
}

impl<TSubMonoid : MonoidWeight + Ord> Display for MonoidWeightWithMaxEdge<TSubMonoid> {
	fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result {
		match self.edge {
			None => write!( f, "{}(-)", self.value ),
			Some( e ) => write!( f, "{}({},{})", self.value, e.0, e.1 )
		}
	}
}

impl<TSubMonoid : MonoidWeight + Ord> MonoidWeight for MonoidWeightWithMaxEdge<TSubMonoid> {
	fn identity() -> Self {
		MonoidWeightWithMaxEdge { value : TSubMonoid::identity(), edge : None }
	}
}
