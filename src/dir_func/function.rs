use std::collections::HashMap;

use crate::{
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Result, Results},
};

use super::variable::{build_variable, Variable};

pub type VariablesTable = HashMap<String, Variable>;

#[derive(PartialEq, Clone, Debug)]
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

    fn insert_variable_from_node(&mut self, node: AstNode, global_fn: &mut Function) -> Result<()> {
        match build_variable(node, &self.variables) {
            Ok((variable, global)) => Ok(match global {
                true => global_fn.insert_variable(variable),
                false => self.insert_variable(variable),
            }),
            Err(error) => match error.is_invalid() {
                true => Ok(()),
                false => Err(error),
            },
        }
    }

    fn insert_variable_from_nodes(
        &mut self,
        nodes: Vec<AstNode>,
        global_fn: &mut Function,
    ) -> Results<Self> {
        let errors: Vec<RaoulError> = nodes
            .into_iter()
            .filter_map(|node| self.insert_variable_from_node(node, global_fn).err())
            .collect();
        if errors.is_empty() {
            Ok(self.to_owned())
        } else {
            Err(errors)
        }
    }
}

impl TryFrom<(AstNode<'_>, &mut Function)> for Function {
    type Error = Vec<RaoulError>;
    fn try_from(tuple: (AstNode, &mut Function)) -> Results<Function> {
        let (v, global_fn) = tuple;
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
                function.insert_variable_from_nodes(args_iter.chain(body_iter).collect(), global_fn)
            }
            AstNode::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                let body_iter = body.clone().into_iter().map(|tuple| tuple.0);
                function.insert_variable_from_nodes(body_iter.collect(), global_fn)
            }
            _ => unreachable!(),
        }
    }
}
