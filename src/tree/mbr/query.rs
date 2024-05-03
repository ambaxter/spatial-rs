// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::geometry::Rect;
use std::fmt::Debug;
use crate::tree::mbr::{MbrLeaf, MbrLeafGeometry, MbrNode};
use crate::FP;

/// Query trait for navigating the tree
pub trait MbrQuery<P: FP, const DIM: usize, LG, T, NODE> {
    /// Returns true if the leaf matches the query
    fn accept_leaf(&self, leaf: &MbrLeaf<P, DIM, LG, T>) -> bool;
    /// Returns true if the level matches the query
    fn accept_level(&self, level: &NODE) -> bool;
}

/// Rect based query
#[derive(Debug, Clone)]
pub enum MbrRectQuery<P: FP, const DIM: usize> {
    /// Matching leaves are ones that are completely contained by this rect
    ContainedBy(Rect<P, DIM>),
    /// Matching leaves are ones that overlap this rect
    Overlaps(Rect<P, DIM>),
}

impl<P: FP, const DIM: usize, LG, T, NODE> MbrQuery<P, DIM, LG, T, NODE> for MbrRectQuery<P, DIM>
where
    LG: MbrLeafGeometry<P, DIM>,
    NODE: MbrNode<P, DIM>,
{
    // Does this query accept the given leaf?
    fn accept_leaf(&self, leaf: &MbrLeaf<P, DIM, LG, T>) -> bool {
        match *self {
            MbrRectQuery::ContainedBy(ref query) => leaf.geometry.contained_by_mbr(query),
            MbrRectQuery::Overlaps(ref query) => leaf.geometry.overlapped_by_mbr(query),
        }
    }

    // Does this query accept the given level?
    fn accept_level(&self, level: &NODE) -> bool {
        match *self {
            MbrRectQuery::ContainedBy(ref query) => level.overlapped_by_mbr(query),
            MbrRectQuery::Overlaps(ref query) => level.overlapped_by_mbr(query),
        }
    }
}
