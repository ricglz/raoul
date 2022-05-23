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

pub type Operand = (usize, Types);

fn safe_address<'a, T>(option: Option<T>, node: &AstNode<'a>) -> Results<'a, T> {
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
    fn safe_add_temp<'a>(&mut self, data_type: Types, node: &AstNode<'a>) -> Results<'a, usize> {
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

    fn get_variable<'a>(&mut self, name: &str, node: &AstNode<'a>) -> Results<'a, &Variable> {
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
        node: &AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        let variable = self.get_variable(name, node)?;
        Ok((variable.address, variable.data_type))
    }

    fn parse_args_exprs<'a>(
        &mut self,
        node: &AstNode<'a>,
        exprs: &[AstNode<'a>],
        args: &[Operand],
    ) -> Results<'a, Vec<Operand>> {
        if args.len() != exprs.len() {
            let kind = RaoulErrorKind::UnmatchArgsAmount {
                expected: args.len(),
                given: exprs.len(),
            };
            return Err(vec![RaoulError::new(node, kind)]);
        }
        let addresses = RaoulError::create_partition(exprs.iter().zip(args).map(
            |(node, (_, arg_type))| -> Results<(usize, Types)> {
                let (v, v_type) = self.parse_expr(node)?;
                v_type.assert_cast(*arg_type, node)?;
                Ok((v, v_type))
            },
        ))?;
        Ok(addresses)
    }

    fn add_era_quad(&mut self, name: &str) {
        let function = self.get_function(name);
        let function_size = function.size();
        let first_quad = function.first_quad;
        self.add_quad(Quadruple::new_args(
            Operator::Era,
            function_size,
            first_quad,
        ));
    }

    fn add_go_sub_quad(&mut self, name: &str) {
        let first_quad = self.get_function(name).first_quad;
        self.add_quad(Quadruple::new_arg(Operator::GoSub, first_quad));
    }

    fn parse_func_call<'a>(
        &mut self,
        name: &str,
        node: &AstNode<'a>,
        exprs: &[AstNode<'a>],
    ) -> Results<'a, ()> {
        self.add_era_quad(name);
        let args = &self.get_function(name).args.clone();
        let addresses = self.parse_args_exprs(node, exprs, args)?;
        addresses
            .into_iter()
            .enumerate()
            .for_each(|(i, (address, _))| {
                self.add_quad(Quadruple::new_un(Operator::Param, address, i));
            });
        self.add_go_sub_quad(name);
        Ok(())
    }

    #[inline]
    fn safe_add_cte<'a>(
        &mut self,
        value: VariableValue,
        node: &AstNode<'a>,
    ) -> Results<'a, (usize, Types)> {
        safe_address(self.memory.add(value), node)
    }

    fn add_binary_op_quad<'a>(
        &mut self,
        operator: Operator,
        op_1: Operand,
        op_2: Operand,
        node: &AstNode<'a>,
    ) -> Results<'a, Operand> {
        let data_type = op_1.1.assert_bin_op(operator, op_2.1, node)?;
        let res = self.safe_add_temp(data_type, node)?;
        self.add_quad(Quadruple::new_com(operator, op_1.0, op_2.0, res));
        Ok((res, data_type))
    }

    fn get_array_val_operand<'a>(
        &mut self,
        name: &str,
        node: &AstNode<'a>,
        idx_1_op: &Operand,
        idx_2_op: Option<Operand>,
    ) -> Results<'a, Operand> {
        let v = (self.get_variable(name, node)?).clone();
        let (dim_1, dim_2) = v.dimensions;
        if dim_1.is_none() {
            return Err(RaoulError::new_vec(
                node,
                RaoulErrorKind::NotList(name.to_owned()),
            ));
        }
        match (dim_2.is_none(), idx_2_op.is_none()) {
            (true, false) => Err(RaoulError::new_vec(
                node,
                RaoulErrorKind::NotMatrix(name.to_owned()),
            )),
            (false, true) => Err(RaoulError::new_vec(node, RaoulErrorKind::UsePrimitive)),
            _ => Ok(()),
        }?;
        let v_address_op = self.safe_add_cte(v.address.into(), node)?;
        let dim_1_op = self.safe_add_cte(dim_1.unwrap().into(), node)?;
        self.add_quad(Quadruple::new_args(Operator::Ver, idx_1_op.0, dim_1_op.0));
        let address: usize = match idx_2_op {
            None => {
                let pointer = self.pointer_memory.get_pointer();
                self.add_quad(Quadruple::new_com(
                    Operator::Sum,
                    v_address_op.0,
                    idx_1_op.0,
                    pointer,
                ));
                pointer
            }
            Some(idx_2_op) => {
                let dim_2_op = self.safe_add_cte(dim_2.unwrap().into(), node)?;
                let mult_op =
                    self.add_binary_op_quad(Operator::Times, *idx_1_op, dim_2_op, node)?;
                self.add_quad(Quadruple::new_args(Operator::Ver, idx_2_op.0, dim_2_op.0));
                let (sum_res, _) =
                    self.add_binary_op_quad(Operator::Sum, v_address_op, mult_op, node)?;
                let pointer = self.pointer_memory.get_pointer();
                self.add_quad(Quadruple::new_com(
                    Operator::Sum,
                    sum_res,
                    idx_2_op.0,
                    pointer,
                ));
                pointer
            }
        };
        Ok((address, v.data_type))
    }

    fn arr_val_op_node<'a>(
        &mut self,
        name: &str,
        node: &AstNode<'a>,
        idx_1: &AstNode<'a>,
        idx_2: Option<Box<AstNode<'a>>>,
    ) -> Results<'a, Operand> {
        let idx_1_op = &self.assert_expr_type(idx_1, Types::Int)?;
        let idx_2_op = match idx_2 {
            Some(idx_2) => Some(self.assert_expr_type(&*idx_2, Types::Int)?),
            None => None,
        };
        self.get_array_val_operand(name, node, idx_1_op, idx_2_op)
    }

    fn assert_dataframe<'a>(&mut self, name: &str, node: &AstNode<'a>) -> Results<'a, ()> {
        let data_type = self.get_variable(name, node)?.data_type;
        data_type.assert_cast(Types::Dataframe, node)
    }

    fn dataframe_op<'a>(
        &mut self,
        name: &str,
        node: &AstNode<'a>,
        operator: Operator,
        op_1: usize,
        op_2: Option<usize>,
    ) -> Results<'a, Operand> {
        self.assert_dataframe(name, node)?;
        let data_type = Types::Float;
        let res = self.safe_add_temp(data_type, node)?;
        self.add_quad(Quadruple::new(operator, Some(op_1), op_2, Some(res)));
        Ok((res, data_type))
    }

    fn parse_expr<'a>(&mut self, node: &AstNode<'a>) -> Results<'a, Operand> {
        match &node.kind {
            AstNodeKind::Bool(_)
            | AstNodeKind::Float(_)
            | AstNodeKind::Integer(_)
            | AstNodeKind::String(_) => self.safe_add_cte(VariableValue::from(&node.kind), node),
            AstNodeKind::UnaryOperation { operator, operand } => {
                let (op, op_type) = self.parse_expr(&*operand)?;
                let res_type = match operator {
                    Operator::Not => match op_type {
                        Types::Bool | Types::Int => Types::Bool,
                        op_type => {
                            let kind = RaoulErrorKind::InvalidCast {
                                from: op_type,
                                to: Types::Bool,
                            };
                            return Err(vec![RaoulError::new(node, kind)]);
                        }
                    },
                    _ => unreachable!(),
                };
                let res = self.safe_add_temp(res_type, node)?;
                self.add_quad(Quadruple::new_un(*operator, op, res));
                Ok((res, res_type))
            }
            AstNodeKind::Id(name) => {
                let variable = self.get_variable(name, node)?;
                match variable.dimensions.0 {
                    None => Ok((variable.address, variable.data_type)),
                    _ => Err(RaoulError::new_vec(node, RaoulErrorKind::UsePrimitive)),
                }
            }
            AstNodeKind::Read => {
                let data_type = Types::String;
                let res = self.safe_add_temp(data_type, node)?;
                self.add_quad(Quadruple::new_res(Operator::Read, res));
                Ok((res, data_type))
            }
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                let op_1 = self.parse_expr(&*lhs)?;
                let op_2 = self.parse_expr(&*rhs)?;
                self.add_binary_op_quad(*operator, op_1, op_2, node)
            }
            AstNodeKind::FuncCall { name, ref exprs } => {
                self.parse_func_call(name, node, exprs)?;
                let (fn_address, return_type) = self.get_variable_name_address(name, node)?;
                let temp_address = self.safe_add_temp(return_type, node)?;
                self.add_quad(Quadruple::new_un(
                    Operator::Assignment,
                    fn_address,
                    temp_address,
                ));
                Ok((temp_address, return_type))
            }
            AstNodeKind::ArrayVal {
                ref name,
                idx_1,
                idx_2,
            } => self.arr_val_op_node(name, node, &*idx_1, idx_2.clone()),
            AstNodeKind::PureDataframeOp { operator, ref name } => {
                self.assert_dataframe(name, node)?;
                let data_type = Types::Int;
                let res = self.safe_add_temp(data_type, node)?;
                self.add_quad(Quadruple::new_res(*operator, res));
                Ok((res, data_type))
            }
            AstNodeKind::UnaryDataframeOp {
                operator,
                ref name,
                column,
            } => {
                let (column_address, _) = self.assert_expr_type(&*column, Types::String)?;
                self.dataframe_op(name, node, *operator, column_address, None)
            }
            AstNodeKind::Correlation {
                ref name,
                column_1,
                column_2,
            } => {
                let (col_1, _) = self.assert_expr_type(&*column_1, Types::String)?;
                let (col_2, _) = self.assert_expr_type(&*column_2, Types::String)?;
                let operator = Operator::Corr;
                self.dataframe_op(name, node, operator, col_1, Some(col_2))
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    fn assert_expr_type<'a>(
        &mut self,
        expr: &AstNode<'a>,
        to: Types,
    ) -> Results<'a, (usize, Types)> {
        let (res_address, res_type) = self.parse_expr(expr)?;
        res_type.assert_cast(to, expr)?;
        Ok((res_address, res_type))
    }

    #[inline]
    fn parse_body<'a>(&mut self, body: &[AstNode<'a>]) -> Results<'a, ()> {
        RaoulError::create_results(body.iter().map(|node| self.parse_statement(node)))
    }

    fn parse_return_body<'a>(&mut self, body: &[AstNode<'a>]) -> Results<'a, bool> {
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
        self.add_quad(Quadruple::new(goto_type, condition, None, None));
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

    fn add_assign_quad<'a>(&mut self, res: usize, value: &AstNode<'a>) -> Results<'a, ()> {
        let (op_1, _) = self.parse_expr(value)?;
        self.add_quad(Quadruple::new_un(Operator::Assignment, op_1, res));
        Ok(())
    }

    fn parse_array<'a>(
        &mut self,
        assignee: &AstNode<'a>,
        exprs: &[AstNode<'a>],
        node: &AstNode<'a>,
    ) -> Results<'a, ()> {
        let name = String::from(assignee);
        let variable = self.get_variable(&name, assignee)?.clone();
        let dim_2 = variable.dimensions.1;
        if dim_2.is_none() {
            RaoulError::create_results(exprs.iter().enumerate().map(|(i, expr)| -> Results<()> {
                let idx_1_op = self.safe_add_cte(i.into(), expr)?;
                let (variable_address, _) =
                    self.get_array_val_operand(&name, node, &idx_1_op, None)?;
                self.add_assign_quad(variable_address, expr)
            }))
        } else {
            RaoulError::create_results(exprs.iter().enumerate().map(|(i, exprs)| -> Results<()> {
                let idx_1_op = self.safe_add_cte(i.into(), exprs)?;
                RaoulError::create_results(exprs.expand_array().iter().enumerate().map(
                    |(j, expr)| -> Results<()> {
                        let idx_2_op = self.safe_add_cte(j.into(), expr)?;
                        let (variable_address, _) =
                            self.get_array_val_operand(&name, node, &idx_1_op, Some(idx_2_op))?;
                        self.add_assign_quad(variable_address, expr)
                    },
                ))
            }))
        }
    }

    fn parse_assignment<'a>(
        &mut self,
        assignee: AstNode<'a>,
        global: bool,
        value: AstNode<'a>,
        node: &AstNode<'a>,
    ) -> Results<'a, ()> {
        match value.kind {
            AstNodeKind::ArrayDeclaration { .. } => Ok(()),
            AstNodeKind::Array(exprs) => self.parse_array(&assignee, &exprs, node),
            AstNodeKind::ReadCSV(file_node) => {
                let (file_address, _) = self.assert_expr_type(&*file_node, Types::String)?;
                self.add_quad(Quadruple::new_arg(Operator::ReadCSV, file_address));
                Ok(())
            }
            _ => {
                let variable_address = if let AstNodeKind::ArrayVal {
                    ref name,
                    idx_1,
                    idx_2,
                } = assignee.kind
                {
                    let op = self.arr_val_op_node(name, node, &*idx_1, idx_2)?;
                    op.0
                } else {
                    let name: String = assignee.into();
                    self.get_variable_address(global, &name)
                };
                self.add_assign_quad(variable_address, &value)
            }
        }
    }

    fn parse_for<'a>(
        &mut self,
        assignment: &AstNode<'a>,
        expr: &AstNode<'a>,
        statements: &[AstNode<'a>],
        node: &AstNode<'a>,
    ) -> Results<'a, ()> {
        let name = String::from(assignment);
        self.parse_statement(assignment)?;
        self.jump_list.push(self.quad_list.len());
        let (res_address, _) = self.assert_expr_type(expr, Types::Bool)?;
        self.add_goto(Operator::GotoF, Some(res_address));
        self.parse_return_body(statements)?;
        let (var_address, var_type) = self.get_variable_name_address(&name, node)?;
        var_type.assert_cast(Types::Int, node)?;
        self.add_quad(Quadruple::new_res(Operator::Inc, var_address));
        let index = self.jump_list.pop().unwrap();
        let goto_res = self.jump_list.pop().unwrap();
        self.add_quad(Quadruple::new_res(Operator::Goto, goto_res));
        self.fill_goto_index(index);
        Ok(())
    }

    fn parse_statement<'a>(&mut self, node: &AstNode<'a>) -> Results<'a, ()> {
        match &node.kind {
            AstNodeKind::Assignment {
                assignee,
                global,
                value,
            } => self.parse_assignment(*assignee.clone(), *global, *value.clone(), node),
            AstNodeKind::Write { exprs } => {
                RaoulError::create_results(exprs.iter().map(|expr| -> Results<()> {
                    let (address, _) = self.parse_expr(expr)?;
                    self.add_quad(Quadruple::new_arg(Operator::Print, address));
                    Ok(())
                }))?;
                self.add_quad(Quadruple::new_empty(Operator::PrintNl));
                Ok(())
            }
            AstNodeKind::Decision {
                expr,
                statements,
                else_block,
            } => {
                let (res_address, _) = self.assert_expr_type(&*expr, Types::Bool)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                let if_misses_return = self.parse_return_body(statements)?;
                if let Some(node) = else_block {
                    let index = self.jump_list.pop().unwrap();
                    self.add_goto(Operator::Goto, None);
                    self.fill_goto_index(index);
                    self.parse_statement(&*node)?;
                    self.fill_goto();
                    if if_misses_return && !self.missing_return {
                        self.missing_return = true;
                    }
                } else {
                    self.fill_goto();
                }
                Ok(())
            }
            AstNodeKind::ElseBlock { statements } => self.parse_body(statements),
            AstNodeKind::While { expr, statements } => {
                self.jump_list.push(self.quad_list.len());
                let (res_address, _) = self.assert_expr_type(&*expr, Types::Bool)?;
                self.add_goto(Operator::GotoF, Some(res_address));
                self.parse_return_body(statements)?;
                let index = self.jump_list.pop().unwrap();
                let goto_res = self.jump_list.pop().unwrap();
                self.add_quad(Quadruple::new_res(Operator::Goto, goto_res));
                self.fill_goto_index(index);
                Ok(())
            }
            AstNodeKind::For {
                assignment,
                expr,
                statements,
            } => self.parse_for(&*assignment, &*expr, statements, node),
            AstNodeKind::Return(expr) => {
                let return_type = self.function().return_type;
                let (expr_address, _) = self.assert_expr_type(&*expr, return_type)?;
                self.missing_return = false;
                self.add_quad(Quadruple::new_arg(Operator::Return, expr_address));
                Ok(())
            }
            AstNodeKind::FuncCall { ref name, exprs } => {
                if self.dir_func.functions.get(name).is_some() {
                    self.parse_func_call(name, node, exprs)
                } else {
                    let kind = RaoulErrorKind::UndeclaredFunction2(name.to_string());
                    Err(RaoulError::new_vec(node, kind))
                }
            }
            AstNodeKind::Plot {
                name,
                column_1,
                column_2,
            } => {
                self.assert_dataframe(name, node)?;
                let (col_1, _) = self.assert_expr_type(&*column_1, Types::String)?;
                let (col_2, _) = self.assert_expr_type(&*column_2, Types::String)?;
                self.add_quad(Quadruple::new_args(Operator::Plot, col_1, col_2));
                Ok(())
            }
            AstNodeKind::Histogram { bins, column, name } => {
                self.assert_dataframe(name, node)?;
                let (col, _) = self.assert_expr_type(&*column, Types::String)?;
                let (bins, _) = self.assert_expr_type(&*bins, Types::Int)?;
                self.add_quad(Quadruple::new_args(Operator::Histogram, col, bins));
                Ok(())
            }
            kind => unreachable!("{kind:?}"),
        }
    }

    #[inline]
    fn update_quad(&mut self, first_quad: usize) {
        self.function_mut().update_quad(first_quad);
    }

    pub fn parse<'a>(&mut self, node: &AstNode<'a>) -> Results<'a, ()> {
        match &node.kind {
            AstNodeKind::Main {
                body,
                functions,
                assignments,
            } => {
                self.add_goto(Operator::Goto, None);
                RaoulError::create_results(functions.iter().map(|node| self.parse(node)))?;
                self.fill_goto();
                self.function_name = "main".to_owned();
                RaoulError::create_results(
                    assignments.iter().map(|node| self.parse_statement(node)),
                )?;
                self.parse_body(body)?;
                self.add_quad(Quadruple::new_empty(Operator::End));
                Ok(())
            }
            AstNodeKind::Function {
                name,
                body,
                return_type,
                ..
            } => {
                self.function_name = name.clone();
                let first_quad = self.quad_list.len();
                self.update_quad(first_quad);
                if *return_type != Types::Void {
                    self.missing_return = true;
                }
                self.parse_body(body)?;
                if self.missing_return {
                    let kind = RaoulErrorKind::MissingReturn(self.function_name.clone());
                    return Err(vec![RaoulError::new(node, kind)]);
                }
                self.add_quad(Quadruple::new_empty(Operator::EndProc));
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
