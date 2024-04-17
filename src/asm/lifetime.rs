
use crate::ir;
use crate::parser;


pub struct Lifetime {
    name: String,
    start: i64,
    end: i64
}
pub struct LifetimeChecker {
    lifetimes: Vec<Lifetime>
}

impl LifetimeChecker {
    pub fn new() -> Self {
        LifetimeChecker {
            lifetimes: vec![],
        }
    }

    pub fn print_all(&mut self) {
        println!("\nLifetime-Checker:");
        for lifetime in &self.lifetimes {
            println!("Var: {} - Start: {}, End: {}", lifetime.name, lifetime.start, lifetime.end);
        }
    }

    pub fn get_start_lifetime(&mut self, name: &str) -> i64 {
        let lifetime_option = self.lifetimes.iter().find(|l| l.name == name);
        if lifetime_option.is_none() {
            panic!("Trying to get start-lifetime of non-existant variable: {}", name);
        }
        lifetime_option.unwrap().start.to_owned()

    }

    pub fn get_end_lifetime(&mut self, name: &str) -> i64 {
        let lifetime_option = self.lifetimes.iter().find(|l| l.name == name);
        if lifetime_option.is_none() {
            panic!("Trying to get end-lifetime of non-existant variable: {}", name);
        }
        lifetime_option.unwrap().end.to_owned()
    }

    fn set_start_lifetime(&mut self, name: String, line: i64) {
        self.lifetimes.push(Lifetime { name: name, start: line, end: line})
    }

    fn set_end_lifetime(&mut self, name: String, line: i64) {
        for lifetime in &mut self.lifetimes {
            if lifetime.name == name {
                lifetime.end = line;
            }
        }
    }
}

fn check_end_lifetime(data: &ir::Data, line: i64,  lifetime_checker: &mut LifetimeChecker) {
    match data {
        ir::Data::Number(_) => (),
        ir::Data::Variable(v) => lifetime_checker.set_end_lifetime(v.to_owned(), line)
    }
}

pub fn get_checker(instructions: &Vec<ir::IrInstruction>, parameters: &parser::Parameters) -> LifetimeChecker {
    let mut checker = LifetimeChecker::new();

    for parameter in parameters {
        checker.set_start_lifetime(parameter.to_owned(), 0);
    }

    for (line, inst) in instructions.iter().enumerate() {

        match inst {
            ir::IrInstruction::Jump(_) => (),
            ir::IrInstruction::JumpFalse(d, _) => check_end_lifetime(d, line as i64, &mut checker),
            ir::IrInstruction::Label(_) => (),
            ir::IrInstruction::FunctionCall(res_var, _, args) => {
                for d in args {
                    check_end_lifetime(d, line as i64, &mut checker);
                }
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Addition(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Subtraction(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Multiplication(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Division(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Modulo(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Greater(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::GreaterEquals(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Less(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::LessEquals(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Equals(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::NotEquals(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::LogicAnd(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::LogicOr(res_var, d1, d2) => {
                check_end_lifetime(d1, line as i64, &mut checker);
                check_end_lifetime(d2, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            },
            ir::IrInstruction::Assignment(res_var, d) => {
                check_end_lifetime(d, line as i64, &mut checker);
                checker.set_start_lifetime(res_var.to_owned(), line as i64);
            }
            ir::IrInstruction::Return(d) => check_end_lifetime(d, line as i64, &mut checker),
            ir::IrInstruction::KeepAlive(var) => {
                checker.set_end_lifetime(var.to_owned(), line as i64)
            }
        }
    }

    checker
}
