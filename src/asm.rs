use std::collections::HashMap;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use iced_x86::code_asm::*;
mod lifetime;
mod var_allocator;
use crate::ir::{self, Data};
use crate::parser;
use crate::jit;

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
        None => {
            let l = generator.code_assembler.create_label();
            generator.code_assembler.jmp(l)?;
            generator.labels.insert(label.to_owned(), l);
        }
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
        None => {
            let l = generator.code_assembler.create_label();
            generator.code_assembler.jz(l)?;
            generator.labels.insert(label.to_owned(), l);
        }
    };
    Ok(())
}

fn generate_label(label: &String, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    generator.code_assembler.nop()?;
    match generator.labels.get_mut(label){
        None => {
            let mut l = generator.code_assembler.create_label();
            generator.labels.insert(label.to_owned(), l);
            generator.code_assembler.set_label(&mut l)?;
        },
        Some(l) => {
            generator.code_assembler.set_label(l)?;
        }
    };
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

fn generate_return(data: &ir::Data, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let data_loc = get_data(data, line, &mut generator.variable_allocator, &mut generator.lifetime_checker);
    move_to(VariableLocation::Register(rax), data_loc, generator)?;
    //restore register
    generator.code_assembler.mov(rbx, rbp)?;
    generator.code_assembler.sub(rbx, 48)?;
    generator.code_assembler.mov(rsp, rbx)?;
    generator.code_assembler.pop(r15)?;
    generator.code_assembler.pop(r14)?;
    generator.code_assembler.pop(r13)?;
    generator.code_assembler.pop(r12)?;
    generator.code_assembler.pop(rbx)?;
    generator.code_assembler.pop(rbp)?;
    generator.code_assembler.ret()?;
    Ok(())
}


fn set_arguments(args: &Vec<Data>, line: u64, generator: &mut CodeGenerator) -> Result<u64, IcedError> {
    #[cfg(target_os = "windows")]
    let arg_regs = [
        rcx,
        rdx,
        r8,
        r9
    ];

    #[cfg(target_os = "linux")]
    let arg_regs = [
        rdi,
        rsi,
        rdx,
        rcx,
        r8,
        r9
    ];

    let mut pushed_args = 0;

    for (i, arg) in args.iter().enumerate() {
        if i < arg_regs.len() {
            match arg {
                Data::Number(n) => generator.code_assembler.mov(arg_regs[i], n.to_owned())?,
                Data::Variable(v) => {
                    match generator.variable_allocator.get(v, line, &mut generator.lifetime_checker){
                        VariableLocation::Register(r) => generator.code_assembler.mov(arg_regs[i], r)?,
                        VariableLocation::Stack(s) => generator.code_assembler.mov(arg_regs[i], rbp + s)?
                    };
                }
            };
        }else{
            pushed_args += 1;
            match arg {
                Data::Number(n) =>{
                    generator.code_assembler.mov(rax, n.to_owned())?;
                    generator.code_assembler.push(rax)?
                },
                Data::Variable(v) => {
                    match generator.variable_allocator.get(v, 0, &mut generator.lifetime_checker){
                        VariableLocation::Register(r) => generator.code_assembler.push(r)?,
                        VariableLocation::Stack(s) => {
                            generator.code_assembler.mov(rax, rbp + s)?;
                            generator.code_assembler.push(rax)?;
                        }
                    };
                }
            };
        }
    }
    Ok(pushed_args)
}


fn unset_arguments(pushed_args: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {
    let mut i = 0;
    while i < pushed_args{
        generator.code_assembler.pop(rbx)?;
        i += 1;
    }
    Ok(())
}

fn save_registers(mut number_of_args: u64, generator: &mut CodeGenerator) -> Result<Vec<AsmRegister64>, IcedError>{
    let mut saved_vec = vec![];

    #[cfg(target_os = "windows")]
    let registers = [rcx, rdx, r8, r9, r10, r11];

    #[cfg(target_os = "windows")]
    let max_reg_args = 4;

    #[cfg(target_os = "linux")]
    let registers = [rcx, rdx, rsi, rsp, r8, r9, r10, r11];

    #[cfg(target_os = "linux")]
    let max_reg_args = 6;

    //cap number of args
    if number_of_args > max_reg_args{
        number_of_args = max_reg_args;
    }

    let mut args = 0;
    for reg in registers{
        if generator.variable_allocator.is_allocated(reg) || args < number_of_args {
            generator.code_assembler.mov(rbx, reg)?;
            generator.code_assembler.push(reg)?;
            generator.code_assembler.mov(reg, rbx)?;
            saved_vec.push(reg);
        }
        args += 1;
    }
    Ok(saved_vec)
}

fn restore_registers(saved_regs: Vec<AsmRegister64>, generator: &mut CodeGenerator) -> Result<(), IcedError>{
    for reg in saved_regs.iter().copied().rev() {
        generator.code_assembler.pop(reg)?;
    }
    Ok(())
}


fn generate_function_call(res_var: &String, fun_name: &String, args: &Vec<Data>, function_tracker: &mut jit::FunctionTracker, line: u64, generator: &mut CodeGenerator) -> Result<(), IcedError> {

    let mut jit_args = vec![];
    jit_args.push(Data::Number(function_tracker as *const _ as i64));
    jit_args.push(Data::Number(function_tracker.get_id(fun_name)));

    let saved_regs = save_registers(jit_args.len() as u64, generator)?;
    let pushed_args = set_arguments(&jit_args, line, generator)?;
    generator.code_assembler.mov(rbx, rdi)?;
    generator.code_assembler.push(rdi)?;
    generator.code_assembler.mov(rdi, rbx)?;
    generator.code_assembler.call(jit::jit_callback as u64)?;
    generator.code_assembler.pop(rdi)?;
    unset_arguments(pushed_args, generator)?;
    restore_registers(saved_regs, generator)?;


    let saved_regs = save_registers(args.len() as u64, generator)?;
    let pushed_args = set_arguments(args, line, generator)?;
    generator.code_assembler.mov(rbx, rdi)?;
    generator.code_assembler.push(rdi)?;
    generator.code_assembler.mov(rdi, rbx)?;
    generator.code_assembler.call(rax)?;
    generator.code_assembler.pop(rdi)?;
    unset_arguments(pushed_args,  generator)?;
    restore_registers(saved_regs, generator)?;

    let res_loc: VariableLocation = generator.variable_allocator.get(&res_var, line, &mut generator.lifetime_checker);
    match res_loc {
        VariableLocation::Register(r) => generator.code_assembler.mov(r, rax)?,
        VariableLocation::Stack(s) => generator.code_assembler.mov(rbp + s, rax)?
    }

    Ok(())
}


pub struct CodeGenerator {
    lifetime_checker: LifetimeChecker,
    variable_allocator: var_allocator::VariableAllocator,
    code_assembler: CodeAssembler,
    labels: HashMap<String, CodeLabel>
}




#[allow(dead_code)]
pub fn generate(instructions: &Vec<ir::IrInstruction>, parameters: &parser::Parameters, function_tracker: &mut jit::FunctionTracker) -> Result<Vec<Instruction>, IcedError> {
    let mut _lifetime = lifetime::get_checker(instructions, parameters);
    let mut generator = CodeGenerator {
        code_assembler: CodeAssembler::new(64)?,
        labels: HashMap::new(),
        variable_allocator: var_allocator::VariableAllocator::new(parameters, &mut _lifetime),
        lifetime_checker: _lifetime
    };

    //save callee registers
    generator.code_assembler.push(rbp)?;
    //fix rbp
    generator.code_assembler.mov(rax, rsp)?;
    generator.code_assembler.add(rax, 8)?;
    generator.code_assembler.mov(rbp, rax)?;
    //save the rest of the register
    generator.code_assembler.push(rbx)?;
    generator.code_assembler.push(r12)?;
    generator.code_assembler.push(r13)?;
    generator.code_assembler.push(r14)?;
    generator.code_assembler.push(r15)?;

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
                generate_function_call(res_var, fun_name, args, function_tracker, line as u64, &mut generator)?;
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
            ir::IrInstruction::Return(data) => {
                generate_return(data, line as u64, &mut generator)?;
            }
        }
    }

    //always return
    generate_return(&Data::Number(0), 0, &mut generator)?;

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
        a.add_instruction(inst.to_owned())?;
    }
    Ok(a.assemble(ip)?)
}
