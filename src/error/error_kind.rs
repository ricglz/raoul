use core::fmt;

use crate::enums::Types;

#[derive(PartialEq, Eq, Clone)]
pub enum RaoulErrorKind {
    Invalid,
    UndeclaredVar { name: String },
    RedeclaredFunction { name: String },
    RedefinedType { name: String, from: Types, to: Types },
    InvalidCast { from: Types, to: Types },
    MemoryExceded,
}

impl fmt::Debug for RaoulErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RaoulErrorKind::Invalid => unreachable!(),
            RaoulErrorKind::UndeclaredVar { name } => {
                write!(f, "Variable \"{}\" was not declared", name)
            }
            RaoulErrorKind::RedeclaredFunction { name } => {
                write!(f, "Function \"{}\" was already declared before", name)
            }
            RaoulErrorKind::RedefinedType { name, from, to } => {
                write!(
                    f,
                    "\"{}\" was originally defined as {:?} and you're attempting to redefined it as a {:?}",
                    name,
                    from,
                    to,
                )
            }
            RaoulErrorKind::InvalidCast { from, to } => {
                write!(f, "Cannot cast from {:?} to {:?}", from, to)
            }
            RaoulErrorKind::MemoryExceded => {
                write!(f, "Memory was exceded")
            }
        }
    }
}
