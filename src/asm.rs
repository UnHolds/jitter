use std::collections::HashMap;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter, Register};
use iced_x86::code_asm::*;
mod lifetime;
mod var_allocator;
use crate::memory::{self, Executable, Memory, Writeable};
use crate::ir::{self, IrInstruction};
use crate::parser;

use self::lifetime::LifetimeChecker;
use self::var_allocator::VariableLocation;

#[derive(Debug, PartialEq)]
pub enum DataLocation {
    Register(AsmRegister64),
    Stack(AsmMemoryOperand),
    Number(i64)
}

fn get_data(data: &ir::Data, line: u64,  var_allocator: &mut var_allocator::VariableAllocator, lifetime_checker: &mut LifetimeChecker) -> DataLocation {
    match data {
        ir::Data::Number(n) => {
            DataLocation::Number(n.to_owned())
        },
        ir::Data::Variable(v) => {
            match var_allocator.get(v, line, lifetime_checker) {
                var_allocator::VariableLocation::Register(r) => DataLocation::Register(r),
                var_allocator::VariableLocation::Stack(s) => DataLocation::Stack(rsp + s)
            }
        }
    }
}

fn move_to(to: VariableLocation, from: DataLocation, generator: &mut CodeGenerator)  -> Result<(), IcedError>{
    match from {
        DataLocation::Number(n) => {
            match to {
                VariableLocation::Register(r) => generator.code_assembler.mov(r, n)?,
                VariableLocation::Stack(s) =>  generator.code_assembler.mov(rbp + s, n as i32)?,
            }
        }
        DataLocation::Register(sr) => {
            match to {
                VariableLocation::Register(r) => generator.code_assembler.mov(r, sr)?,
                VariableLocation::Stack(s) =>  generator.code_assembler.mov(rbp + s, sr)?,
            }
        }
        DataLocation::Stack(st) => {
            match to {
                VariableLocation::Register(r) => generator.code_assembler.mov(r, st)?,
                VariableLocation::Stack(s) =>  {
                    generator.code_assembler.mov(rax, st)?;
                    generator.code_assembler.mov(rbp + s, rax)?;
                },
            }
        }
    };
    Ok(())
}

fn generate_jump(label: &String, generator: &mut CodeGenerator) -> Result<(), IcedError>{
    match generator.labels.get_mut(label) {
        Some(l) => {
            generator.code_assembler.jmp(l.to_owned())?;
        },
        None => panic!("TODO error")
    };
    Ok(())
}

fn generate_jump_false(data: &ir::Data, label: &String, line: u64,  generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let d = get_data(data, line as u64, &mut generator.variable_allocator, &mut generator.lifetime_checker);
    match d {
        DataLocation::Number(n) => {
            generator.code_assembler.mov(rax, n)?;
            generator.code_assembler.test(rax, rax)?;
        },
        DataLocation::Register(r) =>  generator.code_assembler.test(r,r)?,
        DataLocation::Stack(s) => {
            generator.code_assembler.mov(rax, s)?;
            generator.code_assembler.test(rax, rax)?;
        }
    }
    match  generator.labels.get_mut(label) {
        Some(l) => {
            generator.code_assembler.jz(l.to_owned())?;
        },
        None => panic!("TODO error")
    };
    Ok(())
}

fn generate_label(label: &String, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let mut l = generator.code_assembler.create_label();
    match generator.labels.get(label){
        None => generator.labels.insert(label.to_owned(), l),
        Some(_) => panic!("TODO error")
    };
    generator.code_assembler.set_label(&mut l)?;
    Ok(())
}

fn generate_addition(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator)-> Result<(), IcedError> {
    let res_loc = generator.variable_allocator.get(&res_var, line, &mut generator.lifetime_checker);
    move_to(res_loc, get_data(data1, line, &mut generator.variable_allocator, &mut generator.lifetime_checker), generator)?;

    match get_data(data2, line, &mut generator.variable_allocator, &mut generator.lifetime_checker) {
        DataLocation::Number(n) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.add( r, n as i32)?,
                VariableLocation::Stack(s) => generator.code_assembler.add(rbp + s, n as i32)?,
            }
        },
        DataLocation::Register(rr) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.add( r, rr)?,
                VariableLocation::Stack(s) => generator.code_assembler.add(rbp + s, rr)?,
            }
        },
        DataLocation::Stack(st) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.add( r, st)?,
                VariableLocation::Stack(s) => {
                    generator.code_assembler.mov(rax, st)?;
                    generator.code_assembler.add(rbp + s, rax)?;
                },
            }
        }
    }

    Ok(())
}


fn generate_subtraction(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator)-> Result<(), IcedError> {
    let res_loc = generator.variable_allocator.get(&res_var, line, &mut generator.lifetime_checker);
    move_to(res_loc, get_data(data1, line, &mut generator.variable_allocator, &mut generator.lifetime_checker), generator)?;

    match get_data(data2, line, &mut generator.variable_allocator, &mut generator.lifetime_checker) {
        DataLocation::Number(n) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.sub( r, n as i32)?,
                VariableLocation::Stack(s) => generator.code_assembler.sub(rbp + s, n as i32)?,
            }
        },
        DataLocation::Register(rr) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.sub( r, rr)?,
                VariableLocation::Stack(s) => generator.code_assembler.sub(rbp + s, rr)?,
            }
        },
        DataLocation::Stack(st) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.sub( r, st)?,
                VariableLocation::Stack(s) => {
                    generator.code_assembler.mov(rax, st)?;
                    generator.code_assembler.sub(rbp + s, rax)?;
                },
            }
        }
    }

    Ok(())
}


fn generate_multiplication(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator)-> Result<(), IcedError> {
    let res_loc = generator.variable_allocator.get(&res_var, line, &mut generator.lifetime_checker);
    move_to(res_loc, get_data(data1, line, &mut generator.variable_allocator, &mut generator.lifetime_checker), generator)?;

    match get_data(data2, line, &mut generator.variable_allocator, &mut generator.lifetime_checker) {
        DataLocation::Number(n) => {
            match res_loc {
                VariableLocation::Register(r) => {
                    generator.code_assembler.mov(rax, n)?;
                    generator.code_assembler.imul_2(r, rax)?;
                },
                VariableLocation::Stack(s) => {
                    generator.code_assembler.mov(rbx, n)?;
                    generator.code_assembler.mov(rax, rbp + s)?;
                    generator.code_assembler.imul_2(rax, rbx)?;
                    generator.code_assembler.mov(rbp + s, rax)?;
                },
            }
        },
        DataLocation::Register(rr) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.imul_2( r, rr)?,
                VariableLocation::Stack(s) => {
                    generator.code_assembler.mov(rax, rbp + s)?;
                    generator.code_assembler.imul_2( rax, rr)?;
                    generator.code_assembler.mov(rbp + s, rax)?;
                },
            }
        },
        DataLocation::Stack(st) => {
            match res_loc {
                VariableLocation::Register(r) => generator.code_assembler.imul_2( r, st)?,
                VariableLocation::Stack(s) => {
                    generator.code_assembler.mov(rbx, st)?;
                    generator.code_assembler.mov(rax, rbp + s)?;
                    generator.code_assembler.imul_2(rax, rbx)?;
                    generator.code_assembler.mov(rbp + s, rax)?;
                },
            }
        }
    }

    Ok(())
}


fn generate_compare(data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator)-> Result<(), IcedError> {
    let data1_loc = get_data(data1, line, &mut generator.variable_allocator, &mut generator.lifetime_checker);
    let data2_loc = get_data(data2, line, &mut generator.variable_allocator, &mut generator.lifetime_checker);

    match data1_loc {
        DataLocation::Number(n1) => {
            match data2_loc {
                DataLocation::Number(n2) => {
                    generator.code_assembler.mov(rax, n1)?;
                    generator.code_assembler.cmp(rax, n2 as i32)?;
                },
                DataLocation::Register(r2) => {
                    generator.code_assembler.mov(rax, n1)?;
                    generator.code_assembler.cmp(rax, r2)?
                },
                DataLocation::Stack(s2) => {
                    generator.code_assembler.mov(rax, n1)?;
                    generator.code_assembler.cmp(rax, s2)?;
                },
            }
        },
        DataLocation::Register(r1) => {
            match data2_loc {
                DataLocation::Number(n2) => generator.code_assembler.cmp(r1, n2 as i32)?,
                DataLocation::Register(r2) => generator.code_assembler.cmp(r1, r2)?,
                DataLocation::Stack(s2) => generator.code_assembler.cmp(r1, s2)?
            }
        },
        DataLocation::Stack(s1) => {
            match data2_loc {
                DataLocation::Number(n2) => generator.code_assembler.cmp(s1, n2 as i32)?,
                DataLocation::Register(r2) => generator.code_assembler.cmp(s1, r2)?,
                DataLocation::Stack(s2) => {
                    generator.code_assembler.mov(rax, s1)?;
                    generator.code_assembler.cmp(rax, s2)?;
                }
            }
        }
    }

    Ok(())
}

fn store_rax_in_var(var: &String, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let res_loc: VariableLocation = generator.variable_allocator.get(&var, line, &mut generator.lifetime_checker);
    match res_loc {
        VariableLocation::Register(r) => generator.code_assembler.mov(r, rax)?,
        VariableLocation::Stack(s) => generator.code_assembler.mov(rbp + s, rax)?
    };
    Ok(())
}

fn generate_greater(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setg(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_greater_equals(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setge(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_less(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setl(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_less_equals(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setle(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_equals(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.sete(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_not_equals(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, data2, line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setne(al)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_and(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, &ir::Data::Number(0), line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setg(al)?;
    generate_compare(data2, &ir::Data::Number(0), line, generator)?;
    generator.code_assembler.mov(rbx, 0 as i64)?;
    generator.code_assembler.setg(bl)?;
    generator.code_assembler.and(rax, rbx)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_or(res_var: &String, data1: &ir::Data, data2: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generate_compare(data1, &ir::Data::Number(0), line, generator)?;
    generator.code_assembler.mov(rax, 0 as i64)?;
    generator.code_assembler.setg(al)?;
    generate_compare(data2, &ir::Data::Number(0), line, generator)?;
    generator.code_assembler.mov(rbx, 0 as i64)?;
    generator.code_assembler.setg(bl)?;
    generator.code_assembler.or(rax, rbx)?;
    store_rax_in_var(res_var, line, generator)?;
    Ok(())
}

fn generate_assignment(res_var: &String, data: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let res_loc: VariableLocation = generator.variable_allocator.get(&res_var, line, &mut generator.lifetime_checker);
    let data =  get_data(data, line, &mut generator.variable_allocator, &mut generator.lifetime_checker);
    move_to(res_loc, data, generator)?;
    Ok(())
}

pub struct CodeGenerator {
    lifetime_checker: LifetimeChecker,
    variable_allocator: var_allocator::VariableAllocator,
    code_assembler: CodeAssembler,
    labels: HashMap<String, CodeLabel>
}

#[allow(dead_code)]
pub fn generate(instructions: &Vec<ir::IrInstruction>, parameters: &parser::Parameters) -> Result<Vec<Instruction>, IcedError> {
    let mut _lifetime = lifetime::get_checker(instructions, parameters);
    let mut generator = CodeGenerator {
        code_assembler: CodeAssembler::new(64)?,
        labels: HashMap::new(),
        variable_allocator: var_allocator::VariableAllocator::new(parameters, &mut _lifetime),
        lifetime_checker: _lifetime
    };


    for (line, inst) in instructions.iter().enumerate() {

        match inst {
            ir::IrInstruction::Jump(label) => {
                generate_jump(label, &mut generator)?;
            }
            ir::IrInstruction::JumpFalse(data, label) => {
                generate_jump_false(data, label, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Label(label) => {
                generate_label(label, &mut generator)?;
            }
            ir::IrInstruction::FunctionCall(res_var, fun_name, args) => {
                panic!("Function call not implemented yet (ASM)")
            }
            ir::IrInstruction::Addition(res_var, data1, data2) => {
                generate_addition(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Subtraction(res_var, data1, data2) => {
                generate_subtraction(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Multiplication(res_var, data1, data2) => {
                generate_multiplication(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Division(res_var, data1, data2) => {
                panic!("division not implemented yet (ASM)")
            }
            ir::IrInstruction::Modulo(res_var, data1, data2) => {
                panic!("modulo not implemented yet (ASM)")
            }
            ir::IrInstruction::Greater(res_var, data1, data2) => {
                generate_greater(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::GreaterEquals(res_var, data1, data2) => {
                generate_greater_equals(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Less(res_var, data1, data2) => {
                generate_less(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::LessEquals(res_var, data1, data2) => {
                generate_less_equals(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Equals(res_var, data1, data2) => {
                generate_equals(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::NotEquals(res_var, data1, data2) => {
                generate_not_equals(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::LogicAnd(res_var, data1, data2) => {
                generate_and(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::LogicOr(res_var, data1, data2) => {
                generate_or(res_var, data1, data2, line as u64, &mut generator)?;
            }
            ir::IrInstruction::Assignment(res_var, data) => {
                generate_assignment(res_var, data, line as u64, &mut generator)?;
            }
        }
    }


    Ok(generator.code_assembler.take_instructions())
}

pub fn print_decoded_bytes(bytes: &Vec<u8>, rip: u64) {
    let mut decoder =
        Decoder::with_ip(64, &bytes, rip, DecoderOptions::NONE);

    // Formatters: Masm*, Nasm*, Gas* (AT&T) and Intel* (XED).
    let mut formatter = NasmFormatter::new();

    formatter.options_mut().set_digit_separator("`");
    formatter.options_mut().set_first_operand_char_index(10);

    let mut output = String::new();

    let mut instruction = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        output.clear();
        formatter.format(&instruction, &mut output);

        print!("{:016X} ", instruction.ip());
        let start_index = (instruction.ip() - rip) as usize;
        let instr_bytes = &bytes[start_index..start_index + instruction.len()];
        for b in instr_bytes.iter() {
            print!("{:02X}", b);
        }
        if instr_bytes.len() < 10 {
            for _ in 0..10 - instr_bytes.len() {
                print!("  ");
            }
        }
        println!(" {}", output);
    }

}

pub fn assemble(instructions: &Vec<Instruction>, ip: u64) -> Result<Vec<u8>, IcedError> {
    let mut a = CodeAssembler::new(64)?;
    for inst in instructions {
        a.add_instruction(inst.to_owned());
    }
    Ok(a.assemble(ip)?)
}

#[allow(dead_code)]
pub fn test() -> Result<(), IcedError>  {
    let mut a = CodeAssembler::new(64)?;
    a.mov(rax, 10 as u64)?;
    a.push(rax)?;
    a.mov(rbx, rbp - 8)?;
    a.mov(rax, 5 as u64)?;
    let bytes = a.assemble(0x1234_5678)?;
    print_decoded_bytes(&bytes, 0x1234_5678);

    // Encode all added instructions.
    // Use `assemble_options()` if you must get the address of a label
    /*
    let bytes = a.assemble(0x1234_5678)?;
    let mut mem = memory::ExecuteableMemory::new(bytes.len());
    let addr = mem.address() as u64;
    println!("addr: {:?}", addr);
    let bytes = a.assemble(addr)?;
    let inst = a.take_instructions();
    println!("Bytes: {:?}", bytes);
    mem.write(&bytes);
    println!("ok!");
    let f = mem.as_function();
    f();
    println!("ok2");
    */




    Ok(())
}
