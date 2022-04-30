use std::{cmp::Ordering, io::stdin};

use crate::{
    address::{ConstantMemory, Memory, TOTAL_SIZE},
    dir_func::{function::Function, variable_value::VariableValue, FunctionTable},
    enums::Operator,
    quadruple::{quadruple::Quadruple, quadruple_manager::QuadrupleManager},
};

pub struct VMContext {
    quad_pos: usize,
    name: String,
    local_memory: Memory,
    temp_memory: Memory,
}

impl VMContext {
    pub fn new(function: Function) -> Self {
        let local_memory = Memory::new(Box::new(function.local_addresses));
        let temp_memory = Memory::new(Box::new(function.temp_addresses));
        let name = function.name.clone();
        let quad_pos = function.first_quad;
        VMContext {
            quad_pos,
            name,
            local_memory,
            temp_memory,
        }
    }
}

pub struct VM {
    constant_memory: ConstantMemory,
    contexts_stack: Vec<VMContext>,
    functions: FunctionTable,
    global_memory: Memory,
    quad_list: Vec<Quadruple>,
}

impl VM {
    pub fn new(quad_manager: &QuadrupleManager) -> Self {
        let constant_memory = quad_manager.memory.clone();
        let functions = quad_manager.dir_func.functions.clone();
        let global_memory =
            Memory::new(Box::new(quad_manager.dir_func.global_fn.addresses.clone()));
        let quad_list = quad_manager.quad_list.clone();
        let main_function = functions.get("main").unwrap().clone();
        let initial_context = VMContext::new(main_function);
        VM {
            constant_memory,
            contexts_stack: vec![initial_context],
            functions,
            global_memory,
            quad_list,
        }
    }

    #[inline]
    fn current_context(&self) -> &VMContext {
        self.contexts_stack.last().unwrap()
    }

    #[inline]
    fn local_addresses(&self) -> Memory {
        self.current_context().local_memory.clone()
    }

    #[inline]
    fn temp_addresses(&self) -> Memory {
        self.current_context().temp_memory.clone()
    }

    #[inline]
    fn current_context_mut(&mut self) -> &mut VMContext {
        self.contexts_stack.last_mut().unwrap()
    }

    #[inline]
    fn local_addresses_mut(&mut self) -> &mut Memory {
        &mut self.current_context_mut().local_memory
    }

    #[inline]
    fn temp_addresses_mut(&mut self) -> &mut Memory {
        &mut self.current_context_mut().temp_memory
    }

    #[inline]
    fn update_quad_pos(&mut self, quad_pos: usize) {
        self.current_context_mut().quad_pos = quad_pos;
    }

    fn get_current_quad(&self) -> Quadruple {
        let quad_pos = self.current_context().quad_pos;
        self.quad_list.get(quad_pos).unwrap().clone()
    }

    fn get_value(&self, address: usize) -> VariableValue {
        match address / TOTAL_SIZE {
            0 => self.global_memory.get(address).unwrap(),
            1 => self.local_addresses().get(address).unwrap(),
            2 => self.temp_addresses().get(address).unwrap(),
            3 => self.constant_memory.get(address),
            _ => unreachable!(),
        }
    }

    fn write_value(&mut self, value: VariableValue, address: usize) {
        let memory = match address / TOTAL_SIZE {
            0 => &mut self.global_memory,
            1 => self.local_addresses_mut(),
            2 => self.temp_addresses_mut(),
            _ => unreachable!(),
        };
        memory.write(address, value);
    }

    fn process_assign(&mut self) {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap());
        self.write_value(value, quad.res.unwrap());
    }

    fn process_print(&mut self) {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap());
        print!("{value:?} ")
    }

    fn create_value_from_stdin(&self) -> VariableValue {
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        VariableValue::String(line)
    }

    fn process_read(&mut self) {
        let quad = self.get_current_quad();
        let value = self.create_value_from_stdin();
        self.write_value(value, quad.res.unwrap());
    }

    fn unary_operation<F>(&mut self, f: F)
    where
        F: FnOnce(VariableValue) -> VariableValue,
    {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap());
        let value = f(a);
        self.write_value(value, quad.res.unwrap());
    }

    fn binary_operation<F>(&mut self, f: F)
    where
        F: FnOnce(VariableValue, VariableValue) -> VariableValue,
    {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap());
        let b = self.get_value(quad.op_2.unwrap());
        let value = f(a, b);
        self.write_value(value, quad.res.unwrap());
    }

    fn comparison(&mut self) {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap());
        let b = self.get_value(quad.op_2.unwrap());
        let ord = a.partial_cmp(&b);
        let res = match ord {
            None => false,
            Some(ord) => match quad.operator {
                Operator::Lt => ord == Ordering::Less,
                Operator::Lte => ord != Ordering::Greater,
                Operator::Gt => ord == Ordering::Greater,
                Operator::Gte => ord != Ordering::Less,
                Operator::Eq => ord == Ordering::Equal,
                Operator::Ne => ord != Ordering::Equal,
                _ => unreachable!(),
            },
        };
        let value = VariableValue::Bool(res);
        self.write_value(value, quad.res.unwrap());
    }

    pub fn run(&mut self) {
        loop {
            let mut quad_pos = self.current_context().quad_pos;
            let quad = self.quad_list.get(quad_pos).unwrap();
            match quad.operator {
                Operator::End => break,
                Operator::Goto => quad_pos = quad.res.unwrap() - 1,
                Operator::Assignment => self.process_assign(),
                Operator::Print => self.process_print(),
                Operator::PrintNl => println!(""),
                Operator::Read => self.process_read(),
                Operator::Or => self.binary_operation(|a, b| a | b),
                Operator::And => self.binary_operation(|a, b| a & b),
                Operator::Sum => self.binary_operation(|a, b| a + b),
                Operator::Minus => self.binary_operation(|a, b| a - b),
                Operator::Times => self.binary_operation(|a, b| a * b),
                Operator::Div => self.binary_operation(|a, b| a / b),
                Operator::Lt
                | Operator::Lte
                | Operator::Gt
                | Operator::Gte
                | Operator::Eq
                | Operator::Ne => self.comparison(),
                Operator::Not => self.unary_operation(|a| !a),
                kind => todo!("{:?}", kind),
            }
            self.update_quad_pos(quad_pos + 1);
        }
    }
}
