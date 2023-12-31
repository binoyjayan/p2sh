use std::env;
use std::fs;
use std::rc::Rc;

use builtins::functions::BUILTINFNS;
use builtins::variables::BuiltinVarType;
use cliargs::CliArgs;
use compiler::symtab::SymbolTable;
use compiler::*;
use object::array::Array;
use object::Object;
use parser::ast::Program;
use parser::*;
use repl::prompt;
use scanner::*;
use vm::interpreter::GLOBALS_SIZE;
use vm::interpreter::VM;

mod builtins;
mod cliargs;
mod code;
mod compiler;
mod object;
mod parser;
mod repl;
mod scanner;
mod vm;

const HISTORY_LINES: usize = 8;
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let cliargs = CliArgs::new();
    let args = cliargs.get_args().to_vec();
    let filter_mode = cliargs.is_filter();
    let command = cliargs.get_cmd();

    if let Some(cmd) = command {
        run_buf(cmd, args, true, filter_mode);
        return;
    }
    if args.is_empty() {
        run_prompt(args);
    } else {
        run_file(&args[0].clone(), args, filter_mode);
    }
}

/// Function to run the REPL
pub fn run_prompt(args: Vec<String>) {
    println!("{} v{}", PKG_DESC, PKG_VERSION);
    println!("Type quit to quit REPL");

    let mut cmds = vec!["quit".to_string()];

    let mut constants = vec![];
    let data = Rc::new(Object::Null);
    let mut globals = vec![data; GLOBALS_SIZE];

    let mut symtab = SymbolTable::default();
    for (i, sym) in BUILTINFNS.iter().enumerate() {
        // Define the built-in function via an index into the 'BUILTINS' array
        symtab.define_builtin_fn(i, sym.name);
        cmds.push(sym.name.to_string());
    }
    // Define the built-in variables
    for n in BuiltinVarType::range() {
        let name: &str = BuiltinVarType::from(n).into();
        symtab.define_builtin_var(n, name);
        cmds.push(name.to_string());
    }

    let mut prompt = prompt::Prompt::new(HISTORY_LINES, cmds.as_slice());
    loop {
        if let Ok(line) = prompt.show() {
            if line == "quit" {
                break;
            }
            if !line.trim().is_empty() {
                let program = match parse_program(&line) {
                    Some(program) => program,
                    None => {
                        continue;
                    }
                };

                let mut compiler = Compiler::new_with_state(symtab, constants);
                if let Err(e) = compiler.compile(program) {
                    eprintln!("{}", e);
                    symtab = compiler.symtab;
                    constants = compiler.constants;
                    continue;
                }
                let bytecode = compiler.bytecode();
                let mut vm = VM::new_with_global_store(bytecode, globals);
                update_builtin_vars(&mut vm, args.clone());
                let err = vm.run();
                if let Err(err) = err {
                    eprintln!("{}", err);
                    globals = vm.globals;
                    symtab = compiler.symtab;
                    constants = compiler.constants;
                    continue;
                }
                // Get the object at the top of the VM's stack
                let stack_elem = vm.last_popped();
                // print last popped element if it is not null
                if !matches!(stack_elem.as_ref(), Object::Null) {
                    println!("{}", stack_elem);
                }
                globals = vm.globals;
                symtab = compiler.symtab;
                constants = compiler.constants;
            }
        }
    }
    println!("\nExiting...");
}

/// Function to run a script file
/// # Arguments
/// * `path` - Path to the script file
/// * `args` - Arguments to the script
/// * `filter_mode` - Flag to indicate filter mode
pub fn run_file(path: &str, args: Vec<String>, filter_mode: bool) {
    let buf = fs::read_to_string(path);
    if buf.is_err() {
        eprintln!("Failed to read file {}", path);
        return;
    }
    let buf = buf.unwrap();
    run_buf(buf, args, false, filter_mode);
}

/// Function to run a script stored in a buffer
/// # Arguments
/// * `buf` - Buffer containing the script
/// * `args` - Arguments to the script
/// * `cmd_mode` - Flag to indicate command mode
/// * `filter_mode` - Flag to indicate filter mode
pub fn run_buf(buf: String, args: Vec<String>, cmd_mode: bool, _filter_mode: bool) {
    let data = Rc::new(Object::Null);
    let globals = vec![data; GLOBALS_SIZE];

    if buf.trim().is_empty() {
        return;
    }
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
        eprintln!("{}", err);
    }
    if cmd_mode {
        // Get the object at the top of the VM's stack
        let stack_elem = vm.last_popped();
        // print last popped element if it is not null
        if !matches!(stack_elem.as_ref(), Object::Null) {
            println!("{}", stack_elem);
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
