use std::collections::HashMap;

use crate::{
    ast::AstNode,
    error::{RaoulError, Results},
};

use self::function::Function;

mod function;
mod variable;
mod variable_value;

#[derive(PartialEq, Debug)]
pub struct DirFunc {
    functions: HashMap<String, Function>,
}

impl DirFunc {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    fn insert_function(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }

    fn insert_function_from_node(&mut self, node: &AstNode) -> Results<()> {
        let function = Function::try_from(node.to_owned())?;
        Ok(self.insert_function(function))
    }
}

pub fn build_dir_func(dir_func: &mut DirFunc, node: AstNode) -> Results<()> {
    match node {
        AstNode::Main { ref functions, .. } => {
            let errors: Vec<RaoulError> = functions
                .iter()
                .chain(Some(&node))
                .filter_map(|node| dir_func.insert_function_from_node(node).err())
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
