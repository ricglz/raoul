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

    fn gte(_input: Node) -> Result<Operator> {
        Ok(Operator::Gte)
    }

    fn lte(_input: Node) -> Result<Operator> {
        Ok(Operator::Lte)
    }

    fn gt(_input: Node) -> Result<Operator> {
        Ok(Operator::Gt)
    }

    fn lt(_input: Node) -> Result<Operator> {
        Ok(Operator::Lt)
    }

    fn rel_op(input: Node) -> Result<Operator> {
        Ok(match_nodes!(input.into_children();
            [gte(value)] => value,
            [lte(value)] => value,
            [gt(value)] => value,
            [lt(value)] => value,
        ))
    }

    fn eq(_input: Node) -> Result<Operator> {
        Ok(Operator::Eq)
    }

    fn ne(_input: Node) -> Result<Operator> {
        Ok(Operator::Ne)
    }

    fn comp_op(input: Node) -> Result<Operator> {
        Ok(match_nodes!(input.into_children();
            [eq(value)] => value,
            [ne(value)] => value,
        ))
    }

    fn sum(_input: Node) -> Result<Operator> {
        Ok(Operator::Sum)
    }

    fn minus(_input: Node) -> Result<Operator> {
        Ok(Operator::Minus)
    }

    fn art_op(input: Node) -> Result<Operator> {
        Ok(match_nodes!(input.into_children();
            [sum(value)] => value,
            [minus(value)] => value,
        ))
    }

    fn times(_input: Node) -> Result<Operator> {
        Ok(Operator::Times)
    }

    fn div(_input: Node) -> Result<Operator> {
        Ok(Operator::Div)
    }

    fn fact_op(input: Node) -> Result<Operator> {
        Ok(match_nodes!(input.into_children();
            [times(value)] => value,
            [div(value)] => value,
        ))
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

    fn func_call(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [id(id)] => {
                let kind = AstNodeKind::FuncCall { name: String::from(id), exprs: Vec::new() };
                AstNode { kind, span }
            },
            [id(id), exprs(exprs)] => {
                let kind = AstNodeKind::FuncCall { name: String::from(id), exprs };
                AstNode { kind, span }
            },
        ))
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
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [and_term(value)] => value,
            [and_term(lhs), and_term(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator: Operator::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            },
        ))
    }

    fn and_term(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [comp_term(value)] => value,
            [comp_term(lhs), comp_term(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator: Operator::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            },
        ))
    }

    fn comp_term(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [rel_term(value)] => value,
            [rel_term(lhs), comp_op(operator), rel_term(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            }
        ))
    }

    fn rel_term(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [art_term(value)] => value,
            [art_term(lhs), rel_op(operator), art_term(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            }
        ))
    }

    fn art_term(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [fact_term(value)] => value,
            [fact_term(lhs), art_op(operator), fact_term(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            }
        ))
    }

    fn fact_term(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [operand(value)] => value,
            [operand(lhs), fact_op(operator), operand(rhs)] => {
                let kind = AstNodeKind::BinaryOperation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                AstNode { kind, span }
            }
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
            [func_call(call)] => call,
        ))
    }

    fn exprs(input: Node) -> Result<Vec<AstNode>> {
        Ok(match_nodes!(input.into_children();
            [expr(exprs)..] => exprs.collect(),
        ))
    }

    fn read(input: Node) -> Result<AstNode> {
        Ok(AstNode {
            kind: AstNodeKind::Read,
            span: input.as_span().clone(),
        })
    }

    fn assignment_exp(input: Node) -> Result<AstNode> {
        // TODO: Still misses some conditions
        if *input.user_data() {
            println!("assignment_exp");
        }
        Ok(match_nodes!(input.into_children();
            [expr(value)] => value,
            [read(value)] => value,
            [declare_arr(value)] => value,
            [arr_cte(arr)] => arr,
        ))
    }

    // Arrays
    fn declare_arr_type(input: Node) -> Result<Types> {
        Ok(match_nodes!(input.into_children();
            [types(data_type)] => data_type,
        ))
    }

    fn declare_arr(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [declare_arr_type(data_type), int_cte(dim1)] => {
                let kind = AstNodeKind::ArrayDeclaration { data_type, dim1: dim1.into(), dim2: None };
                AstNode {kind, span}
            },
            [declare_arr_type(data_type), int_cte(dim1), int_cte(dim2)] => {
                let kind = AstNodeKind::ArrayDeclaration { data_type, dim1: dim1.into(), dim2: Some(dim2.into()) };
                AstNode {kind, span}
            },
        ))
    }

    fn list_cte(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [exprs(exprs)] => {
                AstNode { kind: AstNodeKind::Array(exprs), span }
            },
        ))
    }

    fn mat_cte(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [list_cte(exprs)..] => {
                AstNode { kind: AstNodeKind::Array(exprs.collect()), span }
            },
        ))
    }

    fn arr_cte(input: Node) -> Result<AstNode> {
        Ok(match_nodes!(input.into_children();
            [list_cte(node)] => node,
            [mat_cte(node)] => node,
        ))
    }

    // Condition
    fn else_block(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [block(statements)] => {
                let kind = AstNodeKind::ElseBlock { statements };
                AstNode {kind, span}
            },
            [decision(decision)] => decision,
        ))
    }

    fn decision(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [expr(expr), block(statements)] => {
                let kind = AstNodeKind::Decision {
                    expr: Box::new(expr),
                    statements,
                    else_block: None
                };
                AstNode {kind, span}
            },
            [expr(expr), block(statements), else_block(else_block)] => {
                let kind = AstNodeKind::Decision {
                    expr: Box::new(expr),
                    statements,
                    else_block: Some(Box::new(else_block))
                };
                AstNode {kind, span}
            },
        ))
    }

    // Loops
    fn while_loop(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [expr(expr), block(statements)] => {
                let kind = AstNodeKind::While {
                    expr: Box::new(expr),
                    statements,
                };
                AstNode {kind, span}
            },
        ))
    }

    fn for_loop(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [assignment(assignment), expr(stop_expr), block(statements)] => {
                let assignment_clone = assignment.clone();
                let expr_clone = stop_expr.clone();
                let id_node = AstNode::new(AstNodeKind::Id(String::from(assignment_clone.kind)), assignment_clone.span);
                let expr_kind = AstNodeKind::BinaryOperation {
                    operator: Operator::Lt,
                    lhs: Box::new(id_node),
                    rhs: Box::new(stop_expr),
                };
                let expr = Box::new(AstNode::new(expr_kind, expr_clone.span));
                let kind = AstNodeKind::For { assignment: Box::new(assignment), expr, statements };
                AstNode::new(kind, span)
            },
        ))
    }

    // Inline statements
    fn assignment(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [global(_), id(id), assignment_exp(value)] => {
                let kind = AstNodeKind::Assignment { global: true, name: String::from(id), value: Box::new(value) };
                AstNode { kind, span }
            },
            [id(id), assignment_exp(value)] => {
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

    fn return_statement(input: Node) -> Result<AstNode> {
        let span = input.as_span().clone();
        Ok(match_nodes!(input.into_children();
            [expr(expr)] => {
                AstNode { kind: AstNodeKind::Return(Box::new(expr)), span }
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
            [decision(node)] => node,
            [while_loop(node)] => node,
            [for_loop(node)] => node,
            [func_call(node)] => node,
            [return_statement(node)] => node,
        ))
    }

    fn block<'a>(input: Node<'a>) -> Result<Vec<AstNode<'a>>> {
        Ok(match_nodes!(input.into_children();
            [statement(statements)..] => statements.collect(),
        ))
    }

    // Function
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
