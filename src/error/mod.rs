#[derive(Debug, PartialEq)]
pub enum RaoulError {
    Invalid,
    UndeclaredVar { name: String },
    UnitializedVar { name: String },
}

pub type Result<T> = std::result::Result<T, RaoulError>;
