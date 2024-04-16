mod lexer;
mod parser;
mod semantic;
mod asm;
mod memory;
mod ir;
mod ssa;
fn main() {

    let code = "
    fun main(a,b,c) {
        a = a + 4 * 3;
    }
    ";
    let program = parser::parse(&mut lexer::lex(code)).unwrap();
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let function = &program_ssa.functions[0];
    let ir = ir::transform(function);
    let is = asm::generate(&ir, &function.parameters).unwrap();
    let bytes = asm::assemble(&is, 0xdeadbeef).unwrap();
    asm::print_decoded_bytes(&bytes, 0xdeadbeef);
}
