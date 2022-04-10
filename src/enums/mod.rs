#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Types {
    INT,
    VOID,
    FLOAT,
    STRING,
    BOOL,
}

#[derive(PartialEq, Debug)]
pub enum Operations {
    NOT,
}
