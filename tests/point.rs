
use serde::Deserialize;

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Eq, PartialEq, Deserialize, Debug)]
struct Point {
    x: u64,
    y: u64,
}

#[test]
fn test_deserialize_point() {
    let mut plist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plist_path.push("tests/point.plist");

    let mut file = fs::File::open(plist_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    assert_eq!(
        bplist::from_slice::<Point>(&data),
        Ok(Point {
            x: 1,
            y: 20
        })
    );
}

#[test]
fn test_deserialize_point_as_object() {
    let mut plist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plist_path.push("tests/point.plist");

    let mut file = fs::File::open(plist_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    assert_eq!(
        bplist::from_slice::<bplist::Object>(&data),
        Ok(bplist::Object::Dictionary({
            let mut map = BTreeMap::new();
            map.insert(bplist::Object::String(String::from("x")), bplist::Object::Integer(1));
            map.insert(bplist::Object::String(String::from("y")), bplist::Object::Integer(20));
            map
        }))
    );
}
