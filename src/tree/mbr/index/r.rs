// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use vecext::{RetainMut, RetainAndAppend};
use geometry::Rect;
use std::fmt::Debug;
use generic_array::ArrayLength;
use tree::mbr::{MbrNode, MbrQuery, MbrLeaf, MbrLeafGeometry};
use tree::mbr::index::{IndexInsert, IndexRemove, RemoveReturn, AT_ROOT, NOT_AT_ROOT};
use std::marker::PhantomData;
use parking_lot::RwLock;
use vecext::{PackRwLocks, UnpackRwLocks};
use std::mem;
use ordered_float::NotNaN;

#[derive(Debug)]
#[must_use]
enum InsertResult<P, DIM, LG, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    Ok,
    Split(MbrNode<P, DIM, LG, T>),
}

pub struct RQuadraticInsert<P, DIM, LG, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    preferred_min: usize,
    max: usize,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P, DIM, LG, T> RQuadraticInsert<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LG: MbrLeafGeometry<P, DIM>,
{
    fn area_cost(&self, mbr: &Rect<P, DIM>, leaf: &MbrLeaf<P, DIM, LG, T>) -> (NotNaN<P>, NotNaN<P>) {
        let mut expanded = mbr.clone();
        leaf.expand_mbr_to_fit(&mut expanded);
        let mbr_area = mbr.area();
        let expanded_area = expanded.area();
        let area_cost = expanded_area - mbr_area;
        (NotNaN::from(area_cost), NotNaN::from(expanded_area))
    }

    fn choose_subnode<'tree>(&self, level: &'tree mut Vec<MbrNode<P, DIM, LG, T>>, leaf: &MbrLeaf<P, DIM, LG, T>) -> &'tree mut MbrNode<P, DIM, LG, T> {
        assert!(!level.is_empty(), "Level should not be empty!");
        level.iter_mut().min_by_key(|a| self.area_cost(a.mbr(), leaf)).unwrap()
    }

    fn split<V: MbrLeafGeometry<P, DIM>>(&self, mbr: &mut Rect<P, DIM>, children: &mut Vec<V>) -> (Rect<P, DIM>, Vec<V>) {
    }

    //OT1
    fn handle_overflow(&self, level: &mut MbrNode<P, DIM, LG, T>) -> InsertResult<P, DIM, LG, T> {
        match *level {
                MbrNode::Leaves{ref mut mbr, ref mut children} => {
                    // unpack all children from the RwLock
                    let mut leaf_children = mem::replace(children, Vec::with_capacity(0))
                        .unpack_rwlocks();

                    // Split
                    let (split_mbr, split_children) = self.split(mbr, &mut leaf_children);

                    // repack non-split children
                    *children = leaf_children.pack_rwlocks();

                    InsertResult::Split(MbrNode::Leaves{mbr: split_mbr, children: split_children.pack_rwlocks()})
                },
                MbrNode::Level{ref mut mbr, ref mut children} => {
                    let (split_mbr, split_children) = self.split(mbr, children);
                    InsertResult::Split(MbrNode::Level{mbr: split_mbr, children: split_children})
                },
        }
    }

    fn insert_into_level(&self, level: &mut MbrNode<P, DIM, LG, T>, leaf: MbrLeaf<P, DIM, LG, T>) -> InsertResult<P, DIM, LG, T> {
        //I4
        leaf.geometry.expand_mbr_to_fit(level.mbr_mut());
        match *level {
            //I2
            MbrNode::Leaves{ref mut children, ..} => {
                children.push(RwLock::new(leaf));
            },
            //I1
            MbrNode::Level{ref mut mbr, ref mut children} => {
                //CS3
                let insert_result = self.insert_into_level(self.choose_subnode(children, &leaf), leaf);
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

impl<P, DIM, LG, T> IndexInsert<P, DIM, LG, T> for RQuadraticInsert<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LG: MbrLeafGeometry<P, DIM>,
{
    fn insert_into_root(&self, mut root: MbrNode<P, DIM, LG, T>, leaf: MbrLeaf<P, DIM, LG, T>) -> MbrNode<P, DIM, LG, T> {
        //let mut level_stack = Vec::new();
        root
    }

    fn preferred_min(&self) -> usize {
        self.preferred_min
    }

    fn new_leaves(&self) -> MbrNode<P, DIM, LG, T> {
        MbrNode::new_leaves()
    }

    fn new_no_alloc_leaves(&self) -> MbrNode<P, DIM, LG, T> {
        MbrNode::new_no_alloc()
    }
}

pub struct RRemove<P, DIM, LG, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    min: usize,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _lg: PhantomData<LG>,
    _t: PhantomData<T>,
}

impl<P, DIM, LG, T> RRemove<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)>,
        LG: MbrLeafGeometry<P, DIM>,
{

    pub fn with_min(min: usize) -> RRemove<P, DIM, LG, T> {
        assert!(min > 0, "min({:?}) must be at least 0.", min);
        RRemove{
            min: min,
            _dim: PhantomData,
            _p: PhantomData,
            _lg: PhantomData,
            _t: PhantomData
        }
    }

/// Removes matching leaves from a leaf level. Return true if the level should be retianed
    fn remove_matching_leaves<Q: MbrQuery<P, DIM, LG, T>, F: FnMut(&T) -> bool>(&self, query: &Q, mbr: &mut Rect<P, DIM>, children: &mut Vec<RwLock<MbrLeaf<P, DIM, LG, T>>>,
        removed: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        f: &mut F,
        at_root: bool) -> bool {

// unpack all children from the RwLock
        let mut leaf_children = mem::replace(children, Vec::with_capacity(0))
            .unpack_rwlocks();

        let orig_len = children.len();
        leaf_children.retain_and_append(removed, |leaf| !query.accept_leaf(leaf) || f(&leaf.item));
        let children_removed = orig_len != children.len();
        if leaf_children.len() < self.min && !at_root {
            to_reinsert.append(&mut leaf_children);
            return false;
        }

        if children_removed {
            *mbr = Rect::max_inverted();
            for child in &leaf_children {
               child.expand_mbr_to_fit(mbr);
            }
        }
// repack non-removed children
        *children = leaf_children.pack_rwlocks();
        true
    }

/// Consume all child leaves and queue them for reinsert
    fn consume_leaves_for_reinsert(&self, nodes: &mut Vec<MbrNode<P, DIM, LG, T>>, to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>) {
        for node in nodes {
            match *node {
                MbrNode::Leaves{ref mut children, ..} => to_reinsert.append(&mut mem::replace(children, Vec::with_capacity(0)).unpack_rwlocks()),
                MbrNode::Level{ref mut children, ..} => self.consume_leaves_for_reinsert(children, to_reinsert)
            }
        }
    }

/// Recursively remove leaves from a level. Return true if the level should be retianed
    fn remove_leaves_from_level<Q: MbrQuery<P, DIM, LG, T>, F: FnMut(&T) -> bool>(&self, query: &Q, level: &mut MbrNode<P, DIM, LG, T>,
        removed: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        to_reinsert: &mut Vec<MbrLeaf<P, DIM, LG, T>>,
        f: &mut F,
        at_root: bool) -> bool {
            if !query.accept_level(level) {
                return true;
            }
            match *level {
                MbrNode::Leaves{ref mut mbr, ref mut children, ..} => return self.remove_matching_leaves(query, mbr, children, removed, to_reinsert, f, at_root),
                MbrNode::Level{ref mut mbr, ref mut children, ..} => {
                    let orig_len = children.len();
                    children.retain_mut(|child| self.remove_leaves_from_level(query, child, removed, to_reinsert, f, NOT_AT_ROOT));
                    let children_removed = orig_len != children.len();
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


impl<P, DIM, LG, T, I> IndexRemove<P, DIM, LG, T, I> for RRemove<P, DIM, LG, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LG: MbrLeafGeometry<P, DIM>,
        I: IndexInsert<P, DIM, LG, T>,
{
    fn remove_from_root<Q: MbrQuery<P, DIM, LG, T>, F: FnMut(&T) -> bool>(&self,
        mut root: MbrNode<P, DIM, LG, T>,
        insert_index: &I,
        query: Q,
        mut f: F) -> RemoveReturn<P, DIM, LG, T> {

            if root.is_empty() {
                (root, Vec::with_capacity(0))
            } else {
                let mut to_reinsert = Vec::new();
                let mut removed = Vec::new();
                self.remove_leaves_from_level(&query, &mut root, &mut removed, &mut to_reinsert, &mut f, AT_ROOT);
// Insert algorithms require an empty root to be for leaves
                if root.is_empty() && root.has_levels() {
                    root = insert_index.new_leaves();
                }
                for leaf in to_reinsert {
                    root = insert_index.insert_into_root(root, leaf);
                }
                (root, removed)
            }
        }
}
