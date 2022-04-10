use crate::{
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Result},
};

use super::{
    function::VariablesTable,
    variable_value::{build_variable_value, VariableValue},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    data_type: Types,
    pub name: String,
    pub value: VariableValue,
}

pub fn build_variable(v: AstNode, variables: &VariablesTable) -> Result<Variable> {
    match v {
        AstNode::Assignment {
            name,
            value: node_value,
            ..
        } => {
            let value = build_variable_value(*node_value, variables)?;
            Ok(Variable {
                data_type: Types::from(value.clone()),
                name,
                value,
            })
        }
        AstNode::Argument {
            arg_type: data_type,
            name,
        } => Ok(Variable {
            data_type,
            name,
            value: VariableValue::Bool(false),
        }),
        _ => Err(RaoulError::Invalid),
    }
}
