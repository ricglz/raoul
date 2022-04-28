use crate::{
    address::{ConstantMemory, Memory},
    dir_func::{function::Function, FunctionTable},
    quadruple::{quadruple::Quadruple, quadruple_manager::QuadrupleManager},
};

pub struct VMContext {
    quad_pos: usize,
    name: String,
    local_memory: Memory,
    temp_memory: Memory,
}

impl VMContext {
    pub fn new(quad_pos: usize, function: Function) -> Self {
        let local_memory = Memory::new(Box::new(function.local_addresses));
        let temp_memory = Memory::new(Box::new(function.temp_addresses));
        let name = function.name.clone();
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
    functions: FunctionTable,
    quad_list: Vec<Quadruple>,
    contexts_stack: Vec<VMContext>,
}

impl VM {
    pub fn new(quad_manager: &QuadrupleManager) -> Self {
        let constant_memory = quad_manager.memory.clone();
        let functions = quad_manager.dir_func.functions.clone();
        let quad_list = quad_manager.quad_list.clone();
        let main_function = functions.get("main").unwrap().clone();
        let initial_context = VMContext::new(0, main_function);
        VM {
            constant_memory,
            functions,
            quad_list,
            contexts_stack: vec![initial_context],
        }
    }
}
