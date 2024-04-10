use iced_x86::code_asm::*;
use crate::memory::{self, Executable, Memory, Writeable};

#[allow(dead_code)]
pub fn test() -> Result<(), IcedError>  {


    let mut a = CodeAssembler::new(64)?;

    a.mov(ecx, 10)?;
    a.mov(edx, 1)?;
    a.ret()?;

    // Encode all added instructions.
    // Use `assemble_options()` if you must get the address of a label
    let bytes = a.assemble(0x1234_5678)?;
    let mut mem = memory::ExecuteableMemory::new(bytes.len());
    let addr = mem.address() as u64;
    println!("addr: {:?}", addr);
    let bytes = a.assemble(addr)?;
    println!("Bytes: {:?}", bytes);
    mem.write(&bytes);
    println!("ok!");
    let f = mem.as_function();
    f();
    println!("ok2");



    Ok(())
}
