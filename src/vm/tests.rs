#![allow(unused_imports)]
use std::rc::Rc;

use super::*;
use crate::common::object::*;
use crate::compiler::*;
use crate::evaluator::*;
use crate::parser::*;
use crate::scanner::*;
use crate::vm::interpreter::VM;

#[cfg(test)]
fn check_parse_errors(parser: &Parser) {
    if parser.print_errors() {
        panic!("{} parse errors", parser.parse_errors().len());
    }
}

#[cfg(test)]
fn test_expected_object(evaluated: Rc<Object>, expected: &Object) {
    match (evaluated.as_ref(), expected) {
        (Object::Number(eval), Object::Number(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong numeric value. got={}, want={}",
                eval, exp
            );
        }
        (Object::Str(eval), Object::Str(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong string value. got={}, want={}",
                eval, exp
            );
        }
        (Object::Bool(eval), Object::Bool(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong boolean value. got={}, want={}",
                eval, exp
            );
        }
        (Object::Nil, Object::Nil) => {
            panic!("object is not Nil. got={:?}", evaluated);
        }
        _ => {
            panic!(
                "invalid object types. got={:?}, want={:?}",
                evaluated, expected
            );
        }
    }
}

#[cfg(test)]
struct VmTestCase {
    input: &'static str,
    expected: Object,
}

#[cfg(test)]
fn test_compile(input: &str) -> Bytecode {
    use crate::compiler::{Bytecode, Compiler};

    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    check_parse_errors(&parser);
    let mut compiler = Compiler::new();
    if let Err(e) = compiler.compile(program) {
        panic!("Compilation error: {}", e);
    }
    compiler.bytecode()
}

#[cfg(test)]
fn run_vm_tests(tests: &[VmTestCase]) {
    for t in tests {
        let bytecode = test_compile(t.input);
        let mut vm = VM::new(bytecode.constants.clone());
        let err = vm.run(&bytecode.instructions.clone());
        if let Err(err) = err {
            panic!("vm error: {}", err);
        }
        // Get the object at the top of the VM's stack
        let stack_elem = vm.last_popped();
        test_expected_object(Rc::clone(&stack_elem), &t.expected);
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1",
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: "2",
            expected: Object::Number(2.),
        },
        VmTestCase {
            input: "1 + 2",
            expected: Object::Number(3.),
        },
        VmTestCase {
            input: "1 - 2",
            expected: Object::Number(-1.),
        },
        VmTestCase {
            input: "1 * 2",
            expected: Object::Number(2.),
        },
        VmTestCase {
            input: "4 / 2",
            expected: Object::Number(2.),
        },
        VmTestCase {
            input: "50 / 2 * 2 + 10 - 5",
            expected: Object::Number(55.),
        },
        VmTestCase {
            input: "5 + 5 + 5 + 5 - 10",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "2 * 2 * 2 * 2 * 2",
            expected: Object::Number(32.),
        },
        VmTestCase {
            input: "5 * 2 + 10",
            expected: Object::Number(20.),
        },
        VmTestCase {
            input: "5 + 2 * 10",
            expected: Object::Number(25.),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Object::Number(60.),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_boolean_expressions() {
    let tests = vec![
        VmTestCase {
            input: "true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "false",
            expected: Object::Bool(false),
        },
    ];

    run_vm_tests(&tests);
}
