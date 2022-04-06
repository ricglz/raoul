use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct MyParser;

pub fn parse(filename: &str, _verbose: bool) -> Result<(), Error<Rule>> {
    let program = std::fs::read_to_string(filename).expect(filename);
    println!("Testing {}", filename);
    if let Err(err) = MyParser::parse(Rule::program, &program) {
        Err(err)
    } else {
        Ok(())
    }
}
