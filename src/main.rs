mod lexer;
mod parser;
mod semantic;
mod asm_generator;
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
    println!("{:?}", ir::transform(&program_ssa.functions[0]))
}
