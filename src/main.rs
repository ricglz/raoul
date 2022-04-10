mod args;

// ANCHOR: Actual parser
mod ast;
mod dir_func;
mod enums;
mod error;
mod parser;
use dir_func::{build_dir_func, DirFunc};
use parser::parse;

// ANCHOR: Testing the examples
mod test_parser;
#[macro_use]
extern crate pest_derive;

use std::process::exit;

use args::parse_args;

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
    let mut dir_func = DirFunc::new();
    if let Err(error) = build_dir_func(&mut dir_func, ast) {
        println!("DirFunc error {:?}", error);
        exit(1);
    }
    if debug {
        println!("Dir func created sucessfully");
        println!("DirFunc:\n{:#?}", dir_func);
    }
}
