use std::char::from_u32;

use crate::parser;

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalFunction {
    pub name: String,
    pub parameters: parser::Parameters,
    pub address: u64
}

extern "C" fn cool() -> u64 {
    println!("cool!");
    return 0;
}

extern "C" fn print_num(i: i64) -> u64 {
    print!("{}", i);
    return 0;
}

extern "C" fn print_char(c: u32) -> u64 {
    let _c = from_u32(c).unwrap();
    print!("{}", _c);
    return 0;
}

extern "C" fn println_num(i: i64) -> u64 {
    println!("{}", i);
    return 0;
}

extern "C" fn println_char(c: u32) -> u64 {
    let _c = from_u32(c).unwrap();
    println!("{}", _c);
    return 0;
}

extern "C" fn read_num() -> u64 {
    let mut input_line = String::new();
    std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");
    return input_line.trim().parse().expect("Input not an integer");
}

pub fn add(program: &mut parser::Program) {
    program.functions.push(parser::Function::External(ExternalFunction {name: "cool".to_owned(), parameters: vec![], address: cool as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "print_num".to_owned(), parameters: vec!["num".to_owned()], address: print_num as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "print_char".to_owned(), parameters: vec!["char".to_owned()], address: print_char as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "println_num".to_owned(), parameters: vec!["num".to_owned()], address: println_num as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "println_char".to_owned(), parameters: vec!["char".to_owned()], address: println_char as u64}));
    program.functions.push(parser::Function::External(ExternalFunction {name: "read_num".to_owned(), parameters: vec![], address: read_num as u64}));
}
