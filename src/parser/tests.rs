use super::*;
use std::fs::read_dir;

#[test]
fn example_files() {
    let paths = read_dir("examples").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/complete.ra" {
            continue;
        }
        let program = std::fs::read_to_string(file).expect(file);
        println!("Testing {:?}", file);
        assert!(parse(&program, true).is_ok());
    }
}
