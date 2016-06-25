// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use generic_array::ArrayLength;
use tree::mbr::{MbrNode, MbrQuery};
use tree::Leaf;
pub mod rstar;
pub mod r;

pub const D_MAX: usize = 64;
const AT_ROOT: bool = true;
const NOT_AT_ROOT: bool = false;
const FORCE_SPLIT: bool = true;
const DONT_FORCE_SPLIT: bool = false;

/// Insert the leaf into the root
pub trait IndexInsert<P, DIM, LS, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn insert_into_root(&self,
                        mut root: MbrNode<P, DIM, LS, T>,
                        leaf: Leaf<P, DIM, LS, T>)
                        -> MbrNode<P, DIM, LS, T>;
    
    fn preferred_min(&self) -> usize;

    fn new_leaves(&self) -> MbrNode<P, DIM, LS, T>;

    fn new_no_alloc_leaves(&self) -> MbrNode<P, DIM, LS, T>;
}

pub type RemoveReturn<P, DIM, LS, T> = (MbrNode<P, DIM, LS, T>, Vec<Leaf<P, DIM, LS, T>>);

/// Remove entries from the tree that match the query, but not the retain function f.
pub trait IndexRemove<P, DIM, LS, T, I>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, DIM, LS, T>
{
    fn remove_from_root<F: FnMut(&T) -> bool>
        (&self,
         mut root: MbrNode<P, DIM, LS, T>,
         insert_index: &I,
         query: MbrQuery<P, DIM>,
         mut f: F)
         -> RemoveReturn<P, DIM, LS, T>;
}
