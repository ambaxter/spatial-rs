// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod mbr;

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use shapes::{LeafShape, Rect};
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::marker::PhantomData;

/// A tree leaf
#[derive(Debug)]
pub struct Leaf<P, DIM, LS, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub shape: LS,
    pub item: T,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
}

impl<P, DIM, LS, T> Leaf<P, DIM, LS, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LS: LeafShape<P, DIM>
{
/// New leaf from shape and item
    pub fn new(shape: LS, item: T) -> Leaf<P, DIM, LS, T> {
        Leaf{shape: shape, item: item, _p: PhantomData, _dim: PhantomData}
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

impl<P, DIM, LS, T> LeafShape<P, DIM> for Leaf<P, DIM, LS, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LS: LeafShape<P, DIM> {

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

/// Query trait for navigating the tree
pub trait SpatialQuery<P, DIM, LS, LEVEL, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Returns true if the leaf matches the query
    fn accept_leaf(&self, leaf: &Leaf<P, DIM, LS, T>) -> bool;
    /// Returns true if the level matches the query
    fn accept_level(&self, level: &LEVEL) -> bool;
}

// TODO: Figure this out later :/
// pub trait SpatialMap<'tree, P, DIM, LS, LEVEL, T>
// where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
// LS: 'tree,
// T: 'tree
// {
// Insert an item
// fn insert(&mut self, shape: LS, item: T);
//
// Remove all items whose shapes are accepted by the query
// fn remove<Q: SpatialQuery<P, DIM, LS, LEVEL, T>>(&mut self, query: Q) -> Vec<(LS, T)>;
//
// Remove all items whose shapes are accepted by the query and where f(&T) returns false
// fn retain<Q, F>(&mut self, query: Q, f: F) -> Vec<(LS, T)>
// where Q: SpatialQuery<P, DIM, LS, LEVEL, T>,
// F: FnMut(&T) -> bool;
//
// Whether the map is empty
// fn is_empty(&self) -> bool;
//
// Length of the map
// fn len(&self) -> usize;
//
// Clear the map
// fn clear(&mut self);
//
// Iter for the map
// fn iter<ITER: Iterator<Item=(&'tree LS, &'tree T)>>(&'tree self) -> ITER;
//
// IterMut for the map
// fn iter_mut<ITERM: Iterator<Item=(&'tree LS, &'tree mut T)>>(&'tree mut self) -> ITERM;
//
// Iter for the map with a given query
// fn iter_query<Q, ITER>(&'tree self, query: Q) -> ITER
// where Q: SpatialQuery<P, DIM, LS, LEVEL, T>,
// ITER: Iterator<Item=(&'tree LS, &'tree T)>;
//
// IterMut for the map with a given query
// fn iter_query_mut<Q, ITERM>(&'tree mut self, query: Q) -> ITERM
// where Q: SpatialQuery<P, DIM, LS, LEVEL, T>,
// ITERM: Iterator<Item=(&'tree LS, &'tree mut T)>;
// }
//
