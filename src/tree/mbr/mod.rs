// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Collection of minimum bounding rectangle spatial trees

mod index;
mod leaf;
mod leafgeometry;
mod map;
mod query;
mod node;

pub use tree::mbr::leaf::MbrLeaf;
pub use tree::mbr::leafgeometry::MbrLeafGeometry;
pub use tree::mbr::map::{Iter, IterMut, MbrMap};
pub use tree::mbr::node::MbrNode;
pub use tree::mbr::query::{MbrQuery, MbrRectQuery};
use tree::mbr::index::IndexInsert;
use tree::mbr::index::r::RRemove;
use tree::mbr::index::rstar::RStarInsert;
use generic_array::ArrayLength;
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;


/// Convenience struct for creating a new R* Tree
pub struct RStar<P, DIM, LG, T> {
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

/// R* Tree Type
pub type RStarTree<P, DIM, LG, T> =  MbrMap<P, DIM, LG, RStarInsert<P, DIM, LG, T>, RRemove<P, DIM, LG, T>, T>;

impl<P, DIM, LG, T> RStar<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          LG: MbrLeafGeometry<P, DIM>,
{
/// Create a new R* tree with min and max children lengths set to 25 and 64, respectively
    pub fn new() -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new())
    }

/// Create a new R* tree with max children lengths as provided. min length will be set to 0.3 * max
    pub fn new_with_max(max: usize) -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new_with_max(max))
    }

/// Creates a mew R* tree with options as provided. min children will be set to reinsert_p.min(split_p) * max
    pub fn new_with_options(max: usize, reinsert_p: f32, split_p: f32, choose_subtree_p: usize) -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new_with_options(max, reinsert_p, split_p, choose_subtree_p))
    }

    fn map_from_insert(rstar_insert: RStarInsert<P, DIM, LG, T>) -> RStarTree<P, DIM, LG, T> {
        let min = rstar_insert.preferred_min();
        MbrMap::new(rstar_insert, RRemove::with_min(min))
    }
}