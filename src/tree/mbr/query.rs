// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use shapes::Rect;
use tree::mbr::leafshape::MbrLeafShape;
use tree::mbr::leaf::MbrLeaf;
use tree::mbr::node::MbrNode;
use std::fmt::Debug;
use generic_array::ArrayLength;

/// Query trait for navigating the tree
pub trait MbrQuery<P, DIM, LS, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Returns true if the leaf matches the query
    fn accept_leaf(&self, leaf: &MbrLeaf<P, DIM, LS, T>) -> bool;
    /// Returns true if the level matches the query
    fn accept_level(&self, level: &MbrNode<P, DIM, LS, T>) -> bool;
}

/// Rect based query
#[derive(Debug, Clone)]
pub enum MbrRectQuery<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Matching leaves are ones that are completely contained by this rect
    ContainedBy(Rect<P, DIM>),
    /// Matching leaves are ones that overlap this rect
    Overlaps(Rect<P, DIM>),
}

impl<P, DIM, LS, T> MbrQuery<P, DIM, LS, T> for MbrRectQuery<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LS: MbrLeafShape<P, DIM>{

// Does this query accept the given leaf?
    fn accept_leaf(&self, leaf: &MbrLeaf<P, DIM, LS, T>) -> bool {
        match *self {
            MbrRectQuery::ContainedBy(ref query) => leaf.shape.contained_by_mbr(query),
            MbrRectQuery::Overlaps(ref query) => leaf.shape.overlapped_by_mbr(query),
        }
    }

// Does this query accept the given level?
    fn accept_level(&self, level: &MbrNode<P, DIM, LS, T>) -> bool {
        match *self {
            MbrRectQuery::ContainedBy(ref query) => level.overlapped_by_mbr(query),
            MbrRectQuery::Overlaps(ref query) => level.overlapped_by_mbr(query),
        }
    }
}