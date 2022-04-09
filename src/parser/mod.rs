use pest::iterators::Pair;
use pest_consume::match_nodes;
use pest_consume::Parser;

use crate::ast::AstNode;
use crate::enums::{Operations, Types};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct LanguageParser;

use pest_consume::Error;
type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, bool>;
pub type Statement<'a> = (AstNode<'a>, Pair<'a, Rule>);
pub type Statements<'a> = Vec<Statement<'a>>;

// This is the other half of the parser, using pest_consume.
#[pest_consume::parser]
impl LanguageParser {
    // Extra
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn global(_input: Node) -> Result<()> {
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
    fn int_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<i64>()
            .map_err(|e| input.error(e))
            .unwrap();
        Ok(AstNode::Integer(value))
    }

    // ID
    fn id(input: Node) -> Result<AstNode> {
        Ok(AstNode::Id(String::from(input.as_str())))
    }

    // Grammar
    fn expr(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [and_term(value)] => value,
        ))
    }

    fn and_term(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [comp_term(value)] => value,
        ))
    }

    fn comp_term(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [rel_term(value)] => value,
        ))
    }

    fn rel_term(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [art_term(value)] => value,
        ))
    }

    fn art_term(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [fact_term(value)] => value,
        ))
    }

    fn fact_term(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [operand(value)] => value,
        ))
    }

    fn operand(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [operand_value(value)] => value,
            [not(operation), operand_value(operand)] => AstNode::UnaryOperation { operation: operation, operand: Box::new(operand) }
        ))
    }

    fn operand_value(input: Node) -> Result<AstNode> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("operand_value");
        }
        Ok(match_nodes!(input.into_children();
            [expr(expr)] => expr,
            [int_cte(number)] => number,
            [id(id)] => id,
        ))
    }

    fn exprs(input: Node) -> Result<Vec<AstNode>> {
        Ok(match_nodes!(input.into_children();
            [expr(exprs)..] => exprs.collect(),
        ))
    }

    fn assignment(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [global(_), id(id), expr(value)] => AstNode::Assignment { global: true, name: String::from(id), value: Box::new(value) },
            [id(id), expr(value)] => AstNode::Assignment { global: false, name: String::from(id), value: Box::new(value) },
        ))
    }

    fn write(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [exprs(exprs)] => AstNode::Write { exprs: exprs },
        ))
    }

    fn statement<'a>(input: Node<'a>) -> Result<Statement<'a>> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("statement");
        }
        let pair = input.as_pair().clone();
        Ok(match_nodes!(input.into_children();
            [assignment(node)] => (node, pair),
            [write(node)] => (node, pair),
        ))
    }

    fn block<'a>(input: Node<'a>) -> Result<Statements<'a>> {
        Ok(match_nodes!(input.into_children();
            [statement(statements)..] => statements.collect(),
        ))
    }

    fn function(input: Node) -> Result<AstNode> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("function");
        }
        Ok(match_nodes!(input.into_children();
            [id(id), _, block(body)] => {
                AstNode::Function {name: String::from(id), body: body}
            },
        ))
    }

    fn program(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [function(functions).., _, block(body), _] => AstNode::Main { functions: functions.collect(), body: body },
        ))
    }
}

pub fn parse<'a>(source: &'a str, debug: bool) -> Result<AstNode<'a>> {
    let inputs = LanguageParser::parse_with_userdata(Rule::program, &source, debug)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    LanguageParser::program(input)
}
