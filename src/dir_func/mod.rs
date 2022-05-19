use std::collections::HashMap;

use crate::{
    address::GenericAddressManager,
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{error_kind::RaoulErrorKind, RaoulError, Result, Results},
};

use self::{
    function::{Function, GlobalScope, Scope},
    variable::Variable,
};

pub mod function;
pub mod variable;
pub mod variable_value;

pub type FunctionTable = HashMap<String, Function>;

#[derive(PartialEq, Debug, Clone)]
pub struct DirFunc {
    pub functions: FunctionTable,
    pub global_fn: GlobalScope,
}

impl DirFunc {
    pub fn new() -> Self {
        Self {
            global_fn: GlobalScope::new(),
            functions: HashMap::new(),
        }
    }

    pub fn clear_variables(&mut self) {
        self.global_fn.variables.clear();
        self.functions
            .values_mut()
            .for_each(|f| f.variables.clear());
    }

    fn insert_function<'a>(&mut self, function: Function, node: AstNode<'a>) -> Result<'a, ()> {
        let name = function.name.clone();
        match self.functions.get(&name) {
            Some(_) => Err(RaoulError::new(
                &node,
                RaoulErrorKind::RedeclaredFunction(name),
            )),
            None => {
                self.functions.insert(name, function);
                Ok(())
            }
        }
    }

    fn insert_function_from_node<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let node_clone = node.clone();
        let mut function = Function::try_create(node, &mut self.global_fn)?;
        if function.return_type != Types::Void {
            let address = self
                .global_fn
                .addresses
                .get_address(function.return_type, (None, None));
            match address {
                Some(address) => {
                    let result = self
                        .global_fn
                        .insert_variable(Variable::from_function(function.clone(), address));
                    if let Err(kind) = result {
                        return Err(vec![RaoulError::new(&node_clone, kind)]);
                    }
                    function.address = address;
                }
                None => {
                    let kind = RaoulErrorKind::MemoryExceded;
                    return Err(vec![RaoulError::new(&node_clone, kind)]);
                }
            }
        }
        match self.insert_function(function, node_clone) {
            Ok(_) => Ok(()),
            Err(error) => Err(vec![error]),
        }
    }

    pub fn build_dir_func<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let clone = node.clone();
        match node.kind {
            AstNodeKind::Main {
                functions,
                assignments,
                ..
            } => {
                RaoulError::create_results(assignments.into_iter().map(|node| -> Results<()> {
                    let variable = Variable::from_global(node.clone(), &mut self.global_fn)?;
                    match self.global_fn.insert_variable(variable) {
                        Ok(_) => Ok(()),
                        Err(kind) => Err(RaoulError::new_vec(node, kind)),
                    }
                }))?;
                RaoulError::create_results(
                    functions
                        .into_iter()
                        .chain(Some(clone))
                        .map(|node| self.insert_function_from_node(node)),
                )
            }
            _ => unreachable!(),
        }
    }
}
