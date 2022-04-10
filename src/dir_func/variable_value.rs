use crate::{
    ast::AstNode,
    enums::Types,
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

pub fn build_variable_value(v: AstNode, variables: &VariablesTable) -> Result<VariableValue> {
    match v {
        AstNode::Integer(value) => Ok(VariableValue::Integer(value)),
        AstNode::Float(value) => Ok(VariableValue::Float(value)),
        AstNode::String(value) => Ok(VariableValue::String(value.clone())),
        AstNode::Bool(value) => Ok(VariableValue::Bool(value)),
        AstNode::Id(name) => {
            if let Some(variable) = variables.get(&name) {
                Ok(variable.value.clone())
            } else {
                Err(RaoulError::UndeclaredId { name })
            }
        }
        AstNode::UnaryOperation { .. } => todo!(),
        _ => Err(RaoulError::Invalid),
    }
}
