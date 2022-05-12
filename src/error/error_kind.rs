use core::fmt;

use crate::enums::Types;

#[derive(PartialEq, Eq, Clone)]
pub enum RaoulErrorKind {
    Invalid,
    MemoryExceded,
    EnteredUnreachable(String),
    UndeclaredVar(String),
    UndeclaredFunction(String),
    UndeclaredFunction2(String),
    RedeclaredFunction(String),
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
    InconsistentSize {
        expected: Option<usize>,
        given: Option<usize>,
    },
    OnlyOneDataframe,
}

impl fmt::Debug for RaoulErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Invalid => unreachable!(),
            Self::UsePrimitive => write!(f, "We can't handle using the complete array"),
            Self::UndeclaredVar(name) => write!(f, "Variable \"{name}\" was not declared"),
            Self::UndeclaredFunction(name) => {
                write!(
                    f,
                    "Function \"{name}\" was not declared or does not return a non-void value",
                )
            }
            Self::UndeclaredFunction2(name) => {
                write!(f, "Function \"{name}\" was not declared")
            }
            Self::RedeclaredFunction(name) => {
                write!(f, "Function \"{name}\" was already declared before")
            }
            Self::RedefinedType { name, from, to } => {
                write!(
                    f,
                    "\"{name}\" was originally defined as {from:?} and you're attempting to redefined it as a {to:?}",
                )
            }
            Self::InvalidCast { from, to } => write!(f, "Cannot cast from {from:?} to {to:?}"),
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
            Self::InconsistentSize { expected, given } => {
                write!(
                    f,
                    "Expecting matrix with second dimension being {} but received {}",
                    expected.unwrap_or(0),
                    given.unwrap_or(0)
                )
            }
            Self::OnlyOneDataframe => write!(f, "Only one dataframe is allowed per program"),
            Self::EnteredUnreachable(kind) => write!(f, "Entered an unreachable statement: {kind}"),
        }
    }
}
