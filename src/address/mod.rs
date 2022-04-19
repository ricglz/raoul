use std::{cmp::Ordering, collections::HashMap};

use crate::{dir_func::variable_value::VariableValue, enums::Types};

type AddressCounter = HashMap<Types, usize>;

#[derive(PartialEq, Clone, Debug)]
pub struct AddressManager {
    base: usize,
    counter: AddressCounter,
}

const THRESHOLD: usize = 250;
const COUNTER_SIZE: usize = 4;
pub const TOTAL_SIZE: usize = THRESHOLD * COUNTER_SIZE;

fn get_type_base(data_type: &Types) -> usize {
    match data_type {
        Types::INT => 0,
        Types::FLOAT => THRESHOLD,
        Types::STRING => THRESHOLD * 2,
        Types::BOOL => THRESHOLD * 3,
        _ => unreachable!(),
    }
}

impl AddressManager {
    pub fn new(base: usize) -> Self {
        let counter = HashMap::from([
            (Types::INT, 0),
            (Types::FLOAT, 0),
            (Types::STRING, 0),
            (Types::BOOL, 0),
        ]);
        debug_assert_eq!(counter.len(), COUNTER_SIZE);
        AddressManager { base, counter }
    }

    pub fn get_address(&mut self, data_type: Types) -> Option<usize> {
        let type_counter = self
            .counter
            .get_mut(&data_type)
            .expect(format!("Get address received {:?}", data_type).as_str());
        let type_counter_clone = type_counter.clone();
        if type_counter.to_owned().cmp(&THRESHOLD) == Ordering::Equal {
            return None;
        }
        *type_counter = *type_counter + 1;
        let type_base = get_type_base(&data_type);
        Some(self.base + type_counter_clone + type_base)
    }
}

type Memory = HashMap<Types, Vec<VariableValue>>;

#[derive(PartialEq, Clone, Debug)]
pub struct ConstantMemory {
    base: usize,
    memory: Memory,
}

impl ConstantMemory {
    pub fn new() -> Self {
        let memory = HashMap::from([
            (Types::INT, vec![]),
            (Types::FLOAT, vec![]),
            (Types::STRING, vec![]),
            (Types::BOOL, vec![]),
        ]);
        ConstantMemory {
            base: TOTAL_SIZE * 3,
            memory,
        }
    }

    fn get_address(&mut self, data_type: &Types, value: VariableValue) -> Option<usize> {
        let type_memory = self
            .memory
            .get_mut(data_type)
            .expect(format!("Get address received {:?}", data_type).as_str());
        let type_base = get_type_base(data_type);
        match type_memory.into_iter().position(|x| x.to_owned() == value) {
            None => {
                if type_memory.len().to_owned().cmp(&THRESHOLD) == Ordering::Equal {
                    return None;
                }
                let position = type_memory.len();
                type_memory.push(value);
                Some(self.base + position + type_base)
            }
            Some(position) => Some(self.base + position + type_base),
        }
    }

    pub fn add(&mut self, value: VariableValue) -> Option<(usize, Types)> {
        let data_type = Types::from(&value);
        let address = self.get_address(&data_type, value)?;
        Some((address, data_type))
    }

    fn get(&self, address: usize) -> Option<VariableValue> {
        let contextless_address = address - self.base;
        let type_determinant = contextless_address / THRESHOLD;
        let address_type = match type_determinant {
            0 => Types::INT,
            1 => Types::FLOAT,
            2 => Types::STRING,
            3 => Types::BOOL,
            _ => unreachable!(),
        };
        let type_memory = self.memory.get(&address_type).unwrap();
        Some(
            type_memory
                .get(contextless_address - type_determinant * THRESHOLD)
                .unwrap()
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests;