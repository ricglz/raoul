use std::fmt;

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
    quadruple::quadruple::Quadruple,
};

#[derive(PartialEq)]
pub struct QuadrupleManager<'a> {
    dir_func: &'a mut DirFunc,
    function_name: String,
    jump_list: Vec<usize>,
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
            jump_list: Vec::new(),
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
        match option.is_temp_address() {
            false => (),
            true => self
                .function_mut()
                .temp_addresses
                .release_address(option.unwrap()),
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
                    None => {
                        let kind = RaoulErrorKind::UndeclaredVar { name };
                        Err(RaoulError::new(node_clone, kind))
                    }
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

    fn parse_expr_results<'a>(&mut self, node: AstNode<'a>) -> Results<'a, (usize, Types)> {
        match self.parse_expr(node) {
            Ok(val) => Ok(val),
            Err(error) => Err(vec![error]),
        }
    }

    fn assert_expr_type<'a>(
        &mut self,
        expr: AstNode<'a>,
        to: Types,
    ) -> Results<'a, (usize, Types)> {
        let expr_clone = expr.clone();
        let (res_address, res_type) = self.parse_expr_results(expr)?;
        match res_type.can_cast(to) {
            true => Ok((res_address, res_type)),
            false => {
                let error = RaoulError::new(
                    expr_clone,
                    RaoulErrorKind::InvalidCast { from: res_type, to },
                );
                Err(vec![error])
            }
        }
    }

    fn parse_body<'a>(&mut self, body: Vec<AstNode<'a>>) -> Results<'a, ()> {
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

    fn add_goto(&mut self, goto_type: Operator, condition: Option<usize>) {
        debug_assert!(goto_type.is_goto());
        self.jump_list.push(self.quad_list.len());
        self.add_quad(Quadruple {
            operator: goto_type,
            op_1: condition,
            op_2: None,
            res: None,
        })
    }

    fn fill_goto_index(&mut self, index: usize) {
        let res = self.quad_list.len();
        let mut quad = self.quad_list.get_mut(index).unwrap();
        debug_assert!(quad.operator.is_goto());
        quad.res = Some(res);
    }

    fn fill_goto(&mut self) {
        let index = self.jump_list.pop().unwrap();
        self.fill_goto_index(index);
    }

    fn parse_function<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        match node.kind {
            AstNodeKind::Assignment {
                global,
                ref name,
                value,
            } => {
                let (value_addr, _) = self.parse_expr_results(*value)?;
                let variable_address = self.get_variable_address(global, name);
                Ok(self.add_quad(Quadruple {
                    operator: Operator::Assignment,
                    op_1: Some(value_addr),
                    op_2: None,
                    res: Some(variable_address),
                }))
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
                Ok(self.add_quad(Quadruple {
                    operator: Operator::PrintNl,
                    op_1: None,
                    op_2: None,
                    res: None,
                }))
            }
            AstNodeKind::Decision {
                expr,
                statements,
                else_block,
            } => {
                let (res_address, _) = self.assert_expr_type(*expr, Types::BOOL)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                self.parse_body(statements)?;
                Ok(if let Some(node) = else_block {
                    let index = self.jump_list.pop().unwrap();
                    self.add_goto(Operator::Goto, None);
                    self.fill_goto_index(index);
                    self.parse_function(*node)?;
                    self.fill_goto();
                } else {
                    self.fill_goto();
                })
            }
            AstNodeKind::ElseBlock { statements } => Ok(self.parse_body(statements)?),
            AstNodeKind::While { expr, statements } => {
                self.jump_list.push(self.quad_list.len());
                let (res_address, _) = self.assert_expr_type(*expr, Types::BOOL)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                self.parse_body(statements)?;
                let index = self.jump_list.pop().unwrap();
                let goto_res = self.jump_list.pop().unwrap();
                self.add_quad(Quadruple {
                    operator: Operator::Goto,
                    op_1: None,
                    op_2: None,
                    res: Some(goto_res),
                });
                Ok(self.fill_goto_index(index))
            }
            _ => unreachable!("{:?}", node.kind),
        }
    }

    pub fn parse<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        match node.kind {
            AstNodeKind::Main { body, .. } => {
                self.function_name = "main".to_owned();
                Ok(self.parse_body(body)?)
            }
            AstNodeKind::Function { .. } => todo!(),
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for QuadrupleManager<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: String = self
            .quad_list
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, quad)| format!("{} - {:?}\n", i, quad))
            .collect();
        write!(f, "{value}")
    }
}
