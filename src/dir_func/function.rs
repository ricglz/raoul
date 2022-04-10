use std::collections::HashMap;

use crate::{ast::AstNode, enums::Types};

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

    fn insert_variable_from_node(&mut self, node: AstNode) {
        let variable = build_variable(node, &self.variables);
        self.insert_variable(variable);
    }
}

impl From<AstNode<'_>> for Function {
    fn from(v: AstNode) -> Self {
        match v {
            AstNode::Function {
                name,
                return_type,
                body,
                arguments,
            } => {
                let mut function = Function::new(name, return_type);
                for node in arguments {
                    function.insert_variable_from_node(node);
                }
                for tuple in body {
                    function.insert_variable_from_node(tuple.0);
                }
                function
            }
            AstNode::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                for tuple in body {
                    function.insert_variable_from_node(tuple.0);
                }
                function
            }
            _ => unreachable!(),
        }
    }
}
