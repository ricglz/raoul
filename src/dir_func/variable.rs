use crate::{
    address::GenericAddressManager,
    ast::ast_kind::AstNodeKind,
    ast::AstNode,
    enums::Types,
    error::error_kind::RaoulErrorKind,
    error::{RaoulError, Results},
};

use super::function::{Function, GlobalScope, Scope};

pub type Dimensions = (Option<usize>, Option<usize>);

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    dimensions: Dimensions,
    pub address: usize,
    pub data_type: Types,
    pub name: String,
}

impl Variable {
    pub fn from_node<'a>(
        v: AstNode<'a>,
        current_fn: &mut Function,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, (Variable, bool)> {
        let node = v.clone();
        match v.kind {
            AstNodeKind::Assignment {
                name,
                value,
                global,
            } => {
                let data_type =
                    Types::from_node(&*value, &current_fn.variables, &global_fn.variables)?;
                let dimensions = value.get_dimensions();
                let address = match global {
                    true => global_fn.get_variable_address(&name, &data_type, dimensions),
                    false => current_fn.get_variable_address(&name, &data_type, dimensions),
                };
                match address {
                    Some(address) => Ok((
                        Variable {
                            data_type,
                            dimensions,
                            name,
                            address,
                        },
                        global,
                    )),
                    None => Err(RaoulError::new_vec(node, RaoulErrorKind::MemoryExceded)),
                }
            }
            AstNodeKind::Argument {
                arg_type: data_type,
                name,
            } => {
                let address = current_fn
                    .local_addresses
                    .get_address(&data_type, (None, None));
                match address {
                    Some(address) => Ok((
                        Variable {
                            address,
                            data_type,
                            name,
                            dimensions: (None, None),
                        },
                        false,
                    )),
                    None => {
                        let kind = RaoulErrorKind::MemoryExceded;
                        Err(RaoulError::new_vec(node, kind))
                    }
                }
            }
            _ => Err(RaoulError::new_vec(v, RaoulErrorKind::Invalid)),
        }
    }

    pub fn from_function(function: Function, address: usize) -> Self {
        Variable {
            address,
            data_type: function.return_type,
            name: function.name,
            dimensions: (None, None),
        }
    }
}
