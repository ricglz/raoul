use core::fmt;

use crate::ast::ast_kind::AstNodeKind;
use crate::ast::AstNode;
use crate::dir_func::function::VariablesTable;
use crate::dir_func::variable::Variable;
use crate::error::error_kind::RaoulErrorKind;
use crate::error::{RaoulError, Results};

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Types {
    Int,
    Void,
    Float,
    String,
    Bool,
    Dataframe,
}

impl Types {
    #[inline]
    pub fn is_boolish(self) -> bool {
        matches!(self, Types::Int | Types::Bool)
    }

    #[inline]
    fn is_number(self) -> bool {
        matches!(self, Types::Int | Types::Float | Types::String)
    }

    pub fn can_cast(self, to: Types) -> bool {
        if self.is_number() && to.is_number() {
            return true;
        }
        if self.is_boolish() && to.is_boolish() {
            return true;
        }
        self == to
    }

    pub fn assert_cast(self, to: Types, node: AstNode) -> Results<()> {
        if self.can_cast(to) {
            return Ok(());
        }
        let error = RaoulError::new_vec(node, RaoulErrorKind::InvalidCast { from: self, to });
        Err(error)
    }

    pub fn binary_operator_type(
        operator: Operator,
        lhs_type: Types,
        rhs_type: Types,
    ) -> std::result::Result<Types, RaoulErrorKind> {
        match operator {
            Operator::Not | Operator::Or | Operator::And => {
                let type_res = Types::Bool;
                match (lhs_type.is_boolish(), rhs_type.is_boolish()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: type_res,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: type_res,
                    }),
                }
            }
            Operator::Gte | Operator::Lte | Operator::Gt | Operator::Lt => {
                let type_res = Types::Bool;
                match (lhs_type.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: type_res,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: type_res,
                    }),
                }
            }
            Operator::Eq | Operator::Ne => {
                if lhs_type.can_cast(rhs_type) {
                    Ok(Types::Bool)
                } else {
                    Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: rhs_type,
                    })
                }
            }
            Operator::Sum | Operator::Minus | Operator::Times | Operator::Div => {
                if lhs_type == rhs_type && lhs_type == Types::Int {
                    return Ok(Types::Int);
                }
                let type_res = Types::Float;
                match (lhs_type.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: type_res,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: type_res,
                    }),
                }
            }
            _ => unreachable!("{:?}", operator),
        }
    }

    #[inline]
    fn get_variable<'a>(
        name: &str,
        variables: &'a VariablesTable,
        global: &'a VariablesTable,
    ) -> Option<&'a Variable> {
        variables.get(name).or_else(|| global.get(name))
    }

    pub fn from_node<'a>(
        v: &AstNode<'a>,
        variables: &VariablesTable,
        global: &VariablesTable,
    ) -> Results<'a, Types> {
        let clone = v.clone();
        match &v.kind {
            AstNodeKind::Integer(_) => Ok(Types::Int),
            AstNodeKind::Float(_)
            | AstNodeKind::UnaryDataframeOp { .. }
            | AstNodeKind::Correlation { .. } => Ok(Types::Float),
            AstNodeKind::String(_) | AstNodeKind::Read => Ok(Types::String),
            AstNodeKind::Bool(_) => Ok(Types::Bool),
            AstNodeKind::Id(name) | AstNodeKind::ArrayVal { name, .. } => {
                match Types::get_variable(name, variables, global) {
                    Some(variable) => Ok(variable.data_type),
                    None => Err(RaoulError::new_vec(
                        clone,
                        RaoulErrorKind::UndeclaredVar(name.to_string()),
                    )),
                }
            }
            AstNodeKind::FuncCall { name, .. } => {
                match Types::get_variable(name, variables, global) {
                    Some(variable) => Ok(variable.data_type),
                    None => Err(RaoulError::new_vec(
                        clone,
                        RaoulErrorKind::UndeclaredFunction(name.to_string()),
                    )),
                }
            }
            AstNodeKind::ArrayDeclaration { data_type, .. } => Ok(*data_type),
            AstNodeKind::Array(exprs) => {
                let (types, errors): (Vec<_>, Vec<_>) = exprs
                    .iter()
                    .map(|node| Types::from_node(node, variables, global))
                    .partition(Results::is_ok);
                if !errors.is_empty() {
                    return Err(errors.into_iter().flat_map(Results::unwrap_err).collect());
                }
                let first_type = types.get(0).unwrap().clone().unwrap();
                let errors: Vec<_> = types
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, v)| {
                        let data_type = v.unwrap();
                        let node = exprs.get(i).unwrap().clone();
                        let res = if data_type.can_cast(first_type) {
                            Ok(())
                        } else {
                            Err(RaoulError::new(
                                node,
                                RaoulErrorKind::InvalidCast {
                                    from: data_type,
                                    to: first_type,
                                },
                            ))
                        };
                        res.err()
                    })
                    .collect();
                if errors.is_empty() {
                    Ok(first_type)
                } else {
                    Err(errors)
                }
            }
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                let lhs_type = Types::from_node(&*lhs, variables, global)?;
                let rhs_type = Types::from_node(&*rhs, variables, global)?;
                match Types::binary_operator_type(*operator, lhs_type, rhs_type) {
                    Err(kind) => Err(RaoulError::new_vec(clone, kind)),
                    Ok(op_type) => Ok(op_type),
                }
            }
            AstNodeKind::UnaryOperation { operator, operand } => match operator {
                Operator::Not => {
                    let operand_type = Types::from_node(&*operand, variables, global)?;
                    if operand_type.is_boolish() {
                        Ok(Types::Bool)
                    } else {
                        Err(RaoulError::new_vec(
                            clone,
                            RaoulErrorKind::InvalidCast {
                                from: operand_type,
                                to: Types::Bool,
                            },
                        ))
                    }
                }
                _ => unreachable!("{:?}", operator),
            },
            AstNodeKind::ReadCSV(_) => Ok(Self::Dataframe),
            kind => Err(RaoulError::new_vec(
                clone,
                RaoulErrorKind::EnteredUnreachable(format!("{kind:?}")),
            )),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Operator {
    // Boolean
    Not,
    Or,
    And,
    // Relational
    Gte,
    Lte,
    Gt,
    Lt,
    // Equality
    Eq,
    Ne,
    // Aritmetic
    Sum,
    Minus,
    Times,
    Div,
    Inc,
    // ByteCode
    Assignment,
    Print,
    PrintNl,
    Read,
    Goto,
    GotoF,
    End,
    // Functions
    Return,
    EndProc,
    Era,
    GoSub,
    Param,
    // Arrays
    Ver,
    // Dataframe
    Average,
    Std,
    Mode,
    Variance,
    Corr,
    ReadCSV,
    Plot,
    Histogram,
}

impl Operator {
    pub fn is_goto(self) -> bool {
        matches!(self, Operator::Goto | Operator::GotoF)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:10}", format!("{:?}", self))
    }
}
