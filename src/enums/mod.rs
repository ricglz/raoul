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
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Operator {
    NOT,
    ASSIGNMENT,
    PRINT,
    PRINTNL,
}
