use crate::{ast::AstNode, enums::Types};

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

pub fn build_variable(v: AstNode, variables: &VariablesTable) -> Variable {
    match v {
        AstNode::Assignment {
            name,
            value: node_value,
            ..
        } => {
            let value = build_variable_value(*node_value, variables);
            let data_type = Types::from(value.clone());
            Variable {
                data_type,
                name,
                value,
            }
        }
        AstNode::Argument {
            arg_type: data_type,
            name,
        } => Variable {
            data_type,
            name,
            value: VariableValue::Bool(false),
        },
        _ => unreachable!(
            "Node {:?}, was attempted to be parsed to a VariableValue",
            v
        ),
    }
}
