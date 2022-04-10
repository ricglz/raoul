use std::collections::HashMap;

use crate::{ast::AstNode, error::Result};

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

    fn insert_function_from_node(&mut self, node: &AstNode) -> Result<()> {
        let function = Function::try_from(node.to_owned())?;
        Ok(self.insert_function(function))
    }
}

pub fn build_dir_func(dir_func: &mut DirFunc, node: AstNode) -> Result<()> {
    match node {
        AstNode::Main { ref functions, .. } => {
            dir_func.insert_function_from_node(&node)?;
            for function in functions {
                dir_func.insert_function_from_node(&function)?;
            }
            Ok(())
        }
        _ => unreachable!(),
    }
}
