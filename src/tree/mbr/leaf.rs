// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::geometry::Rect;
use std::fmt::Debug;
use std::marker::PhantomData;
use crate::tree::mbr::MbrLeafGeometry;
use crate::FP;

/// A tree leaf
#[derive(Debug, Clone)]
pub struct MbrLeaf<P: FP, const DIM: usize, LG, T> {
    pub geometry: LG,
    pub item: T,
    _p: PhantomData<P>,
}

impl<P: FP, const DIM: usize, LG, T> MbrLeaf<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    /// New leaf from geometry and item
    pub fn new(geometry: LG, item: T) -> MbrLeaf<P, DIM, LG, T> {
        MbrLeaf {
            geometry,
            item,
            _p: PhantomData,
        }
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

impl<P: FP, const DIM: usize, LG, T> MbrLeafGeometry<P, DIM> for MbrLeaf<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    fn dim(&self) -> usize {
        self.geometry.dim()
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
}
