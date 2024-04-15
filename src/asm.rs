use std::collections::HashMap;

use iced_x86::code_asm::*;
use iced_x86::Instruction;
mod lifetime;
mod var_allocator;
use crate::memory::{self, Executable, Memory, Writeable};
use crate::ir;
use crate::parser;

fn get_data_in_register(data: &ir::Data, register: AsmRegister64){

}

#[allow(dead_code)]
pub fn generate(instructions: &Vec<ir::IrInstruction>, parameters: &parser::Parameters) -> Result<Vec<Instruction>, IcedError> {
    let mut lifetime_checker = lifetime::get_checker(instructions, parameters);
    let mut variable_allocator = var_allocator::VariableAllocator::new(parameters, &mut lifetime_checker);
    let mut a = CodeAssembler::new(64)?;
    let mut labels: HashMap<String, CodeLabel> = HashMap::new();

    for inst in instructions {
        match inst {
            ir::IrInstruction::Jump(label) => {
                match labels.get_mut(label) {
                    Some(l) => {
                        a.jmp(l.to_owned())?;
                    },
                    None => panic!("TODO error")
                }
            }
            ir::IrInstruction::JumpFalse(data, label) => {
                get_data_in_register(data, rax);
                a.test(rax, rax)?;
                match labels.get_mut(label) {
                    Some(l) => {
                        a.jz(l.to_owned())?;
                    },
                    None => panic!("TODO error")
                }

            }
            ir::IrInstruction::Label(label) => {
                let mut l = a.create_label();
                match labels.get(label){
                    None => labels.insert(label.to_owned(), l),
                    Some(_) => panic!("TODO error")
                };
                a.set_label(&mut l)?;
            }
            ir::IrInstruction::FunctionCall(res_var, fun_name, args) => {
                panic!("Function call not implemented yet (IR)")
            }
            ir::IrInstruction::Addition(res_var, data1, data2) => {

            }
            ir::IrInstruction::Subtraction(res_var, data1, data2) => {

            }
            ir::IrInstruction::Multiplication(res_var, data1, data2) => {

            }
            ir::IrInstruction::Division(res_var, data1, data2) => {

            }
            ir::IrInstruction::Modulo(res_var, data1, data2) => {

            }
            ir::IrInstruction::Greater(res_var, data1, data2) => {

            }
            ir::IrInstruction::GreaterEquals(res_var, data1, data2) => {

            }
            ir::IrInstruction::Less(res_var, data1, data2) => {

            }
            ir::IrInstruction::LessEquals(res_var, data1, data2) => {

            }
            ir::IrInstruction::Equals(res_var, data1, data2) => {

            }
            ir::IrInstruction::NotEquals(res_var, data1, data2) => {

            }
            ir::IrInstruction::LogicAnd(res_var, data1, data2) => {

            }
            ir::IrInstruction::LogicOr(res_var, data1, data2) => {

            }
            ir::IrInstruction::Assignment(res_var, data) => {

            }
        }
    }


    Ok(a.take_instructions())
}

#[allow(dead_code)]
pub fn test() -> Result<(), IcedError>  {
    let mut a = CodeAssembler::new(64)?;
    a.mov(rax, 10 as u64)?;
    a.push(rax)?;

    // Encode all added instructions.
    // Use `assemble_options()` if you must get the address of a label
    /*
    let bytes = a.assemble(0x1234_5678)?;
    let mut mem = memory::ExecuteableMemory::new(bytes.len());
    let addr = mem.address() as u64;
    println!("addr: {:?}", addr);
    let bytes = a.assemble(addr)?;
    let inst = a.take_instructions();
    println!("Bytes: {:?}", bytes);
    mem.write(&bytes);
    println!("ok!");
    let f = mem.as_function();
    f();
    println!("ok2");
    */



    Ok(())
}
