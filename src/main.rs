use crate::asm::generate;

mod lexer;
mod parser;
mod semantic;
mod asm;
mod memory;
mod ir;
mod ssa;
fn main() {

    //println!("{:?}", asm_generator::test());
    //return;
    let code = "
    fun main(a,b,c) {
        if(1 && 2){
            a = 2 && 1 + c;
        }
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let function = &program_ssa.functions[0];
    let ir = ir::transform(function);
    println!("{:?}", ir);
    println!("{:?}", asm::generate(&ir, &function.parameters));
}
