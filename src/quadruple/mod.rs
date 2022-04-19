use crate::{
    address::ConstantMemory,
    ast::{ast_kind::AstNodeKind, AstNode},
    dir_func::{variable_value::VariableValue, DirFunc},
    enums::{Operator, Types},
    error::{error_kind::RaoulErrorKind, RaoulError, Result},
};

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
struct Quadruple {
    operator: Operator,
    op_1: Option<usize>,
    op_2: Option<usize>,
    res: Option<usize>,
}

struct QuadrupleManager<'a> {
    dir_func: &'a DirFunc,
    function_name: String,
    memory: ConstantMemory,
    quad_list: Vec<Quadruple>,
}

impl QuadrupleManager<'_> {
    pub fn new<'a>(dir_func: &'a DirFunc) -> QuadrupleManager<'a> {
        QuadrupleManager {
            dir_func,
            function_name: "".to_owned(),
            memory: ConstantMemory::new(),
            quad_list: Vec::new(),
        }
    }

    pub fn parse_expr<'a>(&mut self, node: AstNode<'a>) -> Result<'a, (usize, Types)> {
        let node_clone = node.clone();
        match node.kind {
            AstNodeKind::Bool(_)
            | AstNodeKind::Float(_)
            | AstNodeKind::Integer(_)
            | AstNodeKind::String(_) => {
                let value = VariableValue::from(node.kind);
                match self.memory.add(value) {
                    Some(tuple) => Ok(tuple),
                    None => {
                        let kind = RaoulErrorKind::MemoryExceded;
                        Err(RaoulError::new(node_clone, kind))
                    }
                }
            }
            AstNodeKind::UnaryOperation { operator, operand } => {
                let (op, op_type) = self.parse_expr(*operand)?;
                let quad = Quadruple {
                    operator,
                    op_1: Some(op),
                    op_2: None,
                    res: None,
                };
                self.quad_list.push(quad);
                Ok((op, op_type))
            }
            _ => unreachable!(),
        }
    }
}
