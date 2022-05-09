use super::AstNode;
use crate::{
    dir_func::variable::Dimensions,
    enums::{Operator, Types},
};
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
    ArrayVal {
        name: String,
        idx_1: Box<AstNode<'a>>,
        idx_2: Option<Box<AstNode<'a>>>,
    },
    Assignment {
        assignee: Box<AstNode<'a>>,
        global: bool,
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
    ReadCSV(Box<AstNode<'a>>),
    UnaryDataframeOp {
        column: Box<AstNode<'a>>,
        name: String,
        operator: Operator,
    },
    Correlation {
        name: String,
        column_1: Box<AstNode<'a>>,
        column_2: Box<AstNode<'a>>,
    },
    Plot {
        name: String,
        column_1: Box<AstNode<'a>>,
        column_2: Box<AstNode<'a>>,
    },
    Histogram {
        column: Box<AstNode<'a>>,
        name: String,
        bins: Box<AstNode<'a>>,
    },
}

impl<'a> From<AstNodeKind<'a>> for String {
    fn from(val: AstNodeKind) -> Self {
        match val {
            AstNodeKind::Integer(n) => n.to_string(),
            AstNodeKind::Id(s) => s.to_string(),
            AstNodeKind::String(s) => s.to_string(),
            AstNodeKind::Assignment { assignee, .. } => assignee.into(),
            AstNodeKind::ArrayVal { name, .. } => name,
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
            Self::Id(s) => write!(f, "Id({})", s),
            Self::Integer(n) => write!(f, "Integer({})", n),
            Self::Float(n) => write!(f, "Float({})", n),
            Self::String(s) => write!(f, "String({})", s),
            Self::Bool(s) => write!(f, "Bool({})", s),
            Self::Array(s) => write!(f, "Array({s:?})"),
            Self::ArrayDeclaration {
                data_type,
                dim1,
                dim2,
            } => {
                write!(f, "ArrayDeclaration({data_type:?}, {dim1}, {dim2:?})")
            }
            Self::ArrayVal { name, idx_1, idx_2 } => {
                write!(f, "ArrayVal({name}, {idx_1:?}, {idx_2:?})")
            }
            Self::Assignment {
                assignee,
                global,
                value,
            } => write!(f, "Assignment({}, {:?}, {:?})", global, assignee, value),
            Self::UnaryOperation {
                operator: operation,
                operand,
            } => {
                write!(f, "Unary({:?}, {:?})", operation, operand)
            }
            Self::Main { functions, body } => write!(f, "Main(({:#?}, {:#?}))", functions, body),
            Self::Argument { arg_type, name } => write!(f, "Argument({:?}, {})", arg_type, name),
            Self::Function {
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
            Self::Write { exprs } => write!(f, "Write({:?})", exprs),
            Self::Read => write!(f, "Read"),
            Self::BinaryOperation { operator, lhs, rhs } => {
                write!(f, "BinaryOperation({:?}, {:?}, {:?})", operator, lhs, rhs)
            }
            Self::Decision {
                expr,
                statements,
                else_block,
            } => {
                write!(f, "Decision({expr:?}, {statements:?}, {else_block:?})")
            }
            Self::ElseBlock { statements } => write!(f, "ElseBlock({:?})", statements),
            Self::While { expr, statements } => write!(f, "While({:?}, {:?})", expr, statements),
            Self::For {
                expr,
                statements,
                assignment,
            } => {
                write!(f, "For({expr:?}, {statements:?}, {assignment:?})")
            }
            Self::FuncCall { name, exprs } => write!(f, "FunctionCall({name}, {exprs:?})"),
            Self::Return(expr) => write!(f, "Return({expr:?})"),
            Self::ReadCSV(file) => write!(f, "ReadCSV({file:?})"),
            Self::UnaryDataframeOp {
                operator,
                name,
                column,
            } => {
                write!(f, "UnaryDataframeOp({operator:?}, {name}, {column:?})")
            }
            Self::Correlation {
                name,
                column_1,
                column_2,
            } => {
                write!(f, "Correlation({name}, {column_1:?}, {column_2:?})")
            }
            Self::Plot {
                name,
                column_1,
                column_2,
            } => write!(f, "Plot({name}, {column_1:?}, {column_2:?})"),
            Self::Histogram { column, name, bins } => {
                write!(f, "Histogram({column:?}, {name}, {bins:?})")
            }
        }
    }
}

impl<'a> AstNodeKind<'a> {
    pub fn is_array(&self) -> bool {
        match self {
            Self::Array(_) | Self::ArrayDeclaration { .. } => true,
            _ => false,
        }
    }

    pub fn get_dimensions(&self) -> Result<Dimensions, Dimensions> {
        if !self.is_array() {
            return Ok((None, None));
        }
        match self {
            Self::ArrayDeclaration { dim1, dim2, .. } => Ok((Some(*dim1), dim2.to_owned())),
            Self::Array(exprs) => {
                let dim1 = Some(exprs.len());
                let dim2 = exprs.get(0).unwrap().get_dimensions()?.0;
                let errors: Vec<_> = exprs
                    .into_iter()
                    .map(|expr| -> Result<(), Dimensions> {
                        let expr_dim_1 = expr.get_dimensions()?.0;
                        match expr_dim_1 == dim2 {
                            true => Ok(()),
                            false => Err((expr_dim_1, dim2)),
                        }
                    })
                    .filter_map(|v| v.err())
                    .collect();
                match errors.is_empty() {
                    true => Ok((dim1, dim2)),
                    false => Err(errors.get(0).unwrap().clone()),
                }
            }
            _ => unreachable!("{self:?}"),
        }
    }
}
