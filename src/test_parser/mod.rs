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

#[allow(dead_code)]
pub fn parse_file(filename: &str, debug: bool) -> Result<(), Error<Rule>> {
    let program = std::fs::read_to_string(filename).expect(filename);
    if debug {
        println!("Testing {:?}", filename);
    }
    parse(&program)
}
#[deny(dead_code)]
#[cfg(test)]
mod tests {
    use std::fs::read_dir;

    use super::*;

    #[test]
    fn valid_files() {
        let paths = read_dir("examples/valid").unwrap();
        for path in paths {
            let file_path = path.expect("File must exist").path();
            let file = file_path.to_str().unwrap();
            assert!(parse_file(file, true).is_ok());
        }
    }

    #[test]
    fn invalid_file() {
        let filename = "examples/invalid/syntax-error.ra";
        assert!(parse_file(&filename, true).is_err());
    }
}
