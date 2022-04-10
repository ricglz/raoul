use std::collections::HashMap;

use crate::{
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Result},
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
            Err(error) => match error {
                RaoulError::UndeclaredVar { .. } => Err(error),
                _ => Ok(()),
            },
        }
    }
}

impl TryFrom<AstNode<'_>> for Function {
    type Error = RaoulError;
    fn try_from(v: AstNode) -> Result<Self> {
        match v {
            AstNode::Function {
                name,
                return_type,
                body,
                arguments,
            } => {
                let mut function = Function::new(name, return_type);
                for node in arguments {
                    function.insert_variable_from_node(node)?;
                }
                for tuple in body {
                    function.insert_variable_from_node(tuple.0)?;
                }
                Ok(function)
            }
            AstNode::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                for tuple in body {
                    function.insert_variable_from_node(tuple.0)?;
                }
                Ok(function)
            }
            _ => unreachable!(),
        }
    }
}
