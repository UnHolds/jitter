use std::collections::HashMap;
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
    id_memory_mapping: HashMap<FunctionId, ExecuteableMemory>,
    program: ssa::SsaProgram,
}
impl FunctionTracker {
    pub fn new(program: ssa::SsaProgram) -> Self {
        let mut name_id_mapping = BiMap::new();
        for (idx, name) in program.functions.iter().map(|f| f.name.to_owned() ).enumerate() {
            name_id_mapping.insert(name, idx as i64);
        }

        FunctionTracker {
            name_id_mapping: name_id_mapping,
            id_memory_mapping: HashMap::new(),
            program: program
        }
    }

    pub fn get_main_function(&mut self) -> fn() -> i64 {
        let id = self.get_id(&"main".to_owned());
        self.complile_function(id)
    }


    pub fn get_id(&mut self, name: &String) -> FunctionId{
        match self.name_id_mapping.get_by_left(name) {
            None =>{
                panic!("All the ids should have been defined!")
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
        match self.id_memory_mapping.get_mut(&id) {
            None => self.complile_function(id) as u64,
            Some(mem) => {
                mem.as_function() as u64
            }
        }
    }


}

#[no_mangle]
pub extern "C" fn jit_callback(function_tracker: &mut FunctionTracker, function_id: FunctionId) -> FunctionAddress{
    function_tracker.get_function_address(function_id)
}
