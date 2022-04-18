use std::collections::HashMap;

use crate::{
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
mod variable;
mod variable_value;

#[derive(PartialEq, Debug)]
pub struct DirFunc {
    functions: HashMap<String, Function>,
    global_fn: GlobalScope,
}

impl DirFunc {
    pub fn new() -> Self {
        Self {
            global_fn: GlobalScope::new(),
            functions: HashMap::new(),
        }
    }

    fn insert_function<'a>(&mut self, function: Function, node: AstNode<'a>) -> Result<'a, ()> {
        let name = function.name.clone();
        match self.functions.get(&name) {
            Some(_) => Err(RaoulError::new(
                node,
                RaoulErrorKind::RedeclaredFunction { name },
            )),
            None => {
                self.functions.insert(name, function);
                Ok(())
            }
        }
    }

    fn insert_function_from_node<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let node_clone = node.clone();
        let function = Function::try_create(node, &mut self.global_fn)?;
        if function.return_type != Types::VOID {
            let address = self.global_fn.addresses.get_address(function.return_type);
            match address {
                Some(address) => self
                    .global_fn
                    .insert_variable(Variable::from_function(function.clone(), address)),
                None => {
                    let kind = RaoulErrorKind::MemoryExceded;
                    return Err(vec![RaoulError::new(node_clone, kind)]);
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
            AstNodeKind::Main { functions, .. } => {
                let errors: Vec<RaoulError> = functions
                    .into_iter()
                    .chain(Some(clone))
                    .filter_map(|node| self.insert_function_from_node(node).err())
                    .flatten()
                    .collect();
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
            _ => unreachable!(),
        }
    }
}
