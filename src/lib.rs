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

use tree::mbr::index::r::RRemove;
use tree::mbr::index::rstar::RStarInsert;
use tree::mbr::MbrMap;
pub use tree::mbr::MbrQuery;
use generic_array::ArrayLength;
pub use shapes::{Shape, Shapes, Point, LineSegment, Rect};
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;


/// Convenience struct for creating a new R* Tree
pub struct RStar<P, DIM, LSHAPE, T> {
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _s: PhantomData<LSHAPE>,
    _t: PhantomData<T>,
}

/// An R* Tree
pub type RStarTree<P, DIM, LSHAPE, T> =  MbrMap<P, DIM, LSHAPE, RStarInsert<P, DIM, LSHAPE, T>, RRemove<P, DIM, LSHAPE, T>, T>;

impl<P, DIM, LSHAPE, T> RStar<P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          LSHAPE: Shape<P, DIM>,
{
/// Create a new R* tree with min and max children lengths set to 25 and 64, respectively
    pub fn new() -> RStarTree<P, DIM, LSHAPE, T> {
        MbrMap::new(RStarInsert::new(), RRemove::new())
    }

/// Create a new R* tree with min and max children lengths as provided
    pub fn new_with_limits(min: usize, max: usize) -> RStarTree<P, DIM, LSHAPE, T> {
        MbrMap::new(RStarInsert::new_with_limits(min, max), RRemove::with_limits(min))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn rstar_integration() {
        let mut tree_map = RStar::new_with_limits(8, 16);
        for i in 0..32 {
            let i_f32 = i as f32;
            tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
            println!("i: {:?}", i);
        }
        assert_eq!(32, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());

        println!("Remove query");
        let removed = tree_map.remove(MbrQuery::ContainedBy(Rect::from_corners(arr![f32; 0.0f32, 0.0f32, 0.0f32], arr![f32; 9.0f32, 9.0f32, 9.0f32])));
        assert_eq!(10, removed.len());
        assert_eq!(22, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());

        println!("Retain query");
        let removed_retain = tree_map.retain(MbrQuery::ContainedBy(Rect::max()), |x| *x >= 20);
        assert_eq!(10, removed_retain.len());
        assert_eq!(12, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());

        println!("Remove all");
        let retain_none = tree_map.remove(MbrQuery::ContainedBy(Rect::max()));
        assert_eq!(12, retain_none.len());
        assert_eq!(0, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());

        for i in 0..32 {
            let i_f32 = i as f32;
            tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
            println!("i: {:?}", i);
        }
        assert_eq!(32, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());
    }
}
