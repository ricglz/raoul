mod args;
mod enums;
mod parser;

use std::process::exit;

use crate::parser::parse;
use crate::args::parse_args;

fn main() {
    let matches = parse_args();
    let filename = matches.value_of("file").expect("required");
    if let Err(error) = parse(filename) {
        println!("Parsing error {}", error.to_string());
        exit(1);
    }
    println!("Successful parsing!")
}
