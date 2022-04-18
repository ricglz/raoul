use std::collections::HashMap;

use crate::{
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Result, Results},
};

use super::variable::{build_variable, Variable};

pub type VariablesTable = HashMap<String, Variable>;

pub trait Scope {
    fn insert_variable(&mut self, variable: Variable);
}

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub return_type: Types,
    variables: VariablesTable,
}

impl Function {
    fn new(name: String, return_type: Types) -> Self {
        Self {
            name,
            return_type,
            variables: HashMap::new(),
        }
    }

    fn insert_variable_from_node<'a>(
        &mut self,
        node: AstNode<'a>,
        global_fn: &mut GlobalScope,
    ) -> Result<'a, ()> {
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

    fn insert_variable_from_nodes<'a>(
        &mut self,
        nodes: Vec<AstNode<'a>>,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, Self> {
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

    pub fn try_create<'a>(v: AstNode<'a>, global_fn: &mut GlobalScope) -> Results<'a, Function> {
        match v.kind {
            AstNodeKind::Function {
                name,
                return_type,
                body,
                arguments,
            } => {
                let mut function = Function::new(name, return_type);
                let args_iter = arguments.clone().into_iter();
                let body_iter = body.clone().into_iter();
                function.insert_variable_from_nodes(args_iter.chain(body_iter).collect(), global_fn)
            }
            AstNodeKind::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                let body_iter = body.clone().into_iter();
                function.insert_variable_from_nodes(body_iter.collect(), global_fn)
            }
            _ => unreachable!(),
        }
    }
}

impl Scope for Function {
    fn insert_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GlobalScope {
    variables: VariablesTable,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Scope for GlobalScope {
    fn insert_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }
}
