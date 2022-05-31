mod args;

// ANCHOR: Actual parser
mod address;
mod ast;
mod dir_func;
mod enums;
mod error;
mod parser;
mod quadruple;
mod vm;

use ast::AstNode;
use dir_func::DirFunc;
use error::Results;
use parser::parse;
use quadruple::quadruple_manager::QuadrupleManager;
use vm::VM;

// ANCHOR: Testing the examples
mod test_parser;
#[macro_use]
extern crate pest_derive;

use std::process::exit;

use args::parse_arguments;

fn parse_ast<'a>(ast: &'a AstNode, debug: bool, quads: bool) -> Results<'a, QuadrupleManager> {
    let mut dir_func = DirFunc::new();
    dir_func.build_dir_func(ast)?;
    if debug {
        println!("Dir func created sucessfully");
        println!("{:#?}", dir_func);
    }
    let mut quad_manager = QuadrupleManager::new(dir_func);
    quad_manager.parse(ast)?;
    if debug || quads {
        println!("Quads created sucessfully");
        println!("{}", quad_manager);
    }
    quad_manager.clear_variables();
    Ok(quad_manager)
}

fn main() {
    let matches = parse_arguments();
    let filename = matches.value_of("file").expect("required");
    let debug = matches.is_present("debug");
    let quads = matches.is_present("quads");
    if debug {
        println!("Starting parsing");
    }
    let file = std::fs::read_to_string(filename).expect(filename);
    let parsing_response = parse(&file, debug);
    if let Err(error) = parsing_response {
        println!("Parsing error {}", error);
        exit(1);
    }
    let ast = parsing_response.unwrap();
    if debug {
        println!("Parsing ended sucessfully");
        println!("AST:\n{:?}", ast);
    }
    let res = parse_ast(&ast, debug, quads);
    if let Err(errors) = res {
        for error in errors {
            println!("{:?}", error);
        }
        exit(1);
    }
    let quad_manager = res.unwrap();
    let mut vm = VM::new(&quad_manager, debug);
    if let Err(error) = vm.run() {
        println!("[Error]: {error}");
        exit(1);
    }
}

#[cfg(test)]
mod tests;
