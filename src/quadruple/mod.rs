use crate::{
    address::ConstantMemory,
    ast::{ast_kind::AstNodeKind, AstNode},
    dir_func::{function::Function, variable_value::VariableValue, DirFunc},
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
    memory: ConstantMemory,
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

    fn get_variable_address(&self, global: bool, name: &str) -> usize {
        let variables = match global {
            true => &self.dir_func.global_fn.variables,
            false => &self.function().variables,
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

    fn safe_address<'a, T>(&self, result: Option<T>, node: AstNode<'a>) -> Result<'a, T> {
        match result {
            Some(value) => Ok(value),
            None => Err(RaoulError::new(node, RaoulErrorKind::MemoryExceded)),
        }
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
                    Operator::NOT => match op_type {
                        Types::BOOL | Types::INT => Types::BOOL,
                        op_type => {
                            let kind = RaoulErrorKind::InvalidCast {
                                from: op_type,
                                to: Types::BOOL,
                            };
                            return Err(RaoulError::new(node_clone, kind));
                        }
                    },
                };
                let result = self.add_temp(&res_type);
                let res = self.safe_address(result, node_clone)?;
                let quad = Quadruple {
                    operator,
                    op_1: Some(op),
                    op_2: None,
                    res: Some(res),
                };
                self.quad_list.push(quad);
                Ok((res, res_type))
            }
            _ => unreachable!(),
        }
    }

    fn parse_function<'a>(&mut self, node: AstNode<'a>) -> Result<'a, ()> {
        match node.kind {
            AstNodeKind::Assignment {
                global,
                ref name,
                value,
            } => {
                let (value_addr, _) = self.parse_expr(*value)?;
                let variable_address = self.get_variable_address(global, name);
                todo!("Add the ASGN operator for this");
            }
            AstNodeKind::Write { .. } => todo!("Add the WRITE operator for this"),
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
