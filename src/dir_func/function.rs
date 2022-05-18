use std::collections::HashMap;

use crate::{
    address::{AddressManager, GenericAddressManager, TempAddressManager, TOTAL_SIZE},
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{error_kind::RaoulErrorKind, RaoulError, Results},
};

use super::variable::{Dimensions, Variable};

pub type VariablesTable = HashMap<String, Variable>;
type InsertResult = std::result::Result<(), RaoulErrorKind>;

pub trait Scope {
    fn get_variable(&self, name: &str) -> Option<&Variable>;
    fn _insert_variable(&mut self, name: String, variable: Variable);
    fn insert_variable(&mut self, variable: Variable) -> InsertResult {
        let name = variable.name.clone();
        match self.get_variable(&name) {
            None => {
                self._insert_variable(variable.name.clone(), variable);
                Ok(())
            }
            Some(stored_var) => match variable.data_type.can_cast(stored_var.data_type) {
                true => Ok(()),
                false => Err(RaoulErrorKind::RedefinedType {
                    name,
                    from: stored_var.data_type,
                    to: variable.data_type,
                }),
            },
        }
    }
    fn _get_variable_address(&mut self, data_type: &Types, dimensions: Dimensions)
        -> Option<usize>;
    fn get_variable_address(
        &mut self,
        name: &str,
        data_type: &Types,
        dimensions: Dimensions,
    ) -> Option<usize> {
        match self.get_variable(name) {
            Some(variable) => Some(variable.address),
            None => self._get_variable_address(data_type, dimensions),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub address: usize,
    pub args: Vec<Variable>,
    pub first_quad: usize,
    pub local_addresses: AddressManager,
    pub name: String,
    pub return_type: Types,
    pub temp_addresses: TempAddressManager,
    pub variables: VariablesTable,
}

impl Function {
    fn new(name: String, return_type: Types) -> Self {
        Self {
            address: usize::MAX,
            args: Vec::new(),
            local_addresses: AddressManager::new(TOTAL_SIZE),
            name,
            return_type,
            temp_addresses: TempAddressManager::new(),
            variables: HashMap::new(),
            first_quad: 0,
        }
    }

    fn insert_variable_from_node<'a>(
        &mut self,
        node: AstNode<'a>,
        global_fn: &mut GlobalScope,
        argument: bool,
    ) -> Results<'a, ()> {
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
                        if argument {
                            self.args.push(variable_clone);
                        }
                        Ok(())
                    }
                    Err(kind) => Err(RaoulError::new_vec(clone, kind)),
                }
            }
            Err(errors) => Err(errors),
        }
    }

    fn insert<'a>(
        &mut self,
        nodes: Vec<AstNode<'a>>,
        global_fn: &mut GlobalScope,
        argument: bool,
    ) -> Vec<RaoulError<'a>> {
        nodes
            .into_iter()
            .flat_map(AstNode::expand_node)
            .filter_map(|node| {
                self.insert_variable_from_node(node.to_owned(), global_fn, argument)
                    .err()
            })
            .flatten()
            .filter(|e| !e.is_invalid())
            .collect()
    }

    fn insert_variable_from_nodes<'a>(
        &mut self,
        nodes: Vec<AstNode<'a>>,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, ()> {
        let errors = self.insert(nodes, global_fn, false);
        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn insert_args_from_nodes<'a>(
        &mut self,
        nodes: Vec<AstNode<'a>>,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, ()> {
        let errors = self.insert(nodes, global_fn, true);
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
                let mut function = Function::new("main".to_string(), Types::Void);
                function.insert_variable_from_nodes(body, global_fn)?;
                Ok(function)
            }
            _ => unreachable!(),
        }
    }

    pub fn size(&self) -> usize {
        self.local_addresses.size() + self.temp_addresses.size()
    }

    pub fn update_quad(&mut self, first_quad: usize) {
        self.first_quad = first_quad;
    }
}

impl Scope for Function {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    fn _insert_variable(&mut self, name: String, variable: Variable) {
        self.variables.insert(name, variable);
    }
    fn _get_variable_address(
        &mut self,
        data_type: &Types,
        dimensions: Dimensions,
    ) -> Option<usize> {
        self.local_addresses.get_address(data_type, dimensions)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GlobalScope {
    has_dataframe: bool,
    pub addresses: AddressManager,
    pub variables: VariablesTable,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self {
            addresses: AddressManager::new(0),
            variables: HashMap::new(),
            has_dataframe: false,
        }
    }

    pub fn add_dataframe(&mut self) -> bool {
        match self.has_dataframe {
            false => {
                self.has_dataframe = true;
                true
            }
            true => false,
        }
    }
}

impl Default for GlobalScope {
  fn default() -> Self {
      Self::new()
  }
}

impl Scope for GlobalScope {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    fn _insert_variable(&mut self, name: String, variable: Variable) {
        self.variables.insert(name, variable);
    }
    fn _get_variable_address(
        &mut self,
        data_type: &Types,
        dimensions: Dimensions,
    ) -> Option<usize> {
        self.addresses.get_address(data_type, dimensions)
    }
}
