// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
pub mod index;

use num::{Signed, Float, Bounded, ToPrimitive, FromPrimitive};
use std::ops::{MulAssign, AddAssign};
use shapes::{Shape, Rect};
use std::slice::Iter as SliceIter;
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::rc::Rc;
use tree::mbr::index::{IndexInsert, IndexRemove};
use tree::{Leaf, SpatialQuery};
use std::ops::Deref;
use std::mem;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Rect based query
#[derive(Debug, Clone)]
pub enum MbrQuery<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Matching leaves are ones that are completely contained by this rect
    ContainedBy(Rect<P, DIM>),
    /// Matching leaves are ones that overlap this rect
    Overlaps(Rect<P, DIM>),
}

impl<P, DIM, LSHAPE, T> SpatialQuery<P, DIM, LSHAPE, MbrNode<P, DIM, LSHAPE, T>, T> for MbrQuery<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LSHAPE: Shape<P, DIM>{

// Does this query accept the given leaf?
    fn accept_leaf(&self, leaf: &Leaf<P, DIM, LSHAPE, T>) -> bool {
        match *self {
            MbrQuery::ContainedBy(ref query) => leaf.shape.contained_by_rect(query),
            MbrQuery::Overlaps(ref query) => leaf.shape.overlapped_by_rect(query),
        }
    }

// Does this query accept the given level?
    fn accept_level(&self, level: &MbrNode<P, DIM, LSHAPE, T>) -> bool {
        match *self {
            MbrQuery::ContainedBy(ref query) => level.overlapped_by_rect(query),
            MbrQuery::Overlaps(ref query) => level.overlapped_by_rect(query),
        }
    }
}

/// Level node of a tree. Either contains other levels or leaves
#[derive(Debug)]
pub enum MbrNode<P, DIM, LSHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Contains only other levels
    Level {
        mbr: Rect<P, DIM>,
        children: Vec<MbrNode<P, DIM, LSHAPE, T>>,
    },
    /// Contains only leaves
    Leaves {
        mbr: Rect<P, DIM>,
        children: Vec<RwLock<Leaf<P, DIM, LSHAPE, T>>>,
    },
}

impl<P, DIM, LSHAPE, T> MbrNode<P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LSHAPE: Shape<P, DIM> {

/// Create an empty leaf level
    pub fn new_leaves() -> MbrNode<P, DIM, LSHAPE, T> {
        MbrNode::Leaves{mbr: Rect::max_inverted(), children: Vec::new()}
    }

/// Create an empty leaf level with no capacity for leaves.
/// Only used for passing ownership of root into the index functions
    fn new_no_alloc() -> MbrNode<P, DIM, LSHAPE, T> {
        MbrNode::Leaves{mbr: Rect::max_inverted(), children: Vec::with_capacity(0)}
    }

/// Does the level point to leaves?
    pub fn has_leaves(&self) -> bool {
        match *self {
            MbrNode::Level{..} => false,
            MbrNode::Leaves{..} => true,
        }
    }

/// Does the level point to other levels?
    pub fn has_levels(&self) -> bool {
        !self.has_leaves()
    }

/// Borrow the level's minimum bounding rectangle
    pub fn mbr(&self) -> &Rect<P, DIM> {
        match *self {
            MbrNode::Level{ref mbr, ..} => mbr,
            MbrNode::Leaves{ref mbr, ..} => mbr,
        }
    }

/// Mutably borrow the level's minimum bounding rectangle
    pub fn mbr_mut(&mut self) -> &mut Rect<P, DIM> {
        match *self {
            MbrNode::Level{ref mut mbr, ..} => mbr,
            MbrNode::Leaves{ref mut mbr, ..} => mbr,
        }
    }

/// Number of level's children
    pub fn len(&self) -> usize {
        match *self {
            MbrNode::Level{ref children, ..} => children.len(),
            MbrNode::Leaves{ref children, ..} => children.len(),
        }
    }

/// Does the level have children?
    pub fn is_empty(&self) -> bool {
        match *self {
            MbrNode::Level{ref children, ..} => children.is_empty(),
            MbrNode::Leaves{ref children, ..} => children.is_empty(),
        }
    }
}


impl<P, DIM, LSHAPE, T> Shape<P, DIM> for MbrNode<P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          LSHAPE: Shape<P, DIM> {

    fn dim(&self) -> usize {
        self.mbr().dim()
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, DIM>) {
        self.mbr().expand_rect_to_fit(edges)
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, DIM>) -> P {
        self.mbr().distance_from_rect_center(edges)
    }

    fn contained_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.mbr().contained_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, DIM>) -> bool {
        self.mbr().overlapped_by_rect(edges)
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, DIM>) -> P {
        self.mbr().area_overlapped_with_rect(edges)
    }

    fn area(&self) -> P {
        self.mbr().area()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.mbr().min_for_axis(dim)
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.mbr().max_for_axis(dim)
    }
}

/// The generic container interface for spatial maps. Will, at the very least, be able to support R, R+, R*, and X trees
pub struct MbrMap<P, DIM, LSHAPE, I, R, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, DIM, LSHAPE, T>,
          R: IndexRemove<P, DIM, LSHAPE, T, I>
{
    insert_index: I,
    remove_index: R,
    root: MbrNode<P, DIM, LSHAPE, T>,
    len: usize,
}

impl<P, DIM, LSHAPE, I, R, T> MbrMap<P, DIM, LSHAPE, I, R, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          LSHAPE: Shape<P, DIM>,
          I: IndexInsert<P, DIM, LSHAPE, T>,
          R: IndexRemove<P, DIM, LSHAPE, T, I>,
{

/// Create a new MbrMap with the given insert and remove indexes
    pub fn new(insert_index: I, remove_index: R) -> MbrMap<P, DIM, LSHAPE, I, R, T> {
        MbrMap{insert_index: insert_index, remove_index: remove_index, root: MbrNode::new_leaves(), len: 0}
    }

/// Insert an item
    pub fn insert(&mut self, shape: LSHAPE, item: T) {
        self.root = self.insert_index.insert_into_root(mem::replace(&mut self.root, MbrNode::new_no_alloc()), Leaf::new(shape, item));
        self.len += 1;
    }

/// Remove all items whose shapes are accepted by the query
    pub fn remove(&mut self, query: MbrQuery<P, DIM>) -> Vec<(LSHAPE, T)> {
        self.retain(query, |_| false)
    }

/// Remove all items whose shapes are accepted by the query and where f(&T) returns false
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, query: MbrQuery<P, DIM>, f: F) -> Vec<(LSHAPE, T)> {
        let (new_root, removed) =
            self.remove_index.remove_from_root(mem::replace(&mut self.root, MbrNode::new_no_alloc()), &self.insert_index, query, f);
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
        self.root = MbrNode::new_leaves();
        self.len = 0;
    }

/// Iter for the map
    pub fn iter(&self) -> Iter<P, DIM, LSHAPE, T> {
        Iter::new(MbrQuery::Overlaps(Rect::max()), &self.root)
    }

/// IterMut for the map
    pub fn iter_mut(&self) -> IterMut<P, DIM, LSHAPE, T> {
        IterMut::new(MbrQuery::Overlaps(Rect::max()), &self.root)
    }

/// Iter for the map with a given query
    pub fn iter_query(&self, query: MbrQuery<P, DIM>) -> Iter<P, DIM, LSHAPE, T> {
        Iter::new(query, &self.root)
    }

/// IterMut for the map with a given query
    pub fn iter_query_mut(&self, query: MbrQuery<P, DIM>) -> IterMut<P, DIM, LSHAPE, T> {
        IterMut::new(query, &self.root)
    }

}

type LeafIter<'tree, P, DIM, LSHAPE, T> = SliceIter<'tree, RwLock<Leaf<P, DIM, LSHAPE, T>>>;

/// Iterate through all `MbrNode::Leaves` matching a query
struct LevelIter<'tree, P, DIM, LSHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LSHAPE: 'tree,
          T: 'tree
{
    query: Rc<MbrQuery<P, DIM>>,
    root: &'tree MbrNode<P, DIM, LSHAPE, T>,
    level_stack: Vec<SliceIter<'tree, MbrNode<P, DIM, LSHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LSHAPE, T> LevelIter<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM> + 'tree,
          T: 'tree {

/// Constructor
    fn new(query: Rc<MbrQuery<P, DIM>>, root: &'tree MbrNode<P, DIM, LSHAPE, T>) -> LevelIter<'tree, P, DIM, LSHAPE, T> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIter{query: query, root: root, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIter{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

/// Select the next matching leaves level
    fn next_leaves(&mut self, mut m_iter: SliceIter<'tree, MbrNode<P, DIM, LSHAPE, T>>) -> Option<LeafIter<'tree, P, DIM, LSHAPE, T>> {
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

impl<'tree, P, DIM, LSHAPE, T> Iterator for LevelIter<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM> + 'tree,
          T: 'tree {
    type Item = LeafIter<'tree, P, DIM, LSHAPE, T>;

    fn next(&mut self) -> Option<LeafIter<'tree, P, DIM, LSHAPE, T>> {
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
pub struct Iter<'tree, P, DIM, LSHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LSHAPE: 'tree,
          T: 'tree
{
    query: Rc<MbrQuery<P, DIM>>,
    level_iter: LevelIter<'tree, P, DIM, LSHAPE, T>,
    leaf_iter: Option<LeafIter<'tree, P, DIM, LSHAPE, T>>,
    leaf_lock: Option<RwLockReadGuard<'tree, Leaf<P, DIM, LSHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LSHAPE, T> Iter<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
/// Constructor
    fn new(query: MbrQuery<P, DIM>, root: &'tree MbrNode<P, DIM, LSHAPE, T>) -> Iter<'tree, P, DIM, LSHAPE, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        Iter{query: rc_query, level_iter: level_iter, leaf_iter: None, leaf_lock: None,  finished: false}
    }

    unsafe fn leaf_as_tuple(&self) -> (&'tree LSHAPE, &'tree T) {
        let leaf: *const Option<RwLockReadGuard<'tree, Leaf<P, DIM, LSHAPE, T>>> = &self.leaf_lock;
        (*leaf).as_ref().unwrap().as_tuple()
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIter<'tree, RwLock<Leaf<P, DIM, LSHAPE, T>>>) -> Option<(&'tree LSHAPE, &'tree T)> {
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

impl<'tree, P, DIM, LSHAPE, T> Iterator for Iter<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
    type Item = (&'tree LSHAPE, &'tree T);

    fn next(&mut self) -> Option<(&'tree LSHAPE, &'tree T)> {
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
pub struct IterMut<'tree, P, DIM, LSHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          LSHAPE: 'tree,
          T: 'tree
{
    query: Rc<MbrQuery<P, DIM>>,
    level_iter: LevelIter<'tree, P, DIM, LSHAPE, T>,
    leaf_iter: Option<LeafIter<'tree, P, DIM, LSHAPE, T>>,
    leaf_lock: Option<RwLockWriteGuard<'tree, Leaf<P, DIM, LSHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, LSHAPE, T> IterMut<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
/// Constructor
    fn new(query: MbrQuery<P, DIM>, root: &'tree MbrNode<P, DIM, LSHAPE, T>) -> IterMut<'tree, P, DIM, LSHAPE, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        IterMut{query: rc_query, level_iter: level_iter, leaf_iter: None, leaf_lock: None, finished: false}
    }

    unsafe fn leaf_as_mut_tuple(&mut self) -> (&'tree LSHAPE, &'tree mut T) {
        let leaf: *mut Option<RwLockWriteGuard<'tree, Leaf<P, DIM, LSHAPE, T>>> = &mut self.leaf_lock;
        (*leaf).as_mut().unwrap().as_mut_tuple()
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIter<'tree, RwLock<Leaf<P, DIM, LSHAPE, T>>>) -> Option<(&'tree LSHAPE, &'tree mut T)> {
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

impl<'tree, P, DIM, LSHAPE, T> Iterator for IterMut<'tree, P, DIM, LSHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          LSHAPE: Shape<P, DIM>,
          T: 'tree
{
    type Item = (&'tree LSHAPE, &'tree mut T);

    fn next(&mut self) -> Option<(&'tree LSHAPE, &'tree mut T)> {
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
