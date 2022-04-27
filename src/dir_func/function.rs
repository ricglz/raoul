use std::collections::HashMap;

use crate::{
    address::{AddressManager, GenericAddressManager, TempAddressManager, TOTAL_SIZE},
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{error_kind::RaoulErrorKind, RaoulError, Result, Results},
};

use super::variable::Variable;

pub type VariablesTable = HashMap<String, Variable>;
type InsertResult = std::result::Result<(), RaoulErrorKind>;

pub trait Scope {
    fn get_variable(&self, name: &str) -> Option<&Variable>;
    fn _insert_variable(&mut self, name: String, variable: Variable);
    fn insert_variable(&mut self, variable: Variable) -> InsertResult {
        let name = variable.name.clone();
        match self.get_variable(&name) {
            None => Ok(self._insert_variable(variable.name.clone(), variable)),
            Some(stored_var) => match stored_var.data_type == variable.data_type {
                true => Ok(()),
                false => Err(RaoulErrorKind::RedefinedType {
                    name,
                    from: stored_var.data_type,
                    to: variable.data_type,
                }),
            },
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub args: Vec<Variable>,
    pub local_addresses: AddressManager,
    pub name: String,
    pub return_type: Types,
    pub temp_addresses: TempAddressManager,
    pub variables: VariablesTable,
}

impl Function {
    fn new(name: String, return_type: Types) -> Self {
        Self {
            args: Vec::new(),
            local_addresses: AddressManager::new(TOTAL_SIZE),
            name,
            return_type,
            temp_addresses: TempAddressManager::new(),
            variables: HashMap::new(),
        }
    }

    fn insert_variable_from_node<'a>(
        &mut self,
        node: AstNode<'a>,
        global_fn: &mut GlobalScope,
        argument: bool,
    ) -> Result<'a, ()> {
        let clone = node.clone();
        match Variable::from_node(node, self, global_fn) {
            Ok((variable, global)) => {
                let variable_clone = variable.clone();
                let result = match global {
                    true => global_fn.insert_variable(variable),
                    false => self.insert_variable(variable),
                };
                match result {
                    Ok(_) => {
                        self.args.push(variable_clone);
                        Ok(())
                    }
                    Err(kind) => Err(RaoulError::new(clone, kind)),
                }
            }
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
    ) -> Results<'a, ()> {
        let errors: Vec<RaoulError> = nodes
            .into_iter()
            .flat_map(AstNode::expand_node)
            .filter_map(|node| {
                self.insert_variable_from_node(node.to_owned(), global_fn, false)
                    .err()
            })
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn insert_args_from_nodes<'a>(
        &mut self,
        nodes: Vec<AstNode<'a>>,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, ()> {
        let errors: Vec<RaoulError> = nodes
            .into_iter()
            .flat_map(AstNode::expand_node)
            .filter_map(|node| {
                self.insert_variable_from_node(node.to_owned(), global_fn, true)
                    .err()
            })
            .collect();
        if errors.is_empty() {
            Ok(())
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
                function.insert_args_from_nodes(arguments, global_fn)?;
                function.insert_variable_from_nodes(body, global_fn)?;
                Ok(function)
            }
            AstNodeKind::Main { body, .. } => {
                let mut function = Function::new("main".to_string(), Types::VOID);
                function.insert_variable_from_nodes(body, global_fn)?;
                Ok(function)
            }
            _ => unreachable!(),
        }
    }

    pub fn size(&self) -> usize {
        self.local_addresses.size() + self.temp_addresses.size()
    }
}

impl Scope for Function {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    fn _insert_variable(&mut self, name: String, variable: Variable) {
        self.variables.insert(name, variable);
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GlobalScope {
    pub addresses: AddressManager,
    pub variables: VariablesTable,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self {
            addresses: AddressManager::new(0),
            variables: HashMap::new(),
        }
    }
}

impl Scope for GlobalScope {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    fn _insert_variable(&mut self, name: String, variable: Variable) {
        self.variables.insert(name, variable);
    }
}
