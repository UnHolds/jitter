mod lexer;
mod parser;
mod semantic;

fn main() {
    let code = "
    fun main(a,b,a) {
    }
    ";
    let program = parser::parse(&mut lexer::lex(code));
    match program {
        Ok(p) => {
            println!("{:?}",semantic::check(&p));
            println!("{:?}", p);
        },
        Err(e) => println!("{:?}", e)
    }

}
