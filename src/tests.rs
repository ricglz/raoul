use std::{
    fs::{read_dir, ReadDir},
    io::Read,
};

use super::{parse, parse_ast, QuadrupleManager, VM};

fn parse_ast_has_error(filename: &str) {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = false;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    let res = parse_ast(ast, debug);
    assert!(res.is_err());
    insta::assert_debug_snapshot!(res.unwrap_err());
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
    let quad_manager = res.unwrap();
    insta::assert_debug_snapshot!(quad_manager);
    quad_manager
}

fn run_vm_is_error(filename: &str) {
    let quad_manager = parse_ast_is_ok(filename);
    let mut vm = VM::new_with_reader(&quad_manager, false, b"test".as_ref());
    let res = vm.run();
    assert!(res.is_err());
    insta::assert_display_snapshot!(res.unwrap_err());
    insta::assert_debug_snapshot!(vm.messages);
}

fn run_vm_is_ok(filename: &str) {
    let quad_manager = parse_ast_is_ok(filename);
    let mut vm = VM::new_with_reader(&quad_manager, false, b"test".as_ref());
    let res = vm.run();
    assert!(res.is_ok());
    insta::assert_debug_snapshot!(vm.messages);
}

fn expect_paths<F>(paths: ReadDir, mut f: F)
where
    F: FnMut(&str),
{
    for path in paths {
        let file_path = path.expect("File must exist").path();
        let file = file_path.to_str().unwrap();
        f(file);
    }
}

#[test]
fn ast_parsing_invalid_files() {
    let paths = read_dir("examples/invalid/static").unwrap();
    expect_paths(paths, parse_ast_has_error);
}

#[test]
fn vm_running_invalid_files() {
    let paths = read_dir("examples/invalid/dynamic").unwrap();
    expect_paths(paths, run_vm_is_error);
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
