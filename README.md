# spatial-rs
N-Dimensional Spatial Tree implementations library for Rust

Currently only implements R* Tree, but I'm trying to keep it generic enough where most spatial trees can be supported in a single interface.

[Documentation](https://ambaxter.github.io/spatial/doc/spatial)

# Roadmap

The march to 0.2.0:
- [x] Bug-fix: Splitting root resulted in incorrect mbrs
- [x] Bug-fix: iter_mut and query_mut should take &mut self to provide Isolation guarantees
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [x] R-Tree: implement, document, test, and benchmark
- [x] Pull out XMbrNode and MbrNode as a trait. Modify everything to use that trait 
- [x] Performance improvement: Remove UnpackRwLocks and PackRwLocks requirement
- [ ] Examples in documentation, README.md
- [ ] Changelog
- [ ] Final polish before publish
- [ ] Publish to crates.io

0.3.0:
- [ ] Inevitable bug fixes
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [ ] X-Tree: implement, document, test, and benchmark (moved to give me time to work on generic-bitset)
- [ ] Optional bindings to rust-geo
- [ ] Move the tree iterator code into macros?
- [ ] Macro so users can create their own Shapes enum?
- [ ] Example application - SLD2 application - insert point at mouse click to visualize the tree
- [ ] test and benchmark individual Index functions in addition to the overall tree
- [ ] macro benchmarks to reduce loc
- [ ] ???

0.4.0:
- [ ] Inevitable bug fixes
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [ ] Performance improvement: Able to re-insert nodes as well, as opposed to just leaves. Remove the need for consume_leaves_for_reinsert
- [ ] Performance improvement: Remove recursion - add another unsafe area :/
- [ ] Performance improvement: Tune children size based off memory block size
- [ ] Performance improvement: Child max/min length different for Level and Leaves  
- [ ] Performance improvement: Use lifeguard with MbrNode
- [ ] Auto-guess performance variables?
- [ ] ???

0.5.0:
- [ ] Inevitable bug fixes
- [ ] Interface changes and code reorganization expected
- [ ] R+Tree: implement, document, test, and benchmark (moved because implementation requires T: + Clone for mbrmap. Is it even a desired feature?)
- [ ] Fast tree creation from sorted data
- [ ] ???

In the future:
- [ ] Saving and updating tree to disk (either with or after 0.5) - Is that even a rabbit hole I should dive down?
- [ ] Compiler supported integer generics (whenever it lands in stable)
- [ ] Profit? (not likely)


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
