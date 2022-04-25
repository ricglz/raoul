use std::{cmp::Ordering, collections::HashMap, fmt};

use crate::{dir_func::variable_value::VariableValue, enums::Types};

const THRESHOLD: usize = 250;
const COUNTER_SIZE: usize = 4;
pub const TOTAL_SIZE: usize = THRESHOLD * COUNTER_SIZE;

pub trait Address {
    fn is_temp_address(&self) -> bool;
}

impl Address for usize {
    fn is_temp_address(&self) -> bool {
        TOTAL_SIZE * 2 < *self && *self < TOTAL_SIZE * 3
    }
}

impl Address for Option<usize> {
    fn is_temp_address(&self) -> bool {
        match self {
            Some(address) => address.is_temp_address(),
            None => false,
        }
    }
}

type AddressCounter = HashMap<Types, usize>;

fn get_type_base(data_type: &Types) -> usize {
    match data_type {
        Types::INT => 0,
        Types::FLOAT => THRESHOLD,
        Types::STRING => THRESHOLD * 2,
        Types::BOOL => THRESHOLD * 3,
        _ => unreachable!(),
    }
}

pub trait GenericAddressManager {
    fn get_address(&mut self, data_type: &Types) -> Option<usize>;
}

#[derive(PartialEq, Clone)]
pub struct AddressManager {
    base: usize,
    counter: AddressCounter,
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
}

impl GenericAddressManager for AddressManager {
    fn get_address(&mut self, data_type: &Types) -> Option<usize> {
        let type_counter = self
            .counter
            .get_mut(data_type)
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

impl fmt::Debug for AddressManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let int_counter = self.counter.get(&Types::INT).unwrap();
        let float_counter = self.counter.get(&Types::FLOAT).unwrap();
        let string_counter = self.counter.get(&Types::STRING).unwrap();
        let bool_counter = self.counter.get(&Types::BOOL).unwrap();
        write!(
            f,
            "AddressManager({:?}, {:?}, {:?}, {:?})",
            int_counter, float_counter, string_counter, bool_counter
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct TempAddressManager {
    address_manager: AddressManager,
    released: HashMap<Types, Vec<usize>>,
}

impl TempAddressManager {
    pub fn new() -> Self {
        let released = HashMap::from([
            (Types::INT, Vec::new()),
            (Types::FLOAT, Vec::new()),
            (Types::STRING, Vec::new()),
            (Types::BOOL, Vec::new()),
        ]);
        debug_assert_eq!(released.len(), COUNTER_SIZE);
        TempAddressManager {
            address_manager: AddressManager::new(TOTAL_SIZE * 2),
            released,
        }
    }

    fn address_type(&self, address: usize) -> Types {
        let contextless_address = address - self.address_manager.base;
        let type_determinant = contextless_address / THRESHOLD;
        match type_determinant {
            0 => Types::INT,
            1 => Types::FLOAT,
            2 => Types::STRING,
            3 => Types::BOOL,
            _ => unreachable!(
                "{:?}, {:?}, {:?}",
                address, contextless_address, type_determinant
            ),
        }
    }

    fn type_released_addresses(&mut self, data_type: &Types) -> &mut Vec<usize> {
        self.released.get_mut(data_type).unwrap()
    }

    pub fn release_address(&mut self, address: usize) {
        let data_type = self.address_type(address);
        self.type_released_addresses(&data_type).push(address);
    }
}

impl GenericAddressManager for TempAddressManager {
    fn get_address(&mut self, data_type: &Types) -> Option<usize> {
        self.type_released_addresses(data_type)
            .pop()
            .or(self.address_manager.get_address(data_type))
    }
}

impl fmt::Debug for TempAddressManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TempAddressManager({:#?})", self.released)
    }
}

type Memory = HashMap<Types, Vec<VariableValue>>;

#[derive(PartialEq, Clone)]
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

impl fmt::Debug for ConstantMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConstantMemory({:?})", self.memory)
    }
}

#[cfg(test)]
mod tests;
