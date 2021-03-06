// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Various geometric shapes to insert into spatial trees

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::convert::{AsRef, AsMut};
use generic_array::{ArrayLength, GenericArray};

/// An n-dimensional point
#[derive(Debug, Clone)]
pub struct Point<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub coords: GenericArray<P, DIM>,
}

impl<P, DIM> Point<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
    DIM: ArrayLength<P> + ArrayLength<(P,P)>
{
/// New Point from a `GenericArray`
    pub fn new(coords: GenericArray<P, DIM>) -> Point<P, DIM> {
        for coord in coords.deref() {
            assert!(coord.is_finite(), "{:?} should be finite", coord);
        }
        Point{coords: coords}
    }
/// New Point from a slice
    pub fn from_slice(slice: &[P]) -> Point<P, DIM> {
        Point::new(GenericArray::from_slice(slice))
    }
}

impl<P, DIM> Deref for Point<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    type Target = [P];

    fn deref(&self) -> &[P] {
        self.coords.deref()
    }
}

impl<P, DIM> DerefMut for Point<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn deref_mut(&mut self) -> &mut [P] {
        self.coords.deref_mut()
    }
}

impl<P, DIM> AsRef<[P]> for Point<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn as_ref(&self) -> &[P] {
        self.deref()
    }
}

impl<P, DIM> AsMut<[P]> for Point<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn as_mut(&mut self) -> &mut [P] {
        self.deref_mut()
    }
}

/// An n-dimensional line segment
#[derive(Debug, Clone)]
pub struct LineSegment<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    // TODO: Would this be better as [(P,P)]?
    pub x: Point<P, DIM>,
    pub y: Point<P, DIM>,
}

impl<P, DIM> LineSegment<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
    DIM: ArrayLength<P> + ArrayLength<(P,P)> {

/// New LineSegment from two GenericArrays representing either end
    pub fn new(x: GenericArray<P, DIM>, y: GenericArray<P, DIM>) -> LineSegment<P, DIM> {
        LineSegment{x: Point::new(x), y: Point::new(y)}
    }
/// New LineSegment from two slices representing either end
    pub fn from_slices(x: &[P], y: &[P]) -> LineSegment<P, DIM> {
        LineSegment{x: Point::from_slice(x), y: Point::from_slice(y)}
    }
}

/// An n-dimensional rectangle
#[derive(Debug, Clone)]
pub struct Rect<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub edges: GenericArray<(P, P), DIM>,
}

impl<P, DIM> Rect<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
    DIM: ArrayLength<P> + ArrayLength<(P,P)> {

/// New Rect from a `GenericArray`
    pub fn new(mut edges: GenericArray<(P,P), DIM>) -> Rect<P,DIM> {
// ensure that the edge coordinates are valid and ordered correctly
        for &mut (ref mut x, ref mut y) in edges.deref_mut() {
            assert!(x.is_finite(), "{:?} should be finite", x);
            assert!(y.is_finite(), "{:?} should be finite", y);
            *x = x.min(*y);
            *y = x.max(*y);
        }
        Rect{edges: edges}
    }

/// New Rect from corners
    pub fn from_corners(x: GenericArray<P, DIM>, y: GenericArray<P, DIM>) -> Rect<P, DIM>{
        use tree::mbr::MbrLeafGeometry;
        let mut edges = Rect::max_inverted();
        Point::new(x).expand_mbr_to_fit(&mut edges);
        Point::new(y).expand_mbr_to_fit(&mut edges);
        edges
    }

/// An inverted Rect where ever dimension's (x, y) coordinates are (MAX, MIN). Simplifies finding boundaries.
    pub fn max_inverted() -> Rect<P, DIM> {
        let mut edges = GenericArray::new();
        for &mut (ref mut x, ref mut y) in edges.as_mut() {
            *x = Bounded::max_value();
            *y = Bounded::min_value();
        }
        Rect{edges: edges}
    }

/// The largest possible rect
    pub fn max() -> Rect<P, DIM> {
        let mut edges = GenericArray::new();
        for &mut (ref mut x, ref mut y) in edges.as_mut() {
            *x = Bounded::min_value();
            *y = Bounded::max_value();
        }
        Rect{edges: edges}
    }
}

impl<P, DIM> Deref for Rect<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    type Target = [(P, P)];

    fn deref(&self) -> &[(P, P)] {
        self.edges.deref()
    }
}

impl<P, DIM> DerefMut for Rect<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn deref_mut(&mut self) -> &mut [(P, P)] {
        self.edges.deref_mut()
    }
}

impl<P, DIM> AsRef<[(P, P)]> for Rect<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn as_ref(&self) -> &[(P, P)] {
        self.deref()
    }
}

impl<P, DIM> AsMut<[(P, P)]> for Rect<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn as_mut(&mut self) -> &mut [(P, P)] {
        self.deref_mut()
    }
}

// When trying to use Other(Box<Shape<P>>)
// the trait bound `shapes::Shape<P>: std::marker::Sized` is not satisfied [E0277]
// the trait bound `shapes::Shape<P>: std::clone::Clone` is not satisfied [E0277]
// the trait bound `shapes::Shape<P> + 'static: std::fmt::Debug` is not satisfied [E0277]
//
/// A convenience enum that contains `Point`, `LineSegment`, and `Rect`
#[derive(Debug, Clone)]
pub enum Shapes<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    Point(Point<P, DIM>),
    LineSegment(LineSegment<P, DIM>),
    Rect(Rect<P, DIM>), // Other(Box<Shape<P>>)
}
