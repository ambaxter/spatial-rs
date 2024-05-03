// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Spatial Trees

#[macro_use]
extern crate itertools;

extern crate num;
extern crate ordered_float;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub trait FP:
    Float
    + Signed
    + Bounded
    + MulAssign
    + AddAssign
    + ToPrimitive
    + FromPrimitive
    + Copy
    + Debug
    + Default
    + FloatCore
    + TryInto<NotNan<Self>>
{
}

impl FP for f32 {}
impl FP for f64 {}

pub mod geometry;
pub mod tree;
mod vecext;

use num::{Bounded, Float, FromPrimitive, Signed, ToPrimitive};
use ordered_float::{FloatCore, NotNan};
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::{AddAssign, MulAssign};
pub use tree::mbr::{RLinearTree, RQuadraticTree, RStar, RStarTree, RTree};
