use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Serializer;
use serde_json::ser::PrettyFormatter;

pub fn save_to_json<P: AsRef<Path>, T: Serialize>(path: P, contents: &T) {
    let mut bytes = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"\t");
    let mut ser = Serializer::with_formatter(&mut bytes, formatter);
    contents.serialize(&mut ser).expect("serialize to json");
    bytes.push(b'\n'); // final newline
    fs::write(path, bytes).expect("write file");
}
