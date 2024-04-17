use memory::{Executable, Writeable};

use crate::memory::Memory;

mod lexer;
mod parser;
mod semantic;
mod asm;
mod memory;
mod ir;
mod ssa;
mod jit;
fn main() {

    let code = "
    fun main() {
        return test(4) + test(5);
    }
    fun test(a) {
        return 5 + a;
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let mut function_tracker = jit::FunctionTracker::new(program_ssa);
    let function = function_tracker.get_main_function();
    println!("Executing!");
    println!("Return: {:?}", function());

}
