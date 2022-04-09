use std::collections::HashMap;

use crate::{ast::AstNode, enums::Types};

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Copy, PartialEq, Debug)]
enum VariableValue {
    Integer(i64),
}

impl From<VariableValue> for Types {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::INTEGER,
        }
    }
}

fn build_variable_value(v: AstNode<'_>) -> Result<VariableValue> {
    match v {
        AstNode::Integer(value) => Ok(VariableValue::Integer(value)),
        AstNode::Id(_) => todo!(),
        _ => unreachable!(),
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Variable {
    data_type: Types,
    name: String,
    value: VariableValue,
}

fn build_variable<'a>(v: AstNode<'a>) -> Result<Variable> {
    match v {
        AstNode::Assignment {
            global: _,
            name,
            value: node,
        } => {
            let res_value = build_variable_value(*node);
            if let Err(error) = res_value {
                return Err(error);
            }
            let value = res_value.unwrap();
            let data_type = Types::from(value);
            Ok(Variable {
                data_type,
                name,
                value,
            })
        }
        _ => unreachable!(),
    }
}

type VariablesTable<'a> = &'a mut HashMap<String, Variable>;

struct Function<'a> {
    name: String,
    return_type: Types,
    variables: VariablesTable<'a>,
}

fn build_function<'a>(v: AstNode<'a>, variables: VariablesTable<'a>) -> Result<Function<'a>> {
    match v {
        AstNode::Function { name, body } => {
            for node in body {
                if let Ok(variable) = build_variable(node.0) {
                    variables.insert(variable.name.to_string(), variable);
                }
            }
            Ok(Function {
                name,
                return_type: Types::VOID,
                variables,
            })
        }
        AstNode::Main { functions: _, body } => {
            for node in body {
                if let Ok(variable) = build_variable(node.0) {
                    variables.insert(variable.name.to_string(), variable);
                }
            }
            Ok(Function {
                name: "main".to_string(),
                return_type: Types::VOID,
                variables,
            })
        }
        _ => unreachable!(),
    }
}
