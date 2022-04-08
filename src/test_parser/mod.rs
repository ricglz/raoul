use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct MyParser;

fn parse(source: &str) -> Result<(), Error<Rule>> {
    if let Err(err) = MyParser::parse(Rule::program, &source) {
        Err(err)
    } else {
        Ok(())
    }
}

pub fn parse_file(filename: &str, debug: bool) -> Result<(), Error<Rule>> {
    let program = std::fs::read_to_string(filename).expect(filename);
    if debug {
        println!("Testing {:?}", filename);
    }
    parse(&program)
}
