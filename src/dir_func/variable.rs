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
    pub address: usize,
    pub data_type: Types,
    pub dimensions: Dimensions,
    pub name: String,
}

fn get_value_dimensions<'a>(value: &AstNode<'a>, node: &AstNode<'a>) -> Results<'a, Dimensions> {
    match value.get_dimensions() {
        Ok(dimensions) => Ok(dimensions),
        Err((expected, given)) => {
            let kind = RaoulErrorKind::InconsistentSize { expected, given };
            Err(RaoulError::new_vec(node, kind))
        }
    }
}

fn assert_dataframe<'a>(
    data_type: Types,
    global_fn: &mut GlobalScope,
    node: &AstNode<'a>,
) -> Results<'a, ()> {
    if data_type != Types::Dataframe {
        return Ok(());
    }
    if global_fn.add_dataframe() {
        Ok(())
    } else {
        Err(RaoulError::new_vec(node, RaoulErrorKind::OnlyOneDataframe))
    }
}

impl Variable {
    pub fn from_global<'a>(v: &AstNode<'a>, global_fn: &mut GlobalScope) -> Results<'a, Variable> {
        match &v.kind {
            AstNodeKind::Assignment {
                assignee, value, ..
            } => {
                let data_type =
                    Types::from_node(&*value, &global_fn.variables, &global_fn.variables)?;
                assert_dataframe(data_type, global_fn, v)?;
                let dimensions = get_value_dimensions(value, v)?;
                let name: String = assignee.into();
                match global_fn.get_variable_address(&name, data_type, dimensions) {
                    Some(address) => Ok(Variable {
                        address,
                        data_type,
                        dimensions,
                        name,
                    }),
                    None => Err(RaoulError::new_vec(v, RaoulErrorKind::MemoryExceded)),
                }
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    pub fn from_node<'a>(
        v: &AstNode<'a>,
        current_fn: &mut Function,
        global_fn: &mut GlobalScope,
    ) -> Results<'a, (Variable, bool)> {
        match v.kind.clone() {
            AstNodeKind::Assignment {
                assignee,
                value,
                global,
            } => {
                let data_type =
                    Types::from_node(&*value, &current_fn.variables, &global_fn.variables)?;
                assert_dataframe(data_type, global_fn, v)?;
                let dimensions = get_value_dimensions(&value, v)?;
                let name: String = assignee.into();
                let address = if global {
                    global_fn.get_variable_address(&name, data_type, dimensions)
                } else {
                    current_fn.get_variable_address(&name, data_type, dimensions)
                };
                match address {
                    Some(address) => Ok((
                        Variable {
                            address,
                            data_type,
                            dimensions,
                            name,
                        },
                        global,
                    )),
                    None => Err(RaoulError::new_vec(v, RaoulErrorKind::MemoryExceded)),
                }
            }
            AstNodeKind::Argument {
                arg_type: data_type,
                name,
            } => {
                let address = current_fn
                    .local_addresses
                    .get_address(data_type, (None, None));
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
                        Err(RaoulError::new_vec(v, kind))
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn from_function(function: &Function, address: usize) -> Self {
        Variable {
            address,
            data_type: function.return_type,
            name: function.name.clone(),
            dimensions: (None, None),
        }
    }
}
