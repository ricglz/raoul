use std::fmt;

use crate::enums::Operator;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Quadruple {
    pub operator: Operator,
    pub op_1: Option<usize>,
    pub op_2: Option<usize>,
    pub res: Option<usize>,
}

impl Quadruple {
    fn format_address(option: Option<usize>) -> String {
        match option {
            None => "-".to_owned(),
            Some(address) => address.to_string(),
        }
    }

    pub fn new(
        operator: Operator,
        op_1: Option<usize>,
        op_2: Option<usize>,
        res: Option<usize>,
    ) -> Self {
        Quadruple {
            operator,
            op_1,
            op_2,
            res,
        }
    }

    pub fn new_empty(operator: Operator) -> Self {
        Self::new(operator, None, None, None)
    }

    pub fn new_arg(operator: Operator, op_1: usize) -> Self {
        Self::new(operator, Some(op_1), None, None)
    }

    pub fn new_res(operator: Operator, res: usize) -> Self {
        Self::new(operator, None, None, Some(res))
    }

    pub fn new_un(operator: Operator, op_1: usize, res: usize) -> Self {
        Self::new(operator, Some(op_1), None, Some(res))
    }

    pub fn new_args(operator: Operator, op_1: usize, op_2: usize) -> Self {
        Self::new(operator, Some(op_1), Some(op_2), None)
    }

    pub fn new_com(operator: Operator, op_1: usize, op_2: usize, res: usize) -> Self {
        Self::new(operator, Some(op_1), Some(op_2), Some(res))
    }
}

impl fmt::Debug for Quadruple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:5} {:5} {}",
            self.operator,
            Quadruple::format_address(self.op_1),
            Quadruple::format_address(self.op_2),
            Quadruple::format_address(self.res),
        )
    }
}
