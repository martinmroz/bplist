
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn test_deserialize_integer_list() {
    let mut plist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plist_path.push("tests/integer_list.plist");

    let mut file = fs::File::open(plist_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    assert_eq!(
        bplist::from_slice::<Vec<u8>>(&data),
        Ok(vec![1,2,3,4,5])
    );
}

#[test]
fn test_deserialize_integer_list_as_object() {
    let mut plist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plist_path.push("tests/integer_list.plist");

    let mut file = fs::File::open(plist_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    assert_eq!(
        bplist::from_slice::<bplist::Object>(&data),
        Ok(bplist::Object::Array(
            vec![
                bplist::Object::Integer(1),
                bplist::Object::Integer(2),
                bplist::Object::Integer(3),
                bplist::Object::Integer(4),
                bplist::Object::Integer(5),
            ]
        ))
    );
}
