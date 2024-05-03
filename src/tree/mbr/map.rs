// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::{Iter as SliceIter, IterMut as SliceIterMut};

use geometry::Rect;
use tree::mbr::index::{IndexInsert, IndexRemove};
use tree::mbr::{MbrLeaf, MbrLeafGeometry, MbrNode, MbrQuery, MbrRectQuery, RTreeNode};
use FP;

/// The generic container interface for spatial maps. Will, at the very least, be able to support R, R+, R*, and X trees
pub struct MbrMap<NODE, I, R> {
    insert_index: I,
    remove_index: R,
    root: NODE,
    len: usize,
}

impl<P: FP, const DIM: usize, LG, I, R, T> MbrMap<RTreeNode<P, DIM, LG, T>, I, R>
where
    LG: MbrLeafGeometry<P, DIM>,
    I: IndexInsert<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
    R: IndexRemove<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>, I>,
{
    /// Create a new MbrMap with the given insert and remove indexes
    pub fn new(insert_index: I, remove_index: R) -> MbrMap<RTreeNode<P, DIM, LG, T>, I, R> {
        let new_root = insert_index.new_leaves();
        MbrMap {
            insert_index: insert_index,
            remove_index: remove_index,
            root: new_root,
            len: 0,
        }
    }

    /// Insert an item
    pub fn insert(&mut self, geometry: LG, item: T) {
        self.root = self.insert_index.insert_into_root(
            mem::replace(&mut self.root, self.insert_index.new_no_alloc_leaves()),
            MbrLeaf::new(geometry, item),
        );
        self.len += 1;
    }

    /// Remove all items whose shapes are accepted by the query. Returns removed entries.
    pub fn remove<Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>>(
        &mut self,
        query: Q,
    ) -> Vec<(LG, T)> {
        self.retain(query, |_| false)
    }

    /// Remove all items whose shapes are accepted by the query and where f(&T) returns false. Returns removed entries
    pub fn retain<Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>, F: FnMut(&T) -> bool>(
        &mut self,
        query: Q,
        f: F,
    ) -> Vec<(LG, T)> {
        let (new_root, removed) = self.remove_index.remove_from_root(
            mem::replace(&mut self.root, self.insert_index.new_no_alloc_leaves()),
            &self.insert_index,
            query,
            f,
        );
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
    pub fn iter(&self) -> Iter<P, DIM, LG, T, MbrRectQuery<P, DIM>> {
        Iter::new(MbrRectQuery::Overlaps(Rect::max()), &self.root)
    }

    /// IterMut for the map
    pub fn iter_mut(&mut self) -> IterMut<P, DIM, LG, T, MbrRectQuery<P, DIM>> {
        IterMut::new(MbrRectQuery::Overlaps(Rect::max()), &mut self.root)
    }

    /// Iter for the map with a given query
    pub fn iter_query<Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>>(
        &self,
        query: Q,
    ) -> Iter<P, DIM, LG, T, Q> {
        Iter::new(query, &self.root)
    }

    /// IterMut for the map with a given query
    pub fn iter_query_mut<Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>>(
        &mut self,
        query: Q,
    ) -> IterMut<P, DIM, LG, T, Q> {
        IterMut::new(query, &mut self.root)
    }
}

type LeafIter<'tree, P: FP, const DIM: usize, LG, T> = SliceIter<'tree, MbrLeaf<P, DIM, LG, T>>;

/// Iterate through all `MbrNode::Leaves` matching a query
struct LevelIter<'tree, P: FP, const DIM: usize, LG, T, Q>
where
    LG: 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    query: Rc<Q>,
    root: &'tree RTreeNode<P, DIM, LG, T>,
    level_stack: Vec<SliceIter<'tree, RTreeNode<P, DIM, LG, T>>>,
    finished: bool,
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> LevelIter<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    /// Constructor
    fn new(
        query: Rc<Q>,
        root: &'tree RTreeNode<P, DIM, LG, T>,
    ) -> LevelIter<'tree, P, DIM, LG, T, Q> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIter {
                query: query,
                root: root,
                level_stack: Vec::with_capacity(0),
                finished: true,
            };
        }
        LevelIter {
            query: query,
            root: root,
            level_stack: Vec::new(),
            finished: false,
        }
    }

    /// Select the next matching leaves level
    fn next_leaves(
        &mut self,
        mut m_iter: SliceIter<'tree, RTreeNode<P, DIM, LG, T>>,
    ) -> Option<LeafIter<'tree, P, DIM, LG, T>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match *node {
                RTreeNode::Leaves { ref children, .. } => return Some(children.iter()),
                RTreeNode::Level { ref children, .. } => {
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

impl<'tree, P: FP, const DIM: usize, LG, T, Q> Iterator for LevelIter<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    type Item = LeafIter<'tree, P, DIM, LG, T>;

    fn next(&mut self) -> Option<LeafIter<'tree, P, DIM, LG, T>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match *self.root {
                RTreeNode::Leaves { ref children, .. } => {
                    self.finished = true;
                    return Some(children.iter());
                }
                RTreeNode::Level { ref children, .. } => {
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

type LeafIterMut<'tree, P: FP, const DIM: usize, LG, T> =
    SliceIterMut<'tree, MbrLeaf<P, DIM, LG, T>>;

/// Iterate mutably through all `MbrNode::Leaves` matching a query
struct LevelIterMut<'tree, P: FP, const DIM: usize, LG, T, Q>
where
    LG: 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    query: Rc<Q>,
    root: &'tree mut RTreeNode<P, DIM, LG, T>,
    level_stack: Vec<SliceIterMut<'tree, RTreeNode<P, DIM, LG, T>>>,
    finished: bool,
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> LevelIterMut<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    /// Constructor
    fn new(
        query: Rc<Q>,
        root: &'tree mut RTreeNode<P, DIM, LG, T>,
    ) -> LevelIterMut<'tree, P, DIM, LG, T, Q> {
        if root.is_empty() || !query.accept_level(root) {
            return LevelIterMut {
                query: query,
                root: root,
                level_stack: Vec::with_capacity(0),
                finished: true,
            };
        }
        LevelIterMut {
            query: query,
            root: root,
            level_stack: Vec::new(),
            finished: false,
        }
    }

    unsafe fn unpack_root_lifetime(&mut self) -> &'tree mut RTreeNode<P, DIM, LG, T> {
        let root: *mut RTreeNode<P, DIM, LG, T> = self.root;
        &mut *root
    }

    /// Select the next matching leaves level
    fn next_leaves(
        &mut self,
        mut m_iter: SliceIterMut<'tree, RTreeNode<P, DIM, LG, T>>,
    ) -> Option<LeafIterMut<'tree, P, DIM, LG, T>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match *node {
                RTreeNode::Leaves {
                    ref mut children, ..
                } => return Some(children.iter_mut()),
                RTreeNode::Level {
                    ref mut children, ..
                } => {
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

impl<'tree, P: FP, const DIM: usize, LG, T, Q> Iterator for LevelIterMut<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    type Item = LeafIterMut<'tree, P, DIM, LG, T>;

    fn next(&mut self) -> Option<LeafIterMut<'tree, P, DIM, LG, T>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match unsafe { self.unpack_root_lifetime() } {
                &mut RTreeNode::Leaves {
                    ref mut children, ..
                } => {
                    self.finished = true;
                    return Some(children.iter_mut());
                }
                &mut RTreeNode::Level {
                    ref mut children, ..
                } => {
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

/// Iter all `Leaf` items matching a query
pub struct Iter<'tree, P: FP, const DIM: usize, LG, T, Q>
where
    LG: 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    query: Rc<Q>,
    level_iter: LevelIter<'tree, P, DIM, LG, T, Q>,
    leaf_iter: Option<LeafIter<'tree, P, DIM, LG, T>>,
    finished: bool,
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> Iter<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    /// Constructor
    fn new(query: Q, root: &'tree RTreeNode<P, DIM, LG, T>) -> Iter<'tree, P, DIM, LG, T, Q> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        Iter {
            query: rc_query,
            level_iter: level_iter,
            leaf_iter: None,
            finished: false,
        }
    }

    /// Select the next matching leaf
    fn next_leaf(
        &mut self,
        mut iter: SliceIter<'tree, MbrLeaf<P, DIM, LG, T>>,
    ) -> Option<(&'tree LG, &'tree T)> {
        while let Some(ref mut leaf) = iter.next() {
            if !self.query.accept_leaf(leaf) {
                continue;
            }
            self.leaf_iter = Some(iter);
            return Some(leaf.as_tuple());
        }
        None
    }
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> Iterator for Iter<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    type Item = (&'tree LG, &'tree T);

    fn next(&mut self) -> Option<(&'tree LG, &'tree T)> {
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

/// Mutably iterate all `Leaf` entries matching a query
pub struct IterMut<'tree, P: FP, const DIM: usize, LG, T, Q>
where
    LG: 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    query: Rc<Q>,
    level_iter: LevelIterMut<'tree, P, DIM, LG, T, Q>,
    leaf_iter: Option<LeafIterMut<'tree, P, DIM, LG, T>>,
    finished: bool,
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> IterMut<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM> + 'tree,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    /// Constructor
    fn new(
        query: Q,
        root: &'tree mut RTreeNode<P, DIM, LG, T>,
    ) -> IterMut<'tree, P, DIM, LG, T, Q> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIterMut::new(rc_query.clone(), root);
        IterMut {
            query: rc_query,
            level_iter: level_iter,
            leaf_iter: None,
            finished: false,
        }
    }

    /// Select the next matching leaf
    fn next_leaf(
        &mut self,
        mut iter: SliceIterMut<'tree, MbrLeaf<P, DIM, LG, T>>,
    ) -> Option<(&'tree LG, &'tree mut T)> {
        while let Some(ref mut leaf) = iter.next() {
            if !self.query.deref().accept_leaf(leaf) {
                continue;
            }
            self.leaf_iter = Some(iter);
            let unsafe_leaf = unsafe {
                let unsafe_leaf: *mut MbrLeaf<P, DIM, LG, T> = *leaf;
                &mut *unsafe_leaf
            };
            return Some(unsafe_leaf.as_mut_tuple());
        }
        None
    }
}

impl<'tree, P: FP, const DIM: usize, LG, T, Q> Iterator for IterMut<'tree, P, DIM, LG, T, Q>
where
    LG: MbrLeafGeometry<P, DIM>,
    T: 'tree,
    Q: MbrQuery<P, DIM, LG, T, RTreeNode<P, DIM, LG, T>>,
{
    type Item = (&'tree LG, &'tree mut T);

    fn next(&mut self) -> Option<(&'tree LG, &'tree mut T)> {
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
