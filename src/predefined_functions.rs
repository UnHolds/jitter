use crate::parser;

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalFunction {
    pub name: String,
    pub parameters: parser::Parameters,
    pub address: u64
}

extern "C" fn cool(){
    println!("cool!");
}

extern "C" fn print_num(i: i64){
    println!("{}", i);
}

pub fn add(program: &mut parser::Program) {
    program.functions.push(parser::Function::External(ExternalFunction {name: "cool".to_owned(), parameters: vec![], address: cool as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "print_num".to_owned(), parameters: vec!["num".to_owned()], address: print_num as u64}));
}
