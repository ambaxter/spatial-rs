// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Zero, Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign, Range, Deref};
use tree::mbr::MbrNode;
use tree::Leaf;
use tree::mbr::index::IndexInsert;
use shapes::{Shape, Rect};
use std::fmt::Debug;
use std::marker::PhantomData;
use ordered_float::NotNaN;
use std::cmp;
use typenum::Unsigned;
use generic_array::ArrayLength;

const AT_ROOT: bool = true;
const NOT_AT_ROOT: bool = false;
const FORCE_SPLIT: bool = true;
const DONT_FORCES_SPLIT: bool = false;

const D_REINSERT_P: f32 = 0.30f32;
const D_SPLIT_P: f32 = 0.40f32;
const D_CHOOSE_SUBTREE_P: usize = 32;

/// The sum of all of the Shapes's edges. Used in the R* algorithm
pub trait Margin<P> {
    fn margin(&self) -> P;
}

impl<P, DIM> Margin<P> for Rect<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
          DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn margin(&self) -> P {
        self.edges.deref().iter()
            .fold(Zero::zero(), | margin,  &(x, y)| margin + y - x)
    }
}

#[derive(Debug)]
#[must_use]
enum InsertResult<P, DIM, LSHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    Ok,
    Reinsert(Vec<Leaf<P, DIM, LSHAPE, T>>),
    Split(MbrNode<P, DIM, LSHAPE, T>),
}

impl<P, DIM, LSHAPE, T> InsertResult<P, DIM, LSHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn is_reinsert(&self) -> bool {
        match *self {
            InsertResult::Reinsert(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct RStarInsert<P, DIM, LSHAPE, MIN, MAX, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          MIN: Unsigned,
          MAX: Unsigned
{
    reinsert_m: usize,
    choose_subtree_p: usize,
    min_k: usize,
    max_k: usize,
    _dim: PhantomData<DIM>,
    _p: PhantomData<P>,
    _shape: PhantomData<LSHAPE>,
    _min: PhantomData<MIN>,
    _max: PhantomData<MAX>,
    _t: PhantomData<T>,
}


impl<P, DIM, LSHAPE, MIN, MAX, T> RStarInsert<P, DIM, LSHAPE, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LSHAPE: Shape<P, DIM>,
        MIN: Unsigned,
        MAX: Unsigned,
{

    pub fn new() -> RStarInsert<P, DIM, LSHAPE, MIN, MAX, T> {
        RStarInsert::new_with_options(D_REINSERT_P, D_SPLIT_P, D_CHOOSE_SUBTREE_P)
    }

    pub fn new_with_options(reinsert_p: f32, split_p: f32, choose_subtree_p: usize) -> RStarInsert<P, DIM, LSHAPE, MIN, MAX, T> {
        assert!(MIN::to_usize() > 0, "MIN({:?}) must be at least 1.", MIN::to_usize());
        assert!(MAX::to_usize() > MIN::to_usize(), "MAX({:?}) must be greater than MIN({:?})", MAX::to_usize(), MIN::to_usize());

        let reinsert_m = MAX::to_usize() - cmp::max((MAX::to_usize() as f32 * reinsert_p) as usize, 1);
        // TODO: What if min_k < MIN?
        let min_k = cmp::max((MAX::to_usize() as f32 * split_p) as usize, 1);
        let max_k = cmp::max(MAX::to_usize() - (2 * min_k) + 1, min_k + 1);
        // On max_k==min_k, the iterator in split is a no-op and vec splits occur at index 0. Happens with M - 2m + 1 and M - 2m + 2 for various small Ms.
        // The above line should prevent this, but assert in case code changes
        assert!(max_k > min_k, "max_k({:?}) must be greater than min_k({:?})", max_k, min_k);
        RStarInsert{
            reinsert_m: reinsert_m,
            choose_subtree_p: choose_subtree_p,
            min_k: min_k,
            max_k: max_k,
            _dim: PhantomData,
            _p: PhantomData,
            _shape: PhantomData,
            _min: PhantomData,
            _max: PhantomData,
            _t: PhantomData
        }
    }

    fn area_cost(&self, mbr: &Rect<P, DIM>, leaf: &Leaf<P, DIM, LSHAPE, T>) -> (NotNaN<P>, NotNaN<P>) {
        let mut expanded = mbr.clone();
        leaf.expand_rect_to_fit(&mut expanded);
        let mbr_area = mbr.area();
        let area_cost = expanded.area() - mbr_area;
        (NotNaN::from(area_cost), NotNaN::from(mbr_area))
    }

    fn overlap_cost(&self, mbr: &Rect<P, DIM>, leaf: &Leaf<P, DIM, LSHAPE, T>) -> NotNaN<P> {
        let overlap = leaf.area_overlapped_with_rect(mbr);
        let overlap_cost = leaf.area() - overlap;
        NotNaN::from(overlap_cost)
    }

    fn overlap_area_cost(&self, mbr: &Rect<P, DIM>, leaf: &Leaf<P, DIM, LSHAPE, T>) -> (NotNaN<P>, NotNaN<P>, NotNaN<P>) {
        let (area_cost, mbr_area) = self.area_cost(mbr, leaf);
        let overlap_cost = self.overlap_cost(mbr, leaf);
        (overlap_cost, area_cost, mbr_area)
    }

    // CS2 + optimizations
    fn choose_subnode<'tree>(&self, level: &'tree mut Vec<MbrNode<P, DIM, LSHAPE, T>>, leaf: &Leaf<P, DIM, LSHAPE, T>) -> &'tree mut MbrNode<P, DIM, LSHAPE, T> {
        if level.first().unwrap().has_leaves() {
            if level.len() > self.choose_subtree_p {
                level.sort_by_key(|a| self.area_cost(a.mbr(), leaf));
                let (left, _) = level.split_at_mut(self.choose_subtree_p);
                return left.iter_mut().min_by_key(|a| self.overlap_cost(a.mbr(), leaf)).unwrap();
            } else {
                return level.iter_mut().min_by_key(|a| self.overlap_area_cost(a.mbr(), leaf)).unwrap();
            }
        }
        level.iter_mut().min_by_key(|a| self.area_cost(a.mbr(), leaf)).unwrap()
    }

    fn split_for_reinsert(&self, mbr: &mut Rect<P, DIM>, children: &mut Vec<Leaf<P, DIM, LSHAPE, T>>) -> Vec<Leaf<P, DIM, LSHAPE, T>> {
        // RI1 & RI2
        children.sort_by_key(|a| NotNaN::from(a.distance_from_rect_center(mbr)));
        //RI3
        let split = children.split_off(self.reinsert_m);
        *mbr = Rect::max_inverted();
        for child in children {
            child.expand_rect_to_fit(mbr);
        }
        split
    }

    fn insert_into_level(&self, level: &mut MbrNode<P, DIM, LSHAPE, T>, leaf: Leaf<P, DIM, LSHAPE, T>, at_root: bool, force_split: bool) -> InsertResult<P, DIM, LSHAPE, T> {
        //I4
        leaf.shape.expand_rect_to_fit(level.mbr_mut());
        match *level {
            //I2
            MbrNode::Leaves{ref mut children, ..} => {
                children.push(leaf);
            },
            //I1
            MbrNode::Level{ref mut mbr, ref mut children} => {
                //CS3
                let insert_result = self.insert_into_level(self.choose_subnode(children, &leaf), leaf, NOT_AT_ROOT, force_split);
                //I3
                if let InsertResult::Split(child) = insert_result {
                    children.push(child);
                } else {
                    //I4
                    if insert_result.is_reinsert() {
                        *mbr = Rect::max_inverted();
                        for child in children {
                            child.mbr().expand_rect_to_fit(mbr);
                        }
                    }
                    return insert_result;
                }
            }
        }
        //I2 & I3
        if level.len() > MAX::to_usize() {
            return self.handle_overflow(level, at_root, force_split);
        }
        InsertResult::Ok
    }


    // fn best_position_for_axis -> (margin, (axis, edge, index))
    fn best_split_position_for_axis<V: Shape<P, DIM>>(&self, axis: usize, children: &mut Vec<V>) ->(P, (usize, usize, usize)) {
        let mut margin:P = Zero::zero();
        let mut d_area:P = Float::max_value();
        let mut d_overlap:P = Float::max_value();
        let mut d_edge: usize = 0;
        let mut d_index: usize = 0;

        for edge in 0..2 {
            if edge == 0 {
                children.sort_by_key(|child| NotNaN::from(child.min_for_axis(axis)));
            } else {
                children.sort_by_key(|child| NotNaN::from(child.max_for_axis(axis)));
            }

            for k in self.min_k..self.max_k {
                let mut r1 = Rect::max_inverted();
                let mut r2 = Rect::max_inverted();

                let (left, right) = children.split_at(k);
                for child in left {
                    child.expand_rect_to_fit(&mut r1);
                }
                for child in right {
                    child.expand_rect_to_fit(&mut r2);
                }

                // (I)
                let area = r1.area() + r2.area();
                // (II)
                margin += r1.margin() + r2.margin();
                // (III)
                let overlap = r1.area_overlapped_with_rect(&r2);

                // CSI1
                if (overlap, area) < (d_overlap, d_area) {
                    d_overlap = overlap;
                    d_area = area;
                    d_edge = edge;
                    d_index = k;
                }
            }
        }
        (margin, (axis, d_edge, d_index))
    }

    fn split<V: Shape<P, DIM>>(&self, mbr: &mut Rect<P, DIM>, children: &mut Vec<V>) -> (Rect<P, DIM>, Vec<V>) {
        // S1 & S2
        let (s_axis, s_edge, s_index) = Range{start: 0, end: DIM::to_usize()}
            // CSA1
            .map(|axis| self.best_split_position_for_axis(axis, children))
            // CSA2
            .min_by_key(|&(margin, _)| NotNaN::from(margin))
            .unwrap().1;

        if s_edge == 0 {
            children.sort_by_key(|child| NotNaN::from(child.min_for_axis(s_axis)));
        } else {
            children.sort_by_key(|child| NotNaN::from(child.max_for_axis(s_axis)));
        }
        // S3
        let split_children = children.split_off(s_index);
        *mbr = Rect::max_inverted();
        let mut split_mbr = Rect::max_inverted();
        for child in &*children {
            child.expand_rect_to_fit(mbr);
        }
        for split_child in &split_children {
            split_child.expand_rect_to_fit(&mut split_mbr);
        }
        (split_mbr, split_children)
    }

    //OT1
    fn handle_overflow(&self, level: &mut MbrNode<P, DIM, LSHAPE, T>, at_root: bool, force_split: bool) -> InsertResult<P, DIM, LSHAPE, T> {
        if !at_root && !force_split {
            match *level {
                MbrNode::Leaves{ref mut mbr, ref mut children} => return InsertResult::Reinsert(self.split_for_reinsert(mbr, children)),
                _ => unreachable!(),
            }

        }
        match *level {
                MbrNode::Leaves{ref mut mbr, ref mut children} => {
                    let (split_mbr, split_children) = self.split(mbr, children);
                    InsertResult::Split(MbrNode::Leaves{mbr: split_mbr, children: split_children})
                },
                MbrNode::Level{ref mut mbr, ref mut children} => {
                    let (split_mbr, split_children) = self.split(mbr, children);
                    InsertResult::Split(MbrNode::Level{mbr: split_mbr, children: split_children})
                },
        }
    }

    fn handle_split_root(&self, root: MbrNode<P, DIM, LSHAPE, T>, split: MbrNode<P, DIM, LSHAPE, T>) -> MbrNode<P, DIM, LSHAPE, T> {
        let mut split_mbr = Rect::max_inverted();
        root.expand_rect_to_fit(&mut split_mbr);
        let mut split_children = Vec::with_capacity(MIN::to_usize());
        split_children.push(root);
        split_children.push(split);
        MbrNode::Level{mbr: split_mbr, children: split_children}
    }
}

impl<P, DIM, LSHAPE, MIN, MAX, T> IndexInsert<P, DIM, LSHAPE, T> for RStarInsert<P, DIM, LSHAPE, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LSHAPE: Shape<P, DIM>,
        MIN: Unsigned,
        MAX: Unsigned
{
    fn insert_into_root(&self, mut root: MbrNode<P, DIM, LSHAPE, T>, leaf: Leaf<P, DIM, LSHAPE, T>) -> MbrNode<P, DIM, LSHAPE, T> {
        let insert_results = self.insert_into_level(&mut root, leaf, FORCE_SPLIT, DONT_FORCES_SPLIT);
        match insert_results {
            InsertResult::Split(child) => {
                self.handle_split_root(root, child)
            },
            // RI4
            InsertResult::Reinsert(mut leaves) => {
                while let Some(leaf) = leaves.pop() {
                    match self.insert_into_level(&mut root, leaf, AT_ROOT, FORCE_SPLIT) {
                        InsertResult::Split(child) => {
                            root = self.handle_split_root(root, child);
                        },
                        InsertResult::Reinsert(_) => unreachable!(),
                        InsertResult::Ok => continue
                    }
                }
                root
            }
            _ => root
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use shapes::Rect;
    use generic_array::GenericArray;
    use typenum::consts::U3;

    #[test]
    fn margin() {
        let g_one: GenericArray<f64, U3> = arr![f64; 1.0f64, 1.0f64, 1.0f64];
        let g_zero: GenericArray<f64, U3> = arr![f64; 0.0f64, 0.0f64, 0.0f64];

        // contained
        let zero_one = Rect::from_corners(g_zero.clone(), g_one.clone());
        // margin
        assert_relative_eq!(3.0f64, zero_one.margin());
    }
}