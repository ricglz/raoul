use std::collections::HashMap;

use crate::{
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Result, Results},
};

use super::variable::{build_variable, Variable};

pub type VariablesTable = HashMap<String, Variable>;

#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: String,
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

    fn insert_variable_from_node(&mut self, node: AstNode) -> Result<()> {
        match build_variable(node, &self.variables) {
            Ok(variable) => Ok(self.insert_variable(variable)),
            Err(error) => match error.is_invalid() {
                true => Ok(()),
                false => Err(error),
            },
        }
    }
}

impl TryFrom<AstNode<'_>> for Function {
    type Error = Vec<RaoulError>;
    fn try_from(v: AstNode) -> Results<Function> {
        match v {
            AstNode::Function {
                name,
                return_type,
                body,
                arguments,
            } => {
                let mut function = Function::new(name, return_type);
                let args_iter = arguments.clone().into_iter();
                let body_iter = body.clone().into_iter().map(|tuple| tuple.0);
                let errors: Vec<RaoulError> = args_iter
                    .chain(body_iter)
                    .filter_map(|node| function.insert_variable_from_node(node).err())
                    .collect();
                if errors.is_empty() {
                    Ok(function)
                } else {
                    Err(errors)
                }
            }
            AstNode::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                let body_iter = body.clone().into_iter().map(|tuple| tuple.0);
                let errors: Vec<RaoulError> = body_iter
                    .filter_map(|node| function.insert_variable_from_node(node).err())
                    .collect();
                if errors.is_empty() {
                    Ok(function)
                } else {
                    Err(errors)
                }
            }
            _ => unreachable!(),
        }
    }
}
