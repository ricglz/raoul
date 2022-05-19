use std::{cmp::Ordering, collections::HashMap, fmt};

use crate::{
    dir_func::{variable::Dimensions, variable_value::VariableValue},
    enums::Types,
    vm::VMResult,
};

const THRESHOLD: usize = 250;
const COUNTER_SIZE: usize = 4;
pub const TOTAL_SIZE: usize = THRESHOLD * COUNTER_SIZE;

pub trait Address {
    fn is_temp_address(&self) -> bool;
    fn is_pointer_address(&self) -> bool;
}

impl Address for usize {
    fn is_temp_address(&self) -> bool {
        TOTAL_SIZE * 2 < *self && *self < TOTAL_SIZE * 3
    }

    fn is_pointer_address(&self) -> bool {
        *self >= TOTAL_SIZE * 4
    }
}

impl Address for Option<usize> {
    fn is_temp_address(&self) -> bool {
        match self {
            Some(address) => address.is_temp_address(),
            None => false,
        }
    }

    fn is_pointer_address(&self) -> bool {
        match self {
            Some(address) => address.is_pointer_address(),
            None => false,
        }
    }
}

type AddressCounter = HashMap<Types, usize>;

fn get_type_base(data_type: Types) -> usize {
    match data_type {
        Types::Int => 0,
        Types::Float => THRESHOLD,
        Types::String => THRESHOLD * 2,
        Types::Bool => THRESHOLD * 3,
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
    fn get_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize>;
    fn size(&self) -> usize;
    fn get_base(&self) -> usize;
}

#[derive(PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AddressManager {
    base: usize,
    counter: AddressCounter,
}

impl AddressManager {
    pub fn new(base: usize) -> Self {
        let counter = HashMap::from([
            (Types::Int, 0),
            (Types::Float, 0),
            (Types::String, 0),
            (Types::Bool, 0),
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
    fn get_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize> {
        if data_type == Types::Dataframe {
            return Some(10_000);
        }
        let type_counter = self
            .counter
            .get_mut(&data_type)
            .unwrap_or_else(|| panic!("{:?}", data_type));
        let prev = *type_counter;
        let amount = get_amount(dimensions);
        let new_counter = prev + amount;
        if new_counter > THRESHOLD {
            return None;
        }
        *type_counter = new_counter;
        let type_base = get_type_base(data_type);
        Some(self.base + prev + type_base)
    }
    #[inline]
    fn size(&self) -> usize {
        self.counter
            .iter()
            .map(|v| v.1)
            .copied()
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
        let int_counter = self.counter.get(&Types::Int).unwrap();
        let float_counter = self.counter.get(&Types::Float).unwrap();
        let string_counter = self.counter.get(&Types::String).unwrap();
        let bool_counter = self.counter.get(&Types::Bool).unwrap();
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
            (Types::Int, Vec::new()),
            (Types::Float, Vec::new()),
            (Types::String, Vec::new()),
            (Types::Bool, Vec::new()),
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
            0 => Types::Int,
            1 => Types::Float,
            2 => Types::String,
            3 => Types::Bool,
            _ => unreachable!(
                "{:?}, {:?}, {:?}",
                address, contextless_address, type_determinant
            ),
        }
    }

    #[inline]
    fn type_released_addresses(&mut self, data_type: &Types) -> &mut Vec<usize> {
        self.released.get_mut(data_type).unwrap()
    }

    pub fn release_address(&mut self, address: usize) {
        let data_type = self.address_type(address);
        self.type_released_addresses(&data_type).push(address);
    }
}

impl Default for TempAddressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericAddressManager for TempAddressManager {
    #[inline]
    fn get_address_counter(&self) -> AddressCounter {
        self.address_manager.get_address_counter()
    }
    #[inline]
    fn get_address(&mut self, data_type: Types, dimensions: Dimensions) -> Option<usize> {
        self.type_released_addresses(&data_type)
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
        0 => Types::Int,
        1 => Types::Float,
        2 => Types::String,
        3 => Types::Bool,
        _ => unreachable!(),
    };
    (contextless_address, type_determinant, address_type)
}

impl ConstantMemory {
    pub fn new() -> Self {
        let memory = HashMap::from([
            (Types::Int, vec![]),
            (Types::Float, vec![]),
            (Types::String, vec![]),
            (Types::Bool, vec![]),
        ]);
        ConstantMemory {
            base: TOTAL_SIZE * 3,
            memory,
        }
    }

    fn get_address(&mut self, data_type: Types, value: VariableValue) -> Option<usize> {
        let type_memory = self
            .memory
            .get_mut(&data_type)
            .unwrap_or_else(|| panic!("Get address received {:?}", data_type));
        let type_base = get_type_base(data_type);
        match type_memory.iter_mut().position(|x| *x == value) {
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
        let address = self.get_address(data_type, value)?;
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
            .clone()
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
    // TODO: Maybe fix it
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(manager: Box<dyn GenericAddressManager>) -> Self {
        let counter = manager.get_address_counter();
        let base = manager.get_base();
        let int_pointer: usize = 0;
        let float_pointer = int_pointer + counter.get(&Types::Int).unwrap();
        let string_pointer = float_pointer + counter.get(&Types::Float).unwrap();
        let bool_pointer = string_pointer + counter.get(&Types::String).unwrap();
        let total_size = bool_pointer + counter.get(&Types::Bool).unwrap();
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
            Types::Int => self.int_pointer,
            Types::Float => self.float_pointer,
            Types::String => self.string_pointer,
            Types::Bool => self.bool_pointer,
            data_type => unreachable!("{:?}", data_type),
        };
        (type_index + pointer, address_type)
    }

    pub fn get(&self, address: usize) -> Option<VariableValue> {
        let index = self.get_index(address).0;
        self.space.get(index).unwrap().clone()
    }

    pub fn write(&mut self, address: usize, uncast: &VariableValue) -> VMResult<()> {
        let (index, address_type) = self.get_index(address);
        let value = uncast.cast_to(address_type)?;
        *self.space.get_mut(index).unwrap() = Some(value);
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PointerMemory {
    counter: usize,
    pointers: HashMap<usize, usize>,
}

impl PointerMemory {
    pub fn new() -> Self {
        Self {
            counter: TOTAL_SIZE * 4,
            pointers: HashMap::new(),
        }
    }

    pub fn get_pointer(&mut self) -> usize {
        let prev_counter = self.counter;
        self.counter += 1;
        prev_counter
    }

    pub fn write(&mut self, address: usize, var: VariableValue) {
        self.pointers.insert(address, var.into());
    }

    pub fn get(&self, address: usize) -> usize {
        self.pointers.get(&address).unwrap().to_owned()
    }
}

#[cfg(test)]
mod tests;
