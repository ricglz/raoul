use std::{cmp::Ordering, collections::HashMap, fmt};

use crate::{
    dir_func::{variable::Dimensions, variable_value::VariableValue},
    enums::Types,
};

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

fn get_amount(dimensions: Dimensions) -> usize {
    let dim_1 = dimensions.0.unwrap_or(0);
    let dim_2 = dimensions.1.unwrap_or(1);
    match dim_1 * dim_2 {
        0 => 1,
        v => v,
    }
}

pub trait GenericAddressManager {
    fn get_address_counter(&self) -> AddressCounter;
    fn get_address(&mut self, data_type: &Types, dimensions: Dimensions) -> Option<usize>;
    fn size(&self) -> usize;
    fn get_base(&self) -> usize;
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
    #[inline]
    fn get_address_counter(&self) -> AddressCounter {
        self.counter.clone()
    }
    fn get_address(&mut self, data_type: &Types, dimensions: Dimensions) -> Option<usize> {
        let type_counter = self
            .counter
            .get_mut(data_type)
            .expect(format!("Get address received {:?}", data_type).as_str());
        let prev = type_counter.clone();
        let amount = get_amount(dimensions);
        let new_counter = prev + amount;
        if new_counter > THRESHOLD {
            return None;
        }
        *type_counter = new_counter;
        let type_base = get_type_base(&data_type);
        Some(self.base + prev + type_base)
    }
    #[inline]
    fn size(&self) -> usize {
        self.counter
            .to_owned()
            .into_iter()
            .map(|v| v.1)
            .reduce(|a, v| a + v)
            .unwrap_or(0)
    }
    #[inline]
    fn get_base(&self) -> usize {
        self.base
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
    #[inline]
    fn get_address_counter(&self) -> AddressCounter {
        self.address_manager.get_address_counter()
    }
    #[inline]
    fn get_address(&mut self, data_type: &Types, dimensions: Dimensions) -> Option<usize> {
        self.type_released_addresses(data_type)
            .pop()
            .or_else(|| self.address_manager.get_address(data_type, dimensions))
    }
    #[inline]
    fn size(&self) -> usize {
        self.address_manager.size()
    }
    #[inline]
    fn get_base(&self) -> usize {
        self.address_manager.base
    }
}

impl fmt::Debug for TempAddressManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TempAddressManager({:#?})", self.released)
    }
}

#[derive(PartialEq, Clone)]
pub struct ConstantMemory {
    base: usize,
    memory: HashMap<Types, Vec<VariableValue>>,
}

fn get_address_info(address: usize, base: usize) -> (usize, usize, Types) {
    let contextless_address = address - base;
    let type_determinant = contextless_address / THRESHOLD;
    let address_type = match type_determinant {
        0 => Types::INT,
        1 => Types::FLOAT,
        2 => Types::STRING,
        3 => Types::BOOL,
        _ => unreachable!(),
    };
    (contextless_address, type_determinant, address_type)
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

    pub fn get(&self, address: usize) -> VariableValue {
        let (contextless_address, type_determinant, address_type) =
            get_address_info(address, self.base);
        self.memory
            .get(&address_type)
            .unwrap()
            .get(contextless_address - type_determinant * THRESHOLD)
            .unwrap()
            .to_owned()
    }
}

impl fmt::Debug for ConstantMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConstantMemory({:?})", self.memory)
    }
}

#[derive(Clone, Debug)]
pub struct Memory {
    base: usize,
    int_pointer: usize,
    float_pointer: usize,
    string_pointer: usize,
    bool_pointer: usize,
    space: Vec<Option<VariableValue>>,
}

impl Memory {
    pub fn new(manager: Box<dyn GenericAddressManager>) -> Self {
        let counter = manager.get_address_counter();
        let base = manager.get_base();
        let int_pointer: usize = 0;
        let float_pointer = int_pointer + counter.get(&Types::INT).unwrap();
        let string_pointer = float_pointer + counter.get(&Types::FLOAT).unwrap();
        let bool_pointer = string_pointer + counter.get(&Types::STRING).unwrap();
        let total_size = bool_pointer + counter.get(&Types::BOOL).unwrap();
        let space = vec![None; total_size];
        Memory {
            base,
            int_pointer,
            float_pointer,
            string_pointer,
            bool_pointer,
            space,
        }
    }

    fn get_index(&self, address: usize) -> (usize, Types) {
        let (contextless_address, _, address_type) = get_address_info(address, self.base);
        let type_index = contextless_address % THRESHOLD;
        let pointer = match address_type {
            Types::INT => self.int_pointer,
            Types::FLOAT => self.float_pointer,
            Types::STRING => self.string_pointer,
            Types::BOOL => self.bool_pointer,
            data_type => unreachable!("{:?}", data_type),
        };
        (type_index + pointer, address_type)
    }

    pub fn get(&self, address: usize) -> Option<VariableValue> {
        let index = self.get_index(address).0;
        self.space.get(index).unwrap().to_owned()
    }

    pub fn write(&mut self, address: usize, uncast: VariableValue) {
        let (index, address_type) = self.get_index(address);
        let value = uncast.cast_to(address_type);
        *self.space.get_mut(index).unwrap() = Some(value);
    }
}

#[cfg(test)]
mod tests;
