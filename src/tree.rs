// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Leaf<P, D, S, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    pub shape: S,
    pub item: T,
    _phantom_p: PhantomData<P>,
    _phantom_d: PhantomData<D>,
    
}

#[allow(dead_code)]
impl<P, D, S, T> Leaf<P, D, S, T> 
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)>,
          S: Shape<P, D>
{
    pub fn new(shape: S, item: T) -> Leaf<P, D, S, T> {
        Leaf{shape: shape, item: item, _phantom_p: PhantomData, _phantom_d: PhantomData}
    }

    pub fn extract(self) -> (S, T) {
        (self.shape, self.item)
    }
}

impl<P, D, S, T> Shape<P, D> for Leaf<P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)>,
          S: Shape<P, D> {

    fn dim(&self) -> usize {
        self.shape.dim()
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, D>) {
        self.shape.expand_rect_to_fit(edges)
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, D>) -> P {
        self.shape.distance_from_rect_center(edges)
    }

    fn contained_by_rect(&self, edges: &Rect<P, D>) -> bool {
        self.shape.contained_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, D>) -> bool {
        self.shape.overlapped_by_rect(edges)
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, D>) -> P {
        self.shape.area_overlapped_with_rect(edges)
    }

    fn area(&self) -> P {
        self.shape.area()
    }

    fn min_for_axis(&self, dim: usize) -> P {
        self.shape.min_for_axis(dim)
    }

    fn max_for_axis(&self, dim: usize) -> P {
        self.shape.max_for_axis(dim)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum LevelNode<P, D, S, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    Level {
        mbr: Rect<P, D>,
        children: Vec<LevelNode<P, D, S, T>>,
    },
    Leaves {
        mbr: Rect<P, D>,
        children: Vec<Leaf<P, D, S, T>>,
    },
}

#[allow(dead_code)]
impl<P, D, S, T> LevelNode<P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug,
          D: ArrayLength<P> + ArrayLength<(P,P)>,
          S: Shape<P, D> {

    pub fn has_leaves(&self) -> bool {
        match *self {
            LevelNode::Level{..} => false,
            LevelNode::Leaves{..} => true,
        }
    }

    pub fn is_level(&self) -> bool {
        !self.has_leaves()
    }

    pub fn mbr(&self) -> &Rect<P, D> {
        match *self {
            LevelNode::Level{ref mbr, ..} => mbr,
            LevelNode::Leaves{ref mbr, ..} => mbr,
        }
    }

    pub fn mbr_mut(&mut self) -> &mut Rect<P, D> {
        match *self {
            LevelNode::Level{ref mut mbr, ..} => mbr,
            LevelNode::Leaves{ref mut mbr, ..} => mbr,
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            LevelNode::Level{ref children, ..} => children.len(),
            LevelNode::Leaves{ref children, ..} => children.len(),
        }
    }
}


impl<P, D, S, T> Shape<P, D> for LevelNode<P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)>,
          S: Shape<P, D> {

    fn dim(&self) -> usize {
        self.mbr().dim()
    }

    fn expand_rect_to_fit(&self, edges: &mut Rect<P, D>) {
        self.mbr().expand_rect_to_fit(edges)
    }

    fn distance_from_rect_center(&self, edges: &Rect<P, D>) -> P {
        self.mbr().distance_from_rect_center(edges)
    }

    fn contained_by_rect(&self, edges: &Rect<P, D>) -> bool {
        self.mbr().contained_by_rect(edges)
    }

    fn overlapped_by_rect(&self, edges: &Rect<P, D>) -> bool {
        self.mbr().overlapped_by_rect(edges)
    }

    fn area_overlapped_with_rect(&self, edges: &Rect<P, D>) -> P {
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


#[allow(dead_code)]
#[derive(Debug)]
pub enum Query<P, D, S, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    Contains(Rect<P, D>, PhantomData<S>, PhantomData<T>),
    Overlaps(Rect<P, D>, PhantomData<S>, PhantomData<T>),
}

#[allow(dead_code)]
impl<P, D, S, T> Query<P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)>,
          S: Shape<P, D> {

    pub fn contains(query: Rect<P, D>) -> Query<P, D, S, T>{
        Query::Contains(query, PhantomData, PhantomData)
    }

    pub fn overlaps(query: Rect<P, D>) -> Query<P, D, S, T>{
        Query::Overlaps(query, PhantomData, PhantomData)
    }

    pub fn accept_leaf(&self, leaf: &Leaf<P, D, S, T>) -> bool {
        match self {
            &Query::Contains(ref query, _, _) => leaf.shape.contained_by_rect(&query),
            &Query::Overlaps(ref query, _, _) => leaf.shape.overlapped_by_rect(&query),
        }
    }

    pub fn accept_level(&self, level: &LevelNode<P, D, S, T>) -> bool {
        match self {
            &Query::Contains(ref query, _, _) => level.mbr().overlapped_by_rect(&query),
            &Query::Overlaps(ref query, _, _) => level.mbr().overlapped_by_rect(&query),
        }
    }
}

#[allow(dead_code)]
pub struct SpatialTree<P, D, S, I, R, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, D, S, T>,
          R: IndexRemove<P, D, S, T, I>
{
    insert_index: I,
    remove_index: R,
    root: Option<LevelNode<P, D, S, T>>,
    len: usize,
}

#[allow(dead_code)]
impl<P, D, S, I, R, T> SpatialTree<P, D, S, I, R, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default,
          D: ArrayLength<P> + ArrayLength<(P,P)> + Clone,
          S: Shape<P, D>,
          I: IndexInsert<P, D, S, T>,
          R: IndexRemove<P, D, S, T, I>,
{

    pub fn new(insert_index: I, remove_index: R) -> SpatialTree<P, D, S, I, R, T> {
        SpatialTree{insert_index: insert_index, remove_index: remove_index, root: None, len: 0}
    }

    pub fn insert(&mut self, shape: S, item: T) {
        self.root = self.insert_index.insert_into_root(self.root.take(), Leaf::new(shape, item));
        self.len += 1;
    }

    pub fn remove(&mut self, query: Query<P, D, S, T>) -> Vec<(S, T)> {
        self.retain(query, |_| false)
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, query: Query<P, D, S, T>, f: F) -> Vec<(S, T)> {
        let (new_root, mut removed) =
            self.remove_index.remove_from_root(self.root.take(), &self.insert_index, query, f);
        self.len -= removed.len();
        self.root = new_root;
        let mut removed_extract = Vec::with_capacity(removed.len());
        while let Some(leaf) = removed.pop() {
            removed_extract.push(leaf.extract());
        }
        removed_extract
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    pub fn iter(&self) -> Iter<P, D, S, T> {
        Iter::new(Query::overlaps(Rect::max()), self.root.as_ref())
    }

    pub fn iter_mut(&mut self) -> IterMut<P, D, S, T> {
        IterMut::new(Query::overlaps(Rect::max()), self.root.as_mut())
    }

    pub fn iter_query(&self, query: Query<P, D, S, T>) -> Iter<P, D, S, T> {
        Iter::new(query, self.root.as_ref())
    }

    pub fn iter_query_mut(&mut self, query: Query<P, D, S, T>) -> IterMut<P, D, S, T> {
        IterMut::new(query, self.root.as_mut())
    }
}

#[allow(dead_code)]
pub struct Iter<'tree, P, D, S, T>
    where P: 'tree,
          D: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          S: 'tree,
          T: 'tree
{
    query: Rc<Query<P, D, S, T>>,
    level_iter: LevelIter<'tree, P, D, S, T>,
    leaf_iter: Option<SliceIter<'tree, Leaf<P, D, S, T>>>,
    finished: bool,
}

#[allow(dead_code)]
impl<'tree, P, D, S, T> Iter<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree, 
          T: 'tree
{
    pub fn new(query: Query<P, D, S, T>, root: Option<&'tree LevelNode<P, D, S, T>>) -> Iter<'tree, P, D, S, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIter::new(rc_query.clone(), root);
        Iter{query: rc_query, level_iter: level_iter, leaf_iter: None, finished: false}
    }

    fn next_leaf(&mut self, mut iter: SliceIter<'tree, Leaf<P, D, S, T>>) -> Option<(&'tree S, &'tree T)> {
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

impl<'tree, P, D, S, T> Iterator for Iter<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree,
          T: 'tree
{
    type Item = (&'tree S, &'tree T);

    fn next(&mut self) -> Option<(&'tree S, &'tree T)> {
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


#[allow(dead_code)]
pub struct LevelIter<'tree, P, D, S, T>
    where P: 'tree,
          D: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          S: 'tree,
          T: 'tree
{
    query: Rc<Query<P, D, S, T>>,
    root: Option<&'tree LevelNode<P, D, S, T>>,
    level_stack: Vec<SliceIter<'tree, LevelNode<P, D, S, T>>>,
    finished: bool,
}


#[allow(dead_code)]
impl<'tree, P, D, S, T> LevelIter<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree,
          T: 'tree {

    pub fn new(query: Rc<Query<P, D, S, T>>, root: Option<&'tree LevelNode<P, D, S, T>>) -> LevelIter<'tree, P, D, S, T> {
        if root.is_none() || !query.accept_level(root.as_ref().unwrap()) {
            return LevelIter{query: query, root: None, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIter{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

    fn next_leaves(&mut self, mut m_iter: SliceIter<'tree, LevelNode<P, D, S, T>>) -> Option<SliceIter<'tree, Leaf<P, D, S, T>>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match node {
                &LevelNode::Leaves{ref children, ..} => return Some(children.iter()),
                &LevelNode::Level{ref children, ..} => {
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

impl<'tree, P, D, S, T> Iterator for LevelIter<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree,
          T: 'tree {
    type Item = SliceIter<'tree, Leaf<P, D, S, T>>;

    fn next(&mut self) -> Option<SliceIter<'tree, Leaf<P, D, S, T>>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match *self.root.as_ref().unwrap() {
                &LevelNode::Leaves{ref children, ..} => {
                    self.finished = true;
                    return Some(children.iter());
                },
                &LevelNode::Level{ref children, ..} => {
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

#[allow(dead_code)]
pub struct IterMut<'tree, P, D, S, T>
    where P: 'tree,
          D: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          S: 'tree,
          T: 'tree
{
    query: Rc<Query<P, D, S, T>>,
    level_iter: LevelIterMut<'tree, P, D, S, T>,
    leaf_iter: Option<SliceIterMut<'tree, Leaf<P, D, S, T>>>,
    finished: bool,
}

#[allow(dead_code)]
impl<'tree, P, D, S, T> IterMut<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree,
          T: 'tree
{
    pub fn new(query: Query<P, D, S, T>, root: Option<&'tree mut LevelNode<P, D, S, T>>) -> IterMut<'tree, P, D, S, T> {
        let rc_query = Rc::new(query);
        let level_iter = LevelIterMut::new(rc_query.clone(), root);
        IterMut{query: rc_query, level_iter: level_iter, leaf_iter: None, finished: false}
    }

    fn next_leaf(&mut self, mut iter: SliceIterMut<'tree, Leaf<P, D, S, T>>) -> Option<(&'tree S, &'tree mut T)> {
        while let Some(ref mut leaf) = iter.next() {
                if !self.query.deref().accept_leaf(leaf) {
                    continue;
                }
                self.leaf_iter = Some(iter);
                // The lifetime checker doesn't understand that the leaf isn't going to be going anywhere. There must be a better way, though
                let mut cast_leaf = unsafe {
                    let cast_leaf: *mut Leaf<P, D, S, T> = *leaf;
                    &mut *cast_leaf
                };
                return Some((&cast_leaf.shape, &mut cast_leaf.item));
        }
        None
    }
}

impl<'tree, P, D, S, T> Iterator for IterMut<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D>,
          T: 'tree
{
    type Item = (&'tree S, &'tree mut T);

    fn next(&mut self) -> Option<(&'tree S, &'tree mut T)> {
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



#[allow(dead_code)]
pub struct LevelIterMut<'tree, P, D, S, T>
    where P: 'tree,
          D: ArrayLength<P> + ArrayLength<(P, P)> + 'tree,
          S: 'tree,
          T: 'tree
{
    query: Rc<Query<P, D, S, T>>,
    root: Option<&'tree mut LevelNode<P, D, S, T>>,
    level_stack: Vec<SliceIterMut<'tree, LevelNode<P, D, S, T>>>,
    finished: bool,
}

#[allow(dead_code)]
impl<'tree, P, D, S, T> LevelIterMut<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default +'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D> + 'tree,
          T: 'tree {

    pub fn new(query: Rc<Query<P, D, S, T>>, root: Option<&'tree mut LevelNode<P, D, S, T>>) -> LevelIterMut<'tree, P, D, S, T> {
        if root.is_none() || !query.accept_level(root.as_ref().unwrap()) {
            return LevelIterMut{query: query, root: None, level_stack: Vec::with_capacity(0), finished: true};
        }
        LevelIterMut{query: query, root: root, level_stack: Vec::new(), finished: false}
    }

// access the root node. Must be unsafe because &'a mut self and &'tree mut self.root borrows cause a conflict
// We're not modifying the tree structure, just the leaf nodes, so this should be ok
    unsafe fn get_root_node(&mut self) -> &'tree mut LevelNode<P, D, S, T> {
        let root: *mut Option<&'tree mut LevelNode<P, D, S, T>> = &mut self.root;
        *(*root).as_mut().unwrap()
    }

fn next_leaves(&mut self, mut m_iter: SliceIterMut<'tree, LevelNode<P, D, S, T>>) -> Option<SliceIterMut<'tree, Leaf<P, D, S, T>>> {
        let mut iter_node = m_iter.next();
        while let Some(node) = iter_node {
            if !self.query.accept_level(node) {
                iter_node = m_iter.next();
                continue;
            }
            self.level_stack.push(m_iter);
            match node {
                &mut LevelNode::Leaves{ref mut children, ..} => return Some(children.iter_mut()),
                &mut LevelNode::Level{ref mut children, ..} => {
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


impl<'tree, P, D, S, T> Iterator for LevelIterMut<'tree, P, D, S, T>
    where P: Float + Signed + Bounded + MulAssign + AddAssign + ToPrimitive + FromPrimitive + Copy + Debug + Default + 'tree,
          D: ArrayLength<P> + ArrayLength<(P,P)> + 'tree,
          S: Shape<P, D>,
          T: 'tree {
    type Item = SliceIterMut<'tree, Leaf<P, D, S, T>>;

    fn next(&mut self) -> Option<SliceIterMut<'tree, Leaf<P, D, S, T>>> {
        if self.finished {
            return None;
        }
        if self.level_stack.is_empty() {
            match unsafe {self.get_root_node()} {
                &mut LevelNode::Leaves{ref mut children, ..} => {
                    self.finished = true;
                    return Some(children.iter_mut());
                },
                &mut LevelNode::Level{ref mut children, ..} => {
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
