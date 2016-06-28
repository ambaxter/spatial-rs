// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A collection of spatial trees

pub mod mbr;

// TODO: Figure this out later :/
// pub trait SpatialMap<'tree, P, DIM, LG, LEVEL, T>
// where DIM: ArrayLength<P> + ArrayLength<(P, P)>,
// LG: 'tree,
// T: 'tree
// {
// Insert an item
// fn insert(&mut self, geometry: LG, item: T);
//
// Remove all items whose shapes are accepted by the query
// fn remove<Q: SpatialQuery<P, DIM, LG, LEVEL, T>>(&mut self, query: Q) -> Vec<(LG, T)>;
//
// Remove all items whose shapes are accepted by the query and where f(&T) returns false
// fn retain<Q, F>(&mut self, query: Q, f: F) -> Vec<(LG, T)>
// where Q: SpatialQuery<P, DIM, LG, LEVEL, T>,
// F: FnMut(&T) -> bool;
//
// Whether the map is empty
// fn is_empty(&self) -> bool;
//
// Length of the map
// fn len(&self) -> usize;
//
// Clear the map
// fn clear(&mut self);
//
// Iter for the map
// fn iter<ITER: Iterator<Item=(&'tree LG, &'tree T)>>(&'tree self) -> ITER;
//
// IterMut for the map
// fn iter_mut<ITERM: Iterator<Item=(&'tree LG, &'tree mut T)>>(&'tree mut self) -> ITERM;
//
// Iter for the map with a given query
// fn iter_query<Q, ITER>(&'tree self, query: Q) -> ITER
// where Q: SpatialQuery<P, DIM, LG, LEVEL, T>,
// ITER: Iterator<Item=(&'tree LG, &'tree T)>;
//
// IterMut for the map with a given query
// fn iter_query_mut<Q, ITERM>(&'tree mut self, query: Q) -> ITERM
// where Q: SpatialQuery<P, DIM, LG, LEVEL, T>,
// ITERM: Iterator<Item=(&'tree LG, &'tree mut T)>;
// }
//
