use std::{fs::read_dir, io::Read};

use super::{parse, parse_ast, QuadrupleManager, VM};

fn parse_ast_has_error(filename: &str) {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = false;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    assert!(parse_ast(ast, debug).is_err());
}

impl<R: Read> VM<R> {
    pub fn new_with_reader(quad_manager: &QuadrupleManager, debug: bool, reader: R) -> Self {
        VM::base_new(quad_manager, debug, Some(reader))
    }
}

fn parse_ast_is_ok(filename: &str) -> QuadrupleManager {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = false;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    let res = parse_ast(ast, debug);
    assert!(res.is_ok());
    res.unwrap()
}

fn run_vm_is_error(filename: &str) {
    let quad_manager = parse_ast_is_ok(filename);
    let mut vm = VM::new_with_reader(&quad_manager, false, b"test".as_ref());
    assert!(vm.run().is_err());
}

fn run_vm_is_ok(filename: &str) {
    let quad_manager = parse_ast_is_ok(filename);
    let mut vm = VM::new_with_reader(&quad_manager, false, b"test".as_ref());
    assert!(vm.run().is_ok());
}

#[test]
fn ast_parsing_invalid_files() {
    let paths = read_dir("examples/invalid").unwrap();
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        if file == "examples/invalid/syntax-error.ra"
            || file == "examples/invalid/array-list-index.ra"
            || file == "examples/invalid/div-0.ra"
        {
            continue;
        }
        parse_ast_has_error(file);
    }
}

#[test]
fn vm_running_invalid_files() {
    let files = vec![
        "examples/invalid/array-list-index.ra",
        "examples/invalid/div-0.ra",
    ];
    files.iter().for_each(|v| run_vm_is_error(*v));
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
        run_vm_is_ok(file);
    }
}
