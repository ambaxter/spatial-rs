// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Spatial Trees

#[macro_use]
extern crate itertools;
#[macro_use]
extern crate generic_array;

extern crate ordered_float;
extern crate num;
extern crate typenum;

#[cfg(feature="geo")]
extern crate geo;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod geometry;
pub mod tree;
mod vecext;

pub use tree::mbr::{RTree, RQuadraticTree, RLinearTree, RStar, RStarTree};
