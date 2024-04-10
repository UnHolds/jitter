use iced_x86::code_asm::*;
use iced_x86::Instruction;
mod lifetime;
use crate::memory::{self, Executable, Memory, Writeable};
use crate::ir;
use crate::parser;

#[allow(dead_code)]
pub fn generate(instructions: &Vec<ir::IrInstruction>, parameters: &parser::Parameters) -> Result<Vec<Instruction>, IcedError> {
    let mut lifetime_checker = lifetime::get_checker(instructions, parameters);
    lifetime_checker.print_all();
    let mut a = CodeAssembler::new(64)?;

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
