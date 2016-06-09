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

pub struct RRemove<P, D, S, MIN, MAX, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>,
          MIN: Unsigned,
          MAX: Unsigned
{
    _phantom_p: PhantomData<P>,
    _phantom_d: PhantomData<D>,
    _phantom_s: PhantomData<S>,
    _phantom_min: PhantomData<MIN>,
    _phantom_max: PhantomData<MAX>,
    _phantom_t: PhantomData<T>,
}

impl<P, D, S, MIN, MAX, T> RRemove<P, D, S, MIN, MAX, T>
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
        D: ArrayLength<P> + ArrayLength<(P, P)>,
        S: Shape<P, D>,
        MIN: Unsigned,
        MAX: Unsigned
{
    pub fn new() -> RRemove<P, D, S, MIN, MAX, T> {
        RRemove{
            _phantom_d: PhantomData,
            _phantom_p: PhantomData,
            _phantom_s: PhantomData,
            _phantom_min: PhantomData,
            _phantom_max: PhantomData,
            _phantom_t: PhantomData
        }
    }

// Removes matching leaves from a leaf level
// Returns if parent node should retain this
    fn remove_matching_leaves<F: FnMut(&T) -> bool>(&self, query: &Query<P, D, S, T>, mbr: &mut Rect<P, D>, children: &mut Vec<Leaf<P, D, S, T>>,
        removed: &mut Vec<Leaf<P, D, S, T>>,
        to_reinsert: &mut Vec<Leaf<P, D, S, T>>,
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
    fn consume_leaves_for_reinsert(&self, children: &mut Vec<LevelNode<P, D, S, T>>, to_reinsert: &mut Vec<Leaf<P, D, S, T>>) {
        for ref mut child in children {
            match *child {
                &mut LevelNode::Leaves{ref mut children, ..} => to_reinsert.append(children),
                &mut LevelNode::Level{ref mut children, ..} => self.consume_leaves_for_reinsert(children, to_reinsert)
            }
        }
    }

    fn remove_leaves_from_level<F: FnMut(&T) -> bool>(&self, query: &Query<P, D, S, T>, level: &mut LevelNode<P, D, S, T>,
        removed: &mut Vec<Leaf<P, D, S, T>>,
        to_reinsert: &mut Vec<Leaf<P, D, S, T>>,
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


impl<P, D, S, MIN, MAX, T, I> IndexRemove<P, D, S, T, I> for RRemove<P, D, S, MIN, MAX, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
        D: ArrayLength<P> + ArrayLength<(P, P)> + Clone,
        S: Shape<P, D>,
        I: IndexInsert<P, D, S, T>,
        MIN: Unsigned,
        MAX: Unsigned
{
    fn remove_from_root<F: FnMut(&T) -> bool>(&self,
        root: Option<LevelNode<P, D, S, T>>,
        insert_index: &I,
        query: Query<P, D, S, T>,
        mut f: F) -> (Option<LevelNode<P, D, S, T>>, Vec<Leaf<P, D, S, T>>) {

            if root.is_none() {
                return (None, Vec::with_capacity(0));
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
                return (Some(root_node), removed);
            }
        }
}
