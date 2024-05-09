mod lexer;
mod parser;
mod semantic;
mod asm;
mod memory;
mod ir;
mod ssa;
mod jit;
mod predefined_functions;
use structopt::StructOpt;
use std::str::FromStr;
use std::path::PathBuf;
use std::fs;
use log::{debug, error};

#[derive(Debug)]
enum LogLevel {
    Debug,
    Info
}

type ParseError = &'static str;

impl FromStr for LogLevel {
    type Err = ParseError;
    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day.to_lowercase().as_ref() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            _ => Err("Could not parse log level"),
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "Info"
        }.to_owned()
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Jitter", about = "Usage of the jitter JIT compiler")]
struct Opt {
    /// The log level of the application
    #[structopt(short = "l", long = "level", default_value="info")]
    log_level: LogLevel,

    /// The file that contains the source code
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// arguments for the passed program
    #[structopt()]
    args: Vec<i64>
}


fn execute_code(code: &str, args: Vec<i64>) -> Result<i64, ()>{
    debug!("Lexing and parsing code");
    let parse_res = parser::parse(&mut lexer::lex(&code));

    let mut program = match parse_res {
        Ok(p) => p,
        Err(err) => {
            error!("Parsing failed: {}", err);
            return Err(());
        }
    };

    debug!("Adding predefined functions");
    predefined_functions::add(&mut program);
    let semantic_res = semantic::check(&program);
    match semantic_res {
        Ok(_) => (),
        Err(err) => {
            error!("Semantic check failed: {}", err);
            return Err(());
        }
    };

    debug!("Converting program to SSA form");
    let program_ssa = ssa::convert(&program);

    let mut function_tracker = jit::FunctionTracker::new(program_ssa);
    let mut main_function = function_tracker.get_main_function();
    debug!("Executing main function");
    let return_value = main_function.execute(args);
    match return_value {
        Ok(value) => {
            debug!("Return value:");
            println!("{}", value);
            Ok(value)
        },
        Err(err) => {
            error!("Error occured during execution: {}", err);
            Err(())
        }
    }
}


fn main() {
    let opt = Opt::from_args();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", opt.log_level.to_string())
    }
    env_logger::init();

    debug!("Reading source file");
    let code = fs::read_to_string(opt.file).expect("Couldn't read source code file");
    match execute_code(&code, opt.args) {
        Err(()) => (),
        Ok(_) => ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_code_1() {
        let code = fs::read_to_string("test/test1.ji").expect("Couldn't read source code file");
        assert_eq!(execute_code(&code, vec![1, 2]).is_ok(), true);
    }

    #[test]
    fn stack_arguments_1() {
        let code = fs::read_to_string("test/test2.ji").expect("Couldn't read source code file");
        assert_eq!(execute_code(&code, vec![]).is_ok(), true);
    }

    #[test]
    fn stack_arguments_2() {
        let code = fs::read_to_string("test/test3.ji").expect("Couldn't read source code file");
        assert_eq!(execute_code(&code, vec![]).unwrap(), 45);
    }

    #[test]
    fn stack_arguments_3() {
        let code = fs::read_to_string("test/test4.ji").expect("Couldn't read source code file");
        assert_eq!(execute_code(&code, vec![]).unwrap(), 36);
    }

}
