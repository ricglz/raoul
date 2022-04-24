use crate::{
    address::{Address, ConstantMemory, GenericAddressManager},
    ast::{ast_kind::AstNodeKind, AstNode},
    dir_func::{
        function::{Function, VariablesTable},
        variable_value::VariableValue,
        DirFunc,
    },
    enums::{Operator, Types},
    error::{error_kind::RaoulErrorKind, RaoulError, Result, Results},
};

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub struct Quadruple {
    operator: Operator,
    op_1: Option<usize>,
    op_2: Option<usize>,
    res: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub struct QuadrupleManager<'a> {
    dir_func: &'a mut DirFunc,
    function_name: String,
    pub memory: ConstantMemory,
    pub quad_list: Vec<Quadruple>,
}

impl QuadrupleManager<'_> {
    pub fn new<'a>(dir_func: &'a mut DirFunc) -> QuadrupleManager<'a> {
        QuadrupleManager {
            dir_func,
            function_name: "".to_owned(),
            memory: ConstantMemory::new(),
            quad_list: Vec::new(),
        }
    }

    fn function(&self) -> &Function {
        self.dir_func
            .functions
            .get(&self.function_name)
            .expect(&self.function_name)
    }

    fn function_variables(&self) -> &VariablesTable {
        &self.function().variables
    }

    fn global_variables(&self) -> &VariablesTable {
        &self.dir_func.global_fn.variables
    }

    fn get_variable_address(&self, global: bool, name: &str) -> usize {
        let variables = match global {
            true => self.global_variables(),
            false => self.function_variables(),
        };
        variables.get(name).expect(name).address
    }

    fn function_mut(&mut self) -> &mut Function {
        self.dir_func
            .functions
            .get_mut(&self.function_name)
            .unwrap()
    }

    fn add_temp(&mut self, data_type: &Types) -> Option<usize> {
        self.function_mut().temp_addresses.get_address(data_type)
    }

    fn safe_address<'a, T>(&self, option: Option<T>, node: AstNode<'a>) -> Result<'a, T> {
        match option {
            Some(value) => Ok(value),
            None => Err(RaoulError::new(node, RaoulErrorKind::MemoryExceded)),
        }
    }

    fn safe_add_temp<'a>(&mut self, data_type: &Types, node: AstNode<'a>) -> Result<'a, usize> {
        let option = self.add_temp(data_type);
        self.safe_address(option, node)
    }

    fn safe_remove_temp_address(&mut self, option: Option<usize>) {
        match option {
            None => (),
            Some(address) => match address.is_temp_address() {
                true => self.function_mut().temp_addresses.release_address(address),
                _ => (),
            },
        }
    }

    fn add_quad(&mut self, quad: Quadruple) {
        self.quad_list.push(quad);
        self.safe_remove_temp_address(quad.op_1);
        self.safe_remove_temp_address(quad.op_2);
    }

    fn parse_expr<'a>(&mut self, node: AstNode<'a>) -> Result<'a, (usize, Types)> {
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Bool(_)
            | AstNodeKind::Float(_)
            | AstNodeKind::Integer(_)
            | AstNodeKind::String(_) => {
                let value = VariableValue::from(node.kind);
                let result = self.memory.add(value);
                self.safe_address(result, node_clone)
            }
            AstNodeKind::UnaryOperation { operator, operand } => {
                let (op, op_type) = self.parse_expr(*operand)?;
                let res_type = match operator {
                    Operator::Not => match op_type {
                        Types::BOOL | Types::INT => Types::BOOL,
                        op_type => {
                            let kind = RaoulErrorKind::InvalidCast {
                                from: op_type,
                                to: Types::BOOL,
                            };
                            return Err(RaoulError::new(node_clone, kind));
                        }
                    },
                    _ => unreachable!(),
                };
                let res = self.safe_add_temp(&res_type, node_clone)?;
                let quad = Quadruple {
                    operator,
                    op_1: Some(op),
                    op_2: None,
                    res: Some(res),
                };
                self.add_quad(quad);
                Ok((res, res_type))
            }
            AstNodeKind::Id(name) => {
                match self
                    .function_variables()
                    .get(&name)
                    .or(self.global_variables().get(&name))
                {
                    Some(variable) => Ok((variable.address, variable.data_type)),
                    None => unreachable!(),
                }
            }
            AstNodeKind::Read => {
                let data_type = Types::STRING;
                let res = self.safe_add_temp(&data_type, node_clone)?;
                self.add_quad(Quadruple {
                    operator: Operator::Read,
                    op_1: None,
                    op_2: None,
                    res: Some(res),
                });
                Ok((res, data_type))
            }
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                let (op_1, lhs_type) = self.parse_expr(*lhs)?;
                let (op_2, rhs_type) = self.parse_expr(*rhs)?;
                let data_type = Types::binary_operator_type(operator, lhs_type, rhs_type).unwrap();
                let res = self.safe_add_temp(&data_type, node_clone)?;
                self.add_quad(Quadruple {
                    operator,
                    op_1: Some(op_1),
                    op_2: Some(op_2),
                    res: Some(res),
                });
                Ok((res, data_type))
            }
            kind => unreachable!("{:?}", kind),
        }
    }

    fn parse_function<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        match node.kind {
            AstNodeKind::Assignment {
                global,
                ref name,
                value,
            } => {
                let result = self.parse_expr(*value);
                if let Err(error) = result {
                    return Err(vec![error]);
                }
                let (value_addr, _) = result.unwrap();
                let variable_address = self.get_variable_address(global, name);
                self.add_quad(Quadruple {
                    operator: Operator::Assignment,
                    op_1: Some(value_addr),
                    op_2: None,
                    res: Some(variable_address),
                });
                Ok(())
            }
            AstNodeKind::Write { exprs } => {
                let (addresses, errors): (Vec<_>, Vec<_>) = exprs
                    .into_iter()
                    .map(|node| self.parse_expr(node))
                    .partition(Result::is_ok);
                let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
                if !errors.is_empty() {
                    return Err(errors);
                }
                addresses
                    .into_iter()
                    .map(Result::unwrap)
                    .for_each(|(address, _)| {
                        self.add_quad(Quadruple {
                            operator: Operator::Print,
                            op_1: Some(address),
                            op_2: None,
                            res: None,
                        })
                    });
                self.add_quad(Quadruple {
                    operator: Operator::PrintNl,
                    op_1: None,
                    op_2: None,
                    res: None,
                });
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn parse<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        match node.kind {
            AstNodeKind::Main { body, .. } => {
                self.function_name = "main".to_owned();
                let errors: Vec<RaoulError> = body
                    .into_iter()
                    .filter_map(|node| self.parse_function(node).err())
                    .flatten()
                    .collect();
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
            AstNodeKind::Function { .. } => todo!(),
            _ => unreachable!(),
        }
    }
}
