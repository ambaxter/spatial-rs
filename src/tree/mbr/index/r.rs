// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

use itertools::Itertools;
use num::{Bounded, One, Zero};
use ordered_float::NotNan;

use crate::geometry::Rect;
use crate::tree::mbr::index::{
    IndexInsert, IndexRemove, MbrNodeSplit, RemoveReturn, AT_ROOT, D_MAX, NOT_AT_ROOT,
};
use crate::tree::mbr::{MbrLeaf, MbrLeafGeometry, MbrNode, MbrQuery, RTreeNode};
use crate::vecext::RetainAndAppend;
use crate::FP;

#[derive(Debug)]
#[must_use]
enum InsertResult<P: FP, const DIM: usize, LG, T> {
    Ok,
    Split(RTreeNode<P, DIM, LG, T>),
}

pub trait PickSeed<P: FP, const DIM: usize, LG, T> {
    fn pick_seed<V: MbrLeafGeometry<P, DIM>>(
        &self,
        mbr: &Rect<P, DIM>,
        children: &Vec<V>,
    ) -> (usize, usize);
}

pub struct Quadratic;

impl<P: FP, const DIM: usize, LG, T> PickSeed<P, DIM, LG, T> for Quadratic
where
    LG: MbrLeafGeometry<P, DIM>,
{
    fn pick_seed<V: MbrLeafGeometry<P, DIM>>(
        &self,
        _mbr: &Rect<P, DIM>,
        children: &Vec<V>,
    ) -> (usize, usize) {
        let (_, k, l) = children
            .iter()
            .enumerate()
            .tuple_combinations()
            // PS1
            .map(|((k, k_child), (l, l_child))| {
                let mut mbr = Rect::max_inverted();
                k_child.expand_mbr_to_fit(&mut mbr);
                l_child.expand_mbr_to_fit(&mut mbr);
                (mbr.area() - k_child.area() - l_child.area(), k, l)
            })
            // PS2
            .max_by_key(|&(j, _, _)| j.try_into().ok().unwrap())
            .unwrap();
        (k, l)
    }
}

pub struct Linear;

impl<P: FP, const DIM: usize, LG, T> PickSeed<P, DIM, LG, T> for Linear
where
    LG: MbrLeafGeometry<P, DIM>,
{
    fn pick_seed<V: MbrLeafGeometry<P, DIM>>(
        &self,
        mbr: &Rect<P, DIM>,
        children: &Vec<V>,
    ) -> (usize, usize) {
        let mut widths = [P::default(); DIM];
        izip!(widths.iter_mut(), mbr.deref()).for_each(|(width, &(min, max))| {
            *width = max - min;
            if *width <= Zero::zero() {
                *width = One::one();
            }
        });
        let mut greatest_lower = [(Bounded::max_value(), 0); DIM];
        let mut least_upper = [(Bounded::min_value(), 0); DIM];

        // LPS1
        children.iter().enumerate().for_each(|(i, child)| {
            izip!(least_upper.iter_mut(), greatest_lower.iter_mut())
                .enumerate()
                .for_each(
                    |(dim, (&mut (ref mut lmax, ref mut li), &mut (ref mut gmin, ref mut gi)))| {
                        let min_for_axis = child.min_for_axis(dim);
                        let max_for_axis = child.max_for_axis(dim);
                        if min_for_axis > *lmax {
                            *lmax = min_for_axis;
                            *li = i;
                        }
                        if max_for_axis < *gmin {
                            *gmin = min_for_axis;
                            *gi = i;
                        }
                    },
                );
        });

        let (_, mut k, mut l) = izip!(widths.iter(), least_upper.iter(), greatest_lower.iter())
            // LPS2
            .map(|(width, &(lmax, li), &(gmin, gi))| (((gmin - lmax) / *width), li, gi))
            // LPS3
            .max_by_key(|&(separation, _, _)| separation.try_into().ok().unwrap())
            .unwrap();

        if k > l {
            mem::swap(&mut k, &mut l);
        }
        (k, l)
    }
}

pub struct SeedSplit<P: FP, const DIM: usize, LG, T, PS> {
    pick_seed: PS,
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P: FP, const DIM: usize, LG, T, PS> SeedSplit<P, DIM, LG, T, PS>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    pub fn linear() -> SeedSplit<P, DIM, LG, T, Linear> {
        SeedSplit {
            pick_seed: Linear,
            _p: PhantomData,
            _lg: PhantomData,
            _t: PhantomData,
        }
    }

    pub fn quadratic() -> SeedSplit<P, DIM, LG, T, Quadratic> {
        SeedSplit {
            pick_seed: Quadratic,
            _p: PhantomData,
            _lg: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<P: FP, const DIM: usize, LG, T, PS> MbrNodeSplit<P, DIM> for SeedSplit<P, DIM, LG, T, PS>
where
    LG: MbrLeafGeometry<P, DIM>,
    PS: PickSeed<P, DIM, LG, T>,
{
    fn split<V: MbrLeafGeometry<P, DIM>>(
        &self,
        min: usize,
        mbr: &mut Rect<P, DIM>,
        children: &mut Vec<V>,
    ) -> (Rect<P, DIM>, Vec<V>) {
        assert!(!children.is_empty(), "Empty children should not be split.");
        // QS1
        let (mut k, mut l) = self.pick_seed.pick_seed(mbr, children);

        // in the unlikely scenario of a tie, just pick something
        if k == l {
            if k < children.len() - 1 {
                l += 1;
            } else {
                k -= 1;
            }
        }

        assert!(k < l, "k {:?} must be less than l {:?}", k, l);

        let mut k_mbr = Rect::max_inverted();
        let k_child = children.remove(k);
        k_child.expand_mbr_to_fit(&mut k_mbr);
        let mut k_children = Vec::new();
        k_children.push(k_child);

        let mut l_mbr = Rect::max_inverted();
        // -1 because we removed k
        let l_child = children.remove(l - 1);
        l_child.expand_mbr_to_fit(&mut l_mbr);
        let mut l_children = Vec::new();
        l_children.push(l_child);

        loop {
            // QS2
            if children.is_empty() {
                break;
            }
            if k_children.len() + children.len() == min {
                for child in children.iter() {
                    child.expand_mbr_to_fit(&mut k_mbr);
                }
                k_children.append(children);
                break;
            }
            if l_children.len() + children.len() == min {
                for child in children.iter() {
                    child.expand_mbr_to_fit(&mut l_mbr);
                }
                l_children.append(children);
                break;
            }
            // QS3
            if let Some(child) = children.pop() {
                let mut k_expanded = k_mbr.clone();
                child.expand_mbr_to_fit(&mut k_expanded);
                let mut l_expanded = l_mbr.clone();
                child.expand_mbr_to_fit(&mut l_expanded);
                let k_area = k_mbr.area();
                let l_area = l_mbr.area();
                if (k_expanded.area() - k_area, k_area, k_children.len())
                    < (l_expanded.area() - l_area, l_area, l_children.len())
                {
                    k_mbr = k_expanded;
                    k_children.push(child);
                } else {
                    l_mbr = l_expanded;
                    l_children.push(child);
                }
            }
        }
        *mbr = k_mbr;
        children.append(&mut k_children);
        (l_mbr, l_children)
    }
}

pub struct RInsert<P: FP, const DIM: usize, LG, T, NS> {
    preferred_min: usize,
    max: usize,
    splitter: NS,
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P: FP, const DIM: usize, LG, T, NS> RInsert<P, DIM, LG, T, NS>
where
    LG: MbrLeafGeometry<P, DIM>,
    NS: MbrNodeSplit<P, DIM>,
{
    pub fn new(splitter: NS) -> RInsert<P, DIM, LG, T, NS> {
        // TODO: This type specification shouldn't be needed?
        RInsert::new_with_max(splitter, D_MAX)
    }

    pub fn new_with_max(splitter: NS, max: usize) -> RInsert<P, DIM, LG, T, NS> {
        let min = (max as f32 * 0.3f32) as usize;
        RInsert {
            preferred_min: min,
            max,
            splitter,
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
        let expanded_area = expanded.area();
        let area_cost = expanded_area - mbr_area;
        (
            area_cost.try_into().ok().unwrap(),
            expanded_area.try_into().ok().unwrap(),
        )
    }

    fn choose_subnode<'tree>(
        &self,
        level: &'tree mut Vec<RTreeNode<P, DIM, LG, T>>,
        leaf: &MbrLeaf<P, DIM, LG, T>,
    ) -> &'tree mut RTreeNode<P, DIM, LG, T> {
        assert!(!level.is_empty(), "Level should not be empty!");
        level
            .iter_mut()
            .min_by_key(|a| self.area_cost(a.mbr(), leaf))
            .unwrap()
    }

    //OT1
    fn handle_overflow(&self, level: &mut RTreeNode<P, DIM, LG, T>) -> InsertResult<P, DIM, LG, T> {
        match *level {
            RTreeNode::Leaves {
                ref mut mbr,
                ref mut children,
            } => {
                let (split_mbr, split_children) =
                    self.splitter.split(self.preferred_min, mbr, children);
                InsertResult::Split(RTreeNode::Leaves {
                    mbr: split_mbr,
                    children: split_children,
                })
            }
            RTreeNode::Level {
                ref mut mbr,
                ref mut children,
            } => {
                let (split_mbr, split_children) =
                    self.splitter.split(self.preferred_min, mbr, children);
                InsertResult::Split(RTreeNode::Level {
                    mbr: split_mbr,
                    children: split_children,
                })
            }
        }
    }

    fn insert_into_level(
        &self,
        level: &mut RTreeNode<P, DIM, LG, T>,
        leaf: MbrLeaf<P, DIM, LG, T>,
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
                ref mut children, ..
            } => {
                //CS3
                let insert_result =
                    self.insert_into_level(self.choose_subnode(children, &leaf), leaf);
                //I3
                if let InsertResult::Split(child) = insert_result {
                    children.push(child);
                }
            }
        }
        //I2 & I3
        if level.len() > self.max {
            return self.handle_overflow(level);
        }
        InsertResult::Ok
    }
}

impl<P: FP, const DIM: usize, LG, T, NS> IndexInsert<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>
    for RInsert<P, DIM, LG, T, NS>
where
    NS: MbrNodeSplit<P, DIM>,
    LG: MbrLeafGeometry<P, DIM>,
{
    fn insert_into_root(
        &self,
        mut root: RTreeNode<P, DIM, LG, T>,
        leaf: MbrLeaf<P, DIM, LG, T>,
    ) -> RTreeNode<P, DIM, LG, T> {
        let result = self.insert_into_level(&mut root, leaf);
        if let InsertResult::Split(split) = result {
            let mut mbr = root.mbr().clone();
            split.expand_mbr_to_fit(&mut mbr);
            let children = vec![root, split];
            root = RTreeNode::Level { mbr, children };
        }
        root
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

pub struct RRemove<P: FP, const DIM: usize, LG, T> {
    min: usize,
    _p: PhantomData<P>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P: FP, const DIM: usize, LG, T> RRemove<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
{
    pub fn with_min(min: usize) -> RRemove<P, DIM, LG, T> {
        assert!(min > 0, "min({:?}) must be at least 0.", min);
        RRemove {
            min,
            _p: PhantomData,
            _lg: PhantomData,
            _t: PhantomData,
        }
    }

    /// Removes matching leaves from a leaf level. Return true if the level should be retained
    fn remove_matching_leaves<
        Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
        F: FnMut(&T) -> bool,
    >(
        &self,
        query: &Q,
        mbr: &mut Rect<P, DIM>,
        children: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        removed: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        f: &mut F,
        at_root: bool,
    ) -> bool {
        let orig_len = children.len();
        // D2
        children.retain_and_append(removed, |leaf| !query.accept_leaf(leaf) || f(&leaf.item));
        let children_removed = orig_len != children.len();
        // CT3
        if children.len() < self.min && !at_root {
            to_reinsert.append(children);
            return false;
        }
        // CT4
        if children_removed {
            *mbr = Rect::max_inverted();
            for child in children {
                child.expand_mbr_to_fit(mbr);
            }
        }
        true
    }

    /// Consume all child leaves and queue them for reinsert
    fn consume_leaves_for_reinsert(
        &self,
        nodes: &mut Vec<RTreeNode<P, DIM, LG, T>>,
        to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
    ) {
        for node in nodes {
            match *node {
                RTreeNode::Leaves {
                    ref mut children, ..
                } => to_reinsert.append(&mut mem::replace(children, Vec::with_capacity(0))),
                RTreeNode::Level {
                    ref mut children, ..
                } => self.consume_leaves_for_reinsert(children, to_reinsert),
            }
        }
    }

    /// Recursively remove leaves from a level. Return true if the level should be retianed
    fn remove_leaves_from_level<
        Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
        F: FnMut(&T) -> bool,
    >(
        &self,
        query: &Q,
        level: &mut RTreeNode<P, DIM, LG, T>,
        removed: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        f: &mut F,
        at_root: bool,
    ) -> bool {
        // FL1
        if !query.accept_level(level) {
            return true;
        }
        match *level {
            // FL2
            RTreeNode::Leaves {
                ref mut mbr,
                ref mut children,
                ..
            } => {
                return self.remove_matching_leaves(
                    query,
                    mbr,
                    children,
                    removed,
                    to_reinsert,
                    f,
                    at_root,
                )
            }
            RTreeNode::Level {
                ref mut mbr,
                ref mut children,
                ..
            } => {
                let orig_len = children.len();
                children.retain_mut(|child| {
                    self.remove_leaves_from_level(
                        query,
                        child,
                        removed,
                        to_reinsert,
                        f,
                        NOT_AT_ROOT,
                    )
                });
                let children_removed = orig_len != children.len();
                // CT5
                // This technically goes against the original R-Tree paper,
                // but it's a bit simpler given the height-naive data structures and algorithms I've written
                if children.len() < self.min && !at_root {
                    self.consume_leaves_for_reinsert(children, to_reinsert);
                    return false;
                }
                if children_removed {
                    *mbr = Rect::max_inverted();
                    for child in &*children {
                        child.expand_mbr_to_fit(mbr);
                    }
                }
            }
        }
        true
    }
}

impl<P: FP, const DIM: usize, LG, T, I> IndexRemove<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>, I>
    for RRemove<P, DIM, LG, T>
where
    LG: MbrLeafGeometry<P, DIM>,
    I: IndexInsert<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    fn remove_from_root<
        Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
        F: FnMut(&T) -> bool,
    >(
        &self,
        mut root: RTreeNode<P, DIM, LG, T>,
        insert_index: &I,
        query: Q,
        mut f: F,
    ) -> RemoveReturn<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>> {
        if root.is_empty() {
            (root, Vec::with_capacity(0))
        } else {
            // CT1
            let mut to_reinsert = Vec::new();
            let mut removed = Vec::new();
            // D1 && CT2
            self.remove_leaves_from_level(
                &query,
                &mut root,
                &mut removed,
                &mut to_reinsert,
                &mut f,
                AT_ROOT,
            );
            // Insert algorithms require an empty root to be for leaves
            if root.is_empty() && root.has_levels() {
                root = insert_index.new_leaves();
            }
            // CT6
            for leaf in to_reinsert {
                root = insert_index.insert_into_root(root, leaf);
            }
            (root, removed)
        }
    }
}
