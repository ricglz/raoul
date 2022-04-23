use crate::{
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::error_kind::RaoulErrorKind,
    error::{RaoulError, Result},
};

use super::{
    function::{Function, VariablesTable},
    variable_value::{build_variable_value, VariableValue},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    data_type: Types,
    pub name: String,
    pub value: Option<VariableValue>,
}

impl From<Function> for Variable {
    fn from(function: Function) -> Self {
        Variable {
            data_type: function.return_type,
            name: function.name,
            value: None,
        }
    }
}

pub fn build_variable<'a>(
    v: AstNode<'a>,
    variables: &VariablesTable,
) -> Result<'a, (Variable, bool)> {
    match v.kind {
        AstNodeKind::Assignment {
            name,
            value: node_value,
            global,
        } => {
            let value = build_variable_value(*node_value, variables)?;
            Ok((
                Variable {
                    data_type: Types::from(value.clone()),
                    name,
                    value: Some(value),
                },
                global,
            ))
        }
        AstNodeKind::Argument {
            arg_type: data_type,
            name,
        } => Ok((
            Variable {
                data_type,
                name,
                value: None,
            },
            false,
        )),
        _ => Err(RaoulError::new(v, RaoulErrorKind::Invalid)),
    }
}
