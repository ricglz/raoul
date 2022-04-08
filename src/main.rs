mod args;
mod enums;
mod test_parser;
#[macro_use]
extern crate pest_derive;

use std::process::exit;

use args::parse_args;
use test_parser::parse_file;

fn main() {
    let matches = parse_args();
    let filename = matches.value_of("file").expect("required");
    let debug = matches.is_present("debug");
    if debug {
        println!("Starting parsing");
    }
    if let Err(error) = parse_file(filename, debug) {
        println!("Parsing error {}", error.to_string());
        exit(1);
    }
    if debug {
        println!("Parsing ended sucessfully");
    }
}
