use super::AstNode;
use crate::enums::{Operator, Types};
use std::fmt;

#[derive(PartialEq, Clone)]
pub enum AstNodeKind<'a> {
    Id(String),
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<AstNode<'a>>),
    ArrayDeclaration {
        data_type: Types,
        dim1: usize,
        dim2: Option<usize>,
    },
    Assignment {
        global: bool,
        name: String,
        value: Box<AstNode<'a>>,
    },
    UnaryOperation {
        operator: Operator,
        operand: Box<AstNode<'a>>,
    },
    BinaryOperation {
        operator: Operator,
        lhs: Box<AstNode<'a>>,
        rhs: Box<AstNode<'a>>,
    },
    Main {
        functions: Vec<AstNode<'a>>,
        body: Vec<AstNode<'a>>,
    },
    Argument {
        arg_type: Types,
        name: String,
    },
    Function {
        arguments: Vec<AstNode<'a>>,
        body: Vec<AstNode<'a>>,
        name: String,
        return_type: Types,
    },
    Write {
        exprs: Vec<AstNode<'a>>,
    },
    Read,
    Decision {
        expr: Box<AstNode<'a>>,
        statements: Vec<AstNode<'a>>,
        else_block: Option<Box<AstNode<'a>>>,
    },
    ElseBlock {
        statements: Vec<AstNode<'a>>,
    },
    While {
        expr: Box<AstNode<'a>>,
        statements: Vec<AstNode<'a>>,
    },
    For {
        assignment: Box<AstNode<'a>>,
        expr: Box<AstNode<'a>>,
        statements: Vec<AstNode<'a>>,
    },
    FuncCall {
        name: String,
        exprs: Vec<AstNode<'a>>,
    },
    Return(Box<AstNode<'a>>),
}

impl<'a> From<AstNodeKind<'a>> for String {
    fn from(val: AstNodeKind) -> Self {
        match val {
            AstNodeKind::Integer(n) => n.to_string(),
            AstNodeKind::Id(s) => s.to_string(),
            AstNodeKind::String(s) => s.to_string(),
            AstNodeKind::Assignment { name, .. } => name,
            node => unreachable!("Node {:?}, cannot be a string", node),
        }
    }
}

impl<'a> From<AstNodeKind<'a>> for usize {
    fn from(val: AstNodeKind) -> Self {
        match val {
            AstNodeKind::Integer(n) => n.try_into().unwrap_or(0),
            node => unreachable!("{node:?}, cannot be a usize"),
        }
    }
}

impl fmt::Debug for AstNodeKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AstNodeKind::Id(s) => write!(f, "Id({})", s),
            AstNodeKind::Integer(n) => write!(f, "Integer({})", n),
            AstNodeKind::Float(n) => write!(f, "Float({})", n),
            AstNodeKind::String(s) => write!(f, "String({})", s),
            AstNodeKind::Bool(s) => write!(f, "Bool({})", s),
            AstNodeKind::Array(s) => write!(f, "Array({s:?})"),
            Self::ArrayDeclaration {
                data_type,
                dim1,
                dim2,
            } => {
                write!(f, "ArrayDeclaration({data_type:?}, {dim1}, {dim2:?})")
            }
            AstNodeKind::Assignment {
                global,
                name,
                value,
            } => write!(f, "Assignment({}, {}, {:?})", global, name, value),
            AstNodeKind::UnaryOperation {
                operator: operation,
                operand,
            } => {
                write!(f, "Unary({:?}, {:?})", operation, operand)
            }
            AstNodeKind::Main { functions, body } => {
                write!(f, "Main(({:#?}, {:#?}))", functions, body)
            }
            AstNodeKind::Argument { arg_type, name } => {
                write!(f, "Argument({:?}, {})", arg_type, name)
            }
            AstNodeKind::Function {
                arguments,
                body,
                name,
                return_type,
            } => {
                write!(
                    f,
                    "Function({}, {:?}, {:?}, {:#?})",
                    name, return_type, arguments, body
                )
            }
            AstNodeKind::Write { exprs } => write!(f, "Write({:?})", exprs),
            AstNodeKind::Read => write!(f, "Read"),
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                write!(f, "BinaryOperation({:?}, {:?}, {:?})", operator, lhs, rhs)
            }
            AstNodeKind::Decision {
                expr,
                statements,
                else_block,
            } => {
                write!(f, "Decision({expr:?}, {statements:?}, {else_block:?})")
            }
            AstNodeKind::ElseBlock { statements } => {
                write!(f, "ElseBlock({:?})", statements)
            }
            AstNodeKind::While { expr, statements } => {
                write!(f, "While({:?}, {:?})", expr, statements)
            }
            AstNodeKind::For {
                expr,
                statements,
                assignment,
            } => {
                write!(f, "For({expr:?}, {statements:?}, {assignment:?})")
            }
            AstNodeKind::FuncCall { name, exprs } => {
                write!(f, "FunctionCall({name}, {exprs:?})")
            }
            AstNodeKind::Return(expr) => write!(f, "Return({expr:?})"),
        }
    }
}

impl AstNodeKind<'_> {
    pub fn is_array(&self) -> bool {
        match self {
            Self::Array(_) | Self::ArrayDeclaration { .. } => true,
            _ => false,
        }
    }
}
