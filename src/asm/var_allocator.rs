

use core::fmt;
use iced_x86::code_asm::*;
use log::debug;
use crate::parser;

use crate::asm::lifetime;

#[derive(Clone, Copy, PartialEq)]
pub enum VariableLocation {
    Stack(i64), //rbp offset
    Register(AsmRegister64)
}

#[allow(non_upper_case_globals)]
fn register_to_string(reg: AsmRegister64) -> String {
    match reg {
        r15 => "R15".to_owned(),
        r14 => "R14".to_owned(),
        r13 => "R13".to_owned(),
        r12 => "R12".to_owned(),
        r11 => "R11".to_owned(),
        r10 => "R10".to_owned(),
        rbx => "RBX".to_owned(),
        r9 => "R9".to_owned(),
        r8 => "R8".to_owned(),
        rcx => "RCX".to_owned(),
        rdx => "RDX".to_owned(),
        rsi => "RSI".to_owned(),
        rdi => "RDI".to_owned(),
        _ => panic!("missing register_to_string match")
    }
}

impl fmt::Display for VariableLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(register) =>
            write!(f, "REGISTER ({})", register_to_string(register.to_owned())),
            Self::Stack(offset) =>
            write!(f, "STACK ({})", offset),
        }
    }
}

pub struct AllocatedVariable {
    location: VariableLocation,
    lifetime_end: u64,
    name: String
}

pub struct VariableAllocator {
    free_registers: Vec<AsmRegister64>,
    variables: Vec<AllocatedVariable>,
    next_stack_variable_offset: i64,
}

impl VariableAllocator {
    pub fn new(parameters: &parser::Parameters, lifetime_checker: &mut lifetime::LifetimeChecker) -> Self {
        #[cfg(target_os = "linux")]
        let mut free_registers = [
            r15,
            r14,
            r13,
            r12,
            r11,
            r10,
            rbx,
            r9,
            r8,
            rcx,
            rdx,
            rsi,
            rdi
        ].to_vec();

        #[cfg(target_os = "windows")]
        let mut free_registers = [
            r15,
            r14,
            r13,
            r12,
            r11,
            r10,
            rsi,
            rdi,
            rbx,
            r9,
            r8,
            rdx,
            rcx,
        ].to_vec();

        let mut variables = vec![];

        #[cfg(target_os = "linux")]
        let number_of_register_arguments = 6;
        #[cfg(target_os = "windows")]
        let number_of_register_arguments = 4;

        let mut offset = 8 + (parameters.len() as i64 - number_of_register_arguments as i64) * 8;
        //allocate the function parameters
        for (i, parameter) in parameters.iter().enumerate() {
            let lifetime_end = lifetime_checker.get_end_lifetime(parameter) as u64;
            if i < number_of_register_arguments {
                //allocate parameter register
                variables.push(AllocatedVariable { location: VariableLocation::Register(free_registers.pop().unwrap()), lifetime_end: lifetime_end, name: parameter.to_owned()});
            }else{
                offset = offset - 8; //base offset
                variables.push(AllocatedVariable { location: VariableLocation::Stack(offset), lifetime_end: lifetime_end, name: parameter.to_owned()});
            }
        }

        VariableAllocator {
            free_registers,
            variables,
            next_stack_variable_offset: -56,
        }
    }

    pub fn print_allocated(&mut self) {
        println!("\nVariable-Allocator:");
        for var in &self.variables {
            println!("Name: {} - Location: {} - Lifetime end at line {}", var.name, var.location, var.lifetime_end);
        }
    }

    pub fn is_allocated(&mut self, reg: AsmRegister64) -> bool{
        self.free_registers.contains(&reg) == false
    }

    fn check(&mut self, line: u64) {
        for var in &self.variables {
            if var.lifetime_end >= line || var.location == VariableLocation::Stack(0){
                continue;
            }
            match var.location {
                VariableLocation::Register(r) => self.free_registers.push(r),
                VariableLocation::Stack(_) => ()
            }
        }
        self.variables.retain(|v| v.lifetime_end >= line || match v.location {VariableLocation::Register(_) => false, VariableLocation::Stack(_) => true});
    }


    fn allcoate(&mut self, name: &str, line: u64, lifetime_checker: &mut lifetime::LifetimeChecker, code_assembler: &mut CodeAssembler) -> VariableLocation {
        self.check(line);
        let lifetime_end = lifetime_checker.get_end_lifetime(name) as u64;
        if self.free_registers.len() > 0 {
            //register allocation
            let location = VariableLocation::Register(self.free_registers.pop().unwrap());
            self.variables.push(AllocatedVariable { location: location, lifetime_end: lifetime_end, name: name.to_owned() });
            location
        }else{
            let location = VariableLocation::Stack(self.next_stack_variable_offset);
            self.next_stack_variable_offset = self.next_stack_variable_offset - 8;
            self.variables.push(AllocatedVariable { location: location, lifetime_end: lifetime_end, name: name.to_owned() });
            code_assembler.push(rbx).unwrap();
            location
        }
    }

    pub fn get(&mut self, name: &str, line: u64, lifetime_checker: &mut lifetime::LifetimeChecker, code_assembler: &mut CodeAssembler) -> VariableLocation {
        match self.variables.iter().find(|v| v.name == name) {
            None => self.allcoate(name, line, lifetime_checker, code_assembler),
            Some(v) => v.location
        }
    }

    pub fn get_num_stackvars(&mut self) -> u64 {
        self.variables.iter().filter(|v| match v.location {VariableLocation::Register(_) => false, VariableLocation::Stack(v) => v < 0}).count() as u64
    }
}
