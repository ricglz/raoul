extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
struct MyParser;

pub fn parse(filename: &str) {
    let program = std::fs::read_to_string(filename).expect(filename);
    println!("Testing {}", filename);
    if let Err(err) = MyParser::parse(Rule::PROGRAM, &program) {
        println!("{}", err);
    } else {
        println!("Is a valid program")
    }
}

fn main() {
    parse("./example.ra");
}
