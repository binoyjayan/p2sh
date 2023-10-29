#![allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::common::builtins::variables::BuiltinVarType;
use crate::common::object::*;
use crate::compiler::*;
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
        (Object::Integer(eval), Object::Integer(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong integer value. got={}, want={}",
                eval, exp
            );
        }
        (Object::Float(eval), Object::Float(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong floating value. got={}, want={}",
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
                eval.len(),
                exp.len(),
                "array object has wrong length. got={}, want={}",
                eval.len(),
                exp.len()
            );
            for (ex, ev) in exp
                .elements
                .borrow()
                .iter()
                .zip(eval.elements.borrow().iter())
            {
                assert_eq!(ex, ev);
            }
        }
        (Object::Map(eval), Object::Map(exp)) => {
            assert_eq!(
                eval.len(),
                exp.len(),
                "map object has wrong length. got={}, want={}",
                eval.len(),
                exp.len()
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
struct VmTestCaseErr {
    input: &'static str,
    expected: &'static str,
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
    for (i, t) in tests.iter().enumerate() {
        let bytecode = test_compile(t.input);
        let mut vm = VM::new(bytecode);
        let arr = Rc::new(Object::Arr(Rc::new(Array::new(Vec::new()))));
        vm.update_builtin_var(BuiltinVarType::Argv, arr);
        let err = vm.run();
        if let Err(err) = err {
            panic!("Test [{}] vm error: {}", i, err);
        }
        // Get the object at the top of the VM's stack
        let stack_elem = vm.last_popped();
        test_expected_object(Rc::clone(&stack_elem), &t.expected);
    }
}

#[cfg(test)]
fn run_vm_negative_tests(tests: &[VmTestCaseErr]) {
    for (i, t) in tests.iter().enumerate() {
        let bytecode = test_compile(t.input);
        let mut vm = VM::new(bytecode);
        let err = vm.run();
        if let Err(err) = err {
            assert_eq!(err.msg, t.expected, "Test {}", i);
        }
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "2",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "1 + 2",
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: "1 - 2",
            expected: Object::Integer(-1),
        },
        VmTestCase {
            input: "1 * 2",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "4 / 2",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "10 % 3",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "50 / 2 * 2 + 10 - 5",
            expected: Object::Integer(55),
        },
        VmTestCase {
            input: "5 + 5 + 5 + 5 - 10",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "2 * 2 * 2 * 2 * 2",
            expected: Object::Integer(32),
        },
        VmTestCase {
            input: "5 * 2 + 10",
            expected: Object::Integer(20),
        },
        VmTestCase {
            input: "5 + 2 * 10",
            expected: Object::Integer(25),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Object::Integer(60),
        },
        VmTestCase {
            input: "-5",
            expected: Object::Integer(-5),
        },
        VmTestCase {
            input: "-10",
            expected: Object::Integer(-10),
        },
        VmTestCase {
            input: "-50 + 100 + -50",
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
            expected: Object::Integer(50),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_floating_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1.",
            expected: Object::Float(1.),
        },
        VmTestCase {
            input: "int(1.5 + 2.5)",
            expected: Object::Integer(4),
        },
        VmTestCase {
            input: "int(1.5 - 2.5)",
            expected: Object::Integer(-1),
        },
        VmTestCase {
            input: "int(1.5 * 2.5)",
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: "int(1.5 / 2.5)",
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: "int(50. / 2. * 2. + 10. - 5.)",
            expected: Object::Integer(55),
        },
        VmTestCase {
            input: "int(5. + 5. + 5. + 5. - 10.)",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "int(2. * 2. * 2. * 2. * 2.)",
            expected: Object::Integer(32),
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
            input: "1.1 < 2.2",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1.1 > 2.2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1.1 < 1.1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1.1 > 1.1",
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
            input: "!(if false { 5; })",
            expected: Object::Bool(true),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_conditionals() {
    let tests = vec![
        VmTestCase {
            input: "if true { 10 }",
            expected: Object::Integer(10),
        },
        // condition enclosed in parentheses
        VmTestCase {
            input: "if (true) { 10 }",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "if true { 10 } else { 20 }",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "if false { 10 } else { 20 } ",
            expected: Object::Integer(20),
        },
        VmTestCase {
            input: "if 1 { 10 }",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "if 1 < 2 { 10 }",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "if 1 < 2 { 10 } else { 20 }",
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: "if 1 > 2 { 10 } else { 20 }",
            expected: Object::Integer(20),
        },
        VmTestCase {
            input: "if 1 > 2 { 10 }",
            expected: Object::Nil,
        },
        VmTestCase {
            input: "if false { 10 }",
            expected: Object::Nil,
        },
        // nested conditionals
        VmTestCase {
            input: "if (if (false) { 10 }) { 10 } else { 20 }",
            expected: Object::Integer(20),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_global_let_statements() {
    let tests = vec![
        VmTestCase {
            input: "let one = 1; one",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "let one = 1; let two = 2; one + two",
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: "let one = 1; let two = one + one; one + two",
            expected: Object::Integer(3),
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
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(Vec::new()),
            })),
        },
        VmTestCase {
            input: "[1, 2, 3]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ]),
            })),
        },
        VmTestCase {
            input: "[1 + 2, 3 * 4, 5 + 6]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(3)),
                    Rc::new(Object::Integer(12)),
                    Rc::new(Object::Integer(11)),
                ]),
            })),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_array_operations() {
    let tests = vec![
        VmTestCase {
            input: "[] + []",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(Vec::new()),
            })),
        },
        VmTestCase {
            input: "[] + [1]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![Rc::new(Object::Integer(1))]),
            })),
        },
        VmTestCase {
            input: "[1] + [2]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                ]),
            })),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_hash_literals() {
    let tests = vec![
        VmTestCase {
            input: "{}",
            expected: Object::Map(Rc::new(HMap::default())),
        },
        VmTestCase {
            input: "{1: 2, 2: 3}",
            expected: Object::Map({
                let map = HMap::default();
                map.insert(Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2)));
                map.insert(Rc::new(Object::Integer(2)), Rc::new(Object::Integer(3)));
                Rc::new(map)
            }),
        },
        VmTestCase {
            input: "{1 + 1: 2 * 2, 3 + 3: 4 * 4}",
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Integer(2.into())),
                    Rc::new(Object::Integer(4.into())),
                );
                map.pairs.borrow_mut().insert(
                    Rc::new(Object::Integer(6.into())),
                    Rc::new(Object::Integer(16.into())),
                );
                Rc::new(map)
            }),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_hash_literals_expressions_negative() {
    let tests = vec![
        VmTestCaseErr {
            input: "{fn() {}: 2}",
            expected: "KeyError: not a valid key: <closure>.",
        },
        VmTestCaseErr {
            input: "{fn() {}: fn() {}}",
            expected: "KeyError: not a valid key: <closure>.",
        },
    ];
    run_vm_negative_tests(&tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        VmTestCase {
            input: "{1: 1, 2: 2}[1]",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "{1: 1, 2: 2}[2]",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "{1.1: 1, 2.2: 2}[2.2]",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: r#"{"one": 1, "two": 2, "three": 3}["o" + "ne"]"#,
            expected: Object::Integer(1),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_index_expressions_negative() {
    let tests = vec![
        VmTestCaseErr {
            input: "[][0]",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "{1: 1}[0]",
            expected: "KeyError: key not found.",
        },
        VmTestCaseErr {
            input: "{}[fn(){}]",
            expected: "KeyError: not a valid key: <closure>.",
        },
        VmTestCaseErr {
            input: "{}[0]",
            expected: "KeyError: key not found.",
        },
        VmTestCaseErr {
            input: "[1, 2, 3][99]",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "[1, 2, 3][1]",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "[1][-1]",
            expected: "IndexError: index cannot be negative.",
        },
        VmTestCaseErr {
            input: "[[1, 1, 1]][0][0]",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "[1, 2, 3][0 + 2]",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "[][fn(){}]",
            expected: "IndexError: unsupported operation.",
        },
    ];
    run_vm_negative_tests(&tests);
}

#[test]
fn test_calling_functions_without_args() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let fivePlusTen = fn() { 5 + 10; };
            fivePlusTen();
            "#,
            expected: Object::Integer(15),
        },
        VmTestCase {
            input: r#"
            let f1 = fn() { 1 };
            let f2 = fn() { 2 };
            f1() + f2()
            "#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"
            let f1 = fn() { 1 };
            let f2 = fn() { f1() + 1 };
            let f3 = fn() { f2() + 1 };
            f3()
            "#,
            expected: Object::Integer(3),
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
            expected: Object::Integer(99),
        },
        VmTestCase {
            input: r#"
            let earlyExit = fn() { return 99; return 100; };
            earlyExit();
            "#,
            expected: Object::Integer(99),
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
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"
                let returnsOneReturner = fn() {
                let returnsOne = fn() { 1; };
                returnsOne;
                };
                returnsOneReturner()();
            "#,
            expected: Object::Integer(1),
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
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"
                let one_and_two = fn() { let one = 1; let two = 2; one + two; };
                one_and_two();
            "#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"
                let one_and_two = fn() { let one = 1; let two = 2; one + two; };
                let three_and_four = fn() { let three = 3; let four = 4; three + four; };
                one_and_two() + three_and_four();
            "#,
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: r#"
                let first_foobar = fn() { let foobar = 50; foobar; };
                let second_foobar = fn() { let foobar = 100; foobar; };
                first_foobar() + second_foobar();
            "#,
            expected: Object::Integer(150),
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
            expected: Object::Integer(97),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_calling_functions_with_args_and_bindings() {
    let tests = vec![
        VmTestCase {
            input: r#"
                let identity = fn(a) { a; };
                identity(4);
            "#,
            expected: Object::Integer(4),
        },
        VmTestCase {
            input: r#"
                let sum = fn(a, b) { a + b; };
                sum(1, 2);
            "#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"
                let sum = fn(a, b) { let c = a + b; c; };
                sum(1, 2);
            "#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"
                let sum = fn(a, b) { let c = a + b; c; };
                sum(1, 2) + sum(3, 4);
            "#,
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: r#"
                let sum = fn(a, b) { let c = a + b; c; };
                let outer = fn() { sum(1, 2) + sum(3, 4); };
                outer();
            "#,
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: r#"
                let globalNum = 10;
                let sum = fn(a, b) {
                    let c = a + b;
                    c + globalNum;
                };
                let outer = fn() {
                    sum(1, 2) + sum(3, 4) + globalNum;
                };
                outer() + globalNum;
            "#,
            expected: Object::Integer(50),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_wrong_number_of_arguments() {
    let tests: Vec<VmTestCaseErr> = vec![
        VmTestCaseErr {
            input: "fn() { 1; }(1);",
            expected: "wrong number of arguments: want=0, got=1",
        },
        VmTestCaseErr {
            input: "fn(a) { a; }();",
            expected: "wrong number of arguments: want=1, got=0",
        },
        VmTestCaseErr {
            input: "fn(a, b) { a + b; }(1);",
            expected: "wrong number of arguments: want=2, got=1",
        },
    ];

    run_vm_negative_tests(&tests);
}

#[test]
fn test_builtin_functions() {
    let tests = vec![
        VmTestCase {
            input: r#"len("")"#,
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: r#"len("four")"#,
            expected: Object::Integer(4),
        },
        VmTestCase {
            input: r#"len("hello world")"#,
            expected: Object::Integer(11),
        },
        VmTestCase {
            input: r#"len([1, 2, 3])"#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"len([])"#,
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: r#"len({})"#,
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: r#"len({"a": 1, "b": 2})"#,
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: r#"first([1, 2, 3])"#,
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"first([])"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"last([1, 2, 3])"#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"last([])"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"rest([1, 2, 3])"#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ]),
            })),
        },
        VmTestCase {
            input: r#"rest([])"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"let a = []; push(a, 1); a"#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![Rc::new(Object::Integer(1))]),
            })),
        },
        VmTestCase {
            input: r#"
                let array = [1, 2, 3];
                push(array, 4);
                first(rest(array));
            "#,
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: r#"contains({}, "k")"#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: r#"let m = {"k1": 1, "k": 0, "k2": 2}; contains(m, "k")"#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"get({}, "k")"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"let m = {"k1": 999, "k2": 888, "k3": 777}; get(m, "k2")"#,
            expected: Object::Integer(888),
        },
        VmTestCase {
            input: r#"let m = {"k1": 999, "k2": 888, "k3": 777}; get(m, "k")"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"let m = {}; insert(m, "k", 1)"#,
            // insert returns Nil if key was not found already
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"let m = {"k1": 999, "k2": 888}; insert(m, "k3", 777)"#,
            // insert returns Nil if key was not found already
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"let m = {"k1": 999, "k2": 888}; insert(m, "k2", 777)"#,
            // insert returns the previous value if key was found
            expected: Object::Integer(888),
        },
        VmTestCase {
            input: r#"let m = {}; insert(m, "k", 1); m"#,
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Str("k".to_string())),
                    Rc::new(Object::Integer(1)),
                );
                Rc::new(map)
            }),
        },
        VmTestCase {
            // rest([]) is Nil
            input: "str(rest([]))",
            expected: Object::Str(String::from("nil")),
        },
        VmTestCase {
            input: r#"str("hello")"#,
            expected: Object::Str(String::from("hello")),
        },
        VmTestCase {
            input: "str(999)",
            expected: Object::Str(String::from("999")),
        },
        VmTestCase {
            input: "str(str(999))",
            expected: Object::Str(String::from("999")),
        },
        VmTestCase {
            input: "str(true)",
            expected: Object::Str(String::from("true")),
        },
        VmTestCase {
            input: "str([1, 2, 3333, 4, 5])",
            expected: Object::Str(String::from("[1, 2, 3333, 4, 5]")),
        },
        VmTestCase {
            // maps are unordered so use only a single item
            input: r#"str({"a": 1})"#,
            expected: Object::Str(String::from(r#"{"a": 1}"#)),
        },
        VmTestCase {
            input: "str({})",
            expected: Object::Str(String::from("{}")),
        },
        VmTestCase {
            input: r#"int("999")"#,
            expected: Object::Integer(999),
        },
        VmTestCase {
            input: r#"int(999.99)"#,
            expected: Object::Integer(999),
        },
        VmTestCase {
            input: "int(str(999))",
            expected: Object::Integer(999),
        },
        VmTestCase {
            input: "int(true)",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "int(false)",
            expected: Object::Integer(0),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_display() {
    let tests = vec![
        VmTestCase {
            input: r#"puts("hello", "world!")"#,
            expected: Object::Nil,
        },
        VmTestCase {
            input: r#"print("Hello, World!")"#,
            expected: Object::Integer(13),
        },
        VmTestCase {
            input: r#"println("Hello, World!")"#,
            expected: Object::Integer(14),
        },
        VmTestCase {
            input: r#"
                format("{0:<10},{1:0>5},{2},{3:b},{3:o},{4:x},{4:X}",
                "Hello", 1, true, 10, 65535
            )"#,
            expected: Object::Str("Hello     ,00001,true,1010,12,ffff,FFFF".to_string()),
        },
        VmTestCase {
            input: r#"
                print("{0:<10},{1:0>5},{2},{3:b},{3:o},{4:x},{4:X}",
                "Hello", 1, true, 10, 65535
            )"#,
            expected: Object::Integer(39),
        },
        VmTestCase {
            input: r#"
                println("{0:<10},{1:0>5},{2},{3:b},{3:o},{4:x},{4:X}",
                "Hello", 1, true, 10, 65535
            )"#,
            expected: Object::Integer(40),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_function_failures() {
    let tests: Vec<VmTestCaseErr> = vec![
        VmTestCaseErr {
            input: r#"len(1)"#,
            expected: "len: unsupported argument",
        },
        VmTestCaseErr {
            input: r#"len("one", "two")"#,
            expected: "len: takes one argument. got=2",
        },
        VmTestCaseErr {
            input: r#"first(1)"#,
            expected: "first: unsupported argument",
        },
        VmTestCaseErr {
            input: r#"last(1)"#,
            expected: "last: unsupported argument",
        },
        VmTestCaseErr {
            input: r#"push(1, 1)"#,
            expected: "push: unsupported argument",
        },
        VmTestCaseErr {
            input: "str(fn() {})",
            expected: "str: unsupported argument",
        },
        VmTestCaseErr {
            input: "int([])",
            expected: "int: unsupported argument",
        },
        VmTestCaseErr {
            input: "int({})",
            expected: "int: unsupported argument",
        },
        VmTestCaseErr {
            input: r#"int("1.11")"#,
            expected: "int: failed to parse string into an int",
        },
        VmTestCaseErr {
            input: r#"float("a")"#,
            expected: "float: failed to parse string into a float",
        },
    ];

    run_vm_negative_tests(&tests);
}

#[test]
fn test_builtin_variables() {
    // run_vm_tests passes empty argv to the vm
    let tests = vec![
        VmTestCase {
            input: r#"argv"#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(Vec::new()),
            })),
        },
        VmTestCase {
            input: r#"len(argv)"#,
            expected: Object::Integer(0),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_closures() {
    let tests: Vec<VmTestCase> = vec![
        // newClosure returns a closure that closes over a free variable,
        // the parameter 'a' of newClosure
        VmTestCase {
            input: r#"
                let newClosure = fn(a) {
                    fn() { a; }
                }
                let closure = newClosure(99);
                closure();
                "#,
            expected: Object::Integer(99),
        },
        VmTestCase {
            input: r#"
                let newAdder = fn(a, b) {
                    fn(c) { a + b + c; }
                }
                let adder = newAdder(1, 2);
                adder(8);
                "#,
            expected: Object::Integer(11),
        },
        VmTestCase {
            input: r#"
                let newAdder = fn(a, b) {
                    let c = a + b;
                    fn(d) { c + d; }
                }
                let adder = newAdder(1, 2);
                adder(8);
                "#,
            expected: Object::Integer(11),
        },
        VmTestCase {
            input: r#"
                let newAdderOuter = fn(a, b) {
                    let c = a + b;
                    fn(d) {
                        let e = d + c;
                        fn(f) { e + f; };
                    };
                };
                let newAdderInner = newAdderOuter(1, 2);
                let adder = newAdderInner(3);
                adder(8);
                "#,
            expected: Object::Integer(14),
        },
        VmTestCase {
            input: r#"
                let a = 1;
                let newAdderOuter = fn(b) {
                    fn(c) {
                        fn(d) { a + b + c + d };
                    };
                };
                let newAdderInner = newAdderOuter(2);
                let adder = newAdderInner(3);
                adder(8);
                "#,
            expected: Object::Integer(14),
        },
        VmTestCase {
            input: r#"
                let newClosure = fn(a, b) {
                    let one = fn() { a; };
                    let two = fn() { b; };
                    fn() { one() + two(); };
                };
                let closure = newClosure(9, 90);
                closure();
                "#,
            expected: Object::Integer(99),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_recursive_functions() {
    let tests: Vec<VmTestCase> = vec![
        VmTestCase {
            input: r#"
                let countDown = fn(x) {
                    if x == 0 {
                        return 0;
                    } else {
                        countDown(x - 1);
                    }
                };
                countDown(1);
                "#,
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: r#"
                let countDown = fn(x) {
                    if x == 0 {
                        return 0;
                    } else {
                        countDown(x - 1);
                    }
                };
                let wrapper = fn() {
                    countDown(1);
                };
                wrapper();
            "#,
            expected: Object::Integer(0),
        },
        // define a recursive function inside another function and also
        // call it inside this other function.
        VmTestCase {
            input: r#"
                let wrapper = fn() {
                    let countDown = fn(x) {
                        if x == 0 {
                            return 0;
                        } else {
                            countDown(x - 1);
                        }
                    };
                    countDown(1);
                };
                wrapper();
            "#,
            expected: Object::Integer(0),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_recursive_fibonacci() {
    let tests: Vec<VmTestCase> = vec![VmTestCase {
        input: r#"
                let fibonacci = fn(x) {
                    if x == 0 {
                        return 0;
                    } else {
                        if x == 1 {
                            return 1;
                        } else {
                            fibonacci(x - 1) + fibonacci(x - 2);
                        }
                    }
                };
                fibonacci(15);
                "#,
        expected: Object::Integer(610),
    }];

    run_vm_tests(&tests);
}

#[test]
fn test_assignment_expressions() {
    let tests: Vec<VmTestCase> = vec![
        VmTestCase {
            input: "let a = 1; a = 2;",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "let a = 1; a = 2; a",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "let a = true; a = false; a",
            expected: Object::Bool(false),
        },
        // Assignment expressions evaluate to the value assigned
        // Use assignment expression enclosed in parantheses to avoid ambiguity
        VmTestCase {
            input: r#" let a = 0; if (a = 1) { "then" } else { "else" } "#,
            expected: Object::Str("then".to_string()),
        },
        // If expression evaluates to falsey, since 0 is falsey.
        VmTestCase {
            input: r#" let a = 1; if (a = 0) { "then" } else { "else" } "#,
            expected: Object::Str("else".to_string()),
        },
        VmTestCase {
            input: "let a = [1]; a = [2]; a",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![Rc::new(Object::Integer(2))]),
            })),
        },
        VmTestCase {
            input: r#"let m = {"a": 1}; m = {"a": 222}; m"#,
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Str("a".into())),
                    Rc::new(Object::Integer(222)),
                );
                Rc::new(map)
            }),
        },
        VmTestCase {
            input: "let a = 0; let b = 0; a = b = 222; [a, b]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(222)),
                    Rc::new(Object::Integer(222)),
                ]),
            })),
        },
        VmTestCase {
            input: "let a = [0]; let b = [0]; a[0] = b[0] = 222; [a[0], b[0]]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(222)),
                    Rc::new(Object::Integer(222)),
                ]),
            })),
        },
        VmTestCase {
            input: "let a = [0, 0]; a[0] = a[1] = 222; [a[0], a[0]]",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(222)),
                    Rc::new(Object::Integer(222)),
                ]),
            })),
        },
        VmTestCase {
            input: r#"let m = {}; m[0] = m[1] = 222; [m[0], m[1]]"#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(222)),
                    Rc::new(Object::Integer(222)),
                ]),
            })),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_set_index_assignment_expressions() {
    let tests: Vec<VmTestCase> = vec![
        VmTestCase {
            input: "let a = [1, 2, 3]; a[0] = 111; a[1] = 222; a[2] = 333; a",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(111)),
                    Rc::new(Object::Integer(222)),
                    Rc::new(Object::Integer(333)),
                ]),
            })),
        },
        VmTestCase {
            input: r#"
                let mul = fn(a, idx, val) {
                    a[idx] = a[idx] * val
                }
                let a = [1, 2, 3];
                mul(a, 0, 10);
                mul(a, 1, 100);
                mul(a, 2, 1000);
                a
            "#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(10)),
                    Rc::new(Object::Integer(200)),
                    Rc::new(Object::Integer(3000)),
                ]),
            })),
        },
        VmTestCase {
            input: r#"let m = {}; m["a"] = 1; m"#,
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Str("a".into())),
                    Rc::new(Object::Integer(1)),
                );
                Rc::new(map)
            }),
        },
        VmTestCase {
            input: r#"let m = {"a": 1}; m["a"] = 111; m"#,
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Str("a".into())),
                    Rc::new(Object::Integer(111)),
                );
                Rc::new(map)
            }),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_set_index_assignment_expressions_negative() {
    let tests: Vec<VmTestCaseErr> = vec![
        VmTestCaseErr {
            input: "let a = []; a[0] = 1;",
            expected: "IndexError: array index out of range.",
        },
        VmTestCaseErr {
            input: "let a = [0]; a[-1] = 1;",
            expected: "IndexError: index cannot be negative.",
        },
        VmTestCaseErr {
            input: "let a = [1, 2, 3]; a[4] = 4;",
            expected: "IndexError: array index out of range.",
        },
    ];

    run_vm_negative_tests(&tests);
}

#[test]
fn test_set_index_assignment_expressions_free_variables() {
    let tests: Vec<VmTestCase> = vec![
        // newClosure returns a closure that closes over a free variable,
        // the parameter 'a' of newClosure
        VmTestCase {
            input: r#"
                let newClosure = fn(a) {
                    fn() { a = 99; a; }
                }
                let closure = newClosure(88);
                closure();
                "#,
            expected: Object::Integer(99),
        },
        VmTestCase {
            input: r#"
                let newAdder = fn(a, b) {
                    fn(c) {
                        a = a + 10; b = b + 20;
                        a + b + c;
                    }
                }
                let adder = newAdder(1, 2);
                adder(7);
                "#,
            expected: Object::Integer(40),
        },
    ];

    run_vm_tests(&tests);
}
