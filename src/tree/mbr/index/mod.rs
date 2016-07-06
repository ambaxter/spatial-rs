// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Specific implementations for inserting and removing leaves

use generic_array::ArrayLength;
use tree::mbr::{MbrNode, MbrLeaf, MbrQuery};
pub mod rstar;
pub mod r;

pub const D_MAX: usize = 64;
const AT_ROOT: bool = true;
const NOT_AT_ROOT: bool = false;
const FORCE_SPLIT: bool = true;
const DONT_FORCE_SPLIT: bool = false;

/// Insert the leaf into the root
pub trait IndexInsert<P, DIM, LG, T, NODE>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
    NODE: MbrNode<P, DIM>
{
    fn insert_into_root(&self,
                        mut root: NODE,
                        leaf: MbrLeaf<P, DIM, LG, T>)
                        -> NODE;
    
    fn preferred_min(&self) -> usize;

    fn new_leaves(&self) -> NODE;

    fn new_no_alloc_leaves(&self) -> NODE;
}

pub type RemoveReturn<P, DIM, LG, T, NODE> = (NODE, Vec<MbrLeaf<P, DIM, LG, T>>);

/// Remove entries from the tree that match the query, but not the retain function f.
pub trait IndexRemove<P, DIM, LG, T, NODE, I>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          NODE: MbrNode<P, DIM>,
          I: IndexInsert<P, DIM, LG, T, NODE>
{
    fn remove_from_root<Q: MbrQuery<P, DIM, LG, T, NODE>, F: FnMut(&T) -> bool>
        (&self,
         mut root: NODE,
         insert_index: &I,
         query: Q,
         mut f: F)
         -> RemoveReturn<P, DIM, LG, T, NODE>;
}
