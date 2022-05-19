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
    error::{error_kind::RaoulErrorKind, RaoulError, Results},
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

fn safe_address<T>(option: Option<T>, node: AstNode) -> Results<T> {
    match option {
        Some(value) => Ok(value),
        None => Err(vec![RaoulError::new(node, RaoulErrorKind::MemoryExceded)]),
    }
}

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
    pub fn clear_variables(&mut self) {
        self.dir_func.clear_variables();
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
        let variables = if global {
            self.global_variables()
        } else {
            self.function_variables()
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
    fn add_temp(&mut self, data_type: Types) -> Option<usize> {
        self.function_mut()
            .temp_addresses
            .get_address(data_type, (None, None))
    }

    #[inline]
    fn safe_add_temp<'a>(&mut self, data_type: Types, node: AstNode<'a>) -> Results<'a, usize> {
        safe_address(self.add_temp(data_type), node)
    }

    fn safe_remove_temp_address(&mut self, operand: Option<usize>) {
        if !operand.is_temp_address() {
            return;
        }
        self.function_mut()
            .temp_addresses
            .release_address(operand.unwrap());
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
                RaoulErrorKind::UndeclaredVar(name.to_string()),
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

    fn parse_args_exprs<'a>(
        &mut self,
        node: AstNode<'a>,
        exprs: Vec<AstNode<'a>>,
        args: &[Variable],
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
                v_type.assert_cast(arg_type, node)?;
                Ok((v, v_type))
            })
            .partition(Results::is_ok);
        if errors.is_empty() {
            Ok(addresses.into_iter().map(Results::unwrap).collect())
        } else {
            Err(errors.into_iter().flat_map(Results::unwrap_err).collect())
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
        let args = &self.get_function(name).args.clone();
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
                });
            });
        self.add_go_sub_quad(name);
        Ok(())
    }

    #[inline]
    fn safe_add_cte<'a>(
        &mut self,
        value: VariableValue,
        node: AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        safe_address(self.memory.add(value), node)
    }

    fn add_binary_op_quad<'a>(
        &mut self,
        operator: Operator,
        op_1: Operand,
        op_2: Operand,
        node: AstNode<'a>,
    ) -> Results<'a, Operand> {
        let data_type = match Types::binary_operator_type(operator, op_1.1, op_2.1) {
            Ok(data_type) => Ok(data_type),
            Err(kind) => Err(RaoulError::new_vec(node.clone(), kind)),
        }?;
        let res = self.safe_add_temp(data_type, node)?;
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
        idx_1: AstNode<'a>,
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
        let idx_1_op = self.assert_expr_type(idx_1, Types::Int)?;
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
                let idx_2_op = self.assert_expr_type(*idx_2, Types::Int)?;
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

    fn assert_dataframe<'a>(&mut self, name: &str, node: AstNode<'a>) -> Results<'a, ()> {
        let data_type = self.get_variable(name, node.clone())?.data_type;
        if data_type == Types::Dataframe {
            return Ok(());
        }
        let kind = RaoulErrorKind::InvalidCast {
            from: data_type,
            to: Types::Dataframe,
        };
        Err(RaoulError::new_vec(node, kind))
    }

    // TODO: Maybe fix later
    #[allow(clippy::too_many_lines)]
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
                        Types::Bool | Types::Int => Types::Bool,
                        op_type => {
                            let kind = RaoulErrorKind::InvalidCast {
                                from: op_type,
                                to: Types::Bool,
                            };
                            return Err(vec![RaoulError::new(node_clone, kind)]);
                        }
                    },
                    _ => unreachable!(),
                };
                let res = self.safe_add_temp(res_type, node_clone)?;
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
                let data_type = Types::String;
                let res = self.safe_add_temp(data_type, node_clone)?;
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
                let (fn_address, return_type) =
                    self.get_variable_name_address(&name, node_clone.clone())?;
                let temp_address = self.safe_add_temp(return_type, node_clone.clone())?;
                self.add_quad(Quadruple {
                    operator: Operator::Assignment,
                    op_1: Some(fn_address),
                    op_2: None,
                    res: Some(temp_address),
                });
                Ok((temp_address, return_type))
            }
            AstNodeKind::ArrayVal {
                ref name,
                idx_1,
                idx_2,
            } => self.get_array_val_operand(name, node_clone, *idx_1, idx_2),
            AstNodeKind::UnaryDataframeOp {
                operator,
                name,
                column,
            } => {
                self.assert_dataframe(&name, node_clone.clone())?;
                let (column_address, _) = self.assert_expr_type(*column, Types::String)?;
                let data_type = Types::Float;
                let res = self.safe_add_temp(data_type, node_clone)?;
                self.add_quad(Quadruple {
                    operator,
                    op_1: Some(column_address),
                    op_2: None,
                    res: Some(res),
                });
                Ok((res, data_type))
            }
            AstNodeKind::Correlation {
                name,
                column_1,
                column_2,
            } => {
                self.assert_dataframe(&name, node_clone.clone())?;
                let (column_1_address, _) = self.assert_expr_type(*column_1, Types::String)?;
                let (column_2_address, _) = self.assert_expr_type(*column_2, Types::String)?;
                let data_type = Types::Float;
                let res = self.safe_add_temp(data_type, node_clone)?;
                self.add_quad(Quadruple {
                    operator: Operator::Corr,
                    op_1: Some(column_1_address),
                    op_2: Some(column_2_address),
                    res: Some(res),
                });
                Ok((res, data_type))
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    fn assert_expr_type<'a>(
        &mut self,
        expr: AstNode<'a>,
        to: Types,
    ) -> Results<'a, (usize, Types)> {
        let expr_clone = expr.clone();
        let (res_address, res_type) = self.parse_expr(expr)?;
        res_type.assert_cast(to, expr_clone)?;
        Ok((res_address, res_type))
    }

    #[inline]
    fn parse_body<'a>(&mut self, body: Vec<AstNode<'a>>) -> Results<'a, ()> {
        RaoulError::create_results(body.into_iter().map(|node| self.parse_function(node)))
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
        });
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
        self.add_quad(Quadruple {
            operator: Operator::Assignment,
            op_1: Some(value_addr),
            op_2: None,
            res: Some(variable_address),
        });
        Ok(())
    }

    fn parse_array<'a>(
        &mut self,
        assignee: AstNode<'a>,
        exprs: Vec<AstNode<'a>>,
        node: &AstNode<'a>,
    ) -> Results<'a, ()> {
        let name = String::from(assignee.clone());
        let variable = self.get_variable(&name, assignee)?.clone();
        let dim_2 = variable.dimensions.1;
        if dim_2.is_none() {
            RaoulError::create_results(exprs.into_iter().enumerate().map(
                |(i, expr)| -> Results<()> {
                    let idx_1 = Box::new(AstNode::new(AstNodeKind::from(i), expr.span.clone()));
                    let (variable_address, _) =
                        self.get_array_val_operand(&name, node.clone(), *idx_1, None)?;
                    self.add_assign_quad(variable_address, expr)
                },
            ))
        } else {
            RaoulError::create_results(exprs.into_iter().enumerate().map(
                |(i, exprs)| -> Results<()> {
                    let idx_1 = Box::new(AstNode::new(AstNodeKind::from(i), node.span.clone()));
                    RaoulError::create_results(exprs.expand_array().into_iter().enumerate().map(
                        |(j, expr)| -> Results<()> {
                            let idx_2 =
                                Box::new(AstNode::new(AstNodeKind::from(j), expr.span.clone()));
                            let (variable_address, _) = self.get_array_val_operand(
                                &name,
                                node.clone(),
                                *idx_1.clone(),
                                Some(idx_2),
                            )?;
                            self.add_assign_quad(variable_address, expr.clone())
                        },
                    ))
                },
            ))
        }
    }

    fn parse_function<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Assignment {
                assignee,
                global,
                value,
            } => match value.kind {
                AstNodeKind::ArrayDeclaration { .. } => Ok(()),
                AstNodeKind::Array(exprs) => self.parse_array(*assignee, exprs, &node_clone),
                AstNodeKind::ReadCSV(file_node) => {
                    let (file_address, _) = self.assert_expr_type(*file_node, Types::String)?;
                    self.add_quad(Quadruple {
                        operator: Operator::ReadCSV,
                        op_1: Some(file_address),
                        op_2: None,
                        res: None,
                    });
                    Ok(())
                }
                _ => {
                    let variable_address: usize = match assignee.kind {
                        AstNodeKind::ArrayVal {
                            ref name,
                            idx_1,
                            idx_2,
                        } => {
                            let op = self.get_array_val_operand(name, node_clone, *idx_1, idx_2)?;
                            op.0
                        }
                        _ => {
                            let name: String = assignee.into();
                            self.get_variable_address(global, &name)
                        }
                    };
                    self.add_assign_quad(variable_address, *value)
                }
            },
            AstNodeKind::Write { exprs } => {
                RaoulError::create_results(exprs.into_iter().map(|expr| -> Results<()> {
                    let (address, _) = self.parse_expr(expr)?;
                    self.add_quad(Quadruple {
                        operator: Operator::Print,
                        op_1: Some(address),
                        op_2: None,
                        res: None,
                    });
                    Ok(())
                }))?;
                self.add_quad(Quadruple {
                    operator: Operator::PrintNl,
                    op_1: None,
                    op_2: None,
                    res: None,
                });
                Ok(())
            }
            AstNodeKind::Decision {
                expr,
                statements,
                else_block,
            } => {
                let (res_address, _) = self.assert_expr_type(*expr, Types::Bool)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                let if_misses_return = self.parse_return_body(statements)?;
                if let Some(node) = else_block {
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
                }
                Ok(())
            }
            AstNodeKind::ElseBlock { statements } => Ok(self.parse_body(statements)?),
            AstNodeKind::While { expr, statements } => {
                self.jump_list.push(self.quad_list.len());
                let (res_address, _) = self.assert_expr_type(*expr, Types::Bool)?;
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
                self.fill_goto_index(index);
                Ok(())
            }
            AstNodeKind::For {
                assignment,
                expr,
                statements,
            } => {
                let name = String::from(*assignment.clone());
                self.parse_function(*assignment)?;
                self.jump_list.push(self.quad_list.len());
                let (res_address, _) = self.assert_expr_type(*expr, Types::Bool)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                self.parse_return_body(statements)?;
                let (var_address, var_type) =
                    self.get_variable_name_address(&name, node_clone.clone())?;
                var_type.assert_cast(Types::Int, node_clone)?;
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
                self.fill_goto_index(index);
                Ok(())
            }
            AstNodeKind::Return(expr) => {
                let return_type = self.function().return_type;
                let (expr_address, _) = self.assert_expr_type(*expr, return_type)?;
                self.missing_return = false;
                self.add_quad(Quadruple {
                    operator: Operator::Return,
                    op_1: Some(expr_address),
                    op_2: None,
                    res: None,
                });
                Ok(())
            }
            AstNodeKind::FuncCall { ref name, exprs } => {
                match self.dir_func.functions.get(name).is_some() {
                    true => self.parse_func_call(name, node_clone, exprs),
                    false => {
                        let kind = RaoulErrorKind::UndeclaredFunction2(name.to_string());
                        Err(RaoulError::new_vec(node_clone, kind))
                    }
                }
            }
            AstNodeKind::Plot {
                name,
                column_1,
                column_2,
            } => {
                self.assert_dataframe(&name, node_clone.clone())?;
                let (column_1_address, _) = self.assert_expr_type(*column_1, Types::String)?;
                let (column_2_address, _) = self.assert_expr_type(*column_2, Types::String)?;
                self.add_quad(Quadruple {
                    operator: Operator::Plot,
                    op_1: Some(column_1_address),
                    op_2: Some(column_2_address),
                    res: None,
                });
                Ok(())
            }
            AstNodeKind::Histogram { bins, column, name } => {
                self.assert_dataframe(&name, node_clone.clone())?;
                let (column_address, _) = self.assert_expr_type(*column, Types::String)?;
                let (bins_address, _) = self.assert_expr_type(*bins, Types::Int)?;
                self.add_quad(Quadruple {
                    operator: Operator::Histogram,
                    op_1: Some(column_address),
                    op_2: Some(bins_address),
                    res: None,
                });
                Ok(())
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    #[inline]
    fn update_quad(&mut self, first_quad: usize) {
        self.function_mut().update_quad(first_quad);
    }

    pub fn parse<'a>(&mut self, node: AstNode<'a>) -> Results<'a, ()> {
        let clone = node.clone();
        match node.kind {
            AstNodeKind::Main {
                body,
                functions,
                assignments,
            } => {
                self.add_goto(Operator::Goto, None);
                RaoulError::create_results(functions.into_iter().map(|node| self.parse(node)))?;
                self.fill_goto();
                self.function_name = "main".to_owned();
                RaoulError::create_results(
                    assignments
                        .into_iter()
                        .map(|node| self.parse_function(node)),
                )?;
                self.parse_body(body)?;
                self.add_quad(Quadruple {
                    operator: Operator::End,
                    op_1: None,
                    op_2: None,
                    res: None,
                });
                Ok(())
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
                if return_type != Types::Void {
                    self.missing_return = true;
                }
                self.parse_body(body)?;
                if self.missing_return {
                    let kind = RaoulErrorKind::MissingReturn(self.function_name.clone());
                    return Err(vec![RaoulError::new(clone, kind)]);
                }
                self.add_quad(Quadruple {
                    operator: Operator::EndProc,
                    op_1: None,
                    op_2: None,
                    res: None,
                });
                Ok(())
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
            .map(|(i, quad)| format!("{:<4} - {:?}\n", i, quad))
            .collect();
        write!(f, "{value}")
    }
}
