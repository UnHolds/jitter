mod lexer;
mod parser;
mod semantic;
mod asm_generator;
mod memory;
mod ir;

fn main() {

    //println!("{:?}", asm_generator::test());
    //return;
    let code = "
    fun main(a,b,a) {
        a = 1 && 2;
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    println!("{:?}", ir::transform(&program.functions[0]))

}
