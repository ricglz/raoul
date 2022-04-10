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

    fn int(_input: Node) -> Result<Types> {
        Ok(Types::INT)
    }

    fn float(_input: Node) -> Result<Types> {
        Ok(Types::FLOAT)
    }

    fn string(_input: Node) -> Result<Types> {
        Ok(Types::STRING)
    }

    fn bool(_input: Node) -> Result<Types> {
        Ok(Types::BOOL)
    }

    fn atomic_types(input: Node) -> Result<Types> {
        Ok(match_nodes!(input.into_children();
            [int(value)] => value,
            [float(value)] => value,
            [string(value)] => value,
            [bool(value)] => value,
        ))
    }

    fn types(input: Node) -> Result<Types> {
        Ok(match_nodes!(input.into_children();
            [void(value)] => value,
            [atomic_types(value)] => value,
        ))
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

    fn float_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<f64>()
            .map_err(|e| input.error(e))
            .unwrap();
        Ok(AstNode::Float(value))
    }

    fn string_value(input: Node) -> Result<AstNode> {
        Ok(AstNode::String(input.as_str().to_owned()))
    }

    fn bool_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<bool>()
            .map_err(|e| input.error(e))
            .unwrap();
        Ok(AstNode::Bool(value))
    }

    // ID
    fn id(input: Node) -> Result<AstNode> {
        Ok(AstNode::Id(input.as_str().to_owned()))
    }

    // Expressions
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
            [float_cte(number)] => number,
            [string_value(string)] => string,
            [bool_cte(value)] => value,
            [id(id)] => id,
        ))
    }

    fn exprs(input: Node) -> Result<Vec<AstNode>> {
        Ok(match_nodes!(input.into_children();
            [expr(exprs)..] => exprs.collect(),
        ))
    }

    // Inline statements
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

    fn func_arg(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [id(id), atomic_types(arg_type)] => {
                AstNode::Argument { arg_type, name: String::from(id) }
            },
        ))
    }

    fn func_args(input: Node) -> Result<Vec<AstNode>> {
        Ok(match_nodes!(input.into_children();
            [func_arg(args)..] => args.collect(),
        ))
    }

    // Function
    fn function(input: Node) -> Result<AstNode> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("function");
        }
        Ok(match_nodes!(input.into_children();
            [id(id), func_args(arguments), types(return_type), block(body)] => {
                AstNode::Function {arguments, name: String::from(id), body, return_type}
            },
            [id(id), types(return_type), block(body)] => {
                AstNode::Function {arguments: Vec::new(), name: String::from(id), body, return_type}
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
