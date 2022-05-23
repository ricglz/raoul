mod gui;

use std::{cmp::Ordering, collections::HashMap};

use polars::{
    datatypes::{AnyValue, DataType},
    io::SerReader,
    prelude::{DataFrame, Series},
};
use polars_lazy::prelude::{col, pearson_corr, IntoLazy};

use crate::{
    address::{Address, ConstantMemory, Memory, PointerMemory, TOTAL_SIZE},
    dir_func::{function::Function, variable_value::VariableValue},
    enums::Operator,
    quadruple::{quadruple::Quadruple, quadruple_manager::QuadrupleManager},
};

use self::gui::App;

#[derive(Clone, Debug)]
pub struct VMContext {
    address: usize,
    args: Vec<usize>,
    local_memory: Memory,
    quad_pos: usize,
    size: usize,
    temp_memory: Memory,
}

impl VMContext {
    pub fn new(function: Function) -> Self {
        let size = function.size();
        let address = function.address;
        let local_memory = Memory::new(&function.local_addresses);
        let temp_memory = Memory::new(&function.temp_addresses);
        let quad_pos = function.first_quad;
        let args = function.args.into_iter().map(|v| v.address).collect();
        Self {
            address,
            args,
            local_memory,
            quad_pos,
            size,
            temp_memory,
        }
    }
}

pub type VMResult<T> = std::result::Result<T, &'static str>;

#[derive(Debug)]
pub struct VM {
    call_stack: Vec<VMContext>,
    constant_memory: ConstantMemory,
    contexts_stack: Vec<VMContext>,
    debug: bool,
    functions: HashMap<usize, Function>,
    global_memory: Memory,
    pointer_memory: PointerMemory,
    pub messages: Vec<String>,
    quad_list: Vec<Quadruple>,
    stack_size: usize,
    data_frame: Option<DataFrame>,
}

const STACK_SIZE_CAP: usize = 1024;

fn cast_to_f64(v: &AnyValue) -> f64 {
    match v {
        AnyValue::Float64(v) => *v,
        AnyValue::Float32(v) => (*v).try_into().unwrap(),
        AnyValue::Int64(v) => v.to_string().parse::<f64>().unwrap(),
        _ => unreachable!(),
    }
}

fn safe_address(value: &Option<VariableValue>) -> VMResult<VariableValue> {
    match value {
        Some(v) => Ok(v.clone()),
        None => Err("Found initialized value"),
    }
}

#[inline]
fn min(c: &Series) -> f64 {
    c.min().unwrap_or(0.0)
}

#[inline]
fn max(c: &Series) -> f64 {
    c.max().unwrap_or(0.0)
}

impl VM {
    pub fn new(quad_manager: &QuadrupleManager, debug: bool) -> Self {
        let constant_memory = quad_manager.memory.clone();
        let functions = quad_manager.dir_func.functions.clone();
        let global_fn = quad_manager.dir_func.global_fn.clone();
        let pointer_memory = quad_manager.pointer_memory.clone();
        let global_memory = Memory::new(&global_fn.addresses);
        let quad_list = quad_manager.quad_list.clone();
        let main_function = functions.get("main").unwrap().clone();
        let stack_size = main_function.size();
        let initial_context = VMContext::new(main_function);
        Self {
            call_stack: vec![],
            constant_memory,
            contexts_stack: vec![initial_context],
            data_frame: None,
            debug,
            functions: functions
                .into_iter()
                .map(|(_, function)| (function.first_quad, function))
                .collect(),
            global_memory,
            messages: Vec::new(),
            pointer_memory,
            quad_list,
            stack_size,
        }
    }

    fn add_call_stack(&mut self, function: Function) -> VMResult<()> {
        self.stack_size += function.size();
        if self.stack_size > STACK_SIZE_CAP || self.contexts_stack.len() == STACK_SIZE_CAP {
            return Err("Stack overflow!");
        }
        self.call_stack.push(VMContext::new(function));
        Ok(())
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
        *self.quad_list.get(quad_pos).unwrap()
    }

    fn get_value(&self, address: usize) -> VMResult<VariableValue> {
        match address / TOTAL_SIZE {
            0 => safe_address(self.global_memory.get(address)),
            1 => safe_address(self.local_addresses().get(address)),
            2 => safe_address(self.temp_addresses().get(address)),
            3 => Ok(self.constant_memory.get(address).clone()),
            _ => {
                let address = self.pointer_memory.get(address);
                self.get_value(address)
            }
        }
    }

    fn write_value(&mut self, value: VariableValue, address: usize) -> VMResult<()> {
        let determinant = address / TOTAL_SIZE;
        if determinant >= 4 {
            self.pointer_memory.write(address, value);
            return Ok(());
        }
        let memory = match determinant {
            0 => &mut self.global_memory,
            1 => self.local_addresses_mut(),
            2 => self.temp_addresses_mut(),
            _ => unreachable!(),
        };
        memory.write(address, &value)
    }

    fn process_assign(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap())?;
        let mut assignee = quad.res.unwrap();
        if assignee.is_pointer_address() {
            assignee = self.pointer_memory.get(assignee);
        }
        self.write_value(value, assignee)
    }

    fn print_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
        let separator = if message.contains('\n') { "" } else { " " };
        print!("{message}{separator}");
    }

    fn process_print(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap())?;
        self.print_message(&format!("{value:?}"));
        Ok(())
    }

    fn process_read(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let value = VariableValue::from_stdin();
        self.write_value(value, quad.res.unwrap())
    }

    fn unary_operation<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(VariableValue) -> VariableValue,
    {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap())?;
        let value = f(a);
        self.write_value(value, quad.res.unwrap())
    }

    fn binary_operation<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(VariableValue, VariableValue) -> VMResult<VariableValue>,
    {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap())?;
        let b = self.get_value(quad.op_2.unwrap())?;
        let value = f(a, b)?;
        self.write_value(value, quad.res.unwrap())
    }

    fn comparison(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.op_1.unwrap())?;
        let b = self.get_value(quad.op_2.unwrap())?;
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
        self.write_value(value, quad.res.unwrap())
    }

    fn conditional_goto(&mut self, approved: bool) -> VMResult<usize> {
        let quad = self.get_current_quad();
        let cond = self.get_value(quad.op_1.unwrap())?;
        let quad_pos = self.current_context().quad_pos;
        if bool::from(cond) == approved {
            return Ok(quad.res.unwrap() - 1);
        }
        Ok(quad_pos)
    }

    fn process_inc(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let a = self.get_value(quad.res.unwrap())?;
        let value = a.increase()?;
        self.write_value(value, quad.res.unwrap())
    }

    fn process_era(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let first_quad = quad.op_2.unwrap();
        let function = self.get_function(first_quad);
        self.add_call_stack(function)
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

    fn write_value_param(&mut self, value: &VariableValue, address: usize) -> VMResult<()> {
        let memory = match address / TOTAL_SIZE {
            1 => &mut self.current_call_mut().local_memory,
            val => unreachable!("{val}"),
        };
        memory.write(address, value)
    }

    fn process_param(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap())?;
        let index = quad.res.unwrap();
        let address = *self.current_call().args.get(index).unwrap();
        self.write_value_param(&value, address)
    }

    #[inline]
    fn get_context_global_address(&self) -> usize {
        self.current_context().address
    }

    fn process_return(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let value = self.get_value(quad.op_1.unwrap())?;
        let address = self.get_context_global_address();
        self.write_value(value, address)?;
        self.process_end_proc();
        Ok(())
    }

    fn process_ver(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let index = self.get_value(quad.op_1.unwrap())?;
        let limit = self.get_value(quad.op_2.unwrap())?;
        if limit <= index || VariableValue::Integer(0) > index {
            return Err("Index out of range for array");
        }
        Ok(())
    }

    fn read_csv(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let filename = String::from(self.get_value(quad.op_1.unwrap())?);
        let res = polars::io::csv::CsvReader::from_path(&filename);
        if res.is_err() {
            return Err("Could not read the file");
        }
        let res = res.unwrap().has_header(true).finish();
        if res.is_err() {
            return Err("File is not a valid CSV");
        }
        self.data_frame = Some(res.unwrap());
        Ok(())
    }

    fn get_dataframe(&self) -> VMResult<&DataFrame> {
        if self.data_frame.is_none() {
            return Err("No data frame was created. You need to create one using `read_csv`");
        }
        let data_frame = self.data_frame.as_ref().unwrap();
        Ok(data_frame)
    }

    fn pure_df_operation(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let data_frame = self.get_dataframe()?;
        let value = match quad.operator {
            Operator::Rows => data_frame.shape().0,
            Operator::Columns => data_frame.shape().1,
            _ => unreachable!(),
        }
        .into();
        self.write_value(value, quad.res.unwrap())
    }

    fn unary_df_operation<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(&Series) -> f64,
    {
        let quad = self.get_current_quad();
        let column_name = String::from(self.get_value(quad.op_1.unwrap())?);
        let data_frame = self.get_dataframe()?;
        let column = data_frame.column(&column_name);
        if column.is_err() {
            return Err("Dataframe key not found in file");
        }
        let value = f(column.unwrap()).into();
        self.write_value(value, quad.res.unwrap())
    }

    fn correlation(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let data_frame = self.get_dataframe()?;
        let col_1_name = String::from(self.get_value(quad.op_1.unwrap())?);
        let col_2_name = String::from(self.get_value(quad.op_2.unwrap())?);
        let temp = data_frame
            .clone()
            .lazy()
            .select([pearson_corr(
                col(&col_1_name).cast(DataType::Float64),
                col(&col_2_name).cast(DataType::Float64),
            )
            .alias("correlation")])
            .collect()
            .unwrap();
        let value = cast_to_f64(&temp.column("correlation").unwrap().get(0)).into();
        self.write_value(value, quad.res.unwrap())
    }

    fn plot(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let data_frame = self.get_dataframe()?;
        let col_1_name = String::from(self.get_value(quad.op_1.unwrap())?);
        let col_2_name = String::from(self.get_value(quad.op_2.unwrap())?);
        let temp = data_frame
            .clone()
            .lazy()
            .select([
                col(&col_1_name).cast(DataType::Float64).alias("column_1"),
                col(&col_2_name).cast(DataType::Float64).alias("column_2"),
            ])
            .collect()
            .unwrap();
        let app = App::new_plot(temp);
        eframe::run_native(
            "Raoul",
            eframe::NativeOptions::default(),
            Box::new(|_cc| Box::new(app)),
        );
    }

    fn histogram(&mut self) -> VMResult<()> {
        let quad = self.get_current_quad();
        let data_frame = self.get_dataframe()?;
        let col_name = String::from(self.get_value(quad.op_1.unwrap())?);
        let bins_value = self.get_value(quad.op_2.unwrap())?;
        let bins = match bins_value {
            VariableValue::Integer(a) if a <= 0 => Err("The amount of bins should be positive"),
            _ => Ok(usize::from(bins_value)),
        }?;
        let temp = data_frame
            .clone()
            .lazy()
            .select([col(&col_name).cast(DataType::Float64).alias("column")])
            .collect()
            .unwrap();
        let app = App::new_histogram(temp, bins);
        eframe::run_native(
            "Raoul",
            eframe::NativeOptions::default(),
            Box::new(|_cc| Box::new(app)),
        );
    }

    pub fn run(&mut self) -> VMResult<()> {
        loop {
            let mut quad_pos = self.current_context().quad_pos;
            if self.debug {
                self.print_message(&format!("Quad - {quad_pos}\n"));
            }
            let quad = self.quad_list.get(quad_pos).unwrap();
            match quad.operator {
                Operator::End => break,
                Operator::Goto => {
                    quad_pos = quad.res.unwrap() - 1;
                    Ok(())
                }
                Operator::Assignment => self.process_assign(),
                Operator::Print => self.process_print(),
                Operator::PrintNl => {
                    self.print_message("\n");
                    Ok(())
                }
                Operator::Read => self.process_read(),
                Operator::Or => self.binary_operation(|a, b| Ok(a | b)),
                Operator::And => self.binary_operation(|a, b| Ok(a & b)),
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
                Operator::GotoF => {
                    quad_pos = self.conditional_goto(false)?;
                    Ok(())
                }
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
                Operator::Return => {
                    self.process_return()?;
                    continue;
                }
                Operator::Ver => self.process_ver(),
                Operator::ReadCSV => self.read_csv(),
                Operator::Rows | Operator::Columns => self.pure_df_operation(),
                Operator::Average => self.unary_df_operation(|c| c.mean().unwrap_or(0.0)),
                Operator::Std => {
                    self.unary_df_operation(|c| cast_to_f64(&c.std_as_series().get(0)))
                }
                Operator::Variance => {
                    self.unary_df_operation(|c| cast_to_f64(&c.var_as_series().get(0)))
                }
                Operator::Median => self.unary_df_operation(|c| c.median().unwrap_or(0.0)),
                Operator::Min => self.unary_df_operation(min),
                Operator::Max => self.unary_df_operation(max),
                Operator::Range => self.unary_df_operation(|c| max(c) - min(c)),
                Operator::Corr => self.correlation(),
                Operator::Plot => self.plot(),
                Operator::Histogram => self.histogram(),
            }?;
            self.update_quad_pos(quad_pos + 1);
        }
        Ok(())
    }
}
