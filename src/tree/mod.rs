// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

pub mod mbr;

/// A tree leaf
#[derive(Debug)]
pub struct Leaf<P, DIM, SHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub shape: SHAPE,
    pub item: T,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
}

impl<P, DIM, SHAPE, T> Leaf<P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          SHAPE: Shape<P, DIM>
{
    /// New leaf from shape and item
    pub fn new(shape: SHAPE, item: T) -> Leaf<P, DIM, SHAPE, T> {
        Leaf{shape: shape, item: item, _p: PhantomData, _dim: PhantomData}
    }

    /// Consumes self, returning the shape and item
    pub fn extract(self) -> (SHAPE, T) {
        (self.shape, self.item)
    }
}

impl<P, DIM, SHAPE, T> Shape<P, DIM> for Leaf<P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          SHAPE: Shape<P, DIM> {

    fn dim(&self) -> usize {
        self.shape.dim()
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        self.shape.expand_rect_to_fit(edges)
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        self.shape.distance_from_rect_center(edges)
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.shape.contained_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.shape.overlapped_by_rect(edges)
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        self.shape.area_overlapped_with_rect(edges)
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
pub trait SpatialMapQuery<P, DIM, SHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)> 
{
    /// Returns true if the leaf matches the query
    fn accept_leaf(&self, leaf: &Leaf<P, DIM, SHAPE, T>) -> bool;
    /// Returns true if the level matches the query
    fn accept_level(&self, level: &MbrNode<P, DIM, SHAPE, T>) -> bool;
}