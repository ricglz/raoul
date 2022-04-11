use core::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum RaoulError {
    Invalid,
    UndeclaredVar { name: String },
    UnitializedVar { name: String },
}

impl RaoulError {
    pub fn is_invalid(&self) -> bool {
        return self.to_owned() == RaoulError::Invalid;
    }
}

pub type Result<T> = std::result::Result<T, RaoulError>;
pub type Results<T> = std::result::Result<T, Vec<RaoulError>>;

impl fmt::Debug for RaoulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RaoulError::Invalid => unreachable!(),
            RaoulError::UndeclaredVar { name } => write!(f, "Variable \"{}\" is undeclared", name),
            RaoulError::UnitializedVar { name } => write!(f, "Variable \"{}\" is undeclared", name),
        }
    }
}
