use std::char::from_u32;

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
    print!("{}", i);
}

extern "C" fn print_char(c: u32){
    let _c = from_u32(c).unwrap();
    print!("{}", _c);
}

pub fn add(program: &mut parser::Program) {
    program.functions.push(parser::Function::External(ExternalFunction {name: "cool".to_owned(), parameters: vec![], address: cool as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "print_num".to_owned(), parameters: vec!["num".to_owned()], address: print_num as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "print_char".to_owned(), parameters: vec!["char".to_owned()], address: print_char as u64}));
}
