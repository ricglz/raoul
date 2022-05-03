use std::fmt;

use crate::{
    address::{Address, ConstantMemory, GenericAddressManager},
    ast::{ast_kind::AstNodeKind, AstNode},
    dir_func::{
        function::{Function, VariablesTable},
        variable::Variable,
        variable_value::VariableValue,
        DirFunc,
    },
    enums::{Operator, Types},
    error::{error_kind::RaoulErrorKind, RaoulError, Result, Results},
    quadruple::quadruple::Quadruple,
};

#[derive(PartialEq)]
pub struct QuadrupleManager {
    function_name: String,
    jump_list: Vec<usize>,
    pub dir_func: DirFunc,
    pub memory: ConstantMemory,
    pub missing_returns: usize,
    pub quad_list: Vec<Quadruple>,
}

impl QuadrupleManager {
    pub fn new(dir_func: DirFunc) -> QuadrupleManager {
        QuadrupleManager {
            dir_func,
            function_name: "".to_owned(),
            jump_list: Vec::new(),
            memory: ConstantMemory::new(),
            missing_returns: 0,
            quad_list: Vec::new(),
        }
    }

    #[inline]
    fn get_function(&self, name: &str) -> &Function {
        self.dir_func
            .functions
            .get(name)
            .expect(&self.function_name)
    }

    #[inline]
    fn function(&self) -> &Function {
        self.get_function(&self.function_name)
    }

    #[inline]
    fn function_variables(&self) -> &VariablesTable {
        &self.function().variables
    }

    #[inline]
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

    #[inline]
    fn function_mut(&mut self) -> &mut Function {
        self.dir_func
            .functions
            .get_mut(&self.function_name)
            .unwrap()
    }

    #[inline]
    fn add_temp(&mut self, data_type: &Types) -> Option<usize> {
        self.function_mut().temp_addresses.get_address(data_type)
    }

    fn safe_address<'a, T>(&self, option: Option<T>, node: AstNode<'a>) -> Results<'a, T> {
        match option {
            Some(value) => Ok(value),
            None => Err(vec![RaoulError::new(node, RaoulErrorKind::MemoryExceded)]),
        }
    }

    fn safe_add_temp<'a>(&mut self, data_type: &Types, node: AstNode<'a>) -> Results<'a, usize> {
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

    fn get_variable_name_address<'a>(
        &mut self,
        name: String,
        node: AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        match self
            .function_variables()
            .get(&name)
            .or(self.global_variables().get(&name))
        {
            Some(variable) => Ok((variable.address, variable.data_type)),
            None => Err(vec![RaoulError::new(
                node,
                RaoulErrorKind::UndeclaredVar { name },
            )]),
        }
    }

    fn parse_exprs<'a>(&mut self, exprs: Vec<AstNode<'a>>) -> Results<'a, Vec<(usize, Types)>> {
        let (addresses, errors): (Vec<_>, Vec<_>) = exprs
            .into_iter()
            .map(|node| self.parse_expr(node))
            .partition(|res| res.is_ok());
        match errors.is_empty() {
            true => Ok(addresses.into_iter().map(|res| res.unwrap()).collect()),
            false => Err(errors
                .into_iter()
                .flat_map(|res| res.unwrap_err())
                .collect()),
        }
    }

    fn assert_type<'a>(&self, from: Types, to: Types, node: AstNode<'a>) -> Result<'a, ()> {
        match from.can_cast(to) {
            true => Ok(()),
            false => {
                let error = RaoulError::new(node, RaoulErrorKind::InvalidCast { from, to });
                Err(error)
            }
        }
    }

    fn parse_args_exprs<'a>(
        &mut self,
        node: AstNode<'a>,
        exprs: Vec<AstNode<'a>>,
        args: Vec<Variable>,
    ) -> Results<'a, Vec<(usize, Types)>> {
        if args.len() != exprs.len() {
            let kind = RaoulErrorKind::UnmatchArgsAmount {
                expected: args.len(),
                given: exprs.len(),
            };
            return Err(vec![RaoulError::new(node, kind)]);
        }
        let (addresses, errors): (Vec<_>, Vec<_>) = exprs
            .into_iter()
            .enumerate()
            .map(|(i, node)| -> Results<(usize, Types)> {
                let (v, v_type) = self.parse_expr(node.clone())?;
                let arg_type = args.get(i).unwrap().data_type;
                self.assert_type_results(v_type, arg_type, node)?;
                Ok((v, v_type))
            })
            .partition(|res| res.is_ok());
        match errors.is_empty() {
            true => Ok(addresses.into_iter().map(|res| res.unwrap()).collect()),
            false => Err(errors
                .into_iter()
                .flat_map(|res| res.unwrap_err())
                .collect()),
        }
    }

    fn add_era_quad(&mut self, name: &str) {
        let function = self.get_function(name);
        let function_size = function.size();
        let first_quad = function.first_quad;
        self.add_quad(Quadruple {
            operator: Operator::Era,
            op_1: Some(function_size),
            op_2: Some(first_quad),
            res: None,
        });
    }

    fn add_go_sub_quad(&mut self, name: &str) {
        let first_quad = self.get_function(name).first_quad;
        self.add_quad(Quadruple {
            operator: Operator::GoSub,
            op_1: Some(first_quad),
            op_2: None,
            res: None,
        });
    }

    fn parse_func_call<'a>(
        &mut self,
        name: &str,
        node: AstNode<'a>,
        exprs: Vec<AstNode<'a>>,
    ) -> Results<'a, ()> {
        self.add_era_quad(name);
        let args = self.get_function(name).args.clone();
        let addresses = self.parse_args_exprs(node, exprs, args)?;
        addresses
            .into_iter()
            .enumerate()
            .for_each(|(i, (address, _))| {
                self.add_quad(Quadruple {
                    operator: Operator::Param,
                    op_1: Some(address),
                    op_2: None,
                    res: Some(i),
                })
            });
        Ok(self.add_go_sub_quad(name))
    }

    fn parse_expr<'a>(&mut self, node: AstNode<'a>) -> Results<'a, (usize, Types)> {
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
                            return Err(vec![RaoulError::new(node_clone, kind)]);
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
            AstNodeKind::Id(name) => self.get_variable_name_address(name, node_clone),
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
            AstNodeKind::FuncCall { name, exprs } => {
                self.parse_func_call(&name, node_clone.clone(), exprs)?;
                self.get_variable_name_address(name, node_clone)
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    fn assert_type_results<'a>(
        &self,
        from: Types,
        to: Types,
        node: AstNode<'a>,
    ) -> Results<'a, ()> {
        match self.assert_type(from, to, node) {
            Ok(_) => Ok(()),
            Err(error) => Err(vec![error]),
        }
    }

    fn assert_expr_type<'a>(
        &mut self,
        expr: AstNode<'a>,
        to: Types,
    ) -> Results<'a, (usize, Types)> {
        let expr_clone = expr.clone();
        let (res_address, res_type) = self.parse_expr(expr)?;
        self.assert_type_results(res_type, to, expr_clone)?;
        Ok((res_address, res_type))
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

    fn parse_return_body<'a>(&mut self, body: Vec<AstNode<'a>>) -> Results<'a, ()> {
        let prev_missing_returns = self.missing_returns;
        self.parse_body(body)?;
        Ok(if self.missing_returns != prev_missing_returns {
            self.missing_returns = prev_missing_returns;
        })
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
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Assignment {
                global,
                ref name,
                value,
            } => {
                let (value_addr, _) = self.parse_expr(*value)?;
                let variable_address = self.get_variable_address(global, name);
                Ok(self.add_quad(Quadruple {
                    operator: Operator::Assignment,
                    op_1: Some(value_addr),
                    op_2: None,
                    res: Some(variable_address),
                }))
            }
            AstNodeKind::Write { exprs } => {
                let addresses = self.parse_exprs(exprs)?;
                addresses.into_iter().for_each(|(address, _)| {
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
                self.parse_return_body(statements)?;
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
                self.parse_return_body(statements)?;
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
            AstNodeKind::For {
                assignment,
                expr,
                statements,
            } => {
                let name = String::from(*assignment.clone());
                self.parse_function(*assignment)?;
                self.jump_list.push(self.quad_list.len());
                let (res_address, _) = self.assert_expr_type(*expr, Types::BOOL)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                self.parse_return_body(statements)?;
                let (var_address, var_type) =
                    self.get_variable_name_address(name, node_clone.clone())?;
                self.assert_type_results(var_type, Types::INT, node_clone)?;
                self.add_quad(Quadruple {
                    operator: Operator::Inc,
                    op_1: None,
                    op_2: None,
                    res: Some(var_address),
                });
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
            AstNodeKind::Return(expr) => {
                let return_type = self.function().return_type;
                let (expr_address, _) = self.assert_expr_type(*expr, return_type)?;
                self.missing_returns -= 1;
                Ok(self.add_quad(Quadruple {
                    operator: Operator::Return,
                    op_1: Some(expr_address),
                    op_2: None,
                    res: None,
                }))
            }
            AstNodeKind::FuncCall { name, exprs } => {
                self.parse_func_call(&name, node_clone.clone(), exprs)
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    #[inline]
    pub fn update_quad(&mut self, first_quad: usize) {
        self.function_mut().update_quad(first_quad);
    }

    pub fn parse<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let clone = node.clone();
        match node.kind {
            AstNodeKind::Main { body, functions } => {
                self.add_goto(Operator::Goto, None);
                let errors: Vec<_> = functions
                    .into_iter()
                    .filter_map(|node| self.parse(node).err())
                    .flatten()
                    .collect();
                if !errors.is_empty() {
                    return Err(errors);
                }
                self.fill_goto();
                self.function_name = "main".to_owned();
                self.parse_body(body)?;
                Ok(self.add_quad(Quadruple {
                    operator: Operator::End,
                    op_1: None,
                    op_2: None,
                    res: None,
                }))
            }
            AstNodeKind::Function {
                name,
                body,
                return_type,
                ..
            } => {
                self.function_name = name;
                let first_quad = self.quad_list.len();
                self.update_quad(first_quad);
                if return_type != Types::VOID {
                    self.missing_returns = 1;
                }
                self.parse_body(body)?;
                if self.missing_returns > 0 {
                    let kind = RaoulErrorKind::MissingReturn(self.function_name.clone());
                    return Err(vec![RaoulError::new(clone, kind)]);
                }
                Ok(self.add_quad(Quadruple {
                    operator: Operator::EndProc,
                    op_1: None,
                    op_2: None,
                    res: None,
                }))
            }
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for QuadrupleManager {
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
