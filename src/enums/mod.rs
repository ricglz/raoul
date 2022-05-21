use core::fmt;

use crate::ast::ast_kind::AstNodeKind;
use crate::ast::AstNode;
use crate::dir_func::function::VariablesTable;
use crate::dir_func::variable::Variable;
use crate::error::error_kind::RaoulErrorKind;
use crate::error::{RaoulError, Results};

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Types {
    Int,
    Void,
    Float,
    String,
    Bool,
    Dataframe,
}

impl Types {
    #[inline]
    pub fn is_boolish(self) -> bool {
        matches!(self, Types::Int | Types::Bool)
    }

    #[inline]
    fn is_number(self) -> bool {
        matches!(self, Types::Int | Types::Float | Types::String)
    }

    pub fn can_cast(self, to: Types) -> bool {
        if self.is_number() && to.is_number() {
            return true;
        }
        if self.is_boolish() && to.is_boolish() {
            return true;
        }
        self == to
    }

    pub fn assert_cast<'a>(self, to: Types, node: &AstNode<'a>) -> Results<'a, ()> {
        if self.can_cast(to) {
            return Ok(());
        }
        let error = RaoulError::new_vec(node, RaoulErrorKind::InvalidCast { from: self, to });
        Err(error)
    }

    pub fn binary_operator_type(
        self,
        operator: Operator,
        rhs_type: Types,
    ) -> Result<Types, (Types, Types)> {
        match operator {
            Operator::Not | Operator::Or | Operator::And => {
                let type_res = Types::Bool;
                match (self.is_boolish(), rhs_type.is_boolish()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err((rhs_type, type_res)),
                    _ => Err((self, type_res)),
                }
            }
            Operator::Gte | Operator::Lte | Operator::Gt | Operator::Lt => {
                let type_res = Types::Bool;
                match (self.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err((rhs_type, type_res)),
                    _ => Err((self, type_res)),
                }
            }
            Operator::Eq | Operator::Ne => {
                if self.can_cast(rhs_type) {
                    return Ok(Types::Bool);
                }
                Err((self, rhs_type))
            }
            Operator::Sum | Operator::Minus | Operator::Times | Operator::Div => {
                if self == rhs_type && self == Types::Int {
                    return Ok(Types::Int);
                }
                let type_res = Types::Float;
                match (self.is_number(), rhs_type.is_number()) {
                    (true, true) => Ok(type_res),
                    (true, false) => Err((rhs_type, type_res)),
                    _ => Err((self, type_res)),
                }
            }
            _ => unreachable!("{:?}", operator),
        }
    }

    pub fn assert_bin_op<'a>(
        self,
        operator: Operator,
        rhs_type: Types,
        node: &AstNode<'a>,
    ) -> Results<'a, Types> {
        match self.binary_operator_type(operator, rhs_type) {
            Ok(data_type) => Ok(data_type),
            Err((from, to)) => Err(RaoulError::new_vec(
                node,
                RaoulErrorKind::InvalidCast { from, to },
            )),
        }
    }

    #[inline]
    fn get_variable<'a>(
        name: &str,
        variables: &'a VariablesTable,
        global: &'a VariablesTable,
    ) -> Option<&'a Variable> {
        variables.get(name).or_else(|| global.get(name))
    }

    pub fn from_node<'a>(
        v: &AstNode<'a>,
        variables: &VariablesTable,
        global: &VariablesTable,
    ) -> Results<'a, Types> {
        let clone = v.clone();
        match &v.kind {
            AstNodeKind::Integer(_) => Ok(Types::Int),
            AstNodeKind::Float(_)
            | AstNodeKind::UnaryDataframeOp { .. }
            | AstNodeKind::Correlation { .. } => Ok(Types::Float),
            AstNodeKind::String(_) | AstNodeKind::Read => Ok(Types::String),
            AstNodeKind::Bool(_) => Ok(Types::Bool),
            AstNodeKind::Id(name) | AstNodeKind::ArrayVal { name, .. } => {
                match Types::get_variable(name, variables, global) {
                    Some(variable) => Ok(variable.data_type),
                    None => Err(RaoulError::new_vec(
                        &clone,
                        RaoulErrorKind::UndeclaredVar(name.to_string()),
                    )),
                }
            }
            AstNodeKind::FuncCall { name, .. } => {
                match Types::get_variable(name, variables, global) {
                    Some(variable) => Ok(variable.data_type),
                    None => Err(RaoulError::new_vec(
                        &clone,
                        RaoulErrorKind::UndeclaredFunction(name.to_string()),
                    )),
                }
            }
            AstNodeKind::ArrayDeclaration { data_type, .. } => Ok(*data_type),
            AstNodeKind::Array(exprs) => {
                let (types, errors): (Vec<_>, Vec<_>) = exprs
                    .iter()
                    .map(|node| Types::from_node(node, variables, global))
                    .partition(Results::is_ok);
                if !errors.is_empty() {
                    return Err(errors.into_iter().flat_map(Results::unwrap_err).collect());
                }
                let first_type = types.get(0).unwrap().clone().unwrap();
                RaoulError::create_results(types.into_iter().enumerate().map(|(i, v)| {
                    let data_type = v.unwrap();
                    let node = exprs.get(i).unwrap().clone();
                    data_type.assert_cast(first_type, &node)
                }))?;
                Ok(first_type)
            }
            AstNodeKind::BinaryOperation { operator, lhs, rhs } => {
                let lhs_type = Types::from_node(&*lhs, variables, global)?;
                let rhs_type = Types::from_node(&*rhs, variables, global)?;
                lhs_type.assert_bin_op(*operator, rhs_type, &clone)
            }
            AstNodeKind::UnaryOperation { operator, operand } => match operator {
                Operator::Not => {
                    let operand_type = Types::from_node(&*operand, variables, global)?;
                    let res_type = Types::Bool;
                    operand_type.assert_cast(res_type, &clone)?;
                    Ok(res_type)
                }
                _ => unreachable!("{:?}", operator),
            },
            AstNodeKind::ReadCSV(_) => Ok(Self::Dataframe),
            kind => Err(RaoulError::new_vec(
                &clone,
                RaoulErrorKind::EnteredUnreachable(format!("{kind:?}")),
            )),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Operator {
    // Boolean
    Not,
    Or,
    And,
    // Relational
    Gte,
    Lte,
    Gt,
    Lt,
    // Equality
    Eq,
    Ne,
    // Aritmetic
    Sum,
    Minus,
    Times,
    Div,
    Inc,
    // ByteCode
    Assignment,
    Print,
    PrintNl,
    Read,
    Goto,
    GotoF,
    End,
    // Functions
    Return,
    EndProc,
    Era,
    GoSub,
    Param,
    // Arrays
    Ver,
    // Dataframe
    Average,
    Std,
    Mode,
    Variance,
    Min,
    Max,
    Range,
    Corr,
    ReadCSV,
    Plot,
    Histogram,
}

impl Operator {
    pub fn is_goto(self) -> bool {
        matches!(self, Operator::Goto | Operator::GotoF)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:10}", format!("{:?}", self))
    }
}
