// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Zero, One, Signed, Float, Bounded, ToPrimitive, FromPrimitive, pow};
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
    where P: Float
    + Signed
    + Bounded
    + MulAssign
    + AddAssign
    + ToPrimitive
    + FromPrimitive
    + Copy
    + Debug
    + Default,
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

// TODO: I'm not sure if I like this
    /// New Rect from corners
    pub fn from_corners(x: &Point<P, DIM>, y: &Point<P, DIM>) -> Rect<P, DIM>{
        let mut edges = Rect::max_inverted();
        x.expand_rect_to_fit(&mut edges);
        y.expand_rect_to_fit(&mut edges);
        edges
    }

    // An inverted Rect where ever dimension's (x, y) coordinates are (MAX, MIN). Simplifies finding boundaries.
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
///A convenience enum that contains `Point`, `LineSegment`, and `Rect`
#[derive(Debug, Clone)]
pub enum Shapes<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    Point(Point<P, DIM>),
    LineSegment(LineSegment<P, DIM>),
    Rect(Rect<P, DIM>), // Other(Box<Shape<P>>)
}

/// The minimum functionality required to insert a shape into an RTree
/// Until the rust compiler allows compile-time generic integers, the generics here will be kinda painful
/// The parameter 'edges' represents the expanse of the rectangle in that dimension
/// A rectangle whose corners are at (x1, y1), (x2, y2) will have the corresponding edges: (x1, x2), (y1, y2)
///
/// All operations should assume self.dim() == edges.len()

pub trait Shape<P, DIM: ArrayLength<P> + ArrayLength<(P, P)>> {
    /// The shape's dimension count
    fn dim(&self) -> usize;

    /// Determine the area of the shape
    fn area(&self) -> P;

    /// the minimum extent for a given axis
    fn min_for_axis(&self, dim: usize) -> P;

    /// the maximum extent for a given axis
    fn max_for_axis(&self, dim: usize) -> P;

    /// Expand the rectangle to minimally fit the shape
    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>);

    /// Determine the distance from the rectangle's center
    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P;

    /// Determine if the shape is completely contained in the rectangle
    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool;

    /// Determine if the shape overlaps the rectangle
    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool;

    /// Determines the area shared with the rectangle
    /// In cases where the shape and rect overlap, but the shape has no area (point or a line, for example), return 0
    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P;
}

impl<P, DIM> Shape<P, DIM> for Point<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
    DIM: ArrayLength<P> + ArrayLength<(P,P)>
{
    fn dim(&self) -> usize {
        self.coords.len()
    }

    fn area(&self) -> P {
        Zero::zero()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        *self.coords.get(dim).unwrap()
    }

    fn max_for_axis(&self, dim: usize) -> P {
        *self.coords.get(dim).unwrap()
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        for (&mut(ref mut x, ref mut y), &z) in izip!(edges.deref_mut(), self.deref()){
            *x = x.min(z);
            *y = y.max(z);
        }
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(edges.deref(), self.deref())
            .fold(Zero::zero(),
                |distance, (&(x, y), &z)| distance + pow((((x + y)/two) - z), 2));
        dist.sqrt()
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.overlapped_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        for (&(x, y), &z) in izip!(edges.deref(), self.deref()){
            if z < x ||  y < z {
                return false;
            }
        }
        true
    }

    #[allow(unused_variables)]
    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        Zero::zero()
    }
}

impl<P, DIM> Shape<P, DIM> for LineSegment<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
    DIM: ArrayLength<P> + ArrayLength<(P,P)>
{

    fn dim(&self) -> usize {
        self.x.dim()
    }
    fn area(&self) -> P {
        Zero::zero()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.x.coords.get(dim).unwrap().min(*self.y.coords.get(dim).unwrap())
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.x.coords.get(dim).unwrap().max(*self.y.coords.get(dim).unwrap())
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        self.x.expand_rect_to_fit(edges);
        self.y.expand_rect_to_fit(edges);
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(edges.deref(), self.x.deref(), self.y.deref())
            .fold(Zero::zero(),
                |distance, (&(x1, y1), &x2, &y2)| distance + pow(((x1 + y1)/two - (x2 + y2)/two), 2));
        dist.sqrt()
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.x.contained_by_rect(edges) && self.y.contained_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.x.overlapped_by_rect(edges) || self.y.overlapped_by_rect(edges)
    }

    #[allow(unused_variables)]
    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        Zero::zero()
    }
}

impl<P, DIM> Shape<P, DIM> for Rect<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>
{

    fn dim(&self) -> usize {
        self.edges.len()
    }

    fn area(&self) -> P {
        self.deref()
            .iter()
            .fold(One::one(), |area, &(x, y)| area * (y - x))
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.edges.get(dim).unwrap().0
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.edges.get(dim).unwrap().1
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        for (&mut (ref mut x1, ref mut y1), &(x2, y2)) in izip!(edges.deref_mut(), self.deref()) {
            *x1 = x1.min(x2);
            *y1 = y1.max(y2);
        }
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(edges.deref(), self.deref())
            .fold(Zero::zero(), |distance, (&(x1, y1), &(x2, y2))| {
                distance + pow(((x1 + y1) / two - (x2 + y2) / two), 2)
            });
        dist.sqrt()
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        for (&(x1, y1), &(x2, y2)) in izip!(edges.deref(), self.deref()) {
            if x2 < x1 || y1 < y2 {
                return false;
            }
        }
        true
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        for (&(x1, y1), &(x2, y2)) in izip!(edges.deref(), self.deref()) {
            if !(x1 < y2) || !(x2 < y1) {
                return false;
            }
        }
        true
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        izip!(edges.deref(), self.deref()).fold(One::one(), |area, (&(x1, y1), &(x2, y2))| {
            area * (y1.min(y2) - x1.max(x2)).max(Zero::zero())
        })
    }

}

impl<P, DIM> Shape<P, DIM> for Shapes<P, DIM>
where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
    DIM: ArrayLength<P> + ArrayLength<(P,P)>
{

    fn dim(&self) -> usize {
        match self {
            &Shapes::Point(ref point) => point.dim(),
            &Shapes::LineSegment(ref linesegment) => linesegment.dim(),
            &Shapes::Rect(ref rect) => rect.dim()
        }
    }

    fn area(&self) -> P {
        match self {
            &Shapes::Point(ref point) => point.area(),
            &Shapes::LineSegment(ref linesegment) => linesegment.area(),
            &Shapes::Rect(ref rect) => rect.area()
        }
    }

    fn min_for_axis(&self, dim: usize) -> P {
        match self {
            &Shapes::Point(ref point) => point.min_for_axis(dim),
            &Shapes::LineSegment(ref linesegment) => linesegment.min_for_axis(dim),
            &Shapes::Rect(ref rect) => rect.min_for_axis(dim)
        }
    }

    fn max_for_axis(&self, dim: usize) -> P {
        match self {
            &Shapes::Point(ref point) => point.max_for_axis(dim),
            &Shapes::LineSegment(ref linesegment) => linesegment.max_for_axis(dim),
            &Shapes::Rect(ref rect) => rect.max_for_axis(dim)
        }
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        match self {
            &Shapes::Point(ref point) => point.expand_rect_to_fit(edges),
            &Shapes::LineSegment(ref linesegment) => linesegment.expand_rect_to_fit(edges),
            &Shapes::Rect(ref rect) => rect.expand_rect_to_fit(edges)
        }
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        match self {
            &Shapes::Point(ref point) => point.distance_from_rect_center(edges),
            &Shapes::LineSegment(ref linesegment) => linesegment.distance_from_rect_center(edges),
            &Shapes::Rect(ref rect) => rect.distance_from_rect_center(edges)
        }
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        match self {
            &Shapes::Point(ref point) => point.contained_by_rect(edges),
            &Shapes::LineSegment(ref linesegment) => linesegment.contained_by_rect(edges),
            &Shapes::Rect(ref rect) => rect.contained_by_rect(edges)
        }
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        match self {
            &Shapes::Point(ref point) => point.overlapped_by_rect(edges),
            &Shapes::LineSegment(ref linesegment) => linesegment.overlapped_by_rect(edges),
            &Shapes::Rect(ref rect) => rect.overlapped_by_rect(edges)
        }
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        match self {
            &Shapes::Point(ref point) => point.area_overlapped_with_rect(edges),
            &Shapes::LineSegment(ref linesegment) => linesegment.area_overlapped_with_rect(edges),
            &Shapes::Rect(ref rect) => rect.area_overlapped_with_rect(edges)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use super::*;
    use typenum::consts::U3;
    use generic_array::GenericArray;

    const ONE: [f64; 3] = [1.0f64, 1.0f64, 1.0f64];
    const ZERO: [f64; 3] = [0.0f64, 0.0f64, 0.0f64];
    const NEG_ONE: [f64; 3] = [-1.0f64, -1.0f64, -1.0f64];
    const NEG_TWO: [f64; 3] = [-2.0f64, -2.0f64, -2.0f64];

    // distance of [0.5, 0.5, 0.5]
    const EXPECTED_DISTANCE: f64 = 0.86602540378f64;

    #[test]
    fn point() {
        let point: Point<f64, U3> = Point::new(GenericArray::new());
        for i in point.deref() {
            assert_relative_eq!(0.0f64, i);
        }

        let zero: Shapes<f64, U3> = Shapes::Point(Point::from_slice(&ZERO));
        let one: Shapes<f64, U3> = Shapes::Point(Point::from_slice(&ONE));
        let neg_one: Shapes<f64, U3> = Shapes::Point(Point::from_slice(&NEG_ONE));

        // Shape tests
        // dim
        assert_eq!(ZERO.len(), zero.dim());
        // area
        assert_relative_eq!(0.0f64, zero.area());
        // min/max for axis
        for (i, item) in ZERO.iter().enumerate() {
            assert_relative_eq!(*item, zero.min_for_axis(i));
            assert_relative_eq!(*item, zero.max_for_axis(i));
        }
        let mut bounding_rect = Rect::max_inverted();
        // expand_rect_to_fit
        zero.expand_rect_to_fit(&mut bounding_rect);
        one.expand_rect_to_fit(&mut bounding_rect);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_rect.min_for_axis(i));
            assert_relative_eq!(*y, bounding_rect.max_for_axis(i));
        }

        // distance_from_rect_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            zero.distance_from_rect_center(&bounding_rect),
                            max_relative = 0.00000001);

        // contained_by_rect
        assert!(zero.contained_by_rect(&bounding_rect));
        assert!(one.contained_by_rect(&bounding_rect));
        assert!(!neg_one.contained_by_rect(&bounding_rect));

        // overlapped_by_rect
        assert!(zero.overlapped_by_rect(&bounding_rect));
        assert!(one.overlapped_by_rect(&bounding_rect));
        assert!(!neg_one.overlapped_by_rect(&bounding_rect));

        // area_overlapped_with_rect
        assert_relative_eq!(0.0f64, zero.area_overlapped_with_rect(&bounding_rect));
    }

    #[test]
    fn line_segment() {
        // contained
        let zero_one: Shapes<f64, U3> = Shapes::LineSegment(LineSegment::from_slices(&ZERO, &ONE));
        // overlap
        let neg_one_one: Shapes<f64, U3> = Shapes::LineSegment(LineSegment::from_slices(&NEG_ONE,
                                                                                        &ONE));
        // outside
        let neg_two_neg_one: Shapes<f64, U3> =
            Shapes::LineSegment(LineSegment::from_slices(&NEG_TWO, &NEG_ONE));

        // Shape tests
        // dim
        assert_eq!(ZERO.len(), zero_one.dim());

        // area
        assert_relative_eq!(0.0f64, zero_one.area());

        // min/max for axis
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, zero_one.min_for_axis(i));
            assert_relative_eq!(*y, zero_one.max_for_axis(i));
        }

        let mut bounding_rect = Rect::max_inverted();

        // expand_rect_to_fit
        zero_one.expand_rect_to_fit(&mut bounding_rect);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_rect.min_for_axis(i));
            assert_relative_eq!(*y, bounding_rect.max_for_axis(i));
        }

        // distance_from_rect_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            neg_one_one.distance_from_rect_center(&bounding_rect),
                            max_relative = 0.00000001);

        // contained_by_rect
        assert!(zero_one.contained_by_rect(&bounding_rect));
        assert!(!neg_one_one.contained_by_rect(&bounding_rect));
        assert!(!neg_two_neg_one.contained_by_rect(&bounding_rect));

        // overlapped_by_rect
        assert!(zero_one.overlapped_by_rect(&bounding_rect));
        assert!(neg_one_one.overlapped_by_rect(&bounding_rect));
        assert!(!neg_two_neg_one.overlapped_by_rect(&bounding_rect));

        // area_overlapped_with_rect
        assert_relative_eq!(0.0f64, zero_one.area_overlapped_with_rect(&bounding_rect));
    }

    #[test]
    fn rect() {
        let one: Point<f64, U3> = Point::from_slice(&ONE);
        let zero: Point<f64, U3> = Point::from_slice(&ZERO);
        let neg_one: Point<f64, U3> = Point::from_slice(&NEG_ONE);
        let neg_two: Point<f64, U3> = Point::from_slice(&NEG_TWO);

        // contained
        let zero_one = Rect::from_corners(&zero, &one);
        // overlapped
        let neg_one_one = Rect::from_corners(&neg_one, &one);
        // outside
        let neg_two_neg_one = Rect::from_corners(&neg_two, &neg_one);

        // Shape tests
        // dim
        assert_eq!(one.dim(), zero_one.dim());

        // area
        assert_relative_eq!(1.0f64, zero_one.area());

        // min/max for axis
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, zero_one.min_for_axis(i));
            assert_relative_eq!(*y, zero_one.max_for_axis(i));
        }

        let mut bounding_rect = Rect::max_inverted();
        // expand_rect_to_fit
        zero_one.expand_rect_to_fit(&mut bounding_rect);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_rect.min_for_axis(i));
            assert_relative_eq!(*y, bounding_rect.max_for_axis(i));
        }

        // distance_from_rect_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            neg_one_one.distance_from_rect_center(&bounding_rect),
                            max_relative = 0.00000001);

        // contained_by_rect
        assert!(zero_one.contained_by_rect(&bounding_rect));
        assert!(!neg_one_one.contained_by_rect(&bounding_rect));
        assert!(!neg_two_neg_one.contained_by_rect(&bounding_rect));

        // overlapped_by_rect
        assert!(zero_one.overlapped_by_rect(&bounding_rect));
        assert!(neg_one_one.overlapped_by_rect(&bounding_rect));
        assert!(!neg_two_neg_one.overlapped_by_rect(&bounding_rect));

        // area_overlapped_with_rect
        assert_relative_eq!(1.0f64, zero_one.area_overlapped_with_rect(&bounding_rect));
        assert_relative_eq!(1.0f64,
                            neg_one_one.area_overlapped_with_rect(&bounding_rect));

        // margin
        assert_relative_eq!(3.0f64, zero_one.margin());
    }
}
