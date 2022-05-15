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
