use pest_consume::Parser;
use pest_consume::match_nodes;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct LanguageParser;

use pest_consume::Error;
type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

// This is the other half of the parser, using pest_consume.
#[pest_consume::parser]
impl LanguageParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn void(_input: Node) -> Result<crate::enums::Types> {
        Ok(crate::enums::Types::VOID)
    }

    fn block(input: Node) -> Result<i32> {
        println!("block: {:?}", input);
        Ok(42)
    }

    fn statement(input: Node) -> Result<i32> {
        println!("statetement: {:?}", input);
        Ok(42)
    }

    fn function(input: Node) -> Result<i32> {
        println!("function: {:?}", input);
        Ok(42)
    }

    fn program(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [function(_)..] => (),
            [block(_)] => (),
            [void(_)] => (),
            [EOI(_)] => (),
        ))
    }
}

pub fn parse(filename: &str) -> Result<()> {
    let file = std::fs::read_to_string(filename).expect(filename);
    let inputs = LanguageParser::parse(Rule::program, &file)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    LanguageParser::program(input)
}
