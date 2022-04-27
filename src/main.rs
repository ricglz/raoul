mod args;

// ANCHOR: Actual parser
mod address;
mod ast;
mod dir_func;
mod enums;
mod error;
mod parser;
mod quadruple;

use ast::AstNode;
use dir_func::DirFunc;
use error::Results;
use parser::parse;
use quadruple::quadruple_manager::QuadrupleManager;

// ANCHOR: Testing the examples
mod test_parser;
#[macro_use]
extern crate pest_derive;

use std::process::exit;

use args::parse_args;

fn parse_ast<'a>(ast: AstNode<'a>, debug: bool) -> Results<'a, ()> {
    let mut dir_func = DirFunc::new();
    dir_func.build_dir_func(ast.clone())?;
    if debug {
        println!("Dir func created sucessfully");
        println!("{:#?}", dir_func);
    }
    let mut quad_manager = QuadrupleManager::new(&mut dir_func);
    quad_manager.parse(ast.clone())?;
    Ok(if debug {
        println!("Quads created sucessfully");
        println!("{:#?}", quad_manager.memory);
        println!("{:?}", quad_manager);
    })
}

fn main() {
    let matches = parse_args();
    let filename = matches.value_of("file").expect("required");
    let debug = matches.is_present("debug");
    if debug {
        println!("Starting parsing");
    }
    let file = std::fs::read_to_string(filename).expect(filename);
    let parsing_response = parse(&file, debug);
    if let Err(error) = parsing_response {
        println!("Parsing error {}", error.to_string());
        exit(1);
    }
    let ast = parsing_response.unwrap();
    if debug {
        println!("Parsing ended sucessfully");
        println!("AST:\n{:?}", ast);
    }
    if let Err(errors) = parse_ast(ast, debug) {
        for error in errors {
            println!("{:?}", error);
        }
        exit(1);
    }
}

#[cfg(test)]
mod tests;
