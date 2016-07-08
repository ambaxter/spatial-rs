// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use tree::mbr::{MbrNode, MbrLeaf, MbrLeafGeometry};
use geometry::Rect;
use std::fmt::Debug;
use generic_array::ArrayLength;

/// Level node of a tree. Either contains other levels or leaves
#[derive(Debug)]
pub enum XTreeNode<P, DIM, LG, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Contains only other levels
    Level {
        mbr: Rect<P, DIM>,
        // TODO: Replace with bitset
        split_dim: usize,
        super_node_size: Option<usize>,
        children: Vec<XTreeNode<P, DIM, LG, T>>,
    },
    /// Contains only leaves
    Leaves {
        mbr: Rect<P, DIM>,
        super_node_size: Option<usize>,
        children: Vec<MbrLeaf<P, DIM, LG, T>>,
    },
}

impl<P, DIM, LG, T> XTreeNode<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LG: MbrLeafGeometry<P, DIM>
{
    pub fn is_super(&self) -> bool {
        match *self {
            XTreeNode::Level{super_node_size, ..} => super_node_size.is_some(),
            XTreeNode::Leaves{super_node_size, ..} => super_node_size.is_some(),
        }
    }
}

impl<P, DIM, LG, T> MbrNode<P, DIM> for XTreeNode<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LG: MbrLeafGeometry<P, DIM> {

    fn new_leaves() -> XTreeNode<P, DIM, LG, T> {
        XTreeNode::Leaves{mbr: Rect::max_inverted(), super_node_size: None, children: Vec::new()}
    }

    fn new_no_alloc() -> XTreeNode<P, DIM, LG, T> {
        XTreeNode::Leaves{mbr: Rect::max_inverted(), super_node_size: None, children: Vec::with_capacity(0)}
    }

    fn has_leaves(&self) -> bool {
        match *self {
            XTreeNode::Level{..} => false,
            XTreeNode::Leaves{..} => true,
        }
    }

    fn has_levels(&self) -> bool {
        !self.has_leaves()
    }

    fn mbr(&self) -> &Rect<P, DIM> {
        match *self {
            XTreeNode::Level{ref mbr, ..} => mbr,
            XTreeNode::Leaves{ref mbr, ..} => mbr,
        }
    }

    fn mbr_mut(&mut self) -> &mut Rect<P, DIM> {
        match *self {
            XTreeNode::Level{ref mut mbr, ..} => mbr,
            XTreeNode::Leaves{ref mut mbr, ..} => mbr,
        }
    }

    fn len(&self) -> usize {
        match *self {
            XTreeNode::Level{ref children, ..} => children.len(),
            XTreeNode::Leaves{ref children, ..} => children.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match *self {
            XTreeNode::Level{ref children, ..} => children.is_empty(),
            XTreeNode::Leaves{ref children, ..} => children.is_empty(),
        }
    }

}


impl<P, DIM, LG, T> MbrLeafGeometry<P, DIM> for XTreeNode<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LG: MbrLeafGeometry<P, DIM> {

    fn dim(&self) -> usize {
        self.mbr().dim()
    }

    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>) {
        self.mbr().expand_mbr_to_fit(mbr)
    }

    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P {
        self.mbr().distance_from_mbr_center(mbr)
    }

    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        self.mbr().contained_by_mbr(mbr)
    }

    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        self.mbr().overlapped_by_mbr(mbr)
    }

    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P {
        self.mbr().area_overlapped_with_mbr(mbr)
    }

    fn area(&self) -> P {
        self.mbr().area()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.mbr().min_for_axis(dim)
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.mbr().max_for_axis(dim)
    }
}
