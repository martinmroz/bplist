
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn test_deserialize_input_with_cycle() {
    let mut plist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plist_path.push("tests/cycle.plist");

    let mut file = fs::File::open(plist_path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    assert_eq!(
        bplist::from_slice::<Vec<u8>>(&data),
        Err(bplist::Error::CycleDetected)
    );
}
