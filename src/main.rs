mod args;
mod enums;
mod test_parser;
#[macro_use]
extern crate pest_derive;

use std::process::exit;

use crate::args::parse_args;
use crate::test_parser::parse;

fn main() {
    let matches = parse_args();
    let filename = matches.value_of("file").expect("required");
    let debug = matches.is_present("debug");
    if debug {
        println!("Starting parsing");
    }
    if let Err(error) = parse(filename, debug) {
        println!("Parsing error {}", error.to_string());
        exit(1);
    }
    if debug {
        println!("Parsing ended sucessfully");
    }
}
