// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate itertools;
#[macro_use]
extern crate generic_array;

extern crate ordered_float;
extern crate num;
extern crate typenum;
extern crate parking_lot;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod shapes;
mod tree;
mod vecext;

pub use tree::mbr::{MbrLeafShape, MbrMap, MbrRectQuery};
use tree::mbr::index::IndexInsert;
use tree::mbr::index::r::RRemove;
use tree::mbr::index::rstar::RStarInsert;
use generic_array::ArrayLength;
pub use shapes::{Shapes, Point, LineSegment, Rect};
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;


/// Convenience struct for creating a new R* Tree
pub struct RStar<P, DIM, LS, T> {
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _ls: PhantomData<LS>,
    _t: PhantomData<T>,
}

/// R* Tree Type
pub type RStarTree<P, DIM, LS, T> =  MbrMap<P, DIM, LS, RStarInsert<P, DIM, LS, T>, RRemove<P, DIM, LS, T>, T>;

impl<P, DIM, LS, T> RStar<P, DIM, LS, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          LS: MbrLeafShape<P, DIM>,
{
/// Create a new R* tree with min and max children lengths set to 25 and 64, respectively
    pub fn new() -> RStarTree<P, DIM, LS, T> {
        RStar::map_from_insert(RStarInsert::new())
    }

/// Create a new R* tree with max children lengths as provided. min length will be set to 0.3 * max
    pub fn new_with_max(max: usize) -> RStarTree<P, DIM, LS, T> {
        RStar::map_from_insert(RStarInsert::new_with_max(max))
    }

/// Creates a mew R* tree with options as provided. min children will be set to reinsert_p.min(split_p) * max
    pub fn new_with_options(max: usize, reinsert_p: f32, split_p: f32, choose_subtree_p: usize) -> RStarTree<P, DIM, LS, T> {
        RStar::map_from_insert(RStarInsert::new_with_options(max, reinsert_p, split_p, choose_subtree_p))
    }

    fn map_from_insert(rstar_insert: RStarInsert<P, DIM, LS, T>) -> RStarTree<P, DIM, LS, T> {
        let min = rstar_insert.preferred_min();
        MbrMap::new(rstar_insert, RRemove::with_min(min))
    }
}