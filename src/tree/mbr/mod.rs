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
use std::slice::{Iter as SliceIter, IterMut as SliceIterMut};
use std::fmt::Debug;
use generic_array::ArrayLength;
use std::marker::PhantomData;
use std::rc::Rc;
use index::{IndexInsert, IndexRemove};
use std::ops::Deref;
use std::mem;

/// Rect based query
#[derive(Debug, Clone)]
pub enum RectQuery<P, DIM>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Matching leaves are ones that are completely contained by this rect
    ContainedBy(Rect<P, DIM>),
    /// Matching leaves are ones that overlap this rect
    Overlaps(Rect<P, DIM>),
}

impl<P, DIM, SHAPE, T> MbrMapQuery<P, DIM, SHAPE, T> for RectQuery<P, DIM>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          SHAPE: Shape<P, DIM> {

    // Does this query accept the given leaf?
    fn accept_leaf(&self, leaf: &Leaf<P, DIM, SHAPE, T>) -> bool {
        match *self {
            RectQuery::ContainedBy(ref query) => leaf.shape.contained_by_rect(query),
            RectQuery::Overlaps(ref query) => leaf.shape.overlapped_by_rect(query),
        }
    }

    // Does this query accept the given level?
    fn accept_level(&self, level: &MbrNode<P, DIM, SHAPE, T>) -> bool {
        match *self {
            RectQuery::ContainedBy(ref query) => level.mbr().overlapped_by_rect(query),
            RectQuery::Overlaps(ref query) => level.mbr().overlapped_by_rect(query),
        }
    }
}

/// Level node of a tree. Either contains other levels or leaves
#[derive(Debug)]
pub enum MbrNode<P, DIM, SHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    /// Contains only other levels
    Level {
        mbr: Rect<P, DIM>,
        children: Vec<MbrNode<P, DIM, SHAPE, T>>,
    },
    /// Contains only leaves
    Leaves {
        mbr: Rect<P, DIM>,
        children: Vec<Leaf<P, DIM, SHAPE, T>>,
    },
}

impl<P, DIM, SHAPE, T> MbrNode<P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          SHAPE: Shape<P, DIM> {

    /// Create an empty leaf level
    pub fn new_leaves() -> MbrNode<P, DIM, SHAPE, T> {
        MbrNode::Leaves{mbr: Rect::max_inverted(), children: Vec::new()}
    }

    /// Create an empty leaf level with no capacity for leaves.
    /// Only used for passing ownership of root into the index functions
    fn new_no_alloc() -> MbrNode<P, DIM, SHAPE, T> {
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


impl<P, DIM, SHAPE, T> Shape<P, DIM> for MbrNode<P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)>,
          SHAPE: Shape<P, DIM> {

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
pub struct MbrMap<P, DIM, SHAPE, I, R, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, DIM, SHAPE, T>,
          R: IndexRemove<P, DIM, SHAPE, T, I>
{
    insert_index: I,
    remove_index: R,
    root: MbrNode<P, DIM, SHAPE, T>,
    len: usize,
}

impl<P, DIM, SHAPE, I, R, T> MbrMap<P, DIM, SHAPE, I, R, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          SHAPE: Shape<P, DIM>,
          I: IndexInsert<P, DIM, SHAPE, T>,
          R: IndexRemove<P, DIM, SHAPE, T, I>,
{

    /// Create a new MbrMap with the given insert and remove indexes
    pub fn new(insert_index: I, remove_index: R) -> MbrMap<P, DIM, SHAPE, I, R, T> {
        MbrMap{insert_index: insert_index, remove_index: remove_index, root: MbrNode::new_leaves(), len: 0}
    }

    /// Insert an item
    pub fn insert(&mut self, shape: SHAPE, item: T) {
        self.root = self.insert_index.insert_into_root(mem::replace(&mut self.root, MbrNode::new_no_alloc()), Leaf::new(shape, item));
        self.len += 1;
    }

    /// Remove all items whose shapes are accepted by the query
    pub fn remove(&mut self, query: RectQuery<P, DIM>) -> Vec<(SHAPE, T)> {
        self.retain(query, |_| false)
    }

    /// Remove all items whose shapes are accepted by the query and where f(&T) returns false
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, query: RectQuery<P, DIM>, f: F) -> Vec<(SHAPE, T)> {
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
    pub fn iter(&self) -> Iter<P, DIM, SHAPE, T> {
        Iter::new(RectQuery::Overlaps(Rect::max()), &self.root)
    }

    /// IterMut for the map
    pub fn iter_mut(&mut self) -> IterMut<P, DIM, SHAPE, T> {
        IterMut::new(RectQuery::Overlaps(Rect::max()), &mut self.root)
    }

    /// Iter for the map with a given query
    pub fn iter_query(&self, query: RectQuery<P, DIM>) -> Iter<P, DIM, SHAPE, T> {
        Iter::new(query, &self.root)
    }

    /// IterMut for the map with a given query
    pub fn iter_query_mut(&mut self, query: RectQuery<P, DIM>) -> IterMut<P, DIM, SHAPE, T> {
        IterMut::new(query, &mut self.root)
    }
}

/// Iter all `Leaf` items matching a query
pub struct Iter<'tree, P, DIM, SHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          SHAPE: 'tree,
          T: 'tree
{
    query: Rc<RectQuery<P, DIM>>,
    level_iter: LevelIter<'tree, P, DIM, SHAPE, T>,
    leaf_iter: Option<SliceIter<'tree, Leaf<P, DIM, SHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, SHAPE, T> Iter<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
    /// Constructor
    fn new(query: RectQuery<P, DIM>, root: &'tree MbrNode<P, DIM, SHAPE, T>) -> Iter<'tree, P, DIM, SHAPE, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        Iter{query: rc_query, level_iter: level_iter, leaf_iter: None, finished: false}
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIter<'tree, Leaf<P, DIM, SHAPE, T>>) -> Option<(&'tree SHAPE, &'tree T)> {
        while let Some(ref mut leaf) = iter.next() {
                if !self.query.accept_leaf(leaf) {
                    continue;
                }
                self.leaf_iter = Some(iter);
                return Some((&leaf.shape, &leaf.item));
        }
        None
    }
}

impl<'tree, P, DIM, SHAPE, T> Iterator for Iter<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
    type Item = (&'tree SHAPE, &'tree T);

    fn next(&mut self) -> Option<(&'tree SHAPE, &'tree T)> {
        if self.finished {
            return None;
        }
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

/// Iterate through all `MbrNode::Leaves` matching a query
struct LevelIter<'tree, P, DIM, SHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          SHAPE: 'tree,
          T: 'tree
{
    query: Rc<RectQuery<P, DIM>>,
    root: &'tree MbrNode<P, DIM, SHAPE, T>,
    level_stack: Vec<SliceIter<'tree, MbrNode<P, DIM, SHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, SHAPE, T> LevelIter<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree {
    
    /// Constructor
    fn new(query: Rc<RectQuery<P, DIM>>, root: &'tree MbrNode<P, DIM, SHAPE, T>) -> LevelIter<'tree, P, DIM, SHAPE, T> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIter{query: query, root: root, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIter{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

    /// Select the next matching leaves level
    fn next_leaves(&mut self, mut m_iter: SliceIter<'tree, MbrNode<P, DIM, SHAPE, T>>) -> Option<SliceIter<'tree, Leaf<P, DIM, SHAPE, T>>> {
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

impl<'tree, P, DIM, SHAPE, T> Iterator for LevelIter<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree {
    type Item = SliceIter<'tree, Leaf<P, DIM, SHAPE, T>>;

    fn next(&mut self) -> Option<SliceIter<'tree, Leaf<P, DIM, SHAPE, T>>> {
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

/// Mutably iterate all `Leaf` entries matching a query
pub struct IterMut<'tree, P, DIM, SHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          SHAPE: 'tree,
          T: 'tree
{
    query: Rc<RectQuery<P, DIM>>,
    level_iter: LevelIterMut<'tree, P, DIM, SHAPE, T>,
    leaf_iter: Option<SliceIterMut<'tree, Leaf<P, DIM, SHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, SHAPE, T> IterMut<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree
{
    /// Constructor
    fn new(query: RectQuery<P, DIM>, root: &'tree mut MbrNode<P, DIM, SHAPE, T>) -> IterMut<'tree, P, DIM, SHAPE, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIterMut::new(rc_query.clone(), root);
        IterMut{query: rc_query, level_iter: level_iter, leaf_iter: None, finished: false}
    }

    /// Select the next matching leaf
    fn next_leaf(&mut self, mut iter: SliceIterMut<'tree, Leaf<P, DIM, SHAPE, T>>) -> Option<(&'tree SHAPE, &'tree mut T)> {
        while let Some(ref mut leaf) = iter.next() {
                if !self.query.deref().accept_leaf(leaf) {
                    continue;
                }
                self.leaf_iter = Some(iter);
                // The lifetime checker doesn't understand that the leaf isn't going to be going anywhere. There must be a better way, though
                let mut cast_leaf = unsafe {
                    let cast_leaf: *mut Leaf<P, DIM, SHAPE, T> = *leaf;
                    &mut *cast_leaf
                };
                return Some((&cast_leaf.shape, &mut cast_leaf.item));
        }
        None
    }
}

impl<'tree, P, DIM, SHAPE, T> Iterator for IterMut<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM>,
          T: 'tree
{
    type Item = (&'tree SHAPE, &'tree mut T);

    fn next(&mut self) -> Option<(&'tree SHAPE, &'tree mut T)> {
        if self.finished {
            return None;
        }
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

/// Mutably iterate through all `MbrNode::Leaves` matching a query
struct LevelIterMut<'tree, P, DIM, SHAPE, T>
    where P: 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          SHAPE: 'tree,
          T: 'tree
{
    query: Rc<RectQuery<P, DIM>>,
    root: &'tree mut MbrNode<P, DIM, SHAPE, T>,
    level_stack: Vec<SliceIterMut<'tree, MbrNode<P, DIM, SHAPE, T>>>,
    finished: bool,
}

impl<'tree, P, DIM, SHAPE, T> LevelIterMut<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default +'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM> + 'tree,
          T: 'tree {

    /// Constructor
    fn new(query: Rc<RectQuery<P, DIM>>, root: &'tree mut MbrNode<P, DIM, SHAPE, T>) -> LevelIterMut<'tree, P, DIM, SHAPE, T> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIterMut{query: query, root: root, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIterMut{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

    // access the root node. Must be unsafe because &'a mut self and &'tree mut self.root borrows cause a conflict
    // We're not modifying the tree structure, just the leaf nodes, so this should be ok
    unsafe fn get_root_node(&mut self) -> &'tree mut MbrNode<P, DIM, SHAPE, T> {
        let root: *mut MbrNode<P, DIM, SHAPE, T> = self.root;
        &mut *root
    }

    /// Select the next matching leaves level
    fn next_leaves(&mut self, mut m_iter: SliceIterMut<'tree, MbrNode<P, DIM, SHAPE, T>>) -> Option<SliceIterMut<'tree, Leaf<P, DIM, SHAPE, T>>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match *node {
                MbrNode::Leaves{ref mut children, ..} => return Some(children.iter_mut()),
                MbrNode::Level{ref mut children, ..} => {
                    let next = self.next_leaves(children.iter_mut());
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


impl<'tree, P, DIM, SHAPE, T> Iterator for LevelIterMut<'tree, P, DIM, SHAPE, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          DIM: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          SHAPE: Shape<P, DIM>,
          T: 'tree {
    type Item = SliceIterMut<'tree, Leaf<P, DIM, SHAPE, T>>;

    fn next(&mut self) -> Option<SliceIterMut<'tree, Leaf<P, DIM, SHAPE, T>>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match *unsafe {self.get_root_node()} {
                MbrNode::Leaves{ref mut children, ..} => {
                    self.finished = true;
                    return Some(children.iter_mut());
                },
                MbrNode::Level{ref mut children, ..} => {
                    self.level_stack.push(children.iter_mut());
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
