use lazy_static::lazy_static;
use std::cell::RefCell;
use std::env;
use std::io;
use std::io::{BufRead, Write};
use std::rc::Rc;

use common::environment::*;
use common::object::Object;
use compiler::symtab::SymbolTable;
use compiler::*;
use evaluator::*;
use parser::ast::Program;
use parser::*;
use scanner::*;
use vm::interpreter::GLOBALS_SIZE;
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
    static ref AST_EVAL: bool = {
        let env_value = env::var("AST_EVAL").unwrap_or_else(|_| String::from("false"));
        matches!(env_value.as_str(), "true" | "1")
    };
}

fn main() {
    if *AST_EVAL {
        println!("{} v{} [AST Evaluator]", PKG_DESC, PKG_VERSION);
    } else {
        println!(
            "{} v{} [Bytecode compiler AST_EVAL=false]",
            PKG_DESC, PKG_VERSION
        );
    }
    println!("Ctrl+D to quit");
    run_prompt();
}

pub fn run_prompt() {
    // Define globals outside REPL loop so the environment is retained
    let stdin = io::stdin();
    let environment = Rc::new(RefCell::new(Environment::default()));
    let mut evaluator = Evaluator::new();
    let mut constants = vec![];
    let mut symtab = SymbolTable::default();
    let data = Rc::new(Object::Nil);
    let mut globals = vec![data; GLOBALS_SIZE];

    print!(">> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() {
                let program = match parse_program(&line) {
                    Some(program) => program,
                    None => return,
                };
                if *AST_EVAL {
                    let evaluated = evaluator.eval_program(&environment, program);
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
                } else {
                    let mut compiler = Compiler::new_with_state(symtab, constants);

                    if let Err(e) = compiler.compile(program) {
                        eprintln!("Compilation error: {}", e);
                        return;
                    }
                    let bytecode = compiler.bytecode();
                    let mut vm = VM::new_with_global_store(bytecode, globals);
                    let err = vm.run();
                    if let Err(err) = err {
                        eprintln!("vm error: {}", err);
                        return;
                    }
                    // Get the object at the top of the VM's stack
                    let stack_elem = vm.last_popped();
                    println!("{}", stack_elem);
                    globals = vm.globals;
                    symtab = compiler.symtab;
                    constants = compiler.constants;
                }
            }
        }
        print!(">> ");
        io::stdout().flush().unwrap();
    }
    println!("\nExiting...");
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
