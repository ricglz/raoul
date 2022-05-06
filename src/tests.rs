use std::io::Read;

use super::{parse, parse_ast, AstNode, QuadrupleManager, VM};

fn get_ast(program: &str) -> AstNode {
    let ast_response = parse(program, false);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    insta::assert_debug_snapshot!(ast);
    ast
}

fn parse_ast_has_error(filename: &str) {
    println!("Testing {:?}", filename);
    let program = std::fs::read_to_string(filename).expect(filename);
    let ast = get_ast(&program);
    let res = parse_ast(ast, false);
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
    let ast = get_ast(&program);
    let res = parse_ast(ast, false);
    assert!(res.is_ok());
    let quad_manager = res.unwrap();
    insta::assert_display_snapshot!(quad_manager);
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

fn expect_paths<F>(glob_path: &str, mut f: F)
where
    F: FnMut(&str),
{
    insta::glob!(glob_path, |path| {
        let file = path.to_str().unwrap();
        f(file);
    });
}

#[test]
fn ast_parsing_invalid_files() {
    expect_paths("examples/invalid/static/*", parse_ast_has_error);
}

#[test]
fn vm_running_invalid_files() {
    expect_paths("examples/invalid/dynamic/*", run_vm_is_error);
}

#[test]
fn valid_files() {
    expect_paths("examples/valid/*", run_vm_is_ok);
}
