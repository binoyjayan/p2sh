use lazy_static::lazy_static;
use std::cell::RefCell;
use std::env;
use std::io;
use std::io::{BufRead, Write};
use std::rc::Rc;

use common::environment::*;
use compiler::*;
use evaluator::*;
use parser::ast::Program;
use parser::*;
use scanner::*;
use vm::interpreter::VM;

mod code;
mod common;
mod compiler;
mod evaluator;
mod parser;
mod scanner;
mod vm;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");

lazy_static! {
    static ref AST_EVAL: bool = env::var("AST_EVAL")
        .unwrap_or_else(|_| String::from("false"))
        .parse()
        .unwrap_or(true);
}

fn main() {
    let ast_evaluator = *AST_EVAL;
    if ast_evaluator {
        println!("{} v{} [AST Evaluator]", PKG_DESC, PKG_VERSION);
    } else {
        println!("{} v{} [Bytecode compiler]", PKG_DESC, PKG_VERSION);
    }
    println!("Ctrl+D to quit");
    run_prompt(ast_evaluator);
}

pub fn run_prompt(ast_evaluator: bool) {
    // Define environment outside REPL loop so the environment is retained
    let environment = Rc::new(RefCell::new(Environment::default()));
    let mut evaluator = Evaluator::new();
    let stdin = io::stdin();
    print!(">> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() {
                if ast_evaluator {
                    run_eval(&line, &environment, &mut evaluator)
                } else {
                    run_compiler(&line)
                }
            }
        }
        print!(">> ");
        io::stdout().flush().unwrap();
    }
    println!("\nExiting...");
}

// Compile the AST into bytecode and execute it
fn run_compiler(source: &str) {
    let program = match parse_program(source) {
        Some(program) => program,
        None => return,
    };
    let mut compiler = Compiler::new();
    if let Err(e) = compiler.compile(program) {
        eprintln!("Compilation error: {}", e);
        return;
    }
    let bytecode = compiler.bytecode();
    let mut vm = VM::new(bytecode.constants.clone());
    let err = vm.run(&bytecode.instructions);
    if let Err(err) = err {
        eprintln!("vm error: {}", err);
        return;
    }
    // Get the object at the top of the VM's stack
    let stack_elem = vm.last_popped();
    println!("{}", stack_elem);
}

// Evaluate the AST tree without compilation
fn run_eval(source: &str, env: &Rc<RefCell<Environment>>, evaluator: &mut Evaluator) {
    let program = match parse_program(source) {
        Some(program) => program,
        None => return,
    };
    let evaluated = evaluator.eval_program(env, program);
    match evaluated {
        Ok(obj) => {
            if !obj.is_nil() {
                println!("{}", obj);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn parse_program(source: &str) -> Option<Program> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    if print_parse_errors(&parser) {
        None
    } else {
        Some(program)
    }
}

fn print_parse_errors(parser: &parser::Parser) -> bool {
    if parser.print_errors() {
        eprintln!("{} parse errors", parser.parse_errors().len());
        true
    } else {
        false
    }
}
