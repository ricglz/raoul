extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;
mod args;

use crate::parser::parse;
use crate::args::parse_args;

fn main() {
    let matches = parse_args();
    parse(matches.value_of("file").expect("required"));
}
