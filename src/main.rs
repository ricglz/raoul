mod args;
mod enums;
mod parser;

use std::process::exit;

use crate::parser::parse;
use crate::args::parse_args;

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
