use std::io;
use std::io::{BufRead, Write};

mod evaluator;
mod object;
mod parser;
mod scanner;
mod token;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    println!("{} v{}", PKG_DESC, PKG_VERSION);
    println!("Ctrl+D to quit");
    run_prompt();
}

pub fn run_prompt() {
    let stdin = io::stdin();
    print!(">> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() {
                run(&line)
            }
        }
        print!(">> ");
        io::stdout().flush().unwrap();
    }
    println!("\nExiting...");
}

fn run(source: &str) {
    let scanner = scanner::Scanner::new(source);
    let mut parser = parser::Parser::new(scanner);
    let program = parser.parse_program();
    if print_parse_errors(&parser) {
        return;
    }
    let evaluated = evaluator::eval_program(program);
    println!("{}", evaluated);
}

fn print_parse_errors(parser: &parser::Parser) -> bool {
    let errors = parser.parse_errors();
    if errors.is_empty() {
        return false;
    }
    for msg in errors {
        eprintln!("{}", msg);
    }
    eprintln!("{} parse error(s)", errors.len());
    true
}
