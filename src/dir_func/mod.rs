use std::collections::HashMap;

use crate::{
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::{RaoulError, Results},
};

use self::{
    function::{Function, GlobalScope, Scope},
    variable::Variable,
};

mod function;
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

    fn insert_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }

    fn insert_function_from_node<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let function = Function::try_create(node, &mut self.global_fn)?;
        if function.return_type != Types::VOID {
            self.global_fn
                .insert_variable(Variable::from(function.clone()))
        }
        Ok(self.insert_function(function))
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
