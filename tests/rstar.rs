#[macro_use]
extern crate generic_array;
extern crate spatial;
extern crate typenum;

use spatial::geometry::{Point, Rect};
use spatial::tree::mbr::MbrRectQuery;
use spatial::RStar;

#[test]
fn rstar_integration() {
    let mut tree_map = RStar::new_with_max(16);
    for i in 0..32 {
        let i_f32 = i as f32;
        tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
        println!("i: {:?}", i);
    }
    assert_eq!(32, tree_map.len());
    assert_eq!(tree_map.len(), tree_map.iter().count());
    assert_eq!(tree_map.len(), tree_map.iter_mut().count());

    println!("Remove query");
    let removed = tree_map.remove(MbrRectQuery::ContainedBy(Rect::from_corners(
        arr![f32; 0.0f32, 0.0f32, 0.0f32],
        arr![f32; 9.0f32, 9.0f32, 9.0f32],
    )));
    assert_eq!(10, removed.len());
    assert_eq!(22, tree_map.len());
    assert_eq!(tree_map.len(), tree_map.iter().count());

    println!("Retain query");
    let removed_retain = tree_map.retain(MbrRectQuery::ContainedBy(Rect::max()), |x| *x >= 20);
    assert_eq!(10, removed_retain.len());
    assert_eq!(12, tree_map.len());
    assert_eq!(tree_map.len(), tree_map.iter().count());

    println!("Remove all");
    let retain_none = tree_map.remove(MbrRectQuery::ContainedBy(Rect::max()));
    assert_eq!(12, retain_none.len());
    assert_eq!(0, tree_map.len());
    assert_eq!(tree_map.len(), tree_map.iter().count());

    for i in 0..32 {
        let i_f32 = i as f32;
        tree_map.insert(Point::new(arr![f32; i_f32, i_f32, i_f32]), i);
        println!("i: {:?}", i);
    }
    assert_eq!(32, tree_map.len());
    assert_eq!(tree_map.len(), tree_map.iter().count());
}
