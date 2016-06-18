// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate itertools;

extern crate ordered_float;
extern crate num;
extern crate typenum;

#[macro_use]
extern crate generic_array;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod shapes;
mod tree;
mod index;
mod vecext;

use typenum::{U32, U64, Unsigned};
use index::r::RRemove;
use index::rstar::RStarInsert;
use generic_array::ArrayLength;
pub use shapes::{Shape, Shapes, Point, LineSegment, Rect};
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;
pub use tree::{SpatialMap, RectQuery};

/// Convenience struct for creating a new R* Tree
pub struct RStar<P, DIM, SHAPE, T> {
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _s: PhantomData<SHAPE>,
    _t: PhantomData<T>,
}

impl<P, DIM, SHAPE, T> RStar<P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          SHAPE: Shape<P, DIM>
{
    /// Create a new R* tree with min and max children lengths set to 32 and 64, respectively
    pub fn new() -> SpatialMap<P, DIM, SHAPE, RStarInsert<P, DIM, SHAPE, U32, U64, T>, RRemove<P, DIM, SHAPE, U32, T>, T> {
        SpatialMap::new(RStarInsert::new(), RRemove::new())
    }

    /// Create a new R* tree with min and max children lengths as provided
    pub fn new_with_limits<MIN: Unsigned, MAX: Unsigned>() -> SpatialMap<P, DIM, SHAPE, RStarInsert<P, DIM, SHAPE, MIN, MAX, T>, RRemove<P, DIM, SHAPE, MIN, T>, T> {
        SpatialMap::new(RStarInsert::new(), RRemove::new())
    }
}

#[cfg(test)]
mod tests {

    use typenum::consts::{U3, U8};
    use super::*;
    #[test]
    fn rstar_integration() {
        let mut tree_map = RStar::new_with_limits::<U3, U8>();
        for i in 0..25 {
            let i_f32 = i as f32;
            tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
        }
        for leaf in tree_map.iter() {
            println!("All: {:?}, Item: {:?}", leaf.0, leaf.1)
        }
        assert_eq!(tree_map.len(), tree_map.iter().count());
        let query = RectQuery::ContainedBy(Rect::from_corners(arr![f32; 0.0f32, 0.0f32, 0.0f32], arr![f32; 9.0f32, 9.0f32, 9.0f32]));
        for leaf in tree_map.iter_query(query.clone()) {
            println!("ToRemove: {:?}, Item: {:?}", leaf.0, leaf.1)
        }
        let removed = tree_map.remove(query.clone());
        assert_eq!(10, removed.len());
        assert_eq!(15, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());
        let removed_retain = tree_map.retain(RectQuery::ContainedBy(Rect::max()), |x| *x >= 20);
        assert_eq!(10, removed_retain.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());
        for leaf in tree_map.iter() {
            println!("All: {:?}, Item: {:?}", leaf.0, leaf.1)
        }
        println!("Nodes: {:?}", tree_map.root)
    }
}
