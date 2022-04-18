#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Types {
    INT,
    VOID,
    FLOAT,
    STRING,
    BOOL,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operations {
    NOT,
}
