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
        b = 0;
        c = 0;
        while (b < 8) {
            b = b + 1;
            c = c + 5;
        }
        return c;
    }
    fun test(a) {

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
