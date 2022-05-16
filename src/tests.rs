use std::fs::read_dir;

use super::{parse, parse_ast};

fn parse_ast_has_error(filename: &str) {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = false;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    assert!(parse_ast(ast, debug).is_err());
}

fn parse_ast_is_ok(filename: &str) {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = false;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    assert!(parse_ast(ast, debug).is_ok());
}

#[test]
fn invalid_files() {
    let paths = read_dir("examples/invalid").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/invalid/syntax-error.ra" {
            continue;
        }
        parse_ast_has_error(file);
    }
}

#[test]
fn valid_files() {
    let paths = read_dir("examples/valid").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/valid/complete.ra" {
            continue;
        }
        parse_ast_is_ok(file);
    }
}
