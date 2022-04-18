use std::{cmp::Ordering, collections::HashMap};

use crate::enums::Types;

type AddressCounter = HashMap<Types, i32>;

struct AddressManager {
    base: i32,
    counter: AddressCounter,
}

const THRESHOLD: i32 = 250;

impl AddressManager {
    pub fn new(base: i32) -> Self {
        let counter = HashMap::from([
            (Types::INT, 0),
            (Types::FLOAT, 0),
            (Types::STRING, 0),
            (Types::BOOL, 0),
        ]);
        AddressManager { base, counter }
    }

    fn get_type_base(&self, data_type: Types) -> i32 {
        match data_type {
            Types::INT => 0,
            Types::FLOAT => THRESHOLD,
            Types::STRING => THRESHOLD * 2,
            Types::BOOL => THRESHOLD * 3,
            _ => unreachable!(),
        }
    }

    pub fn get_address(&mut self, data_type: Types) -> Option<i32> {
        let type_counter = self
            .counter
            .get_mut(&data_type)
            .expect(format!("Get address received {:?}", data_type).as_str());
        let type_counter_clone = type_counter.clone();
        *type_counter = *type_counter + 1;
        if type_counter.to_owned().cmp(&THRESHOLD) == Ordering::Greater {
            return None;
        }
        let type_base = self.get_type_base(data_type);
        Some(self.base + type_counter_clone + type_base)
    }
}

#[cfg(test)]
mod tests;
