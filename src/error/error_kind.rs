use core::fmt;

use crate::enums::Types;

#[derive(PartialEq, Eq, Clone)]
pub enum RaoulErrorKind {
    Invalid,
    MemoryExceded,
    UndeclaredVar {
        name: String,
    },
    UndeclaredFunction {
        name: String,
    },
    RedeclaredFunction {
        name: String,
    },
    RedefinedType {
        name: String,
        from: Types,
        to: Types,
    },
    InvalidCast {
        from: Types,
        to: Types,
    },
    UnmatchArgsAmount {
        expected: usize,
        given: usize,
    },
    MissingReturn(String),
    NotList(String),
    NotMatrix(String),
    UsePrimitive,
}

impl fmt::Debug for RaoulErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Invalid => unreachable!(),
            Self::UsePrimitive => write!(f, "We can't handle using the complete array"),
            Self::UndeclaredVar { name } => write!(f, "Variable \"{name}\" was not declared"),
            Self::UndeclaredFunction { name } => {
                write!(
                    f,
                    "Function \"{}\" was not declared or does not return a non-void value",
                    name
                )
            }
            Self::RedeclaredFunction { name } => {
                write!(f, "Function \"{name}\" was already declared before")
            }
            Self::RedefinedType { name, from, to } => {
                write!(
                    f,
                    "\"{}\" was originally defined as {:?} and you're attempting to redefined it as a {:?}",
                    name,
                    from,
                    to,
                )
            }
            Self::InvalidCast { from, to } => write!(f, "Cannot cast from {:?} to {:?}", from, to),
            Self::MemoryExceded => write!(f, "Memory was exceded"),
            Self::UnmatchArgsAmount { expected, given } => {
                write!(
                    f,
                    "Wrong args amount: Expected {expected}, but were given {given}"
                )
            }
            Self::MissingReturn(name) => {
                write!(f, "In function {name} not all branches return a value")
            }
            Self::NotList(name) => write!(f, "`{name}` is not a list"),
            Self::NotMatrix(name) => write!(f, "`{name}` is not a matrix"),
        }
    }
}
