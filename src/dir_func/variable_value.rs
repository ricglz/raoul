use crate::{ast::AstNode, enums::Types};

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

pub fn build_variable_value(v: AstNode, variables: &VariablesTable) -> VariableValue {
    match v {
        AstNode::Integer(value) => VariableValue::Integer(value),
        AstNode::Float(value) => VariableValue::Float(value),
        AstNode::String(value) => VariableValue::String(value.clone()),
        AstNode::Bool(value) => VariableValue::Bool(value),
        AstNode::Id(name) => {
            let variable = variables.get(&name).unwrap();
            variable.value.clone()
        }
        AstNode::UnaryOperation { .. } => todo!(),
        _ => unreachable!(
            "Node {:?}, was attempted to be parsed to a VariableValue",
            v
        ),
    }
}
