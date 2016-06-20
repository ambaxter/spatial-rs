// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use] extern crate itertools;
#[macro_use] extern crate generic_array;

extern crate ordered_float;
extern crate num;
extern crate typenum;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod shapes;
mod tree;
mod vecext;

use typenum::{U32, U64, Unsigned};
use tree::mbr::index::r::RRemove;
use tree::mbr::index::rstar::RStarInsert;
pub use tree::mbr::{MbrMap, MbrQuery};
use generic_array::ArrayLength;
pub use shapes::{Shape, Shapes, Point, LineSegment, Rect};
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;


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
          SHAPE: Shape<P, DIM>,
{
    /// Create a new R* tree with min and max children lengths set to 32 and 64, respectively
    pub fn new() -> MbrMap<P, DIM, SHAPE, RStarInsert<P, DIM, SHAPE, U32, U64, T>, RRemove<P, DIM, SHAPE, U32, T>, T> {
        MbrMap::new(RStarInsert::new(), RRemove::new())
    }

    /// Create a new R* tree with min and max children lengths as provided
    pub fn new_with_limits<MIN: Unsigned, MAX: Unsigned>() -> MbrMap<P, DIM, SHAPE, RStarInsert<P, DIM, SHAPE, MIN, MAX, T>, RRemove<P, DIM, SHAPE, MIN, T>, T> {
        MbrMap::new(RStarInsert::new(), RRemove::new())
    }
}

#[cfg(test)]
mod tests {

    use typenum::consts::{U8, U16};
    use super::*;
    #[test]
    fn rstar_integration() {
        let mut tree_map = RStar::new_with_limits::<U8, U16>();
        for i in 0..32 {
            let i_f32 = i as f32;
            tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
        }
        
        assert_eq!(tree_map.len(), tree_map.iter().count());        
        let query = RectQuery::ContainedBy(Rect::from_corners(arr![f32; 0.0f32, 0.0f32, 0.0f32], arr![f32; 9.0f32, 9.0f32, 9.0f32]));

        let removed = tree_map.remove(query.clone());
        assert_eq!(10, removed.len());
        assert_eq!(22, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());
       
        let removed_retain = tree_map.retain(RectQuery::ContainedBy(Rect::max()), |x| *x >= 20);
        assert_eq!(10, removed_retain.len());
        assert_eq!(12, tree_map.len());
        assert_eq!(tree_map.len(), tree_map.iter().count());
    }
}
