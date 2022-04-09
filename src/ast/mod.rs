use std::fmt;

use crate::enums::Operations;
use crate::parser::Statements;

#[derive(PartialEq)]
pub enum AstNode<'a> {
    Id(String),
    Integer(i64),
    Assignment {
        global: bool,
        name: String,
        value: Box<AstNode<'a>>,
    },
    UnaryOperation {
        operation: Operations,
        operand: Box<AstNode<'a>>,
    },
    Main {
        functions: Vec<AstNode<'a>>,
        body: Statements<'a>,
    },
    Function {
        name: String,
        // arguments: Vec<AstNode<'a>>,
        body: Statements<'a>,
    },
    Write {
        exprs: Vec<AstNode<'a>>,
    },
}

impl<'a> From<AstNode<'a>> for String {
    fn from(val: AstNode) -> Self {
        match val {
            AstNode::Function { name, body: _ } => String::from(name),
            AstNode::Integer(n) => n.to_string(),
            AstNode::Id(s) => s.to_string(),
            node => unreachable!("Node {:?}, cannot be a string", node),
        }
    }
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AstNode::Id(s) => write!(f, "Id({})", s),
            AstNode::Integer(n) => write!(f, "Integer({})", n),
            AstNode::Assignment {
                global,
                name,
                value,
            } => write!(f, "Assignment({}, {}, {:?})", global, name, value),
            AstNode::UnaryOperation { operation, operand } => {
                write!(f, "Unary({:?}, {:?})", operation, operand)
            }
            AstNode::Main { functions, body } => {
                let nodes: Vec<&AstNode> = body.iter().map(|x| &x.0).collect();
                write!(f, "Main(({:?}, {:#?}))", functions, nodes)
            }
            AstNode::Function { name, body } => {
                let nodes: Vec<&AstNode> = body.iter().map(|x| &x.0).collect();
                write!(f, "Function({} {:#?})", name, nodes)
            }
            AstNode::Write { exprs } => write!(f, "Write({:?})", exprs),
        }
    }
}
