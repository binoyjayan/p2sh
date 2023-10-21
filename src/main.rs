use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::rc::Rc;

use common::builtins::functions::BUILTINFNS;
use common::builtins::variables::BuiltinVarType;
use common::object::Array;
use common::object::Object;
use compiler::symtab::SymbolTable;
use compiler::*;
use parser::ast::Program;
use parser::*;
use scanner::*;
use vm::interpreter::GLOBALS_SIZE;
use vm::interpreter::VM;

mod code;
mod common;
mod compiler;
mod parser;
mod scanner;
mod vm;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // Remove the interpreter path from the args
    args.remove(0);
    match args.len() {
        0 => run_prompt(args),
        _ => {
            run_file(&args[0].clone(), args);
        }
    }
}

pub fn run_prompt(args: Vec<String>) {
    println!("{} v{}", PKG_DESC, PKG_VERSION);
    println!("Ctrl+D to quit");
    // Define globals outside REPL loop so the environment is retained
    let stdin = io::stdin();
    let mut constants = vec![];
    let data = Rc::new(Object::Nil);
    let mut globals = vec![data; GLOBALS_SIZE];

    let mut symtab = SymbolTable::default();
    for (i, sym) in BUILTINFNS.iter().enumerate() {
        // Define the built-in function via an index into the 'BUILTINS' array
        symtab.define_builtin_fn(i, sym.name);
    }
    // Define the built-in variables
    for n in BuiltinVarType::range() {
        let name: &str = BuiltinVarType::from(n).into();
        symtab.define_builtin_var(n, name);
    }

    print!(">> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() {
                let program = match parse_program(&line) {
                    Some(program) => program,
                    None => return,
                };

                let mut compiler = Compiler::new_with_state(symtab, constants);
                if let Err(e) = compiler.compile(program) {
                    eprintln!("{}", e);
                    return;
                }
                let bytecode = compiler.bytecode();
                let mut vm = VM::new_with_global_store(bytecode, globals);
                update_builtin_vars(&mut vm, args.clone());
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
        print!(">> ");
        io::stdout().flush().unwrap();
    }
    println!("\nExiting...");
}

pub fn run_file(path: &str, args: Vec<String>) {
    let buf = fs::read_to_string(path);
    if buf.is_err() {
        eprintln!("Failed to read file {}", path);
        return;
    }
    let buf = buf.unwrap();
    let data = Rc::new(Object::Nil);
    let globals = vec![data; GLOBALS_SIZE];

    if !buf.trim().is_empty() {
        let program = match parse_program(&buf) {
            Some(program) => program,
            None => return,
        };

        let mut compiler = Compiler::new();
        if let Err(e) = compiler.compile(program) {
            eprintln!("{}", e);
            return;
        }
        let bytecode = compiler.bytecode();
        let mut vm = VM::new_with_global_store(bytecode, globals);
        update_builtin_vars(&mut vm, args);
        let err = vm.run();
        if let Err(err) = err {
            eprintln!("vm error: {}", err);
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

fn update_builtin_vars(vm: &mut VM, args: Vec<String>) {
    let elements: Vec<Rc<Object>> = args.into_iter().map(|s| Rc::new(Object::Str(s))).collect();
    let arr = Rc::new(Object::Arr(Rc::new(Array::new(elements))));
    vm.update_builtin_var(BuiltinVarType::Argv, arr);
}
