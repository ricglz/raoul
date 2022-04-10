use std::collections::HashMap;

use crate::{ast::AstNode, enums::Types};

#[derive(Clone, PartialEq, Debug)]
enum VariableValue {
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

type VariablesTable = HashMap<String, Variable>;

fn build_variable_value(v: AstNode, variables: &VariablesTable) -> VariableValue {
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

#[derive(Clone, PartialEq, Debug)]
struct Variable {
    data_type: Types,
    name: String,
    value: VariableValue,
}

fn build_variable(v: AstNode, variables: &VariablesTable) -> Variable {
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

#[derive(PartialEq, Debug)]
struct Function {
    name: String,
    return_type: Types,
    variables: VariablesTable,
}

impl Function {
    pub fn new(name: String, return_type: Types) -> Self {
        Self {
            name,
            return_type,
            variables: HashMap::new(),
        }
    }

    fn insert_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }
}
