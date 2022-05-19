use super::*;
use std::fs::read_dir;

#[test]
fn valid_files() {
    let paths = read_dir("src/examples/valid").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/valid/complete.ra" {
            continue;
        }
        let program = std::fs::read_to_string(file).expect(file);
        println!("Testing {:?}", file);
        let res = parse(&program, true);
        assert!(res.is_ok());
    }
}

#[test]
fn invalid_file() {
    let filename = "src/examples/invalid/syntax/syntax-error.ra";
    let program = std::fs::read_to_string(filename).expect(filename);
    let res = parse(&program, true);
    assert!(res.is_err());
}
