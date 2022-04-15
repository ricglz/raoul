use core::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum RaoulErrorKind {
    Invalid,
    UndeclaredVar { name: String },
    UnitializedVar { name: String },
}

impl fmt::Debug for RaoulErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RaoulErrorKind::Invalid => unreachable!(),
            RaoulErrorKind::UndeclaredVar { name } => {
                write!(f, "Variable \"{}\" was not declared", name)
            }
            RaoulErrorKind::UnitializedVar { name } => {
                write!(f, "Variable \"{}\" was not initialized", name)
            }
        }
    }
}
