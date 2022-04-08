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

#[cfg(test)]
mod tests {
    use std::fs::read_dir;

    use super::*;

    #[test]
    fn example_files() {
        let paths = read_dir("examples").unwrap();
        for path in paths {
            let file_path = path.expect("File must exist").path();
            let file = file_path.to_str().unwrap();
            assert!(parse_file(file, true).is_ok());
        }
    }
}
