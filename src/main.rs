mod lexer;
mod parser;
fn main() {
    let code = "
    fun main() {
        if(a == 1 && 2 > 3){
            c = 4;
        }
    }
    ";
    println!("{:?}", parser::parse(&mut lexer::lex(code)));
}
