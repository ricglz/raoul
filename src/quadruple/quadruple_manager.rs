use std::fmt;

use crate::{
    address::{Address, ConstantMemory, GenericAddressManager, PointerMemory},
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

#[derive(PartialEq, Debug)]
pub struct QuadrupleManager {
    function_name: String,
    jump_list: Vec<usize>,
    missing_return: bool,
    pub dir_func: DirFunc,
    pub memory: ConstantMemory,
    pub pointer_memory: PointerMemory,
    pub quad_list: Vec<Quadruple>,
}

type Operand = (usize, Types);

impl QuadrupleManager {
    pub fn new(dir_func: DirFunc) -> QuadrupleManager {
        QuadrupleManager {
            dir_func,
            function_name: "".to_owned(),
            jump_list: Vec::new(),
            memory: ConstantMemory::new(),
            missing_return: false,
            pointer_memory: PointerMemory::new(),
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
        self.function_mut()
            .temp_addresses
            .get_address(data_type, (None, None))
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

    fn get_variable<'a>(&mut self, name: &str, node: AstNode<'a>) -> Results<'a, &Variable> {
        match self
            .function_variables()
            .get(name)
            .or_else(|| self.global_variables().get(name))
        {
            Some(var) => Ok(var),
            None => Err(RaoulError::new_vec(
                node,
                RaoulErrorKind::UndeclaredVar {
                    name: name.to_owned(),
                },
            )),
        }
    }

    fn get_variable_name_address<'a>(
        &mut self,
        name: &str,
        node: AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        let variable = self.get_variable(name, node)?;
        Ok((variable.address, variable.data_type))
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

    fn safe_add_cte<'a>(
        &mut self,
        value: VariableValue,
        node: AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        let result = self.memory.add(value);
        self.safe_address(result, node)
    }

    fn add_binary_op_quad<'a>(
        &mut self,
        operator: Operator,
        op_1: Operand,
        op_2: Operand,
        node: AstNode<'a>,
    ) -> Results<'a, Operand> {
        let data_type = Types::binary_operator_type(operator, op_1.1, op_2.1).unwrap();
        let res = self.safe_add_temp(&data_type, node)?;
        self.add_quad(Quadruple {
            operator,
            op_1: Some(op_1.0),
            op_2: Some(op_2.0),
            res: Some(res),
        });
        Ok((res, data_type))
    }

    fn get_array_val_operand<'a>(
        &mut self,
        name: &str,
        node: AstNode<'a>,
        idx_1: Box<AstNode<'a>>,
        idx_2: Option<Box<AstNode<'a>>>,
    ) -> Results<'a, Operand> {
        let v = (self.get_variable(name, node.clone())?).clone();
        let (dim_1, dim_2) = v.dimensions;
        if dim_1.is_none() {
            return Err(RaoulError::new_vec(
                node,
                RaoulErrorKind::NotList(name.to_owned()),
            ));
        }
        match (dim_2.is_none(), idx_2.is_none()) {
            (true, false) => Err(RaoulError::new_vec(
                node.clone(),
                RaoulErrorKind::NotMatrix(name.to_owned()),
            )),
            (false, true) => Err(RaoulError::new_vec(
                node.clone(),
                RaoulErrorKind::UsePrimitive,
            )),
            _ => Ok(()),
        }?;
        let v_address_op = self.safe_add_cte(v.address.into(), node.clone())?;
        let idx_1_op = self.assert_expr_type(*idx_1, Types::INT)?;
        let dim_1_op = self.safe_add_cte(dim_1.unwrap().into(), node.clone())?;
        self.add_quad(Quadruple {
            operator: Operator::Ver,
            op_1: Some(idx_1_op.0),
            op_2: Some(dim_1_op.0),
            res: None,
        });
        let address: usize = match idx_2 {
            None => {
                let pointer = self.pointer_memory.get_pointer();
                self.add_quad(Quadruple {
                    operator: Operator::Sum,
                    op_1: Some(v_address_op.0),
                    op_2: Some(idx_1_op.0),
                    res: Some(pointer),
                });
                pointer
            }
            Some(idx_2) => {
                let dim_2_op = self.safe_add_cte(dim_2.unwrap().into(), node.clone())?;
                let mult_op =
                    self.add_binary_op_quad(Operator::Times, idx_1_op, dim_2_op, node.clone())?;
                let idx_2_op = self.assert_expr_type(*idx_2, Types::INT)?;
                self.add_quad(Quadruple {
                    operator: Operator::Ver,
                    op_1: Some(idx_2_op.0),
                    op_2: Some(dim_2_op.0),
                    res: None,
                });
                let (sum_res, _) =
                    self.add_binary_op_quad(Operator::Sum, v_address_op, mult_op, node)?;
                let pointer = self.pointer_memory.get_pointer();
                self.add_quad(Quadruple {
                    operator: Operator::Sum,
                    op_1: Some(sum_res),
                    op_2: Some(idx_2_op.0),
                    res: Some(pointer),
                });
                pointer
            }
        };
        Ok((address, v.data_type))
    }

    fn parse_expr<'a>(&mut self, node: AstNode<'a>) -> Results<'a, Operand> {
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Bool(_)
            | AstNodeKind::Float(_)
            | AstNodeKind::Integer(_)
            | AstNodeKind::String(_) => {
                self.safe_add_cte(VariableValue::from(node.kind), node_clone)
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
            AstNodeKind::Id(name) => {
                let variable = self.get_variable(&name, node_clone.clone())?.clone();
                match variable.dimensions.0 {
                    None => Ok((variable.address, variable.data_type)),
                    _ => Err(RaoulError::new_vec(
                        node_clone,
                        RaoulErrorKind::UsePrimitive,
                    )),
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
                let op_1 = self.parse_expr(*lhs)?;
                let op_2 = self.parse_expr(*rhs)?;
                self.add_binary_op_quad(operator, op_1, op_2, node_clone)
            }
            AstNodeKind::FuncCall { name, exprs } => {
                self.parse_func_call(&name, node_clone.clone(), exprs)?;
                self.get_variable_name_address(&name, node_clone)
            }
            AstNodeKind::ArrayVal {
                ref name,
                idx_1,
                idx_2,
            } => self.get_array_val_operand(name, node_clone, idx_1, idx_2),
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

    fn parse_return_body<'a>(&mut self, body: Vec<AstNode<'a>>) -> Results<'a, bool> {
        let prev = self.missing_return;
        self.parse_body(body)?;
        let current = self.missing_return;
        if self.missing_return != prev {
            self.missing_return = prev;
        }
        Ok(current)
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

    fn add_assign_quad<'a>(
        &mut self,
        variable_address: usize,
        value: AstNode<'a>,
    ) -> Results<'a, ()> {
        let (value_addr, _) = self.parse_expr(value)?;
        Ok(self.add_quad(Quadruple {
            operator: Operator::Assignment,
            op_1: Some(value_addr),
            op_2: None,
            res: Some(variable_address),
        }))
    }

    fn parse_function<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Assignment {
                assignee,
                global,
                value,
            } => {
                if value.is_array() {
                    match value.kind {
                        AstNodeKind::Array(exprs) => {
                            let name = String::from(*assignee.clone());
                            let variable = self.get_variable(&name, *assignee)?.clone();
                            let dim_2 = variable.dimensions.1;
                            match dim_2.is_none() {
                                true => {
                                    let errors: Vec<_> = exprs
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, expr)| -> Results<()> {
                                            let idx_1 = Box::new(AstNode::new(
                                                AstNodeKind::Integer(i.try_into().unwrap()),
                                                expr.span.clone(),
                                            ));
                                            let (variable_address, _) = self
                                                .get_array_val_operand(
                                                    &name,
                                                    node_clone.clone(),
                                                    idx_1,
                                                    None,
                                                )?;
                                            self.add_assign_quad(variable_address, expr)
                                        })
                                        .filter_map(|v| v.err())
                                        .flatten()
                                        .collect();
                                    match errors.is_empty() {
                                        true => (),
                                        false => return Err(errors),
                                    }
                                }
                                false => {
                                    let errors: Vec<_> = exprs
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, exprs)| -> Results<()> {
                                            let idx_1 = Box::new(AstNode::new(
                                                AstNodeKind::Integer(i.try_into().unwrap()),
                                                node_clone.span.clone(),
                                            ));
                                            let errors: Vec<_> = exprs
                                                .expand_array()
                                                .into_iter()
                                                .enumerate()
                                                .map(|(j, expr)| -> Results<()> {
                                                    let idx_2 = Box::new(AstNode::new(
                                                        AstNodeKind::Integer(j.try_into().unwrap()),
                                                        expr.span.clone(),
                                                    ));
                                                    let (variable_address, _) = self
                                                        .get_array_val_operand(
                                                            &name,
                                                            node_clone.clone(),
                                                            idx_1.clone(),
                                                            Some(idx_2),
                                                        )?;
                                                    self.add_assign_quad(
                                                        variable_address,
                                                        expr.clone(),
                                                    )
                                                })
                                                .filter_map(|v| v.err())
                                                .flatten()
                                                .collect();
                                            match errors.is_empty() {
                                                true => Ok(()),
                                                false => return Err(errors),
                                            }
                                        })
                                        .filter_map(|v| v.err())
                                        .flatten()
                                        .collect();
                                    match errors.is_empty() {
                                        true => (),
                                        false => return Err(errors),
                                    }
                                }
                            }
                        }
                        _ => (),
                    };
                    return Ok(());
                }
                let variable_address: usize = match assignee.kind {
                    AstNodeKind::ArrayVal {
                        ref name,
                        idx_1,
                        idx_2,
                    } => {
                        let op = self.get_array_val_operand(name, node_clone, idx_1, idx_2)?;
                        op.0
                    }
                    _ => {
                        let name: String = assignee.into();
                        self.get_variable_address(global, &name)
                    }
                };
                self.add_assign_quad(variable_address, *value)
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
                let if_misses_return = self.parse_return_body(statements)?;
                Ok(if let Some(node) = else_block {
                    let index = self.jump_list.pop().unwrap();
                    self.add_goto(Operator::Goto, None);
                    self.fill_goto_index(index);
                    self.parse_function(*node)?;
                    self.fill_goto();
                    if if_misses_return & !self.missing_return {
                        self.missing_return = true;
                    }
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
                    self.get_variable_name_address(&name, node_clone.clone())?;
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
                self.missing_return = false;
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
                    self.missing_return = true;
                }
                self.parse_body(body)?;
                if self.missing_return {
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

impl fmt::Display for QuadrupleManager {
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
