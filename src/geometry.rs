// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Various geometric shapes to insert into spatial trees

use num::{Bounded, Float};
use std::convert::{AsMut, AsRef, TryInto};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use FP;

/// An n-dimensional point
#[derive(Debug, Clone)]
pub struct Point<P, const DIM: usize> {
    pub coords: [P; DIM],
}

impl<P: FP, const DIM: usize> Point<P, DIM> {
    /// New Point from a `GenericArray`
    pub fn new(coords: [P; DIM]) -> Point<P, DIM> {
        for coord in &coords {
            assert!(Float::is_finite(*coord), "{:?} should be finite", coord);
        }
        Point { coords }
    }
    /// New Point from a slice
    pub fn from_slice(slice: &[P]) -> Point<P, DIM> {
        Point::new(slice.try_into().unwrap())
    }
}

impl<P: FP, const DIM: usize> Deref for Point<P, DIM> {
    type Target = [P];

    fn deref(&self) -> &[P] {
        &self.coords
    }
}

impl<P: FP, const DIM: usize> DerefMut for Point<P, DIM> {
    fn deref_mut(&mut self) -> &mut [P] {
        &mut self.coords
    }
}

impl<P: FP, const DIM: usize> AsRef<[P]> for Point<P, DIM> {
    fn as_ref(&self) -> &[P] {
        &self.coords
    }
}

impl<P: FP, const DIM: usize> AsMut<[P]> for Point<P, DIM> {
    fn as_mut(&mut self) -> &mut [P] {
        &mut self.coords
    }
}

/// An n-dimensional line segment
#[derive(Debug, Clone)]
pub struct LineSegment<P: FP, const DIM: usize> {
    // TODO: Would this be better as [(P,P)]?
    pub x: Point<P, DIM>,
    pub y: Point<P, DIM>,
}

impl<P: FP, const DIM: usize> LineSegment<P, DIM> {
    /// New LineSegment from two GenericArrays representing either end
    pub fn new(x: [P; DIM], y: [P; DIM]) -> LineSegment<P, DIM> {
        LineSegment {
            x: Point::new(x),
            y: Point::new(y),
        }
    }
    /// New LineSegment from two slices representing either end
    pub fn from_slices(x: &[P], y: &[P]) -> LineSegment<P, DIM> {
        LineSegment {
            x: Point::from_slice(x),
            y: Point::from_slice(y),
        }
    }
}

/// An n-dimensional rectangle
#[derive(Debug, Clone)]
pub struct Rect<P: FP, const DIM: usize> {
    pub edges: [(P, P); DIM],
}

impl<P: FP, const DIM: usize> Rect<P, DIM> {
    /// New Rect from a `GenericArray`
    pub fn new(mut edges: [(P, P); DIM]) -> Rect<P, DIM> {
        // ensure that the edge coordinates are valid and ordered correctly
        for &mut (ref mut x, ref mut y) in &mut edges {
            assert!(Float::is_finite(*x), "{:?} should be finite", x);
            assert!(Float::is_finite(*y), "{:?} should be finite", y);
            *x = Float::min(*x, *y);
            *y = Float::max(*x, *y);
        }
        Rect { edges }
    }

    /// New Rect from corners
    pub fn from_corners(x: [P; DIM], y: [P; DIM]) -> Rect<P, DIM> {
        use tree::mbr::MbrLeafGeometry;
        let mut edges = Rect::max_inverted();
        Point::new(x).expand_mbr_to_fit(&mut edges);
        Point::new(y).expand_mbr_to_fit(&mut edges);
        edges
    }

    /// An inverted Rect where ever dimension's (x, y) coordinates are (MAX, MIN). Simplifies finding boundaries.
    pub fn max_inverted() -> Rect<P, DIM> {
        let edges = [(Bounded::max_value(), Bounded::min_value()); DIM];
        Rect { edges }
    }

    /// The largest possible rect
    pub fn max() -> Rect<P, DIM> {
        let edges = [(Bounded::min_value(), Bounded::max_value()); DIM];
        Rect { edges }
    }
}

impl<P: FP, const DIM: usize> Deref for Rect<P, DIM> {
    type Target = [(P, P)];

    fn deref(&self) -> &[(P, P)] {
        &self.edges
    }
}

impl<P: FP, const DIM: usize> DerefMut for Rect<P, DIM> {
    fn deref_mut(&mut self) -> &mut [(P, P)] {
        &mut self.edges
    }
}

impl<P: FP, const DIM: usize> AsRef<[(P, P)]> for Rect<P, DIM> {
    fn as_ref(&self) -> &[(P, P)] {
        self.deref()
    }
}

impl<P: FP, const DIM: usize> AsMut<[(P, P)]> for Rect<P, DIM> {
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
pub enum Shapes<P: FP, const DIM: usize> {
    Point(Point<P, DIM>),
    LineSegment(LineSegment<P, DIM>),
    Rect(Rect<P, DIM>), // Other(Box<Shape<P>>)
}
