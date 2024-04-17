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
    fun main(a, b) {
        return a + b;
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let mut function_tracker = jit::FunctionTracker::new(program_ssa);
    let mut main_function = function_tracker.get_main_function();
    println!("Executing!");
    println!("Return: {:?}", main_function.execute([2, 6].to_vec()));

}
