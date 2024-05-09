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
    file: PathBuf
}


fn main() {
    let opt = Opt::from_args();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", opt.log_level.to_string())
    }
    env_logger::init();

    let code = fs::read_to_string(opt.file).expect("Couldn't read source code file");

    let mut program = parser::parse(&mut lexer::lex(&code)).unwrap();
    predefined_functions::add(&mut program);
    semantic::check(&program).unwrap();
    let program_ssa = ssa::convert(&program);
    let mut function_tracker = jit::FunctionTracker::new(program_ssa);
    let mut main_function = function_tracker.get_main_function();
    println!("Executing!");
    println!("Return: {:?}", main_function.execute([2, 6].to_vec()));

}
