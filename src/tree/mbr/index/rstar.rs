// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::geometry::Rect;
use num::{Float, Zero};
use ordered_float::NotNan;
use std::cmp;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;
use crate::tree::mbr::index::{IndexInsert, AT_ROOT, DONT_FORCE_SPLIT, D_MAX, FORCE_SPLIT, NOT_AT_ROOT};
use crate::tree::mbr::{MbrLeaf, MbrLeafGeometry, MbrNode, RTreeNode};
use crate::FP;

const D_REINSERT_P: f32 = 0.30f32;
const D_SPLIT_P: f32 = 0.40f32;
const D_CHOOSE_SUBTREE_P: usize = 32;

/// The sum of all of the Shapes's edges. Used in the R* algorithm
pub trait Margin<P> {
    fn margin(&self) -> P;
}

impl<P: FP, const DIM: usize> Margin<P> for Rect<P, DIM> {
    fn margin(&self) -> P {
        self.edges
            .iter()
            .fold(Zero::zero(), |margin, &(x, y)| margin + y - x)
    }
}

#[derive(Debug)]
#[must_use]
enum InsertResult<P: FP, const DIM: usize, LG, T> {
    Ok,
    Reinsert(Vec<MbrLeaf<P, DIM, LG, T>>),
    Split(RTreeNode<P, DIM, LG, T>),
}

impl<P: FP, const DIM: usize, LG, T> InsertResult<P, DIM, LG, T> {
    fn is_reinsert(&self) -> bool {
        match *self {
            InsertResult::Reinsert(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct RStarInsert<P: FP, const DIM: usize, LG, T> {
    max: usize,
    preferred_min: usize,
    reinsert_m: usize,
    choose_subtree_p: usize,
    min_k: usize,
    max_k: usize,
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P: FP, const DIM: usize, LG, T> RStarInsert<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    pub fn new() -> RStarInsert<P, DIM, LG, T> {
        RStarInsert::new_with_options(D_MAX, D_REINSERT_P, D_SPLIT_P, D_CHOOSE_SUBTREE_P)
    }

    pub fn new_with_max(max: usize) -> RStarInsert<P, DIM, LG, T> {
        RStarInsert::new_with_options(max, D_REINSERT_P, D_SPLIT_P, D_CHOOSE_SUBTREE_P)
    }

    pub fn new_with_options(
        max: usize,
        reinsert_p: f32,
        split_p: f32,
        choose_subtree_p: usize,
    ) -> RStarInsert<P, DIM, LG, T> {
        let preferred_min = cmp::max((max as f32 * reinsert_p.min(split_p)) as usize, 1);
        let reinsert_size = cmp::max((max as f32 * reinsert_p) as usize, 1);
        let reinsert_m = max - reinsert_size;

        let min_k = cmp::max((max as f32 * split_p) as usize, 1);
        let max_k = cmp::max(max - (2 * min_k) + 1, min_k + 1);
        // On max_k==min_k, the iterator in split is a no-op and vec splits occur at index 0. Happens with M - 2m + 1 and M - 2m + 2 for various small Ms.
        // The above line should prevent this, but assert in case code changes
        assert!(
            max_k > min_k,
            "max_k({:?}) must be greater than min_k({:?})",
            max_k,
            min_k
        );
        assert!(
            max > max_k,
            "max({:?}) must be greater than max_k({:?})",
            max,
            max_k
        );
        RStarInsert {
            max,
            preferred_min,
            reinsert_m,
            choose_subtree_p,
            min_k,
            max_k,
            _p: PhantomData,
            _lg: PhantomData,
            _t: PhantomData,
        }
    }

    fn area_cost(
        &self,
        mbr: &Rect<P, DIM>,
        leaf: &MbrLeaf<P, DIM, LG, T>,
    ) -> (NotNan<P>, NotNan<P>) {
        let mut expanded = mbr.clone();
        leaf.expand_mbr_to_fit(&mut expanded);
        let mbr_area = mbr.area();
        let area_cost = expanded.area() - mbr_area;
        (
            area_cost.try_into().ok().unwrap(),
            mbr_area.try_into().ok().unwrap(),
        )
    }

    fn overlap_cost(&self, mbr: &Rect<P, DIM>, leaf: &MbrLeaf<P, DIM, LG, T>) -> NotNan<P> {
        let overlap = leaf.area_overlapped_with_mbr(mbr);
        let overlap_cost = leaf.area() - overlap;
        overlap_cost.try_into().ok().unwrap()
    }

    fn overlap_area_cost(
        &self,
        mbr: &Rect<P, DIM>,
        leaf: &MbrLeaf<P, DIM, LG, T>,
    ) -> (NotNan<P>, NotNan<P>, NotNan<P>) {
        let (area_cost, mbr_area) = self.area_cost(mbr, leaf);
        let overlap_cost = self.overlap_cost(mbr, leaf);
        (overlap_cost, area_cost, mbr_area)
    }

    // CS2 + optimizations
    fn choose_subnode<'tree>(
        &self,
        level: &'tree mut Vec<RTreeNode<P, DIM, LG, T>>,
        leaf: &MbrLeaf<P, DIM, LG, T>,
    ) -> &'tree mut RTreeNode<P, DIM, LG, T> {
        assert!(!level.is_empty(), "Level should not be empty!");
        if level.first().unwrap().has_leaves() {
            if level.len() > self.choose_subtree_p {
                level.sort_by_key(|a| self.area_cost(a.mbr(), leaf));
                let (left, _) = level.split_at_mut(self.choose_subtree_p);
                return left
                    .iter_mut()
                    .min_by_key(|a| self.overlap_cost(a.mbr(), leaf))
                    .unwrap();
            } else {
                return level
                    .iter_mut()
                    .min_by_key(|a| self.overlap_area_cost(a.mbr(), leaf))
                    .unwrap();
            }
        }
        level
            .iter_mut()
            .min_by_key(|a| self.area_cost(a.mbr(), leaf))
            .unwrap()
    }

    fn split_for_reinsert(
        &self,
        mbr: &mut Rect<P, DIM>,
        children: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
    ) -> Vec<MbrLeaf<P, DIM, LG, T>> {
        // RI1 & RI2
        children.sort_by_key(|a| a.distance_from_mbr_center(mbr).try_into().ok().unwrap());
        //RI3
        let split = children.split_off(self.reinsert_m);
        *mbr = Rect::max_inverted();
        for child in children {
            child.expand_mbr_to_fit(mbr);
        }
        split
    }

    fn insert_into_level(
        &self,
        level: &mut RTreeNode<P, DIM, LG, T>,
        leaf: MbrLeaf<P, DIM, LG, T>,
        at_root: bool,
        force_split: bool,
    ) -> InsertResult<P, DIM, LG, T> {
        //I4
        leaf.geometry.expand_mbr_to_fit(level.mbr_mut());
        match *level {
            //I2
            RTreeNode::Leaves {
                ref mut children, ..
            } => {
                children.push(leaf);
            }
            //I1
            RTreeNode::Level {
                ref mut mbr,
                ref mut children,
            } => {
                //CS3
                let insert_result = self.insert_into_level(
                    self.choose_subnode(children, &leaf),
                    leaf,
                    NOT_AT_ROOT,
                    force_split,
                );
                //I3
                if let InsertResult::Split(child) = insert_result {
                    children.push(child);
                } else {
                    //I4
                    if insert_result.is_reinsert() {
                        *mbr = Rect::max_inverted();
                        for child in children {
                            child.mbr().expand_mbr_to_fit(mbr);
                        }
                    }
                    return insert_result;
                }
            }
        }
        //I2 & I3
        if level.len() > self.max {
            return self.handle_overflow(level, at_root, force_split);
        }
        InsertResult::Ok
    }

    // fn best_position_for_axis -> (margin, (axis, edge, index))
    fn best_split_position_for_axis<V: MbrLeafGeometry<P, DIM>>(
        &self,
        axis: usize,
        children: &mut Vec<V>,
    ) -> (P, (usize, usize, usize)) {
        let mut margin: P = Zero::zero();
        let mut d_area: P = Float::max_value();
        let mut d_overlap: P = Float::max_value();
        let mut d_edge: usize = 0;
        let mut d_index: usize = 0;

        for edge in 0..2 {
            if edge == 0 {
                children.sort_by_key(|child| child.min_for_axis(axis).try_into().ok().unwrap());
            } else {
                children.sort_by_key(|child| child.max_for_axis(axis).try_into().ok().unwrap());
            }

            for k in self.min_k..self.max_k {
                let mut r1 = Rect::max_inverted();
                let mut r2 = Rect::max_inverted();

                let (left, right) = children.split_at(k);
                for child in left {
                    child.expand_mbr_to_fit(&mut r1);
                }
                for child in right {
                    child.expand_mbr_to_fit(&mut r2);
                }

                // (I)
                let area = r1.area() + r2.area();
                // (II)
                margin += r1.margin() + r2.margin();
                // (III)
                let overlap = r1.area_overlapped_with_mbr(&r2);

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

    fn split<V: MbrLeafGeometry<P, DIM>>(
        &self,
        mbr: &mut Rect<P, DIM>,
        children: &mut Vec<V>,
    ) -> (Rect<P, DIM>, Vec<V>) {
        // S1 & S2
        let (s_axis, s_edge, s_index) = Range { start: 0, end: DIM }
            // CSA1
            .map(|axis| self.best_split_position_for_axis(axis, children))
            // CSA2
            .min_by_key(|&(margin, _)| margin.try_into().ok().unwrap())
            .unwrap()
            .1;

        if s_edge == 0 {
            children.sort_by_key(|child| child.min_for_axis(s_axis).try_into().ok().unwrap());
        } else {
            children.sort_by_key(|child| child.max_for_axis(s_axis).try_into().ok().unwrap());
        }
        // S3
        let split_children = children.split_off(s_index);
        *mbr = Rect::max_inverted();
        let mut split_mbr = Rect::max_inverted();
        for child in &*children {
            child.expand_mbr_to_fit(mbr);
        }
        for split_child in &split_children {
            split_child.expand_mbr_to_fit(&mut split_mbr);
        }
        (split_mbr, split_children)
    }

    //OT1
    fn handle_overflow(
        &self,
        level: &mut RTreeNode<P, DIM, LG, T>,
        at_root: bool,
        force_split: bool,
    ) -> InsertResult<P, DIM, LG, T> {
        if !at_root && !force_split {
            match *level {
                RTreeNode::Leaves {
                    ref mut mbr,
                    ref mut children,
                } => return InsertResult::Reinsert(self.split_for_reinsert(mbr, children)),
                _ => unreachable!(),
            }
        }
        match *level {
            RTreeNode::Leaves {
                ref mut mbr,
                ref mut children,
            } => {
                let (split_mbr, split_children) = self.split(mbr, children);
                InsertResult::Split(RTreeNode::Leaves {
                    mbr: split_mbr,
                    children: split_children,
                })
            }
            RTreeNode::Level {
                ref mut mbr,
                ref mut children,
            } => {
                let (split_mbr, split_children) = self.split(mbr, children);
                InsertResult::Split(RTreeNode::Level {
                    mbr: split_mbr,
                    children: split_children,
                })
            }
        }
    }

    fn handle_split_root(
        &self,
        root: RTreeNode<P, DIM, LG, T>,
        split: RTreeNode<P, DIM, LG, T>,
    ) -> RTreeNode<P, DIM, LG, T> {
        let mut mbr = root.mbr().clone();
        split.expand_mbr_to_fit(&mut mbr);
        RTreeNode::Level {
            mbr: mbr,
            children: vec![root, split],
        }
    }
}

impl<P: FP, const DIM: usize, LG, T> IndexInsert<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>
    for RStarInsert<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    fn insert_into_root(
        &self,
        mut root: RTreeNode<P, DIM, LG, T>,
        leaf: MbrLeaf<P, DIM, LG, T>,
    ) -> RTreeNode<P, DIM, LG, T> {
        let insert_results = self.insert_into_level(&mut root, leaf, FORCE_SPLIT, DONT_FORCE_SPLIT);
        match insert_results {
            InsertResult::Split(child) => self.handle_split_root(root, child),
            // RI4
            InsertResult::Reinsert(leaves) => {
                for leaf in leaves {
                    match self.insert_into_level(&mut root, leaf, AT_ROOT, FORCE_SPLIT) {
                        InsertResult::Split(child) => {
                            root = self.handle_split_root(root, child);
                        }
                        InsertResult::Reinsert(_) => unreachable!(),
                        InsertResult::Ok => continue,
                    }
                }
                root
            }
            _ => root,
        }
    }

    fn preferred_min(&self) -> usize {
        self.preferred_min
    }

    fn new_leaves(&self) -> RTreeNode<P, DIM, LG, T> {
        RTreeNode::new_leaves()
    }

    fn new_no_alloc_leaves(&self) -> RTreeNode<P, DIM, LG, T> {
        RTreeNode::new_no_alloc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn margin() {
        let g_one = [1.0f64, 1.0f64, 1.0f64];
        let g_zero = [0.0f64, 0.0f64, 0.0f64];

        // contained
        let zero_one = Rect::from_corners(g_zero.clone(), g_one.clone());
        // margin
        assert_relative_eq!(3.0f64, zero_one.margin());
    }
}
