use std::io;
use std::io::{BufRead, Write};

use token::*;

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
    let mut scanner = scanner::Scanner::new(source);

    loop {
        let token = scanner.next_token();
        println!("{}", token);
        if token.ttype == TokenType::Eof {
            break;
        }
    }
}
