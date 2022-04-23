use crate::{
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::error_kind::RaoulErrorKind,
    error::{RaoulError, Result},
};

use super::function::VariablesTable;

#[derive(Clone, PartialEq, Debug)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl From<VariableValue> for Types {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::INT,
            VariableValue::Float(_) => Types::FLOAT,
            VariableValue::String(_) => Types::STRING,
            VariableValue::Bool(_) => Types::BOOL,
        }
    }
}

pub fn build_variable_value<'a>(
    v: AstNode<'a>,
    variables: &VariablesTable,
) -> Result<'a, VariableValue> {
    let clone = v.clone();
    match v.kind {
        AstNodeKind::Integer(value) => Ok(VariableValue::Integer(value)),
        AstNodeKind::Float(value) => Ok(VariableValue::Float(value)),
        AstNodeKind::String(value) => Ok(VariableValue::String(value.clone())),
        AstNodeKind::Bool(value) => Ok(VariableValue::Bool(value)),
        AstNodeKind::Id(name) => {
            if let Some(variable) = variables.get(&name) {
                if let Some(value) = &variable.value {
                    Ok(value.to_owned())
                } else {
                    Err(RaoulError::new(
                        clone,
                        RaoulErrorKind::UnitializedVar { name },
                    ))
                }
            } else {
                Err(RaoulError::new(
                    clone,
                    RaoulErrorKind::UndeclaredVar { name },
                ))
            }
        }
        AstNodeKind::UnaryOperation { .. } => todo!(),
        _ => Err(RaoulError::new(clone, RaoulErrorKind::Invalid)),
    }
}
