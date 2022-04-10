use core::fmt;

#[derive(PartialEq)]
pub enum RaoulError {
    Invalid,
    UndeclaredVar { name: String },
    UnitializedVar { name: String },
}

pub type Result<T> = std::result::Result<T, RaoulError>;

impl fmt::Debug for RaoulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RaoulError::Invalid => unreachable!(),
            RaoulError::UndeclaredVar { name } => write!(f, "Variable \"{}\" is undeclared", name),
            RaoulError::UnitializedVar { name } => write!(f, "Variable \"{}\" is undeclared", name),
        }
    }
}
