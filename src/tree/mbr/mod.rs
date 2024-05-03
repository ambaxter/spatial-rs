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
mod node;
mod query;

use std::marker::PhantomData;
use tree::mbr::index::r::{Linear, Quadratic, RInsert, RRemove, SeedSplit};
use tree::mbr::index::rstar::RStarInsert;
use tree::mbr::index::{IndexInsert, MbrNodeSplit};
pub use tree::mbr::leaf::MbrLeaf;
pub use tree::mbr::leafgeometry::MbrLeafGeometry;
pub use tree::mbr::map::{Iter, IterMut, MbrMap};
pub use tree::mbr::node::{MbrNode, RTreeNode};
pub use tree::mbr::query::{MbrQuery, MbrRectQuery};
use FP;

/// Convenience struct for creating a new R Tree
///
/// Agorithms described by Guttman, A. (1984). "R-Trees: A Dynamic Index Structure for Spatial Searching"
pub struct RTree<P: FP, const DIM: usize, LG, T> {
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

/// R Quadratic Tree Type
pub type RQuadraticTree<P, const DIM: usize, LG, T> = MbrMap<
    RTreeNode<P, DIM, LG, T>,
    RInsert<P, DIM, LG, T, SeedSplit<P, DIM, LG, T, Quadratic>>,
    RRemove<P, DIM, LG, T>,
>;
/// R Quadratic Tree Type
pub type RLinearTree<P, const DIM: usize, LG, T> = MbrMap<
    RTreeNode<P, DIM, LG, T>,
    RInsert<P, DIM, LG, T, SeedSplit<P, DIM, LG, T, Linear>>,
    RRemove<P, DIM, LG, T>,
>;

impl<P: FP, const DIM: usize, LG, T> RTree<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    /// Create a new R Tree using the Linear splitting algorithm with min and max children lengths set to 19 and 64, respectively
    pub fn new_linear() -> RLinearTree<P, DIM, LG, T> {
        // TODO: This type specification shouldn't be needed? Compliation error without them :/
        RTree::map_from_insert(RInsert::new(SeedSplit::<P, DIM, LG, T, Linear>::linear()))
    }

    /// Create a new R Tree using the Linear splitting algorithm with max children lengths as provided. min length will be set to 0.3 * max
    pub fn new_linear_with_max(max: usize) -> RLinearTree<P, DIM, LG, T> {
        RTree::map_from_insert(RInsert::new_with_max(
            SeedSplit::<P, DIM, LG, T, Linear>::linear(),
            max,
        ))
    }

    /// Create a new R Tree using the Quadratic splitting algorithm with min and max children lengths set to 19 and 64, respectively
    pub fn new_quadratic() -> RQuadraticTree<P, DIM, LG, T> {
        RTree::map_from_insert(RInsert::new(
            SeedSplit::<P, DIM, LG, T, Quadratic>::quadratic(),
        ))
    }

    /// Create a new R Tree using the Quadratic splitting algorithm with max children lengths as provided. min length will be set to 0.3 * max
    pub fn new_quadratic_with_max(max: usize) -> RQuadraticTree<P, DIM, LG, T> {
        RTree::map_from_insert(RInsert::new_with_max(
            SeedSplit::<P, DIM, LG, T, Quadratic>::quadratic(),
            max,
        ))
    }

    fn map_from_insert<S: MbrNodeSplit<P, DIM>>(
        insert: RInsert<P, DIM, LG, T, S>,
    ) -> MbrMap<RTreeNode<P, DIM, LG, T>, RInsert<P, DIM, LG, T, S>, RRemove<P, DIM, LG, T>> {
        let min = insert.preferred_min();
        MbrMap::new(insert, RRemove::with_min(min))
    }
}

/// R* Tree Type
pub type RStarTree<P, const DIM: usize, LG, T> =
    MbrMap<RTreeNode<P, DIM, LG, T>, RStarInsert<P, DIM, LG, T>, RRemove<P, DIM, LG, T>>;

/// Convenience struct for creating a new R* Tree
///
/// Algorithms descibed by Beckmann, N.; Kriegel, H. P.; Schneider, R.; Seeger, B. (1990). "The R*-tree: an efficient and robust access method for points and rectangles".
pub struct RStar<P: FP, const DIM: usize, LG, T> {
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P: FP, const DIM: usize, LG, T> RStar<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    /// Create a new R* tree with min and max children lengths set to 19 and 64, respectively
    pub fn new() -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new())
    }

    /// Create a new R* tree with max children lengths as provided. min length will be set to 0.3 * max
    pub fn new_with_max(max: usize) -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new_with_max(max))
    }

    /// Creates a mew R* tree with options as provided. min children will be set to reinsert_p.min(split_p) * max
    pub fn new_with_options(
        max: usize,
        reinsert_p: f32,
        split_p: f32,
        choose_subtree_p: usize,
    ) -> RStarTree<P, DIM, LG, T> {
        RStar::map_from_insert(RStarInsert::new_with_options(
            max,
            reinsert_p,
            split_p,
            choose_subtree_p,
        ))
    }

    fn map_from_insert(rstar_insert: RStarInsert<P, DIM, LG, T>) -> RStarTree<P, DIM, LG, T> {
        let min = rstar_insert.preferred_min();
        MbrMap::new(rstar_insert, RRemove::with_min(min))
    }
}
