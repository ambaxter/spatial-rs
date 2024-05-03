#![feature(test)]

extern crate rand;
extern crate spatial;
extern crate test;

use rand::{random};
use spatial::geometry::{Point, Rect};
use spatial::tree::mbr::MbrRectQuery;
use spatial::{RStar, RStarTree};
use test::Bencher;

fn generate_tree_with_size(count: usize) -> RStarTree<f64, 3, Point<f64, 3>, usize> {
    let mut tree_map = RStar::new_with_max(32);
    for i in 0..count {
        tree_map.insert(
            Point::new([random::<f64>(), random::<f64>(), random::<f64>()]),
            i,
        );
    }
    tree_map
}

#[bench]
fn insert_rng_bench_3d_10(b: &mut Bencher) {
    b.iter(|| {
        generate_tree_with_size(10);
    });
}

#[bench]
fn insert_rng_bench_3d_100(b: &mut Bencher) {
    b.iter(|| {
        generate_tree_with_size(100);
    });
}

#[bench]
fn insert_rng_bench_3d_1000(b: &mut Bencher) {
    b.iter(|| {
        generate_tree_with_size(1000);
    });
}

#[bench]
fn insert_rng_bench_3d_10000(b: &mut Bencher) {
    b.iter(|| {
        generate_tree_with_size(10000);
    });
}

fn search_rng_bench_3d(b: &mut Bencher, size: usize) {
    let tree_map = generate_tree_with_size(size);
    b.iter(|| {
        let x_array = [random::<f64>(), random::<f64>(), random::<f64>()];
        let y_array = [random::<f64>(), random::<f64>(), random::<f64>()];
        tree_map
            .iter_query(MbrRectQuery::Overlaps(Rect::from_corners(x_array, y_array)))
            .count();
    });
}

fn remove_rng_bench_3d(b: &mut Bencher, size: usize) {
    let mut tree_map = generate_tree_with_size(size);
    b.iter(|| {
        let x_array = [random::<f64>(), random::<f64>(), random::<f64>()];
        let y_array = [random::<f64>(), random::<f64>(), random::<f64>()];
        let removed = tree_map.remove(MbrRectQuery::Overlaps(Rect::from_corners(x_array, y_array)));
        for (lshape, item) in removed {
            tree_map.insert(lshape, item);
        }
    });
}

#[bench]
fn search_rng_bench_3d_10(b: &mut Bencher) {
    search_rng_bench_3d(b, 10);
}

#[bench]
fn search_rng_bench_3d_100(b: &mut Bencher) {
    search_rng_bench_3d(b, 100);
}

#[bench]
fn search_rng_bench_3d_1000(b: &mut Bencher) {
    search_rng_bench_3d(b, 1000);
}

#[bench]
fn search_rng_bench_3d_10000(b: &mut Bencher) {
    search_rng_bench_3d(b, 10000);
}

#[bench]
fn remove_rng_bench_3d_10(b: &mut Bencher) {
    remove_rng_bench_3d(b, 10);
}

#[bench]
fn remove_rng_bench_3d_100(b: &mut Bencher) {
    remove_rng_bench_3d(b, 100);
}

#[bench]
fn remove_rng_bench_3d_1000(b: &mut Bencher) {
    remove_rng_bench_3d(b, 1000);
}

#[bench]
fn remove_rng_bench_3d_10000(b: &mut Bencher) {
    remove_rng_bench_3d(b, 10000);
}
