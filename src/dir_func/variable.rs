use crate::{
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::error_kind::RaoulErrorKind,
    error::{RaoulError, Result},
};

use super::function::{Function, GlobalScope};

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub data_type: Types,
    pub name: String,
    pub address: usize,
}

impl Variable {
    pub fn from_node<'a>(
        v: AstNode<'a>,
        current_function: &mut Function,
        global_fn: &mut GlobalScope,
    ) -> Result<'a, (Variable, bool)> {
        let node = v.clone();
        match v.kind {
            AstNodeKind::Assignment {
                name,
                value: node_value,
                global,
            } => {
                let data_type = Types::from_node(
                    *node_value,
                    &current_function.variables,
                    &global_fn.variables,
                )?;
                let address = match global {
                    true => global_fn.addresses.get_address(&data_type),
                    false => current_function.local_addresses.get_address(&data_type),
                };
                match address {
                    Some(address) => Ok((
                        Variable {
                            data_type,
                            name,
                            address,
                        },
                        global,
                    )),
                    None => {
                        let kind = RaoulErrorKind::MemoryExceded;
                        Err(RaoulError::new(node, kind))
                    }
                }
            }
            AstNodeKind::Argument {
                arg_type: data_type,
                name,
            } => {
                let address = current_function.local_addresses.get_address(&data_type);
                match address {
                    Some(address) => Ok((
                        Variable {
                            data_type,
                            name,
                            address,
                        },
                        false,
                    )),
                    None => {
                        let kind = RaoulErrorKind::MemoryExceded;
                        Err(RaoulError::new(node, kind))
                    }
                }
            }
            _ => Err(RaoulError::new(v, RaoulErrorKind::Invalid)),
        }
    }

    pub fn from_function(function: Function, address: usize) -> Self {
        Variable {
            address,
            data_type: function.return_type,
            name: function.name,
        }
    }
}
