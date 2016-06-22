// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use vecext::{RetainMut, RetainAndAppend};
use shapes::{Shape, Rect};
use std::fmt::Debug;
use generic_array::ArrayLength;
use tree::mbr::{MbrNode, MbrQuery};
use tree::{Leaf, SpatialQuery};
use tree::mbr::index::{IndexInsert, IndexRemove, RemoveReturn, AT_ROOT, NOT_AT_ROOT};
use std::marker::PhantomData;
use parking_lot::RwLock;
use vecext::{PackRwLocks, UnpackRwLocks};
use std::mem;

pub struct RRemove<P, DIM, LSHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    min: usize,
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _shape: PhantomData<LSHAPE>,
    _t: PhantomData<T>,
}

impl<P, DIM, LSHAPE, T> RRemove<P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)>,
        LSHAPE: Shape<P, DIM>,
{

    pub fn with_min(min: usize) -> RRemove<P, DIM, LSHAPE, T> {
        assert!(min > 0, "min({:?}) must be at least 0.", min);
        RRemove{
            min: min,
            _dim: PhantomData,
            _p: PhantomData,
            _shape: PhantomData,
            _t: PhantomData
        }
    }

/// Removes matching leaves from a leaf level. Return true if the level should be retianed
    fn remove_matching_leaves<F: FnMut(&T) -> bool>(&self, query: &MbrQuery<P, DIM>, mbr: &mut Rect<P, DIM>, children: &mut Vec<RwLock<Leaf<P, DIM, LSHAPE, T>>>,
        removed: &mut Vec<Leaf<P, DIM, LSHAPE, T>>,
        to_reinsert: &mut Vec<Leaf<P, DIM, LSHAPE, T>>,
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
               child.expand_rect_to_fit(mbr);
            }
        }
// repack non-removed children
        *children = leaf_children.pack_rwlocks();
        true
    }

/// Consume all child leaves and queue them for reinsert
    fn consume_leaves_for_reinsert(&self, nodes: &mut Vec<MbrNode<P, DIM, LSHAPE, T>>, to_reinsert: &mut Vec<Leaf<P, DIM, LSHAPE, T>>) {
        for node in nodes {
            match *node {
                MbrNode::Leaves{ref mut children, ..} => to_reinsert.append(&mut mem::replace(children, Vec::with_capacity(0)).unpack_rwlocks()),
                MbrNode::Level{ref mut children, ..} => self.consume_leaves_for_reinsert(children, to_reinsert)
            }
        }
    }

/// Recursively remove leaves from a level. Return true if the level should be retianed
    fn remove_leaves_from_level<F: FnMut(&T) -> bool>(&self, query: &MbrQuery<P, DIM>, level: &mut MbrNode<P, DIM, LSHAPE, T>,
        removed: &mut Vec<Leaf<P, DIM, LSHAPE, T>>,
        to_reinsert: &mut Vec<Leaf<P, DIM, LSHAPE, T>>,
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
                            child.expand_rect_to_fit(mbr);
                        }
                    }
                }
            }
            true
        }
}


impl<P, DIM, LSHAPE, T, I> IndexRemove<P, DIM, LSHAPE, T, I> for RRemove<P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        LSHAPE: Shape<P, DIM>,
        I: IndexInsert<P, DIM, LSHAPE, T>,
{
    fn remove_from_root<F: FnMut(&T) -> bool>(&self,
        mut root: MbrNode<P, DIM, LSHAPE, T>,
        insert_index: &I,
        query: MbrQuery<P, DIM>,
        mut f: F) -> RemoveReturn<P, DIM, LSHAPE, T> {

            if root.is_empty() {
                (root, Vec::with_capacity(0))
            } else {
                let mut to_reinsert = Vec::new();
                let mut removed = Vec::new();
                self.remove_leaves_from_level(&query, &mut root, &mut removed, &mut to_reinsert, &mut f, AT_ROOT);
// Insert algorithms require an empty root to be for leaves
                if root.is_empty() && root.has_levels() {
                    root = MbrNode::new_leaves();
                }
                for leaf in to_reinsert {
                    root = insert_index.insert_into_root(root, leaf);
                }
                (root, removed)
            }
        }
}
