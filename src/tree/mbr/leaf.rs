// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use geometry::Rect;
use tree::mbr::MbrLeafGeometry;
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::marker::PhantomData;

/// A tree leaf
#[derive(Debug, Clone)]
pub struct MbrLeaf<P, DIM, LG, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub geometry: LG,
    pub item: T,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
}

impl<P, DIM, LG, T> MbrLeaf<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LG: MbrLeafGeometry<P, DIM>
{
/// New leaf from geometry and item
    pub fn new(geometry: LG, item: T) -> MbrLeaf<P, DIM, LG, T> {
        MbrLeaf{geometry: geometry, item: item, _p: PhantomData, _dim: PhantomData}
    }

/// Consumes self, returning the geometry and item
    pub fn extract(self) -> (LG, T) {
        (self.geometry, self.item)
    }

    pub fn as_tuple(&self) -> (&LG, &T) {
        (&self.geometry, &self.item)
    }

    pub fn as_mut_tuple(&mut self) -> (&LG, &mut T) {
        (&self.geometry, &mut self.item)
    }
}

impl<P, DIM, LG, T> MbrLeafGeometry<P, DIM> for MbrLeaf<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LG: MbrLeafGeometry<P, DIM> {

    fn dim(&self) -> usize {
        self.geometry.dim()
    }

    fn expand_mbr_to_fit(&self, edges: &mut Rect<P, DIM>) {
        self.geometry.expand_mbr_to_fit(edges)
    }

    fn distance_from_mbr_center(&self, edges: &Rect<P, DIM>) -> P {
        self.geometry.distance_from_mbr_center(edges)
    }

    fn contained_by_mbr(&self, edges: &Rect<P, DIM>) -> bool {
        self.geometry.contained_by_mbr(edges)
    }

    fn overlapped_by_mbr(&self, edges: &Rect<P, DIM>) -> bool {
        self.geometry.overlapped_by_mbr(edges)
    }

    fn area_overlapped_with_mbr(&self, edges: &Rect<P, DIM>) -> P {
        self.geometry.area_overlapped_with_mbr(edges)
    }

    fn area(&self) -> P {
        self.geometry.area()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.geometry.min_for_axis(dim)
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.geometry.max_for_axis(dim)
    }
}