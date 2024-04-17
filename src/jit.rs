use std::collections::HashMap;
use std::mem;
use bimap::BiMap;
use crate::memory::Executable;
use crate::memory::ExecuteableMemory;
use crate::memory::Writeable;
use crate::ssa;
use crate::ir;
use crate::asm;
use crate::memory;
pub type FunctionAddress = u64;
pub type FunctionId = i64;

pub struct FunctionTracker{
    name_id_mapping: BiMap<String, FunctionId>,
    id_external_fun_mapping: HashMap<FunctionId, FunctionAddress>,
    id_memory_mapping: HashMap<FunctionId, ExecuteableMemory>,
    program: ssa::SsaProgram,
}

pub struct MainFunction {
    function: fn() -> i64,
    num_args: u64,
}


impl std::fmt::Display for JitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumberOfArguments(expected, found) =>
                write!(f, "Invalid number of arguments for main function! Expected {:?}. Found {:?}.", expected, found),
            Self::TooManyArguments =>
                write!(f, "The execute function can't handle so many arguments (JIT limitation)"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum JitError {
    InvalidNumberOfArguments(u64, u64),
    TooManyArguments
}

impl MainFunction {
    pub fn execute(&mut self, args: Vec<i64>) -> Result<i64, JitError> {
        unsafe {

            if args.len() as u64 != self.num_args {
                return Err(JitError::InvalidNumberOfArguments(self.num_args, args.len() as u64))
            }

            let res = match args.len() {
                0 => (self.function)(),
                1 => mem::transmute::<fn() -> i64, fn(i64) -> i64>(self.function)(args[0]),
                2 => mem::transmute::<fn() -> i64, fn(i64, i64) -> i64>(self.function)(args[0], args[1]),
                3 => mem::transmute::<fn() -> i64, fn(i64, i64, i64) -> i64>(self.function)(args[0], args[1], args[2]),
                4 => mem::transmute::<fn() -> i64, fn(i64, i64, i64, i64) -> i64>(self.function)(args[0], args[1], args[2], args[3]),
                5 => mem::transmute::<fn() -> i64, fn(i64, i64, i64, i64, i64) -> i64>(self.function)(args[0], args[1], args[2], args[3], args[4]),
                _ => return Err(JitError::TooManyArguments)
            };
            Ok(res)
        }
    }
}


impl FunctionTracker {
    pub fn new(program: ssa::SsaProgram) -> Self {
        let mut name_id_mapping = BiMap::new();
        for (idx, name) in program.functions.iter().map(|f| f.name.to_owned() ).enumerate() {
            name_id_mapping.insert(name, idx as i64);

        }

        let mut id_external_fun_mapping = HashMap::new();
        for (idx, fun) in program.external_functions.iter().enumerate() {
            name_id_mapping.insert(fun.name.to_owned(), -(idx as i64) - 1);
            id_external_fun_mapping.insert(-(idx as i64) - 1, fun.address.to_owned());
        }



        FunctionTracker {
            name_id_mapping: name_id_mapping,
            id_external_fun_mapping: id_external_fun_mapping,
            id_memory_mapping: HashMap::new(),
            program: program
        }
    }

    pub fn get_main_function(&mut self) -> MainFunction {
        let id = self.get_id(&"main".to_owned());
        let fun = self.program.functions.iter().find(|f| f.name == "main".to_owned()).unwrap().clone();
        MainFunction{function: self.complile_function(id), num_args: fun.parameters.len() as u64}
    }


    pub fn get_id(&mut self, name: &String) -> FunctionId{
        match self.name_id_mapping.get_by_left(name) {
            None =>{
                panic!("All the ids should have been defined, but couldn't find id for: {}", name)
            },
            Some(counter) => counter.to_owned()
        }
    }

    fn complile_function(&mut self, id: i64) -> fn() -> i64 {
        let name = self.name_id_mapping.get_by_right(&id).unwrap();

        println!("Compiling function: {} with id {}", name, id);

        let fun = self.program.functions.iter().find(|f| f.name == name.to_owned()).unwrap().clone();
        let ir = ir::transform(&fun);
        let is = asm::generate(&ir, &fun.parameters, self).unwrap();
        let bytes = asm::assemble(&is, 0).unwrap();
        let mut memory = memory::ExecuteableMemory::new(bytes.len());
        memory.write(&bytes);
        let compiled_function = memory.as_function();
        self.id_memory_mapping.insert(id, memory);
        compiled_function
    }

    pub fn get_function_address(&mut self, id: FunctionId) -> FunctionAddress {
        if id >= 0 {
            match self.id_memory_mapping.get_mut(&id) {
                None => self.complile_function(id) as u64,
                Some(mem) => {
                    mem.as_function() as u64
                }
            }
        }else{
            match self.id_external_fun_mapping.get_mut(&id) {
                None => panic!("The function with {} does not exit", id),
                Some(v) => return v.to_owned()
            }
        }

    }


}

#[no_mangle]
pub extern "C" fn jit_callback(function_tracker: &mut FunctionTracker, function_id: FunctionId) -> FunctionAddress{
    function_tracker.get_function_address(function_id)
}
