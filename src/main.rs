use memory::{Executable, Writeable};

use crate::memory::Memory;

mod lexer;
mod parser;
mod semantic;
mod asm;
mod memory;
mod ir;
mod ssa;
fn main() {

    let code = "
    fun main() {
        return 7;
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let function = &program_ssa.functions[0];
    let ir = ir::transform(function);
    let mut function_tracker = asm::FunctionTracker::new();
    let is = asm::generate(&ir, &function.name, &function.parameters, &mut function_tracker).unwrap();
    let bytes = asm::assemble(&is, 0xdeadbeef).unwrap();
    asm::print_decoded_bytes(&bytes, 0xdeadbeef);
    let mut memory = memory::ExecuteableMemory::new(bytes.len());
    println!("Addr: {:?}", memory.address());
    memory.write(&bytes);
    println!("Executing!");
    let f = memory.as_function();
    println!("Return: {:?}", f());
}
