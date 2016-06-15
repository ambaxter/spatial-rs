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
use typenum::Unsigned;
use generic_array::ArrayLength;
use index::{IndexInsert, IndexRemove};
use tree::{LevelNode, Query, Leaf};
use std::marker::PhantomData;

pub struct RRemove<P, DIM, SHAPE, MIN, MAX, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          MIN: Unsigned,
          MAX: Unsigned
{
    _p: PhantomData<P>,
    _dim: PhantomData<DIM>,
    _shape: PhantomData<SHAPE>,
    _min: PhantomData<MIN>,
    _max: PhantomData<MAX>,
    _t: PhantomData<T>,
}

impl<P, DIM, SHAPE, MIN, MAX, T> RRemove<P, DIM, SHAPE, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)>,
        SHAPE: Shape<P, DIM>,
        MIN: Unsigned,
        MAX: Unsigned
{
    pub fn new() -> RRemove<P, DIM, SHAPE, MIN, MAX, T> {
        RRemove{
            _dim: PhantomData,
            _p: PhantomData,
            _shape: PhantomData,
            _min: PhantomData,
            _max: PhantomData,
            _t: PhantomData
        }
    }

// Removes matching leaves from a leaf level
// Returns if parent node should retain this
    fn remove_matching_leaves<F: FnMut(&T) -> bool>(&self, query: &Query<P, DIM, SHAPE, T>, mbr: &mut Rect<P, DIM>, children: &mut Vec<Leaf<P, DIM, SHAPE, T>>,
        removed: &mut Vec<Leaf<P, DIM, SHAPE, T>>,
        to_reinsert: &mut Vec<Leaf<P, DIM, SHAPE, T>>,
        f: &mut F) -> bool {

        let orig_len = children.len();
        children.retain_and_append(removed, |leaf| !query.accept_leaf(leaf) && !f(&leaf.item));
        let children_removed = orig_len != children.len();
        if children.len() < MIN::to_usize() {
            to_reinsert.append(children);
            return false;
        }
        if children_removed {
            *mbr = Rect::max_inverted();
            for child in &*children {
               child.expand_rect_to_fit(mbr);
            }
        }
        true
    }

// Consume all child leaves and queue them for reinsert
    fn consume_leaves_for_reinsert(&self, children: &mut Vec<LevelNode<P, DIM, SHAPE, T>>, to_reinsert: &mut Vec<Leaf<P, DIM, SHAPE, T>>) {
        for ref mut child in children {
            match *child {
                &mut LevelNode::Leaves{ref mut children, ..} => to_reinsert.append(children),
                &mut LevelNode::Level{ref mut children, ..} => self.consume_leaves_for_reinsert(children, to_reinsert)
            }
        }
    }

    fn remove_leaves_from_level<F: FnMut(&T) -> bool>(&self, query: &Query<P, DIM, SHAPE, T>, level: &mut LevelNode<P, DIM, SHAPE, T>,
        removed: &mut Vec<Leaf<P, DIM, SHAPE, T>>,
        to_reinsert: &mut Vec<Leaf<P, DIM, SHAPE, T>>,
        f: &mut F) -> bool {
            if !query.accept_level(level) {
                return true;
            }
            match level {
                &mut LevelNode::Leaves{ref mut mbr, ref mut children, ..} => return self.remove_matching_leaves(query, mbr, children, removed, to_reinsert, f),
                &mut LevelNode::Level{ref mut mbr, ref mut children, ..} => {
                    let orig_len = children.len();
                    children.retain_mut(|child| self.remove_leaves_from_level(query, child, removed, to_reinsert, f));
                    let children_removed = orig_len != children.len();
                    if children.len() < MIN::to_usize() {
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


impl<P, DIM, SHAPE, MIN, MAX, T, I> IndexRemove<P, DIM, SHAPE, T, I> for RRemove<P, DIM, SHAPE, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        DIM: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        SHAPE: Shape<P, DIM>,
        I: IndexInsert<P, DIM, SHAPE, T>,
        MIN: Unsigned,
        MAX: Unsigned
{
    fn remove_from_root<F: FnMut(&T) -> bool>(&self,
        root: Option<LevelNode<P, DIM, SHAPE, T>>,
        insert_index: &I,
        query: Query<P, DIM, SHAPE, T>,
        mut f: F) -> (Option<LevelNode<P, DIM, SHAPE, T>>, Vec<Leaf<P, DIM, SHAPE, T>>) {

            if root.is_none() {
                (None, Vec::with_capacity(0))
            } else {
                let mut root_node = root.unwrap();
                let mut to_reinsert = Vec::new();
                let mut removed = Vec::new();
                let root_empty =
                    self.remove_leaves_from_level(&query, &mut root_node, &mut removed, &mut to_reinsert, &mut f);
                if root_empty {
                    *root_node.mbr_mut() = Rect::max_inverted();
                }
                for leaf in to_reinsert {
                    root_node = insert_index.insert_into_root(Some(root_node), leaf).unwrap();
                }
                (Some(root_node), removed)
            }
        }
}
