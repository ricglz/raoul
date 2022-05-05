use super::*;
use std::fs::read_dir;

#[test]
fn valid_files() {
    let paths = read_dir("examples/valid").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/valid/complete.ra" {
            continue;
        }
        let program = std::fs::read_to_string(file).expect(file);
        println!("Testing {:?}", file);
        assert!(parse(&program, true).is_ok());
    }
}

#[test]
fn invalid_file() {
    let filename = "examples/invalid/syntax/syntax-error.ra";
    let program = std::fs::read_to_string(filename).expect(filename);
    assert!(parse(&program, true).is_err());
}
