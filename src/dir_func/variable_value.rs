use std::fmt;

use crate::{ast::ast_kind::AstNodeKind, enums::Types};

#[derive(Clone, PartialEq)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl From<&VariableValue> for Types {
    fn from(v: &VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::INT,
            VariableValue::Float(_) => Types::FLOAT,
            VariableValue::String(_) => Types::STRING,
            VariableValue::Bool(_) => Types::BOOL,
        }
    }
}

impl From<AstNodeKind<'_>> for VariableValue {
    fn from(v: AstNodeKind) -> Self {
        match v {
            AstNodeKind::Integer(value) => VariableValue::Integer(value),
            AstNodeKind::Float(value) => VariableValue::Float(value),
            AstNodeKind::String(value) => VariableValue::String(value.clone()),
            AstNodeKind::Bool(value) => VariableValue::Bool(value),
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            VariableValue::Bool(value) => value.to_string(),
            VariableValue::Integer(value) => value.to_string(),
            VariableValue::Float(value) => value.to_string(),
            VariableValue::String(value) => value.to_owned(),
        };
        write!(f, "{}", value)
    }
}
