// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Zero, One, Signed, Float, Bounded, ToPrimitive, FromPrimitive, pow};
use std::ops::{MulAssign, AddAssign};
use geometry::{Shapes, Point, LineSegment, Rect};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use generic_array::ArrayLength;

/// The minimum functionality required to insert leaf geometry into `MbrMap`
/// Until the rust compiler allows compile-time generic integers, we'll be using generic_array's `ArrayLength` to specify
/// geometry dimensions at compile time.
///
/// The parameter `mbr` represents a minimum bounding rectangle.
/// An mbr whose corners are at (x1, y1), (x2, y2) will have the corresponding edges: (x1, x2), (y1, y2)
pub trait MbrLeafGeometry<P, DIM: ArrayLength<P> + ArrayLength<(P, P)>> {
    /// The geometry's dimension count
    fn dim(&self) -> usize;

    /// Determine the area of the geometry
    fn area(&self) -> P;

    /// the minimum extent for a given axis
    fn min_for_axis(&self, dim: usize) -> P;

    /// the maximum extent for a given axis
    fn max_for_axis(&self, dim: usize) -> P;

    /// Expand the mbr to minimally fit the leaf
    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>);

    /// Determine the distance from the mbr's center
    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P;

    /// Determine if the leaf is completely contained in the mbr
    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool;

    /// Determine if the leaf overlaps the mbr
    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool;

    /// Determines the leaf area shared with the rectangle.
    /// In cases where the leaf and mbr overlap, but the leaf has no area (point or a line, for example), return 0
    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P;
}

impl<P, DIM> MbrLeafGeometry<P, DIM> for Point<P, DIM>
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

    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>) {
        for (&mut(ref mut x, ref mut y), &z) in izip!(mbr.deref_mut(), self.deref()){
            *x = x.min(z);
            *y = y.max(z);
        }
    }

    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(mbr.deref(), self.deref())
            .fold(Zero::zero(),
                |distance, (&(x, y), &z)| distance + pow((((x + y)/two) - z), 2));
        dist.sqrt()
    }

    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        self.overlapped_by_mbr(mbr)
    }

    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        for (&(x, y), &z) in izip!(mbr.deref(), self.deref()){
            if z < x ||  y < z {
                return false;
            }
        }
        true
    }

    #[allow(unused_variables)]
    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P {
        Zero::zero()
    }
}

impl<P, DIM> MbrLeafGeometry<P, DIM> for LineSegment<P, DIM>
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

    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>) {
        self.x.expand_mbr_to_fit(mbr);
        self.y.expand_mbr_to_fit(mbr);
    }

    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(mbr.deref(), self.x.deref(), self.y.deref())
            .fold(Zero::zero(),
                |distance, (&(x1, y1), &x2, &y2)| distance + pow(((x1 + y1)/two - (x2 + y2)/two), 2));
        dist.sqrt()
    }

    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        self.x.contained_by_mbr(mbr) && self.y.contained_by_mbr(mbr)
    }

    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        self.x.overlapped_by_mbr(mbr) || self.y.overlapped_by_mbr(mbr)
    }

    #[allow(unused_variables)]
    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P {
        Zero::zero()
    }
}

impl<P, DIM> MbrLeafGeometry<P, DIM> for Rect<P, DIM>
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

    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>) {
        for (&mut (ref mut x1, ref mut y1), &(x2, y2)) in izip!(mbr.deref_mut(), self.deref()) {
            *x1 = x1.min(x2);
            *y1 = y1.max(y2);
        }
    }

    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P {
        let two = FromPrimitive::from_usize(2).unwrap();
        let dist: P = izip!(mbr.deref(), self.deref())
            .fold(Zero::zero(), |distance, (&(x1, y1), &(x2, y2))| {
                distance + pow(((x1 + y1) / two - (x2 + y2) / two), 2)
            });
        dist.sqrt()
    }

    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        for (&(x1, y1), &(x2, y2)) in izip!(mbr.deref(), self.deref()) {
            if x2 < x1 || y1 < y2 {
                return false;
            }
        }
        true
    }

    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        for (&(x1, y1), &(x2, y2)) in izip!(mbr.deref(), self.deref()) {
            if !(x1 < y2) || !(x2 < y1) {
                return false;
            }
        }
        true
    }

    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P {
        izip!(mbr.deref(), self.deref()).fold(One::one(), |area, (&(x1, y1), &(x2, y2))| {
            area * (y1.min(y2) - x1.max(x2)).max(Zero::zero())
        })
    }

}

impl<P, DIM> MbrLeafGeometry<P, DIM> for Shapes<P, DIM>
where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
    DIM: ArrayLength<P> + ArrayLength<(P,P)>
{

    fn dim(&self) -> usize {
        match *self {
            Shapes::Point(ref point) => point.dim(),
            Shapes::LineSegment(ref linesegment) => linesegment.dim(),
            Shapes::Rect(ref rect) => rect.dim()
        }
    }

    fn area(&self) -> P {
        match *self {
            Shapes::Point(ref point) => point.area(),
            Shapes::LineSegment(ref linesegment) => linesegment.area(),
            Shapes::Rect(ref rect) => rect.area()
        }
    }

    fn min_for_axis(&self, dim: usize) -> P {
        match *self {
            Shapes::Point(ref point) => point.min_for_axis(dim),
            Shapes::LineSegment(ref linesegment) => linesegment.min_for_axis(dim),
            Shapes::Rect(ref rect) => rect.min_for_axis(dim)
        }
    }

    fn max_for_axis(&self, dim: usize) -> P {
        match *self {
            Shapes::Point(ref point) => point.max_for_axis(dim),
            Shapes::LineSegment(ref linesegment) => linesegment.max_for_axis(dim),
            Shapes::Rect(ref rect) => rect.max_for_axis(dim)
        }
    }

    fn expand_mbr_to_fit(&self, mbr: &mut Rect<P, DIM>) {
        match *self {
            Shapes::Point(ref point) => point.expand_mbr_to_fit(mbr),
            Shapes::LineSegment(ref linesegment) => linesegment.expand_mbr_to_fit(mbr),
            Shapes::Rect(ref rect) => rect.expand_mbr_to_fit(mbr)
        }
    }

    fn distance_from_mbr_center(&self, mbr: &Rect<P, DIM>) -> P {
        match *self {
            Shapes::Point(ref point) => point.distance_from_mbr_center(mbr),
            Shapes::LineSegment(ref linesegment) => linesegment.distance_from_mbr_center(mbr),
            Shapes::Rect(ref rect) => rect.distance_from_mbr_center(mbr)
        }
    }

    fn contained_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        match *self {
            Shapes::Point(ref point) => point.contained_by_mbr(mbr),
            Shapes::LineSegment(ref linesegment) => linesegment.contained_by_mbr(mbr),
            Shapes::Rect(ref rect) => rect.contained_by_mbr(mbr)
        }
    }

    fn overlapped_by_mbr(&self, mbr: &Rect<P, DIM>) -> bool {
        match *self {
            Shapes::Point(ref point) => point.overlapped_by_mbr(mbr),
            Shapes::LineSegment(ref linesegment) => linesegment.overlapped_by_mbr(mbr),
            Shapes::Rect(ref rect) => rect.overlapped_by_mbr(mbr)
        }
    }

    fn area_overlapped_with_mbr(&self, mbr: &Rect<P, DIM>) -> P {
        match *self {
            Shapes::Point(ref point) => point.area_overlapped_with_mbr(mbr),
            Shapes::LineSegment(ref linesegment) => linesegment.area_overlapped_with_mbr(mbr),
            Shapes::Rect(ref rect) => rect.area_overlapped_with_mbr(mbr)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use typenum::consts::U3;
    use geometry::{Shapes, Point, LineSegment, Rect};
    use generic_array::GenericArray;
    use super::*;

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
        let mut bounding_mbr = Rect::max_inverted();
        // expand_mbr_to_fit
        zero.expand_mbr_to_fit(&mut bounding_mbr);
        one.expand_mbr_to_fit(&mut bounding_mbr);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_mbr.min_for_axis(i));
            assert_relative_eq!(*y, bounding_mbr.max_for_axis(i));
        }

        // distance_from_mbr_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            zero.distance_from_mbr_center(&bounding_mbr),
                            max_relative = 0.00000001);

        // contained_by_mbr
        assert!(zero.contained_by_mbr(&bounding_mbr));
        assert!(one.contained_by_mbr(&bounding_mbr));
        assert!(!neg_one.contained_by_mbr(&bounding_mbr));

        // overlapped_by_mbr
        assert!(zero.overlapped_by_mbr(&bounding_mbr));
        assert!(one.overlapped_by_mbr(&bounding_mbr));
        assert!(!neg_one.overlapped_by_mbr(&bounding_mbr));

        // area_overlapped_with_mbr
        assert_relative_eq!(0.0f64, zero.area_overlapped_with_mbr(&bounding_mbr));
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

        let mut bounding_mbr = Rect::max_inverted();

        // expand_mbr_to_fit
        zero_one.expand_mbr_to_fit(&mut bounding_mbr);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_mbr.min_for_axis(i));
            assert_relative_eq!(*y, bounding_mbr.max_for_axis(i));
        }

        // distance_from_mbr_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            neg_one_one.distance_from_mbr_center(&bounding_mbr),
                            max_relative = 0.00000001);

        // contained_by_mbr
        assert!(zero_one.contained_by_mbr(&bounding_mbr));
        assert!(!neg_one_one.contained_by_mbr(&bounding_mbr));
        assert!(!neg_two_neg_one.contained_by_mbr(&bounding_mbr));

        // overlapped_by_mbr
        assert!(zero_one.overlapped_by_mbr(&bounding_mbr));
        assert!(neg_one_one.overlapped_by_mbr(&bounding_mbr));
        assert!(!neg_two_neg_one.overlapped_by_mbr(&bounding_mbr));

        // area_overlapped_with_mbr
        assert_relative_eq!(0.0f64, zero_one.area_overlapped_with_mbr(&bounding_mbr));
    }

    #[test]
    fn rect() {

        let g_one: GenericArray<f64, U3> = arr![f64; 1.0f64, 1.0f64, 1.0f64];
        let g_zero: GenericArray<f64, U3> = arr![f64; 0.0f64, 0.0f64, 0.0f64];
        let g_neg_one: GenericArray<f64, U3> = arr![f64; -1.0f64, -1.0f64, -1.0f64];
        let g_neg_two: GenericArray<f64, U3> = arr![f64; -2.0f64, -2.0f64, -2.0f64];

        // contained
        let zero_one = Rect::from_corners(g_zero.clone(), g_one.clone());
        // overlapped
        let neg_one_one = Rect::from_corners(g_neg_one.clone(), g_one.clone());
        // outside
        let neg_two_neg_one = Rect::from_corners(g_neg_two.clone(), g_neg_one.clone());

        // Shape tests
        // dim
        assert_eq!(zero_one.len(), zero_one.dim());

        // area
        assert_relative_eq!(1.0f64, zero_one.area());

        // min/max for axis
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, zero_one.min_for_axis(i));
            assert_relative_eq!(*y, zero_one.max_for_axis(i));
        }

        let mut bounding_mbr = Rect::max_inverted();
        // expand_mbr_to_fit
        zero_one.expand_mbr_to_fit(&mut bounding_mbr);
        for (i, (x, y)) in izip!(&ZERO, &ONE).enumerate() {
            assert_relative_eq!(*x, bounding_mbr.min_for_axis(i));
            assert_relative_eq!(*y, bounding_mbr.max_for_axis(i));
        }

        // distance_from_mbr_center
        assert_relative_eq!(EXPECTED_DISTANCE,
                            neg_one_one.distance_from_mbr_center(&bounding_mbr),
                            max_relative = 0.00000001);

        // contained_by_mbr
        assert!(zero_one.contained_by_mbr(&bounding_mbr));
        assert!(!neg_one_one.contained_by_mbr(&bounding_mbr));
        assert!(!neg_two_neg_one.contained_by_mbr(&bounding_mbr));

        // overlapped_by_mbr
        assert!(zero_one.overlapped_by_mbr(&bounding_mbr));
        assert!(neg_one_one.overlapped_by_mbr(&bounding_mbr));
        assert!(!neg_two_neg_one.overlapped_by_mbr(&bounding_mbr));

        // area_overlapped_with_mbr
        assert_relative_eq!(1.0f64, zero_one.area_overlapped_with_mbr(&bounding_mbr));
        assert_relative_eq!(1.0f64, neg_one_one.area_overlapped_with_mbr(&bounding_mbr));
    }
}
