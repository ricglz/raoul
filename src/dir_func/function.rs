use std::collections::HashMap;

use crate::{
    address::{AddressManager, GenericAddressManager, TempAddressManager, TOTAL_SIZE},
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{error_kind::RaoulErrorKind, RaoulError, Results},
    quadruple::quadruple_manager::Operand,
};

use super::variable::{Dimensions, Variable};

pub type VariablesTable = HashMap<String, Variable>;
type InsertResult = std::result::Result<(), RaoulErrorKind>;

pub trait Scope {
    fn get_variable(&self, name: &str) -> Option<&Variable>;
    fn _insert_variable(&mut self, name: String, variable: Variable);
    fn insert_variable(&mut self, variable: Variable) -> InsertResult {
        let name = variable.name.clone();
        if let Some(stored_var) = self.get_variable(&name) {
            if !variable.data_type.can_cast(stored_var.data_type) {
                return Err(RaoulErrorKind::RedefinedType {
                    name,
                    from: stored_var.data_type,
                    to: variable.data_type,
                });
            }
        } else {
            self._insert_variable(name, variable);
        }
        Ok(())
    }
    fn _get_variable_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize>;
    fn get_variable_address(
        &mut self,
        name: &str,
        data_type: Types,
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
    pub args: Vec<Operand>,
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
        node: &AstNode<'a>,
        global_fn: &mut GlobalScope,
        argument: bool,
    ) -> Results<'a, ()> {
        match Variable::from_node(node, self, global_fn) {
            Ok((variable, global)) => {
                let address = variable.address;
                let data_type = variable.data_type;
                let result = if global {
                    global_fn.insert_variable(variable)
                } else {
                    self.insert_variable(variable)
                };
                if let Err(kind) = result {
                    return Err(RaoulError::new_vec(node, kind));
                }
                if argument {
                    self.args.push((address, data_type));
                }
                Ok(())
            }
            Err(errors) => Err(errors),
        }
    }

    fn insert_from_nodes<'a>(
        &mut self,
        nodes: &[AstNode<'a>],
        global_fn: &mut GlobalScope,
        is_arg: bool,
    ) -> Results<'a, ()> {
        RaoulError::create_results(
            nodes
                .iter()
                .flat_map(AstNode::expand_node)
                .filter(AstNode::is_declaration)
                .map(|node| self.insert_variable_from_node(&node, global_fn, is_arg)),
        )
    }

    pub fn try_create<'a>(v: &AstNode<'a>, global_fn: &mut GlobalScope) -> Results<'a, Function> {
        match v.kind.clone() {
            AstNodeKind::Function {
                name,
                return_type,
                ref body,
                ref arguments,
            } => {
                let mut function = Function::new(name, return_type);
                function.insert_from_nodes(arguments, global_fn, true)?;
                function.insert_from_nodes(body, global_fn, false)?;
                Ok(function)
            }
            AstNodeKind::Main { ref body, .. } => {
                let mut function = Function::new("main".to_string(), Types::Void);
                function.insert_from_nodes(body, global_fn, false)?;
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
    fn _get_variable_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize> {
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
        if self.has_dataframe {
            false
        } else {
            self.has_dataframe = true;
            true
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
    fn _get_variable_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize> {
        self.addresses.get_address(data_type, dimensions)
    }
}
