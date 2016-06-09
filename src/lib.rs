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
use tree::SpatialTree;
use index::r::RRemove;
use index::rstar::RStarInsert;
use generic_array::ArrayLength;
pub use shapes::{Shape, Shapes, Point, LineSegment, Rect};
use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::marker::PhantomData;
pub use tree::Query;

pub struct RStar<P, D, S, T> {
    _p: PhantomData<P>,
    _d: PhantomData<D>,
    _s: PhantomData<S>,
    _t: PhantomData<T>,
}

impl<P, D, S, T> RStar<P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          S: Shape<P, D>
{
    pub fn new() -> SpatialTree<P, D, S, RStarInsert<P, D, S, U32, U64, T>, RRemove<P, D, S, U32, U64, T>, T> {
        SpatialTree::new(RStarInsert::new(), RRemove::new())
    }

    pub fn new_with_limits<MIN: Unsigned, MAX: Unsigned>() -> SpatialTree<P, D, S, RStarInsert<P, D, S, MIN, MAX, T>, RRemove<P, D, S, MIN, MAX, T>, T> {
        SpatialTree::new(RStarInsert::new(), RRemove::new())
    }
}

#[cfg(test)]
mod tests {

    use typenum::consts::U3;
    use generic_array::GenericArray;
    #[test]
    fn it_works() {
        // let mut tree: RStarTree<f64, String> = RStarTree::new(2);
        // tree.insert(BoundingBox::new_from(&[(1.0f64, 1.0f64), (1.0f64, 1.0f64)]),
        // "1".to_owned());
        // tree.insert(BoundingBox::new_from(&[(2.0f64, 2.0f64), (2.0f64, 2.0f64)]),
        // "2".to_owned());
        // tree.insert(BoundingBox::new_from(&[(3.0f64, 3.0f64), (3.0f64, 3.0f64)]),
        // "3".to_owned());
        // tree.insert(BoundingBox::new_from(&[(4.0f64, 4.0f64), (4.0f64, 4.0f64)]),
        // "4".to_owned());
        // tree.insert(BoundingBox::new_from(&[(5.0f64, 25.0f64), (5.0f64, 25.0f64)]),
        // "5".to_owned());
        // tree.insert(BoundingBox::new_from(&[(6.0f64, 25.0f64), (6.0f64, 25.0f64)]),
        // "6".to_owned());
        // tree.insert(BoundingBox::new_from(&[(7.0f64, 25.0f64), (7.0f64, 25.0f64)]),
        // "7".to_owned());
        // tree.insert(BoundingBox::new_from(&[(8.0f64, 25.0f64), (8.0f64, 25.0f64)]),
        // "8".to_owned());
        // tree.insert(BoundingBox::new_from(&[(9.0f64, 25.0f64), (9.0f64, 25.0f64)]),
        // "9".to_owned());
        // tree.insert(BoundingBox::new_from(&[(10.0f64, 25.0f64), (10.0f64, 25.0f64)]),
        // "10".to_owned());
        // tree.insert(BoundingBox::new_from(&[(11.0f64, 25.0f64), (11.0f64, 25.0f64)]),
        // "11".to_owned());
        // tree.insert(BoundingBox::new_from(&[(12.0f64, 25.0f64), (12.0f64, 25.0f64)]),
        // "12".to_owned());
        // tree.insert(BoundingBox::new_from(&[(13.0f64, 25.0f64), (13.0f64, 25.0f64)]),
        // "13".to_owned());
        // tree.insert(BoundingBox::new_from(&[(14.0f64, 25.0f64), (14.0f64, 25.0f64)]),
        // "14".to_owned());
        // tree.insert(BoundingBox::new_from(&[(15.0f64, 25.0f64), (15.0f64, 25.0f64)]),
        // "15".to_owned());
        // tree.insert(BoundingBox::new_from(&[(16.0f64, 25.0f64), (16.0f64, 25.0f64)]),
        // "16".to_owned());
        // tree.insert(BoundingBox::new_from(&[(17.0f64, 25.0f64), (17.0f64, 25.0f64)]),
        // "17".to_owned());
        // tree.insert(BoundingBox::new_from(&[(18.0f64, 25.0f64), (18.0f64, 25.0f64)]),
        // "18".to_owned());
        // tree.insert(BoundingBox::new_from(&[(19.0f64, 25.0f64), (19.0f64, 25.0f64)]),
        // "19".to_owned());
        // tree.insert(BoundingBox::new_from(&[(20.0f64, 25.0f64), (20.0f64, 25.0f64)]),
        // "20".to_owned());
        //
        // println!("Size: {}", tree.len());
        //
        // for node in tree.iter() {
        // println!("All nodes: {:?}", node);
        // }
        //
        // for node in tree.query(BoundingBox::new_from(&[(0.0f64, 10.1f64), (0.0f64, 10.1f64)])) {
        // println!("10 box nodes: {:?}", node);
        // }
        //
        // for node in tree.query_mut(BoundingBox::new_from(&[(0.0f64, 10.1f64), (0.0f64, 10.1f64)])) {
        // node.1 = "newnew".to_owned();
        // }
        //
        // for node in tree.query(BoundingBox::new_from(&[(0.0f64, 10.1f64), (0.0f64, 10.1f64)])) {
        // println!("10 box nodes: {:?}", node);
        // }
        //
        let test: GenericArray<u32, U3> = arr![u32; 1, 2, 3];
        assert_eq!(test[1], 2);
    }
}
