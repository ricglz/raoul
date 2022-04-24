use pest_consume::match_nodes;
use pest_consume::Parser;

use crate::ast::ast_kind::AstNodeKind;
use crate::ast::AstNode;
use crate::enums::{Operator, Types};

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
    fn not(_input: Node) -> Result<Operator> {
        Ok(Operator::Not)
    }

    // Values
    fn int_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<i64>()
            .map_err(|e| input.error(e))
            .unwrap();
        let kind = AstNodeKind::Integer(value);
        Ok(AstNode {
            kind,
            span: input.as_span(),
        })
    }

    fn float_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<f64>()
            .map_err(|e| input.error(e))
            .unwrap();
        Ok(AstNode {
            kind: AstNodeKind::Float(value),
            span: input.as_span(),
        })
    }

    fn string_value(input: Node) -> Result<AstNode> {
        Ok(AstNode {
            kind: AstNodeKind::String(input.as_str().to_owned()),
            span: input.as_span(),
        })
    }

    fn bool_cte(input: Node) -> Result<AstNode> {
        let value = input
            .as_str()
            .parse::<bool>()
            .map_err(|e| input.error(e))
            .unwrap();
        Ok(AstNode {
            kind: AstNodeKind::Bool(value),
            span: input.as_span(),
        })
    }

    // ID
    fn id(input: Node) -> Result<AstNode> {
        Ok(AstNode {
            kind: AstNodeKind::Id(input.as_str().to_owned()),
            span: input.as_span(),
        })
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
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [operand_value(value)] => value,
            [not(operator), operand_value(operand)] => {
                let kind = AstNodeKind::UnaryOperation { operator, operand: Box::new(operand) };
                AstNode { kind, span }
            }
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
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [global(_), id(id), expr(value)] => {
                let kind = AstNodeKind::Assignment { global: true, name: String::from(id), value: Box::new(value) };
                AstNode { kind, span }
            },
            [id(id), expr(value)] => {
                let kind = AstNodeKind::Assignment { global: false, name: String::from(id), value: Box::new(value) };
                AstNode { kind, span }
            },
        ))
    }

    fn write(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [exprs(exprs)] => {
                AstNode { kind: AstNodeKind::Write { exprs: exprs }, span }
            },
        ))
    }

    fn statement(input: Node) -> Result<AstNode> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("statement");
        }
        Ok(match_nodes!(input.into_children();
            [assignment(node)] => node,
            [write(node)] => node,
        ))
    }

    fn block<'a>(input: Node<'a>) -> Result<Vec<AstNode<'a>>> {
        Ok(match_nodes!(input.into_children();
            [statement(statements)..] => statements.collect(),
        ))
    }

    fn func_arg(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [id(id), atomic_types(arg_type)] => {
                let kind = AstNodeKind::Argument { arg_type, name: String::from(id) };
                AstNode { kind, span }
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
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [id(id), func_args(arguments), types(return_type), block(body)] => {
                let kind = AstNodeKind::Function {arguments, name: String::from(id), body, return_type};
                AstNode { kind, span }
            },
            [id(id), types(return_type), block(body)] => {
                let kind = AstNodeKind::Function {arguments: Vec::new(), name: String::from(id), body, return_type};
                AstNode { kind, span }
            },
        ))
    }

    fn program(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [function(functions).., _, block(body), _] => {
                let kind = AstNodeKind::Main { functions: functions.collect(), body: body };
                AstNode { kind, span }
            },
        ))
    }
}

pub fn parse<'a>(source: &'a str, debug: bool) -> Result<AstNode<'a>> {
    let inputs = LanguageParser::parse_with_userdata(Rule::program, &source, debug)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    LanguageParser::program(input)
}

#[cfg(test)]
mod tests;
