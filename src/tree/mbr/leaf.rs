// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use geometry::Rect;
use tree::mbr::MbrLeafShape;
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::marker::PhantomData;

/// A tree leaf
#[derive(Debug)]
pub struct MbrLeaf<P, DIM, LS, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub shape: LS,
    pub item: T,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
}

impl<P, DIM, LS, T> MbrLeaf<P, DIM, LS, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LS: MbrLeafShape<P, DIM>
{
/// New leaf from shape and item
    pub fn new(shape: LS, item: T) -> MbrLeaf<P, DIM, LS, T> {
        MbrLeaf{shape: shape, item: item, _p: PhantomData, _dim: PhantomData}
    }

/// Consumes self, returning the shape and item
    pub fn extract(self) -> (LS, T) {
        (self.shape, self.item)
    }

    pub fn as_tuple(&self) -> (&LS, &T) {
        (&self.shape, &self.item)
    }

    pub fn as_mut_tuple(&mut self) -> (&LS, &mut T) {
        (&self.shape, &mut self.item)
    }
}

impl<P, DIM, LS, T> MbrLeafShape<P, DIM> for MbrLeaf<P, DIM, LS, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LS: MbrLeafShape<P, DIM> {

    fn dim(&self) -> usize {
        self.shape.dim()
    }

    fn expand_mbr_to_fit(&self, edges: &mut Rect<P, DIM>) {
        self.shape.expand_mbr_to_fit(edges)
    }

    fn distance_from_mbr_center(&self, edges: &Rect<P, DIM>) -> P {
        self.shape.distance_from_mbr_center(edges)
    }

    fn contained_by_mbr(&self, edges: &Rect<P, DIM>) -> bool {
        self.shape.contained_by_mbr(edges)
    }

    fn overlapped_by_mbr(&self, edges: &Rect<P, DIM>) -> bool {
        self.shape.overlapped_by_mbr(edges)
    }

    fn area_overlapped_with_mbr(&self, edges: &Rect<P, DIM>) -> P {
        self.shape.area_overlapped_with_mbr(edges)
    }

    fn area(&self) -> P {
        self.shape.area()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.shape.min_for_axis(dim)
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.shape.max_for_axis(dim)
    }
}