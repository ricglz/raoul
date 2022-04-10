use std::collections::HashMap;

use crate::enums::Types;

#[derive(Clone, Copy, PartialEq, Debug)]
enum VariableValue {
    Integer(i64),
}

impl From<VariableValue> for Types {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::INT,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Variable {
    data_type: Types,
    name: String,
    value: VariableValue,
}

type VariablesTable<'a> = &'a mut HashMap<String, Variable>;

#[derive(PartialEq, Debug)]
struct Function<'a> {
    name: String,
    return_type: Types,
    variables: VariablesTable<'a>,
}
