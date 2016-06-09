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

pub trait IndexInsert<P, D, S, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn insert_into_root(&self,
                        root: Option<LevelNode<P, D, S, T>>,
                        leaf: Leaf<P, D, S, T>)
                        -> Option<LevelNode<P, D, S, T>>;
}

pub trait IndexRemove<P, D, S, T, I>
    where D: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, D, S, T>
{
    fn remove_from_root<F: FnMut(&T) -> bool>
        (&self,
         root: Option<LevelNode<P, D, S, T>>,
         insert_index: &I,
         query: Query<P, D, S, T>,
         mut f: F)
         -> (Option<LevelNode<P, D, S, T>>, Vec<Leaf<P, D, S, T>>);
}
