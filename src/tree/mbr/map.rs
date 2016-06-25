// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.


use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use shapes::Rect;
use std::slice::Iter as SliceIter;
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::rc::Rc;
use tree::mbr::index::{IndexInsert, IndexRemove};
use tree::mbr::{MbrLeafShape, MbrLeaf, MbrRectQuery, MbrQuery, MbrNode};
use std::ops::Deref;
use std::mem;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The generic container interface for spatial maps. Will, at the very least, be able to support R, R+, R*, and X trees
pub struct MbrMap<P, DIM, LS, I, R, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, DIM, LS, T>,
          R: IndexRemove<P, DIM, LS, T, I>
{
    insert_index: I,
    remove_index: R,
    root: MbrNode<P, DIM, LS, T>,
    len: usize,
}

impl<P, DIM, LS, I, R, T> MbrMap<P, DIM, LS, I, R, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          LS: MbrLeafShape<P, DIM>,
          I: IndexInsert<P, DIM, LS, T>,
          R: IndexRemove<P, DIM, LS, T, I>,
{

/// Create a new MbrMap with the given insert and remove indexes
    pub fn new(insert_index: I, remove_index: R) -> MbrMap<P, DIM, LS, I, R, T> {
        let new_root = insert_index.new_leaves();
        MbrMap{insert_index: insert_index, remove_index: remove_index, root: new_root, len: 0}
    }

/// Insert an item
    pub fn insert(&mut self, shape: LS, item: T) {
        self.root = self.insert_index.insert_into_root(mem::replace(&mut self.root, self.insert_index.new_no_alloc_leaves()), MbrLeaf::new(shape, item));
        self.len += 1;
    }

/// Remove all items whose shapes are accepted by the query. Returns removed entries.
    pub fn remove<Q: MbrQuery<P, DIM, LS, T>>(&mut self, query: Q) -> Vec<(LS, T)> {
        self.retain(query, |_| false)
    }

/// Remove all items whose shapes are accepted by the query and where f(&T) returns false. Returns removed entries
    pub fn retain<Q: MbrQuery<P, DIM, LS, T>, F: FnMut(&T) -> bool>(&mut self, query: Q, f: F) -> Vec<(LS, T)> {
        let (new_root, removed) =
            self.remove_index.remove_from_root(mem::replace(&mut self.root, self.insert_index.new_no_alloc_leaves()), &self.insert_index, query, f);
        self.len -= removed.len();
        self.root = new_root;
        let mut removed_extract = Vec::with_capacity(removed.len());
        for leaf in removed {
            removed_extract.push(leaf.extract());
        }
        removed_extract
    }

/// Whether the map is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

/// Length of the map
    pub fn len(&self) -> usize {
        self.len
    }

/// Clear the map
    pub fn clear(&mut self) {
        self.root = self.insert_index.new_leaves();
        self.len = 0;
    }

/// Iter for the map
    pub fn iter(&self) -> Iter<P, DIM, LS, T, MbrRectQuery<P, DIM>> {
        Iter::new(MbrRectQuery::Overlaps(Rect::max()), &self.root)
    }

/// IterMut for the map
    pub fn iter_mut(&self) -> IterMut<P, DIM, LS, T, MbrRectQuery<P, DIM>> {
        IterMut::new(MbrRectQuery::Overlaps(Rect::max()), &self.root)
    }

/// Iter for the map with a given query
    pub fn iter_query<Q: MbrQuery<P, DIM, LS, T>>(&self, query: Q) -> Iter<P, DIM, LS, T, Q> {
        Iter::new(query, &self.root)
    }

/// IterMut for the map with a given query
    pub fn iter_query_mut<Q: MbrQuery<P, DIM, LS, T>>(&self, query: Q) -> IterMut<P, DIM, LS, T, Q> {
        IterMut::new(query, &self.root)
    }

}

type LeafIter<'tree, P, DIM, LS, T> = SliceIter<'tree, RwLock<MbrLeaf<P, DIM, LS, T>>>;

/// Iterate through all `MbrNode::Leaves` matching a query
struct LevelIter<'tree, P, DIM, LS, T, Q>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LS: 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
    query: Rc<Q>,
    root: &'tree MbrNode<P, DIM, LS, T>,
    level_stack: Vec<SliceIter<'tree, MbrNode<P, DIM, LS, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LS, T, Q> LevelIter<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM> + 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{

/// Constructor
    fn new(query: Rc<Q>, root: &'tree MbrNode<P, DIM, LS, T>) -> LevelIter<'tree, P, DIM, LS, T, Q> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIter{query: query, root: root, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIter{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

/// Select the next matching leaves level
    fn next_leaves(&mut self, mut m_iter: SliceIter<'tree, MbrNode<P, DIM, LS, T>>) -> Option<LeafIter<'tree, P, DIM, LS, T>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match *node {
                MbrNode::Leaves{ref children, ..} => return Some(children.iter()),
                MbrNode::Level{ref children, ..} => {
                    let next = self.next_leaves(children.iter());
                    if next.is_none() {
                        m_iter = self.level_stack.pop().unwrap();
                        iter_node = m_iter.next();
                        continue;
                    }
                    return next;
                }
            }
        }
        None
    }
}

impl<'tree, P, DIM, LS, T, Q> Iterator for LevelIter<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM> + 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T> 
{
    type Item = LeafIter<'tree, P, DIM, LS, T>;

    fn next(&mut self) -> Option<LeafIter<'tree, P, DIM, LS, T>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match *self.root {
                MbrNode::Leaves{ref children, ..} => {
                    self.finished = true;
                    return Some(children.iter());
                },
                MbrNode::Level{ref children, ..} => {
                    self.level_stack.push(children.iter());
                }
            }
        }
        let mut m_iter = self.level_stack.pop().unwrap();
        let mut next = self.next_leaves(m_iter);
        while let None = next {
            if !self.level_stack.is_empty() {
                m_iter = self.level_stack.pop().unwrap();
                next = self.next_leaves(m_iter);
            } else {
                self.finished = true;
                break;
            }
        }
        next
    }
}

/// Iter all `Leaf` items matching a query
pub struct Iter<'tree, P, DIM, LS, T, Q>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LS: 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
    query: Rc<Q>,
    level_iter: LevelIter<'tree, P, DIM, LS, T, Q>,
    leaf_iter: Option<LeafIter<'tree, P, DIM, LS, T>>,
    leaf_lock: Option<RwLockReadGuard<'tree, MbrLeaf<P, DIM, LS, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LS, T, Q> Iter<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM> + 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
/// Constructor
    fn new(query: Q, root: &'tree MbrNode<P, DIM, LS, T>) -> Iter<'tree, P, DIM, LS, T, Q> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        Iter{query: rc_query, level_iter: level_iter, leaf_iter: None, leaf_lock: None,  finished: false}
    }

    unsafe fn leaf_as_tuple(&self) -> (&'tree LS, &'tree T) {
        let leaf: *const Option<RwLockReadGuard<'tree, MbrLeaf<P, DIM, LS, T>>> = &self.leaf_lock;
        (*leaf).as_ref().unwrap().as_tuple()
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIter<'tree, RwLock<MbrLeaf<P, DIM, LS, T>>>) -> Option<(&'tree LS, &'tree T)> {
        while let Some(ref mut leaf) = iter.next() {
                let leaf_lock = leaf.read();
                if !self.query.accept_leaf(&leaf_lock) {
                    continue;
                }
                self.leaf_iter = Some(iter);
                self.leaf_lock = Some(leaf_lock);
                return Some(unsafe {self.leaf_as_tuple()});
        }
        None
    }
}

impl<'tree, P, DIM, LS, T, Q> Iterator for Iter<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM> + 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
    type Item = (&'tree LS, &'tree T);

    fn next(&mut self) -> Option<(&'tree LS, &'tree T)> {
        if self.finished {
            return None;
        }
// Release the lock immediately
        self.leaf_lock = None;
        if self.leaf_iter.is_none() {
            self.leaf_iter = self.level_iter.next();
        }
        if self.leaf_iter.is_none() {
            self.finished = true;
            return None;
        }
        let iter = self.leaf_iter.take().unwrap();
        let mut next = self.next_leaf(iter);
        while let None = next {
            let leaf_iter = self.level_iter.next();
            if !leaf_iter.is_none() {
                next = self.next_leaf(leaf_iter.unwrap());
            } else {
                self.finished = true;
                break;
            }
        }
        next
    }
}

/// Mutably iterate all `Leaf` entries matching a query
pub struct IterMut<'tree, P, DIM, LS, T, Q>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LS: 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
    query: Rc<Q>,
    level_iter: LevelIter<'tree, P, DIM, LS, T, Q>,
    leaf_iter: Option<LeafIter<'tree, P, DIM, LS, T>>,
    leaf_lock: Option<RwLockWriteGuard<'tree, MbrLeaf<P, DIM, LS, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LS, T, Q> IterMut<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM> + 'tree,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
/// Constructor
    fn new(query: Q, root: &'tree MbrNode<P, DIM, LS, T>) -> IterMut<'tree, P, DIM, LS, T, Q> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        IterMut{query: rc_query, level_iter: level_iter, leaf_iter: None, leaf_lock: None, finished: false}
    }

    unsafe fn leaf_as_mut_tuple(&mut self) -> (&'tree LS, &'tree mut T) {
        let leaf: *mut Option<RwLockWriteGuard<'tree, MbrLeaf<P, DIM, LS, T>>> = &mut self.leaf_lock;
        (*leaf).as_mut().unwrap().as_mut_tuple()
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIter<'tree, RwLock<MbrLeaf<P, DIM, LS, T>>>) -> Option<(&'tree LS, &'tree mut T)> {
        while let Some(ref mut leaf) = iter.next() {
            let leaf_lock = leaf.write();
                if !self.query.deref().accept_leaf(&leaf_lock) {
                    continue;
                }
                self.leaf_iter = Some(iter);
                self.leaf_lock = Some(leaf_lock);
                return Some(unsafe {self.leaf_as_mut_tuple()});
        }
        None
    }
}

impl<'tree, P, DIM, LS, T, Q> Iterator for IterMut<'tree, P, DIM, LS, T, Q>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LS: MbrLeafShape<P, DIM>,
          T: 'tree,
          Q: MbrQuery<P, DIM, LS, T>
{
    type Item = (&'tree LS, &'tree mut T);

    fn next(&mut self) -> Option<(&'tree LS, &'tree mut T)> {
        if self.finished {
            return None;
        }
        // Release the lock immediately
        self.leaf_lock = None;
        if self.leaf_iter.is_none() {
            self.leaf_iter = self.level_iter.next();
        }
        if self.leaf_iter.is_none() {
            self.finished = true;
            return None;
        }
        let iter = self.leaf_iter.take().unwrap();
        let mut next = self.next_leaf(iter);
        while let None = next {
            let leaf_iter = self.level_iter.next();
            if !leaf_iter.is_none() {
                next = self.next_leaf(leaf_iter.unwrap());
            } else {
                self.finished = true;
                break;
            }
        }
        next
    }
}
