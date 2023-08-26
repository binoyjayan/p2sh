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
        (Object::Arr(eval), Object::Arr(exp)) => {
            assert_eq!(
                eval.elements.len(),
                exp.elements.len(),
                "array object has wrong length. got={}, want={}",
                eval.elements.len(),
                exp.elements.len()
            );
            for (ex, ev) in exp.elements.iter().zip(eval.elements.iter()) {
                assert_eq!(ex, ev);
            }
        }
        (Object::Map(eval), Object::Map(exp)) => {
            assert_eq!(
                eval.pairs.len(),
                exp.pairs.len(),
                "map object has wrong length. got={}, want={}",
                eval.pairs.len(),
                exp.pairs.len()
            );
            assert_eq!(eval, exp);
        }
        (_, Object::Nil) => {
            assert_eq!(
                evaluated,
                Rc::new(Object::Nil),
                "object is not Nil. got={:?}",
                evaluated
            );
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
        let mut vm = VM::new(bytecode);
        let err = vm.run();
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
        VmTestCase {
            input: "-5",
            expected: Object::Number(-5.),
        },
        VmTestCase {
            input: "-10",
            expected: Object::Number(-10.),
        },
        VmTestCase {
            input: "-50 + 100 + -50",
            expected: Object::Number(0.),
        },
        VmTestCase {
            input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
            expected: Object::Number(50.),
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
        VmTestCase {
            input: "1 < 2",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 > 2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 < 1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 > 1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 == 1",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 != 1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 == 2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 != 2",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true == true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "false == false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true == false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "true != false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "false != true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "(1 < 2) == true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "(1 < 2) == false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "(1 > 2) == true",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "(1 > 2) == false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!true",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "!false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!5",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "!!true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!!false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "!!5",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "(10 + 50 + -5 - 5 * 2 / 2) < (100 - 35)",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!(if (false) { 5; })",
            expected: Object::Bool(true),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_conditionals() {
    let tests = vec![
        VmTestCase {
            input: "if (true) { 10 }",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "if (true) { 10 } else { 20 }",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "if (false) { 10 } else { 20 } ",
            expected: Object::Number(20.),
        },
        VmTestCase {
            input: "if (1) { 10 }",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "if (1 < 2) { 10 }",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "if (1 < 2) { 10 } else { 20 }",
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: "if (1 > 2) { 10 } else { 20 }",
            expected: Object::Number(20.),
        },
        VmTestCase {
            input: "if (1 > 2) { 10 }",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "if (false) { 10 }",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "if ((if (false) { 10 })) { 10 } else { 20 }",
            expected: Object::Number(20.),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_global_let_statements() {
    let tests = vec![
        VmTestCase {
            input: "let one = 1; one",
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: "let one = 1; let two = 2; one + two",
            expected: Object::Number(3.),
        },
        VmTestCase {
            input: "let one = 1; let two = one + one; one + two",
            expected: Object::Number(3.),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_string_expressions() {
    let tests = vec![
        VmTestCase {
            input: r#""p2sh""#,
            expected: Object::Str("p2sh".to_string()),
        },
        VmTestCase {
            input: r#""p2" + "sh""#,
            expected: Object::Str("p2sh".to_string()),
        },
        VmTestCase {
            input: r#""p2" + "sh" + "banana""#,
            expected: Object::Str("p2shbanana".to_string()),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_array_literals() {
    let tests = vec![
        VmTestCase {
            input: "[]",
            expected: Object::Arr(Array {
                elements: Vec::new(),
            }),
        },
        VmTestCase {
            input: "[1, 2, 3]",
            expected: Object::Arr(Array {
                elements: vec![
                    Rc::new(Object::Number(1.0)),
                    Rc::new(Object::Number(2.0)),
                    Rc::new(Object::Number(3.0)),
                ],
            }),
        },
        VmTestCase {
            input: "[1 + 2, 3 * 4, 5 + 6]",
            expected: Object::Arr(Array {
                elements: vec![
                    Rc::new(Object::Number(3.0)),
                    Rc::new(Object::Number(12.0)),
                    Rc::new(Object::Number(11.0)),
                ],
            }),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_hash_literals() {
    let tests = vec![
        VmTestCase {
            input: "{}",
            expected: Object::Map(HMap::default()),
        },
        VmTestCase {
            input: "{1: 2, 2: 3}",
            expected: Object::Map({
                let mut map = HMap::default();
                map.pairs.insert(
                    Rc::new(Object::Number(1.into())),
                    Rc::new(Object::Number(2.into())),
                );
                map.pairs.insert(
                    Rc::new(Object::Number(2.into())),
                    Rc::new(Object::Number(3.into())),
                );
                map
            }),
        },
        VmTestCase {
            input: "{1 + 1: 2 * 2, 3 + 3: 4 * 4}",
            expected: Object::Map({
                let mut map = HMap::default();
                map.pairs.insert(
                    Rc::new(Object::Number(2.into())),
                    Rc::new(Object::Number(4.into())),
                );
                map.pairs.insert(
                    Rc::new(Object::Number(6.into())),
                    Rc::new(Object::Number(16.into())),
                );
                map
            }),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        VmTestCase {
            input: "[1, 2, 3][1]",
            expected: Object::Number(2.),
        },
        VmTestCase {
            input: "[1, 2, 3][0 + 2]",
            expected: Object::Number(3.),
        },
        VmTestCase {
            input: "[[1, 1, 1]][0][0]",
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: "[][0]",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "[1, 2, 3][99]",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "[1][-1]",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "{1: 1, 2: 2}[1]",
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: "{1: 1, 2: 2}[2]",
            expected: Object::Number(2.),
        },
        VmTestCase {
            input: "{1: 1}[0]",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "{}[0]",
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"{"one": 1, "two": 2, "three": 3}["o" + "ne"]"#,
            expected: Object::Number(1.),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_calling_functions_without_args() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let fivePlusTen = fn() { 5 + 10; };
            fivePlusTen();
            "#,
            expected: Object::Number(15.),
        },
        VmTestCase {
            input: r#"
            let f1 = fn() { 1 };
            let f2 = fn() { 2 };
            f1() + f2()
            "#,
            expected: Object::Number(3.),
        },
        VmTestCase {
            input: r#"
            let f1 = fn() { 1 };
            let f2 = fn() { f1() + 1 };
            let f3 = fn() { f2() + 1 };
            f3()
            "#,
            expected: Object::Number(3.),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_calling_functions_with_return() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let earlyExit = fn() { return 99; 100; };
            earlyExit();
            "#,
            expected: Object::Number(99.),
        },
        VmTestCase {
            input: r#"
            let earlyExit = fn() { return 99; return 100; };
            earlyExit();
            "#,
            expected: Object::Number(99.),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_calling_functions_without_return() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let noReturn = fn() { };
            noReturn();
            "#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"
            let noReturn1 = fn() { };
            let noReturn2 = fn() { noReturn1(); };
            noReturn1();
            noReturn2();
            "#,
            expected: Object::Nil,
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_first_class_functions_() {
    let tests = vec![
        VmTestCase {
            input: r#"
                let returnsOne = fn() { 1; };
                let returnsOneReturner = fn() { returnsOne; };
                returnsOneReturner()();
            "#,
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: r#"
                let returnsOneReturner = fn() {
                let returnsOne = fn() { 1; };
                returnsOne;
                };
                returnsOneReturner()();
            "#,
            expected: Object::Number(1.),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_calling_functions_with_bindings() {
    let tests = vec![
        VmTestCase {
            input: r#"
                let one = fn() { let one = 1; one };
                one();
            "#,
            expected: Object::Number(1.),
        },
        VmTestCase {
            input: r#"
                let one_and_two = fn() { let one = 1; let two = 2; one + two; };
                one_and_two();
            "#,
            expected: Object::Number(3.),
        },
        VmTestCase {
            input: r#"
                let one_and_two = fn() { let one = 1; let two = 2; one + two; };
                let three_and_four = fn() { let three = 3; let four = 4; three + four; };
                one_and_two() + three_and_four();
            "#,
            expected: Object::Number(10.),
        },
        VmTestCase {
            input: r#"
                let first_foobar = fn() { let foobar = 50; foobar; };
                let second_foobar = fn() { let foobar = 100; foobar; };
                first_foobar() + second_foobar();
            "#,
            expected: Object::Number(150.),
        },
        VmTestCase {
            input: r#"
                let global_seed = 50;
                let minus_one = fn() {
                    let num = 1;
                    global_seed - num;
                };
                let minus_two = fn() {
                    let num = 2;
                    global_seed - num;
                };
                minus_one() + minus_two();
            "#,
            expected: Object::Number(97.),
        },
    ];
    run_vm_tests(&tests);
}
