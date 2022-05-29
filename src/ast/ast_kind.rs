use super::{BoxedNode, Nodes};
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
    Array(Nodes<'a>),
    ArrayDeclaration {
        data_type: Types,
        dim1: usize,
        dim2: Option<usize>,
    },
    ArrayVal {
        name: String,
        idx_1: BoxedNode<'a>,
        idx_2: Option<BoxedNode<'a>>,
    },
    Assignment {
        assignee: BoxedNode<'a>,
        global: bool,
        value: BoxedNode<'a>,
    },
    UnaryOperation {
        operator: Operator,
        operand: BoxedNode<'a>,
    },
    BinaryOperation {
        operator: Operator,
        lhs: BoxedNode<'a>,
        rhs: BoxedNode<'a>,
    },
    Main {
        assignments: Nodes<'a>,
        body: Nodes<'a>,
        functions: Nodes<'a>,
    },
    Argument {
        arg_type: Types,
        name: String,
    },
    Function {
        arguments: Nodes<'a>,
        body: Nodes<'a>,
        name: String,
        return_type: Types,
    },
    Write {
        exprs: Nodes<'a>,
    },
    Read,
    Decision {
        expr: BoxedNode<'a>,
        statements: Nodes<'a>,
        else_block: Option<BoxedNode<'a>>,
    },
    ElseBlock {
        statements: Nodes<'a>,
    },
    While {
        expr: BoxedNode<'a>,
        statements: Nodes<'a>,
    },
    For {
        assignment: BoxedNode<'a>,
        expr: BoxedNode<'a>,
        statements: Nodes<'a>,
    },
    FuncCall {
        name: String,
        exprs: Nodes<'a>,
    },
    Return(BoxedNode<'a>),
    ReadCSV(BoxedNode<'a>),
    PureDataframeOp {
        name: String,
        operator: Operator,
    },
    UnaryDataframeOp {
        column: BoxedNode<'a>,
        name: String,
        operator: Operator,
    },
    Correlation {
        name: String,
        column_1: BoxedNode<'a>,
        column_2: BoxedNode<'a>,
    },
    Plot {
        name: String,
        column_1: BoxedNode<'a>,
        column_2: BoxedNode<'a>,
    },
    Histogram {
        column: BoxedNode<'a>,
        name: String,
        bins: BoxedNode<'a>,
    },
}

impl From<&AstNodeKind<'_>> for String {
    fn from(val: &AstNodeKind) -> Self {
        match val {
            AstNodeKind::Integer(n) => n.to_string(),
            AstNodeKind::Id(s) | AstNodeKind::String(s) => s.clone(),
            AstNodeKind::Assignment { assignee, .. } => assignee.into(),
            AstNodeKind::ArrayVal { name, .. } => name.clone(),
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

impl<'a> From<usize> for AstNodeKind<'a> {
    fn from(i: usize) -> Self {
        AstNodeKind::Integer(i.try_into().unwrap())
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
            Self::Main {
                assignments,
                body,
                functions,
            } => write!(f, "Main(({assignments:#?}, {:#?}, {:#?}))", functions, body),
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
            Self::PureDataframeOp { name, operator } => {
                write!(f, "PureDataframeOp({operator:?}, {name})")
            }
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
        matches!(self, Self::Array(_) | Self::ArrayDeclaration { .. })
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self, Self::Assignment { .. } | Self::Argument { .. })
    }

    pub fn get_dimensions(&self) -> Result<Dimensions, Dimensions> {
        if !self.is_array() {
            return Ok((None, None));
        }
        match self {
            Self::ArrayDeclaration { dim1, dim2, .. } => Ok((Some(*dim1), *dim2)),
            Self::Array(exprs) => {
                let dim1 = Some(exprs.len());
                let dim2 = exprs.get(0).unwrap().get_dimensions()?.0;
                let errors: Vec<_> = exprs
                    .iter()
                    .map(|expr| {
                        let expr_dim_1 = expr.get_dimensions()?.0;
                        if expr_dim_1 == dim2 {
                            Ok(())
                        } else {
                            Err((expr_dim_1, dim2))
                        }
                    })
                    .filter_map(Result::err)
                    .collect();
                if errors.is_empty() {
                    Ok((dim1, dim2))
                } else {
                    Err(*errors.get(0).unwrap())
                }
            }
            _ => unreachable!("{self:?}"),
        }
    }
}
