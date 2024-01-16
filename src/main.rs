use std::env;
use std::fs;
use std::io;
use std::rc::Rc;

use builtins::functions::BUILTINFNS;
use builtins::pcap::Pcap;
use builtins::variables::BuiltinVarType;
use cliargs::CliArgs;
use compiler::symtab::SymbolTable;
use compiler::*;
use object::array::Array;
use object::file::FileHandle;
use object::func::CompiledFunction;
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
    let command = cliargs.get_cmd();
    let skip_pcap = cliargs.skip_pcap();

    if let Some(cmd) = command {
        run_buf(cmd, args, true, skip_pcap);
        return;
    }
    if args.is_empty() {
        run_prompt(args);
    } else {
        run_file(&args[0].clone(), args, skip_pcap);
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
                init_builtin_vars(&vm, args.clone());
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
pub fn run_file(path: &str, args: Vec<String>, skip_pcap: bool) {
    let buf = fs::read_to_string(path);
    if buf.is_err() {
        eprintln!("Failed to read file {}", path);
        return;
    }
    let buf = buf.unwrap();
    run_buf(buf, args, false, skip_pcap);
}

/// Function to run a script stored in a buffer
/// # Arguments
/// * `buf` - Buffer containing the script
/// * `args` - Arguments to the script
/// * `cmd_mode` - Flag to indicate command mode
/// * `filter_mode` - Flag to indicate filter mode
pub fn run_buf(buf: String, args: Vec<String>, cmd_mode: bool, skip_pcap: bool) {
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
    let filters = bytecode.filters.clone();
    let filter_end = bytecode.filter_end.clone();

    let filter_mode = !filters.is_empty() || filter_end.is_some();

    // Run the bytecode that excludes the filter statements
    let mut vm = VM::new_with_global_store(bytecode, globals);
    init_builtin_vars(&vm, args);
    let err = vm.run();
    if let Err(err) = err {
        eprintln!("{}", err);
    }

    if cmd_mode && !filter_mode {
        // Get the object at the top of the VM's stack
        let stack_elem = vm.last_popped();
        // print last popped element if it is not null
        if !matches!(stack_elem.as_ref(), Object::Null) {
            println!("{}", stack_elem);
        }
    }

    // Run all the filter statements
    if filter_mode {
        vm.update_builtin_var(BuiltinVarType::NP, Rc::new(Object::Integer(0)));
        run_filters(vm, filters, filter_end, skip_pcap);
    }
}

/// Run the filter statements on the input pcap stream and
/// write the output pcap stream to stdout
/// # Arguments
/// * `vm` - VM instance
/// * `filters` - Vector of filter statements
fn run_filters(
    mut vm: VM,
    filters: Vec<Rc<CompiledFunction>>,
    filter_end: Option<Rc<CompiledFunction>>,
    skip_pcap: bool,
) {
    let pcap_in = match Pcap::from_file(Rc::new(FileHandle::Stdin)) {
        Ok(pcap) => pcap,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let magic = pcap_in.get_magic_number_raw();
    let pcap_out = if skip_pcap {
        None
    } else {
        let out = match Pcap::new_with_magic(Rc::new(FileHandle::Stdout), magic) {
            Ok(pcap) => pcap,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };
        Some(out)
    };

    // Read packet stream from stdin and write to stdout in a loop
    let mut count = 1;
    'out: loop {
        let result = pcap_in.next_packet();
        match result {
            Ok(pkt) => {
                vm.set_curr_pkt(pkt.clone());
                vm.update_builtin_var(BuiltinVarType::NP, Rc::new(Object::Integer(count)));
                // Run filter statements on the packet
                for filter in &filters {
                    if let Err(err) = vm.push_filter_frame(filter) {
                        eprintln!("{}", err);
                        break 'out;
                    }
                    if let Err(err) = vm.run() {
                        eprintln!("{}", err);
                        break 'out;
                    }
                    // If the result of the filter is true, then write the packet to stdout
                    // The result is true when the action is not specified and the pattern
                    // evaluates to true.
                    match vm.pop_filter_frame() {
                        Ok(true) => {
                            if let Some(out) = &pcap_out {
                                if let Err(err) = out.write_all(pkt.clone()) {
                                    eprintln!("{}", err);
                                    break 'out;
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            break;
                        }
                        Ok(false) => {}
                    }
                }
                count += 1;
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::UnexpectedEof {
                    eprintln!("{}", err);
                }
                break;
            }
        }
    }
    // Reset built-in variables for packets
    vm.update_builtin_var(BuiltinVarType::PL, Rc::new(Object::Null));
    vm.update_builtin_var(BuiltinVarType::WL, Rc::new(Object::Null));
    // Call the end filter
    if let Some(filter) = filter_end {
        if let Err(err) = vm.push_filter_frame(&filter) {
            eprintln!("{}", err);
            return;
        }
        if let Err(err) = vm.run() {
            eprintln!("{}", err);
            return;
        }
        // There is nothing to write to stdout for the end filter
        // as there is always an action specified for the end filter
        match vm.pop_filter_frame() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", err);
            }
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

fn init_builtin_vars(vm: &VM, args: Vec<String>) {
    let elements: Vec<Rc<Object>> = args.into_iter().map(|s| Rc::new(Object::Str(s))).collect();
    let arr = Rc::new(Object::Arr(Rc::new(Array::new(elements))));
    vm.update_builtin_var(BuiltinVarType::Argv, arr);
    vm.update_builtin_var(BuiltinVarType::NP, Rc::new(Object::Null));
    vm.update_builtin_var(BuiltinVarType::PL, Rc::new(Object::Null));
    vm.update_builtin_var(BuiltinVarType::WL, Rc::new(Object::Null));
    vm.update_builtin_var(BuiltinVarType::Tss, Rc::new(Object::Null));
    vm.update_builtin_var(BuiltinVarType::Tsu, Rc::new(Object::Null));
}
