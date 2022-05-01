use std::{cmp::Ordering, collections::HashMap, io::stdin, process::exit, vec};

use crate::{
    address::{ConstantMemory, Memory, TOTAL_SIZE},
    dir_func::{
        function::{Function, VariablesTable},
        variable_value::VariableValue,
    },
    enums::Operator,
    quadruple::{quadruple::Quadruple, quadruple_manager::QuadrupleManager},
};

#[derive(Clone, Debug)]
pub struct VMContext {
    args: Vec<usize>,
    local_memory: Memory,
    name: String,
    quad_pos: usize,
    size: usize,
    temp_memory: Memory,
}

impl VMContext {
    pub fn new(function: Function) -> Self {
        let size = function.size();
        let local_memory = Memory::new(Box::new(function.local_addresses));
        let temp_memory = Memory::new(Box::new(function.temp_addresses));
        let quad_pos = function.first_quad;
        let name = function.name;
        let args = function.args.into_iter().map(|v| v.address).collect();
        Self {
            args,
            local_memory,
            name,
            quad_pos,
            size,
            temp_memory,
        }
    }
}

#[derive(Debug)]
pub struct VM {
    call_stack: Vec<VMContext>,
    constant_memory: ConstantMemory,
    contexts_stack: Vec<VMContext>,
    functions: HashMap<usize, Function>,
    global_memory: Memory,
    global_variables: VariablesTable,
    quad_list: Vec<Quadruple>,
    stack_size: usize,
}

const STACK_SIZE_CAP: usize = 1024;

impl VM {
    pub fn new(quad_manager: &QuadrupleManager) -> Self {
        let constant_memory = quad_manager.memory.clone();
        let functions = quad_manager.dir_func.functions.clone();
        let global_fn = quad_manager.dir_func.global_fn.clone();
        let global_memory = Memory::new(Box::new(global_fn.addresses));
        let global_variables = global_fn.variables;
        let quad_list = quad_manager.quad_list.clone();
        let main_function = functions.get("main").unwrap().clone();
        let stack_size = main_function.size();
        let initial_context = VMContext::new(main_function);
        VM {
            call_stack: vec![],
            constant_memory,
            contexts_stack: vec![initial_context],
            functions: functions
                .into_iter()
                .map(|(_, function)| (function.first_quad.clone(), function))
                .collect(),
            global_memory,
            global_variables,
            quad_list,
            stack_size,
        }
    }

    fn add_call_stack(&mut self, function: Function) {
        self.stack_size += function.size();
        if self.stack_size > STACK_SIZE_CAP {
            println!("Stack overflow!");
            exit(1);
        }
        self.call_stack.push(VMContext::new(function));
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

    #[inline]
    fn get_function(&self, first_quad: usize) -> Function {
        self.functions.get(&first_quad).unwrap().clone()
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

    fn conditional_goto(&mut self, approved: bool) -> usize {
        let quad = self.get_current_quad();
        let cond = self.get_value(quad.op_1.unwrap());
        let quad_pos = self.current_context().quad_pos;
        match bool::from(cond) == approved {
            true => quad.res.unwrap() - 1,
            false => quad_pos,
        }
    }

    fn process_inc(&mut self) {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.res.unwrap());
        let value = a + VariableValue::Integer(1);
        self.write_value(value, quad.res.unwrap());
    }

    fn process_era(&mut self) {
        let quad = self.get_current_quad();
        let first_quad = quad.op_2.unwrap();
        let function = self.get_function(first_quad);
        self.add_call_stack(function);
    }

    fn process_go_sub(&mut self) {
        let quad_pos = self.current_context().quad_pos;
        self.update_quad_pos(quad_pos + 1);
        let call = self.call_stack.pop().unwrap();
        self.contexts_stack.push(call);
    }

    fn process_end_proc(&mut self) {
        let context = self.contexts_stack.pop().unwrap();
        self.stack_size -= context.size;
    }

    #[inline]
    fn current_call(&self) -> VMContext {
        self.call_stack.last().unwrap().clone()
    }

    #[inline]
    fn current_call_mut(&mut self) -> &mut VMContext {
        self.call_stack.last_mut().unwrap()
    }

    fn write_value_param(&mut self, value: VariableValue, address: usize) {
        let memory = match address / TOTAL_SIZE {
            1 => &mut self.current_call_mut().local_memory,
            val => unreachable!("{val}"),
        };
        memory.write(address, value);
    }

    fn process_param(&mut self) {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap());
        let index = quad.res.unwrap();
        let address = self.current_call().args.get(index).unwrap().clone();
        self.write_value_param(value, address);
    }

    fn get_context_global_address(&self) -> usize {
        let name = &self.current_context().name;
        self.global_variables.get(name).unwrap().address
    }

    fn process_return(&mut self) {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap());
        let address = self.get_context_global_address();
        self.write_value(value, address);
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
                Operator::GotoF => quad_pos = self.conditional_goto(false),
                Operator::Inc => self.process_inc(),
                Operator::Era => self.process_era(),
                Operator::GoSub => {
                    self.process_go_sub();
                    continue;
                }
                Operator::EndProc => {
                    self.process_end_proc();
                    continue;
                }
                Operator::Param => self.process_param(),
                Operator::Return => self.process_return(),
                // kind => todo!("{:?}", kind),
            }
            self.update_quad_pos(quad_pos + 1);
        }
    }
}
