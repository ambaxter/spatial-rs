// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use generic_array::ArrayLength;
use tree::{LevelNode, Query, Leaf};
mod rstar;
mod r;

pub trait IndexInsert<P, D, T>
    where D: ArrayLength<P> + ArrayLength<(P, P)>
{
    fn insert_into_root(&self,
                        root: Option<LevelNode<P, D, T>>,
                        leaf: Leaf<P, D, T>)
                        -> Option<LevelNode<P, D, T>>;
}

pub trait IndexRemove<P, D, T, I>
    where D: ArrayLength<P> + ArrayLength<(P, P)>,
          I: IndexInsert<P, D, T>
{
    fn remove_from_root(&self,
                        root: Option<LevelNode<P, D, T>>,
                        insert_index: &I,
                        query: Query<P, D, T>)
                        -> (Option<LevelNode<P, D, T>>, Vec<Leaf<P, D, T>>);
}
