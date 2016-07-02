# spatial-rs
N-Dimensional Spatial Tree implementations library for Rust

Currently only implements R* Tree, but I'm trying to keep it generic enough where most spatial trees can be supported in a single interface.

[Documentation](https://ambaxter.github.io/spatial/doc/spatial/index.html)

# Roadmap

The march to 0.2.0:
- [ ] Inevitable bug fixes
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [x] Bug-fix: iter_mut and query_mut should take &mut self to provide Isolation guarantees
- [ ] R-Tree: ~~implement~~, document, test, and benchmark
- [ ] X-Tree: implement, document, test, and benchmark
- [x] Performance improvement: Remove UnpackRwLocks and PackRwLocks requirement
- [ ] Examples in documentation, README.md
- [ ] Changelog
- [ ] Final polish before publish
- [ ] Publish to crates.io

0.3.0:
- [ ] Inevitable bug fixes
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [ ] Move the tree iterator code into macros?
- [ ] Macro so users can create their own Shapes enum?
- [ ] Example application - SLD2 application - insert point at mouse click to visualize the tree
- [ ] test and benchmark individual Index functions as opposed to simply the overall tree
- [ ] ???

0.4.0:
- [ ] Inevitable bug fixes
- [ ] No public library interface changes (pub fn and type names only) unless bug fixes demand it
- [ ] Performance improvement: remove recursion - add another unsafe area :/
- [ ] Performance improvement: Use lifeguard with MbrNode
- [ ] Auto-guess performance variables?
- [ ] ???

0.5.0:
- [ ] Inevitable bug fixes
- [ ] Interface changes and code reorganization expected
- [ ] R+Tree: implement, document, test, and benchmark (moved because implementation requires T: + Clone for mbrmap)
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
