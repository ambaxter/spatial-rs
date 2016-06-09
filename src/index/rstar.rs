// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Zero, Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign, Range};
use tree::{LevelNode, Leaf};
use index::IndexInsert;
use shapes::{Shape, Rect};
use std::fmt::Debug;
use std::marker::PhantomData;
use ordered_float::NotNaN;
use std::cmp;
use typenum::Unsigned;
use generic_array::ArrayLength;

const D_REINSERT_P: f32 = 0.30f32;
const D_CHOOSE_SUBTREE_P: usize = 32;

#[allow(dead_code)]
#[derive(Debug)]
#[must_use]
enum InsertResult<P, D, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    Ok,
    Reinsert(Vec<Leaf<P, D, T>>),
    Split(LevelNode<P, D, T>),
}

#[allow(dead_code)]
impl<P, D, T> InsertResult<P, D, T>
    where P: Float
        + Signed
        + Bounded
        + MulAssign
        + AddAssign
        + ToPrimitive
        + FromPrimitive
        + Copy
        + Debug,
        D: ArrayLength<P> + ArrayLength<(P, P)> {

    pub fn is_ok(&self) -> bool {
        if let InsertResult::Ok = *self {
            return true;
        }
        false
    }

    pub fn is_reinsert(&self) -> bool {
        if let InsertResult::Reinsert(_) = *self {
            return true;
        }
        false
    }

    pub fn is_split(&self) -> bool {
        if let InsertResult::Split(_) = *self {
            return true;
        }
        false
    }
}


#[allow(dead_code)]
#[derive(Debug)]
struct RStarInsert<P, D, MIN, MAX, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>,
          MIN: Unsigned,
          MAX: Unsigned
{
    reinsert_p: f32,
    choose_subtree_p: usize,
    _phantom_d: PhantomData<D>,
    _phantom_p: PhantomData<P>,
    _phantom_min: PhantomData<MIN>,
    _phantom_max: PhantomData<MAX>,
    _phantom_t: PhantomData<T>,
}


#[allow(dead_code)]
impl<P, D, MIN, MAX, T> RStarInsert<P, D, MIN, MAX, T>
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
        D: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        MIN: Unsigned,
        MAX: Unsigned
{

    pub fn new() -> RStarInsert<P, D, MIN, MAX, T> {
        RStarInsert{reinsert_p: D_REINSERT_P,
            choose_subtree_p: D_CHOOSE_SUBTREE_P,
            _phantom_d: PhantomData,
            _phantom_p: PhantomData,
            _phantom_min: PhantomData,
            _phantom_max: PhantomData,
            _phantom_t: PhantomData
        }
    }

    fn area_cost(&self, mbr: &Rect<P, D>, leaf: &Leaf<P, D, T>) -> (NotNaN<P>, NotNaN<P>) {
        let mut expanded = mbr.clone();
        leaf.expand_rect_to_fit(&mut expanded);
        let mbr_area = mbr.area();
        let area_cost = expanded.area() - mbr_area;
        (NotNaN::from(area_cost), NotNaN::from(mbr_area))
    }

    fn overlap_cost(&self, mbr: &Rect<P, D>, leaf: &Leaf<P, D, T>) -> NotNaN<P> {
        let overlap = leaf.area_overlapped_with_rect(&mbr);
        let overlap_cost = leaf.area() - overlap;
        NotNaN::from(overlap_cost)
    }

    fn overlap_area_cost(&self, mbr: &Rect<P, D>, leaf: &Leaf<P, D, T>) -> (NotNaN<P>, NotNaN<P>, NotNaN<P>) {
        let (area_cost, mbr_area) = self.area_cost(mbr, leaf);
        let overlap_cost = self.overlap_cost(mbr, leaf);
        (overlap_cost, area_cost, mbr_area)
    }

    fn choose_subnode<'tree>(&self, level: &'tree mut Vec<LevelNode<P, D, T>>, leaf: &Leaf<P, D, T>) -> &'tree mut LevelNode<P, D, T> {
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

    fn split_for_reinsert(&self, mbr: &mut Rect<P, D>, children: &mut Vec<Leaf<P, D, T>>) -> Vec<Leaf<P, D, T>> {
        let potential_p = (children.len() as f32 * self.reinsert_p) as usize;
        let left_count = children.len() - cmp::max(potential_p, 1);
        children.sort_by_key(|a| NotNaN::from(a.distance_from_rect_center(mbr)));
        let split = children.split_off(left_count);
        *mbr = Rect::max_inverted();
        for child in children {
            child.expand_rect_to_fit(mbr);
        }
        split
    }

    fn insert_into_level(&self, level: &mut LevelNode<P, D, T>, leaf: Leaf<P, D, T>, at_root: bool, force_split: bool) -> InsertResult<P, D, T> {
        leaf.shape.expand_rect_to_fit(level.mbr_mut());
        match *level {
            LevelNode::Leaves{ref mut children, ..} => {
                children.push(leaf);
            },
            LevelNode::Level{ref mut children, ..} => {
                let insert_result = self.insert_into_level(self.choose_subnode(children, &leaf), leaf, false, force_split);
                if let InsertResult::Split(child) = insert_result {
                    children.push(child);
                } else {
                    return insert_result;
                }
            }
        }
        if level.len() > MAX::to_usize() {
            return self.handle_overflow(level, at_root, force_split);
        }
        InsertResult::Ok
    }


    // fn best_position_for_axis -> (margin, (axis, edge, index))
    fn best_split_position_for_axis<S: Shape<P, D>>(&self, axis: usize, children: &mut Vec<S>) ->(P, (usize, usize, usize)) {
        let k_start = MIN::to_usize();
        // TODO: How do we put this at compile time?
        let k_end = MAX::to_usize() - k_start + 2;
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

            for k in k_start..k_end {
                let mut r1 = Rect::max_inverted();
                let mut r2 = Rect::max_inverted();

                let (left, right) = children.split_at(k);
                for child in left {
                    child.expand_rect_to_fit(&mut r1);
                }
                for child in right {
                    child.expand_rect_to_fit(&mut r2);
                }

                margin += r1.margin() + r2.margin();
                let area = r1.area() + r2.area();
                let overlap = r1.area_overlapped_with_rect(&r2);

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

    fn split<S: Shape<P, D>>(&self, mbr: &mut Rect<P, D>, children: &mut Vec<S>) -> (Rect<P, D>, Vec<S>) {
        let max_inverted = Rect::max_inverted();
        let (s_axis, s_edge, s_index) = Range{start: 0, end: max_inverted.dim()}
            .map(|axis| self.best_split_position_for_axis(axis, children))
            .min_by_key(|&(margin, _)| NotNaN::from(margin))
            .unwrap().1;

        if s_edge == 0 {
            children.sort_by_key(|child| NotNaN::from(child.min_for_axis(s_axis)));
        } else {
            children.sort_by_key(|child| NotNaN::from(child.max_for_axis(s_axis)));
        }
        let split_children = children.split_off(s_index);
        *mbr = max_inverted.clone();
        let mut split_mbr = max_inverted.clone();
        for child in &*children {
            child.expand_rect_to_fit(mbr);
        }
        for split_child in &split_children {
            split_child.expand_rect_to_fit(&mut split_mbr);
        }
        (split_mbr, split_children)
    }

    fn handle_overflow(&self, level: &mut LevelNode<P, D, T>, at_root: bool, force_split: bool) -> InsertResult<P, D, T> {
        if !at_root && !force_split {
            match *level {
                LevelNode::Leaves{ref mut mbr, ref mut children} => return InsertResult::Reinsert(self.split_for_reinsert(mbr, children)),
                _ => unreachable!(),
            }

        }
        match *level {
                LevelNode::Leaves{ref mut mbr, ref mut children} => {
                    let (split_mbr, split_children) = self.split(mbr, children);
                    return InsertResult::Split(LevelNode::Leaves{mbr: split_mbr, children: split_children})
                },
                LevelNode::Level{ref mut mbr, ref mut children} => {
                    let (split_mbr, split_children) = self.split(mbr, children);
                    return InsertResult::Split(LevelNode::Level{mbr: split_mbr, children: split_children})
                },
        }
    }

    fn handle_split_root(&self, root: LevelNode<P, D, T>, split: LevelNode<P, D, T>) -> LevelNode<P, D, T> {
        let mut split_mbr = Rect::max_inverted();
        root.expand_rect_to_fit(&mut split_mbr);
        let mut split_children = Vec::with_capacity(MIN::to_usize());
        split_children.push(root);
        split_children.push(split);
        LevelNode::Level{mbr: split_mbr, children: split_children}
    }
}

impl<P, D, MIN, MAX, T> IndexInsert<P, D, T> for RStarInsert<P, D, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        D: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        MIN: Unsigned,
        MAX: Unsigned
{
    fn insert_into_root(&self, root: Option<LevelNode<P, D, T>>, leaf: Leaf<P, D, T>) -> Option<LevelNode<P, D, T>> {
        if root.is_none() {
            let mut mbr = Rect::max_inverted();
            leaf.expand_rect_to_fit(&mut mbr);
            let mut root_nodes = Vec::with_capacity(MIN::to_usize());
            root_nodes.push(leaf);
            return Some(LevelNode::Leaves{mbr: mbr, children:root_nodes});
        } else {
            let mut root_node = root.unwrap();
            let insert_results = self.insert_into_level(&mut root_node, leaf, true, false);
            return match insert_results {
                InsertResult::Split(child) => {
                    Some(self.handle_split_root(root_node, child))
                },
                InsertResult::Reinsert(mut leaves) => {
                    while let Some(leaf) = leaves.pop() {
                        match self.insert_into_level(&mut root_node, leaf, true, true) {
                            InsertResult::Split(child) => {
                                root_node = self.handle_split_root(root_node, child);
                            },
                            InsertResult::Reinsert(_) => unreachable!(),
                            InsertResult::Ok => continue
                        }
                    }
                    Some(root_node)
                }
                _ => Some(root_node)
            }
        }
    }
}
