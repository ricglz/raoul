#[derive(Debug, PartialEq)]
pub enum RaoulError {
    Invalid,
    UndeclaredId { name: String },
}

pub type Result<T> = std::result::Result<T, RaoulError>;
