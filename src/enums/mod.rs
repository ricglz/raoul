use crate::ast::ast_kind::AstNodeKind;
use crate::ast::AstNode;
use crate::dir_func::function::VariablesTable;
use crate::error::error_kind::RaoulErrorKind;
use crate::error::{RaoulError, Result};

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Types {
    INT,
    VOID,
    FLOAT,
    STRING,
    BOOL,
}

impl Types {
    fn is_boolish(&self) -> bool {
        match self {
            Types::INT | Types::BOOL => true,
            _ => false,
        }
    }

    fn is_number(&self) -> bool {
        match self {
            Types::INT | Types::FLOAT | Types::STRING => true,
            _ => false,
        }
    }

    fn binary_operator_type(
        operator: Operator,
        lhs_type: Types,
        rhs_type: Types,
    ) -> std::result::Result<Types, RaoulErrorKind> {
        match operator {
            Operator::Not | Operator::Or | Operator::And => {
                let res_type = Types::BOOL;
                match (lhs_type.is_boolish(), rhs_type.is_boolish()) {
                    (true, true) => Ok(res_type),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: res_type,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: res_type,
                    }),
                }
            }
            Operator::Gte | Operator::Lte | Operator::Gt | Operator::Lt => {
                let res_type = Types::BOOL;
                match (lhs_type.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(res_type),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: res_type,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: res_type,
                    }),
                }
            }
            Operator::Eq | Operator::Ne => match lhs_type == rhs_type {
                true => Ok(Types::BOOL),
                false => Err(RaoulErrorKind::InvalidCast {
                    from: lhs_type,
                    to: rhs_type,
                }),
            },
            Operator::Sum | Operator::Minus | Operator::Times | Operator::Div => {
                if lhs_type == rhs_type && lhs_type == Types::INT {
                    return Ok(Types::INT);
                }
                let res_type = Types::FLOAT;
                match (lhs_type.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(res_type),
                    (true, false) => Err(RaoulErrorKind::InvalidCast {
                        from: rhs_type,
                        to: res_type,
                    }),
                    _ => Err(RaoulErrorKind::InvalidCast {
                        from: lhs_type,
                        to: res_type,
                    }),
                }
            }
            _ => unreachable!("{:?}", operator),
        }
    }

    pub fn from_node<'a>(
        v: AstNode<'a>,
        variables: &VariablesTable,
        global: &VariablesTable,
    ) -> Result<'a, Types> {
        let clone = v.clone();
        match v.kind {
            AstNodeKind::Integer(_) => Ok(Types::INT),
            AstNodeKind::Float(_) => Ok(Types::FLOAT),
            AstNodeKind::String(_) => Ok(Types::STRING),
            AstNodeKind::Bool(_) => Ok(Types::BOOL),
            AstNodeKind::Id(name) => {
                if let Some(variable) = variables.get(&name).or(global.get(&name)) {
                    Ok(variable.data_type)
                } else {
                    Err(RaoulError::new(
                        clone,
                        RaoulErrorKind::UndeclaredVar { name },
                    ))
                }
            }
            AstNodeKind::Read => Ok(Types::STRING),
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                let lhs_type = Types::from_node(*lhs, variables, global)?;
                let rhs_type = Types::from_node(*rhs, variables, global)?;
                match Types::binary_operator_type(operator, lhs_type, rhs_type) {
                    Err(kind) => Err(RaoulError::new(clone, kind)),
                    Ok(op_type) => Ok(op_type),
                }
            }
            AstNodeKind::UnaryOperation { operator, operand } => match operator {
                Operator::Not => {
                    let operand_type = Types::from_node(*operand, variables, global)?;
                    match operand_type.is_boolish() {
                        true => Ok(Types::BOOL),
                        false => Err(RaoulError::new(
                            clone,
                            RaoulErrorKind::InvalidCast {
                                from: operand_type,
                                to: Types::BOOL,
                            },
                        )),
                    }
                }
                _ => unreachable!("{:?}", operator),
            },
            kind => unreachable!("{:?}", kind),
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
    // ByteCode
    Assignment,
    Print,
    PrintNl,
    Read,
}
