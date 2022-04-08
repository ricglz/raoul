use pest_consume::match_nodes;
use pest_consume::Parser;

use crate::enums::{Operations, Types};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct LanguageParser;

use pest_consume::Error;
type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, bool>;

// This is the other half of the parser, using pest_consume.
#[pest_consume::parser]
impl LanguageParser {
    // Extra
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    // Types
    fn void(_input: Node) -> Result<Types> {
        Ok(Types::VOID)
    }

    // Operations
    fn not(_input: Node) -> Result<Operations> {
        Ok(Operations::NOT)
    }

    // Values
    fn int_cte(input: Node) -> Result<i64> {
        input
            .as_str()
            .parse::<i64>()
            // `input.error` links the error to the location in the input file where it occurred.
            .map_err(|e| input.error(e))
    }

    // ID
    fn id(input: Node) -> Result<&str> {
        Ok(input.as_str())
    }

    // Grammar
    fn expr(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [and_term(_)] => (),
        ))
    }

    fn and_term(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [comp_term(_)] => (),
        ))
    }

    fn comp_term(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [rel_term(_)] => (),
        ))
    }

    fn rel_term(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [art_term(_)] => (),
        ))
    }

    fn art_term(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [fact_term(_)] => (),
        ))
    }

    fn fact_term(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [operand(_)] => (),
        ))
    }

    fn operand(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [operand_value(_)] => (),
            [not(_), operand_value(_)] => (),
        ))
    }

    fn operand_value(input: Node) -> Result<()> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("operand_value");
        }
        Ok(match_nodes!(input.into_children();
            [expr(_)] => (),
            [int_cte(_)] => (),
            [id(_)] => (),
        ))
    }

    fn exprs(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [expr(_)..] => (),
        ))
    }

    fn assignment(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [id(_), expr(_)] => (),
        ))
    }

    fn write(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [exprs(_)] => (),
        ))
    }

    fn statement(input: Node) -> Result<()> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("statement");
        }
        Ok(match_nodes!(input.into_children();
            [assignment(_)] => (),
            [write(_)] => (),
        ))
    }

    fn block(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [statement(_statements)..] => (),
        ))
    }

    fn function(input: Node) -> Result<()> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("function");
        }
        Ok(match_nodes!(input.into_children();
            [assignment(_)] => (),
            [write(_)] => (),
        ))
    }

    fn program(input: Node) -> Result<()> {
        Ok(match_nodes!(input.into_children();
            [function(_).., _, block(_), _] => (),
            [_, block(_), _] => (),
        ))
    }
}

fn parse(source: &str, debug: bool) -> Result<()> {
    let inputs = LanguageParser::parse_with_userdata(Rule::program, &source, debug)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    LanguageParser::program(input)
}

pub fn parse_file(filename: &str, debug: bool) -> Result<()> {
    let file = std::fs::read_to_string(filename).expect(filename);
    parse(file, debug)
}
