// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use generic_array::ArrayLength;
use tree::{LevelNode, Query, Leaf};
pub mod rstar;
pub mod r;

/// Insert the leaf into the root
pub trait IndexInsert<P, DIM, SHAPE, T>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn insert_into_root(&self,
                        root: Option<LevelNode<P, DIM, SHAPE, T>>,
                        leaf: Leaf<P, DIM, SHAPE, T>)
                        -> Option<LevelNode<P, DIM, SHAPE, T>>;
}

/// Remove entries from the tree that match the query, but not the retain function f.
pub trait IndexRemove<P, DIM, SHAPE, T, I>
    where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, DIM, SHAPE, T>
{
    fn remove_from_root<F: FnMut(&T) -> bool>
        (&self,
         root: Option<LevelNode<P, DIM, SHAPE, T>>,
         insert_index: &I,
         query: Query<P, DIM, SHAPE, T>,
         mut f: F)
         -> (Option<LevelNode<P, DIM, SHAPE, T>>, Vec<Leaf<P, DIM, SHAPE, T>>);
}
