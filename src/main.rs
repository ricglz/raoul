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
    let verbose = matches.is_present("verbose");
    if verbose {
        println!("Starting parsing");
    }
    if let Err(error) = parse(filename, verbose) {
        println!("Parsing error {}", error.to_string());
        exit(1);
    }
    if verbose {
        println!("Parsing ended sucessfully");
    }
}
