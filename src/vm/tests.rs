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
        (Object::Char(eval), Object::Char(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong char value. got={}, want={}",
                eval, exp
            );
        }
        (Object::Byte(eval), Object::Byte(exp)) => {
            assert_eq!(
                eval, exp,
                "object has wrong byte value. got={}, want={}",
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
        (_, Object::Null) => {
            assert_eq!(
                evaluated,
                Rc::new(Object::Null),
                "object is not Null. got={:?}",
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
        println!("[{}] VM Test", i);
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
fn test_mixed_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1. + 2",
            expected: Object::Float(3.),
        },
        VmTestCase {
            input: "1 + 2.",
            expected: Object::Float(3.),
        },
        VmTestCase {
            input: "1 + b'2'",
            expected: Object::Integer(51),
        },
        VmTestCase {
            input: "b'1' - 2",
            expected: Object::Integer(47),
        },
        VmTestCase {
            input: "1. + b'2'",
            expected: Object::Float(51.),
        },
        VmTestCase {
            input: "b'1' - 2.",
            expected: Object::Float(47.),
        },
        VmTestCase {
            input: "1. + 2.",
            expected: Object::Float(3.),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_division_by_zero() {
    let tests = vec![
        VmTestCaseErr {
            input: "1 / 0",
            expected: "Division by zero.",
        },
        VmTestCaseErr {
            input: "1. / 0",
            expected: "Division by zero.",
        },
    ];
    run_vm_negative_tests(&tests);
}

#[test]
fn test_unary_expressions() {
    let tests = vec![
        VmTestCase {
            input: "-10",
            expected: Object::Integer(-10),
        },
        VmTestCase {
            input: "-10.2",
            expected: Object::Float(-10.2),
        },
        VmTestCase {
            input: "!2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "~10",
            expected: Object::Integer(-11),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_unary_expressions_negative() {
    let tests = vec![
        VmTestCaseErr {
            input: r#"-"a""#,
            expected: "bad operand type for unary '-'",
        },
        VmTestCaseErr {
            input: r#"~"a""#,
            expected: "bad operand type for unary '~'",
        },
        VmTestCaseErr {
            input: r#"-'a'"#,
            expected: "bad operand type for unary '-'",
        },
        VmTestCaseErr {
            input: r#"-b'a'"#,
            expected: "bad operand type for unary '-'",
        },
        VmTestCaseErr {
            input: r#"~'a'"#,
            expected: "bad operand type for unary '~'",
        },
        VmTestCaseErr {
            input: r#"~b'a'"#,
            expected: "bad operand type for unary '~'",
        },
    ];
    run_vm_negative_tests(&tests);
}

#[test]
fn test_bitwise_operators() {
    let tests = vec![
        VmTestCase {
            input: "10 & 12",
            expected: Object::Integer(8),
        },
        VmTestCase {
            input: "10 | 12",
            expected: Object::Integer(14),
        },
        VmTestCase {
            input: "10 ^ 12",
            expected: Object::Integer(6),
        },
        VmTestCase {
            input: "10 << 2",
            expected: Object::Integer(40),
        },
        VmTestCase {
            input: "10 >> 2",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "((7 | (12 ^ 5)) & (9 << 2)) >> 2",
            expected: Object::Integer(1),
        },
    ];

    run_vm_tests(&tests);
}

#[test]
fn test_bitwise_operators_negative() {
    let tests = vec![
        VmTestCaseErr {
            input: "1.1 & 2.1",
            expected: "Invalid bitwise operation.",
        },
        VmTestCaseErr {
            input: "10 << 2.",
            expected: "Invalid bitwise operation.",
        },
        VmTestCaseErr {
            input: "10. >> 2",
            expected: "Invalid bitwise operation.",
        },
    ];
    run_vm_negative_tests(&tests);
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
            input: "1 <= 2",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 > 2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 >= 2",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 < 1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 <= 1",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 > 1",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 >= 1",
            expected: Object::Bool(true),
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
            input: "!null == true",
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
            input: "!!null",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "!!5",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!!'a'",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!b'a'",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "(10 + 50 + -5 - 5 * 2 / 2) < (100 - 35)",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "!(if false { 5; })",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#""a" < "a""#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: r#""a" > "a""#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: r#""a" <= "a""#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#""a" >= "a""#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"'a' <= 'a'"#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"b'a' < b'a'"#,
            expected: Object::Bool(false),
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
            expected: Object::Null,
        },
        VmTestCase {
            input: "if true { }",
            expected: Object::Null,
        },
        VmTestCase {
            input: "if false { 10 }",
            expected: Object::Null,
        },
        VmTestCase {
            input: "if false { } else { }",
            expected: Object::Null,
        },
        // nested conditionals
        VmTestCase {
            input: "if (if false { 10 }) { 10 } else { 20 }",
            expected: Object::Integer(20),
        },
        VmTestCase {
            input: "if 1 > 2 { 10 } else if 2 > 3 { 20 } else { 30 }",
            expected: Object::Integer(30),
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
fn test_char_and_byte_expressions() {
    let tests = vec![
        VmTestCase {
            input: "'c'",
            expected: Object::Char('c'),
        },
        VmTestCase {
            input: "b'c'",
            expected: Object::Byte(b'c'),
        },
        VmTestCase {
            input: "'1' + '2'",
            expected: Object::Str("12".to_string()),
        },
        VmTestCase {
            input: "b'1' + b'2'",
            expected: Object::Byte(99),
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
            input: "map {}",
            expected: Object::Map(Rc::new(HMap::default())),
        },
        VmTestCase {
            input: "map {1: 2, 2: 3}",
            expected: Object::Map({
                let map = HMap::default();
                map.insert(Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2)));
                map.insert(Rc::new(Object::Integer(2)), Rc::new(Object::Integer(3)));
                Rc::new(map)
            }),
        },
        VmTestCase {
            input: "map {1 + 1: 2 * 2, 3 + 3: 4 * 4}",
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
            input: "map {fn() {}: 2}",
            expected: "KeyError: not a valid key: <closure>.",
        },
        VmTestCaseErr {
            input: "map {fn() {}: fn() {}}",
            expected: "KeyError: not a valid key: <closure>.",
        },
    ];
    run_vm_negative_tests(&tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        VmTestCase {
            input: "map {1: 1, 2: 2}[1]",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "map {1: 1, 2: 2}[2]",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "map {1.1: 1, 2.2: 2}[2.2]",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: r#"map {"one": 1, "two": 2, "three": 3}["o" + "ne"]"#,
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: "map {'a': 1, 'b': 2, 'c': 3}['b']",
            expected: Object::Integer(2),
        },
        VmTestCase {
            input: "map {b'a': 1, b'b': 2, b'c': 3}[b'b']",
            expected: Object::Integer(2),
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
            input: "map {1: 1}[0]",
            expected: "KeyError: key not found.",
        },
        VmTestCaseErr {
            input: "map {}[fn(){}]",
            expected: "KeyError: not a valid key: <closure>.",
        },
        VmTestCaseErr {
            input: "map {}[0]",
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
            let earlyExit = fn() { return; 100; };
            earlyExit();
            "#,
            expected: Object::Null,
        },
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
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"
            let noReturn1 = fn() { };
            let noReturn2 = fn() { noReturn1(); };
            noReturn1();
            noReturn2();
            "#,
            expected: Object::Null,
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
fn test_builtin_function_len() {
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
            input: r#"len(map {})"#,
            expected: Object::Integer(0),
        },
        VmTestCase {
            input: r#"len(map {"a": 1, "b": 2})"#,
            expected: Object::Integer(2),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_arrays() {
    let tests = vec![
        VmTestCase {
            input: r#"first([1, 2, 3])"#,
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"first([])"#,
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"last([1, 2, 3])"#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"last([])"#,
            expected: Object::Null,
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
            expected: Object::Null,
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
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_maps() {
    let tests = vec![
        VmTestCase {
            input: r#"contains(map {}, "k")"#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: r#"let m = map {"k1": 1, "k": 0, "k2": 2}; contains(m, "k")"#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"get(map {}, "k")"#,
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"let m = map {"k1": 999, "k2": 888, "k3": 777}; get(m, "k2")"#,
            expected: Object::Integer(888),
        },
        VmTestCase {
            input: r#"let m = map {"k1": 999, "k2": 888, "k3": 777}; get(m, "k")"#,
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"let m = map {}; insert(m, "k", 1)"#,
            // insert returns Null if key was not found already
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"let m = map {"k1": 999, "k2": 888}; insert(m, "k3", 777)"#,
            // insert returns Null if key was not found already
            expected: Object::Null,
        },
        VmTestCase {
            input: r#"let m = map {"k1": 999, "k2": 888}; insert(m, "k2", 777)"#,
            // insert returns the previous value if key was found
            expected: Object::Integer(888),
        },
        VmTestCase {
            input: r#"let m = map {}; insert(m, "k", 1); m"#,
            expected: Object::Map({
                let map = HMap::default();
                map.insert(
                    Rc::new(Object::Str("k".to_string())),
                    Rc::new(Object::Integer(1)),
                );
                Rc::new(map)
            }),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_conversions() {
    let tests = vec![
        VmTestCase {
            // rest([]) is Null
            input: "str(rest([]))",
            expected: Object::Str(String::from("null")),
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
            input: r#"str(map {"a": 1})"#,
            expected: Object::Str(String::from(r#"map {"a": 1}"#)),
        },
        VmTestCase {
            input: "str(map {})",
            expected: Object::Str(String::from("map {}")),
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
        VmTestCase {
            input: "int(b'1')",
            expected: Object::Integer(49),
        },
        VmTestCase {
            input: "int('1')",
            expected: Object::Integer(49),
        },
        VmTestCase {
            input: "float(b'1')",
            expected: Object::Float(49.),
        },
        VmTestCase {
            input: "float('1')",
            expected: Object::Float(49.),
        },
        VmTestCase {
            input: r#"str(b'1')"#,
            expected: Object::Str(String::from("49")),
        },
        VmTestCase {
            input: r#"str('1')"#,
            expected: Object::Str(String::from("1")),
        },
        VmTestCase {
            input: "char(b'1')",
            expected: Object::Char('1'),
        },
        VmTestCase {
            input: "byte('1')",
            expected: Object::Byte(b'1'),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_math() {
    let tests = vec![VmTestCase {
        input: r#"round(3.141592653589793238, 2)"#,
        expected: Object::Float(3.14),
    }];
    run_vm_tests(&tests);
}

#[test]
fn test_builtin_functions_display() {
    let tests = vec![
        VmTestCase {
            input: r#"puts("hello", "world!")"#,
            expected: Object::Null,
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
fn test_builtin_functions_file_io() {
    // Preserve the sequence of tests here
    let tests = vec![
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt", "w");
              write(f, "Hey")
            "#,
            expected: Object::Integer(3),
        },
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt", "T");
              write(f, "Hello")
            "#,
            expected: Object::Integer(5),
        },
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt", "a");
              let n1 = write(f, [b' ', b'W', b'o', b'r', b'l', b'd', b'!']);
              let n2 = write(f, byte(10));
              n1 + n2
            "#,
            expected: Object::Integer(8),
        },
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt");
              let a = read(f, 5);
              decode_utf8(a)
            "#,
            expected: Object::Str(String::from("Hello")),
        },
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt");
              let a = read(f);
              decode_utf8(a)
            "#,
            expected: Object::Str(String::from("Hello World!\n")),
        },
        VmTestCase {
            input: r#"
              let n = 0;
              let f = open("/tmp/__p2sh_test.txt");
              loop {
                let a = read(f, 1);
                if len(a) < 1 {
                    break;
                }
                n = n + 1;
              }
              n
            "#,
            expected: Object::Integer(13),
        },
        // Write without truncation
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt", "w");
              write(f, "Hey  ")
            "#,
            expected: Object::Integer(5),
        },
        VmTestCase {
            input: r#"
              let f = open("/tmp/__p2sh_test.txt");
              let a = read(f);
              decode_utf8(a)
            "#,
            expected: Object::Str(String::from("Hey   World!\n")),
        },
        VmTestCase {
            input: r#"decode_utf8([b'H', b'e', b'l', b'l', b'o'])"#,
            expected: Object::Str(String::from("Hello")),
        },
        VmTestCase {
            input: r#"encode_utf8("Hello")"#,
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Byte(b'H')),
                    Rc::new(Object::Byte(b'e')),
                    Rc::new(Object::Byte(b'l')),
                    Rc::new(Object::Byte(b'l')),
                    Rc::new(Object::Byte(b'o')),
                ]),
            })),
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
            input: "int(map {})",
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
        VmTestCaseErr {
            input: r#"open()"#,
            expected: "open: takes one or two arguments. got=0",
        },
        VmTestCaseErr {
            input: r#"read()"#,
            expected: "read: takes one or two arguments. got=0",
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
fn test_iterative_fibonacci_loop() {
    let tests: Vec<VmTestCase> = vec![VmTestCase {
        input: r#"
            fn fibonacci(n) {
                if n == 0 {
                    return 0;
                } else {
                    if n == 1 {
                        return 1;
                    }
                }

                let a = 0;
                let b = 1;
                let result = 0;
                let i = 2;

                loop {
                    if i > n {
                        break;
                    }

                    result = a + b;
                    a = b;
                    b = result;
                    i = i + 1;
                }

                return result;
            }

            fibonacci(15);
        "#,
        expected: Object::Integer(610),
    }];

    run_vm_tests(&tests);
}

#[test]
fn test_iterative_fibonacci_while() {
    let tests: Vec<VmTestCase> = vec![VmTestCase {
        input: r#"
            fn fibonacci(n) {
                if n == 0 {
                    return 0;
                } else {
                    if n == 1 {
                        return 1;
                    }
                }

                let a = 0;
                let b = 1;
                let result = 0;
                let i = 2;

                while i <= n {
                    result = a + b;
                    a = b;
                    b = result;
                    i = i + 1;
                }

                return result;
            }

            fibonacci(15);
        "#,
        expected: Object::Integer(610),
    }];

    run_vm_tests(&tests);
}

#[test]
fn test_closures_with_depth() {
    let tests: Vec<VmTestCase> = vec![
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
                fn(a) {
                    {
                        let a = 10;
                    }
                    fn(b) {
                        a + b
                    }
                }(1)(2)
                "#,
            expected: Object::Integer(3),
        },
    ];

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
            input: "let a = 1.; a = 2.; a",
            expected: Object::Float(2.),
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
            input: r#"let m = map {"a": 1}; m = map {"a": 222}; m"#,
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
            input: r#"let m = map {}; m[0] = m[1] = 222; [m[0], m[1]]"#,
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
            input: r#"let m = map {}; m["a"] = 1; m"#,
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
            input: r#"let m = map {"a": 1}; m["a"] = 111; m"#,
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

#[test]
fn test_logical_and_expressions() {
    let tests = vec![
        VmTestCase {
            input: "true && true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true && false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "false && true",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "false && false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "true && 1",
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"true && "hello""#,
            expected: Object::Str("hello".to_string()),
        },
        VmTestCase {
            input: r#"false && "hello""#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: r#""hello" && false"#,
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 < 2 && 3 < 4",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 < 2 && 3 > 4",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "1 > 2 && 3 < 4",
            expected: Object::Bool(false),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_logical_or_expressions() {
    let tests = vec![
        VmTestCase {
            input: "true || true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true || false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "false || true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "false || false",
            expected: Object::Bool(false),
        },
        VmTestCase {
            input: "true || 1",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"true || "hello""#,
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: r#"false || "hello""#,
            expected: Object::Str("hello".to_string()),
        },
        VmTestCase {
            input: r#""hello" || false"#,
            expected: Object::Str("hello".to_string()),
        },
        VmTestCase {
            input: "1 < 2 || 3 < 4",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 < 2 || 3 > 4",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 > 2 || 3 < 4",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "1 > 2 || 3 > 4",
            expected: Object::Bool(false),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_logical_expressions_combined() {
    let tests = vec![
        VmTestCase {
            input: "true && true || false",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true && false || true",
            expected: Object::Bool(true),
        },
        VmTestCase {
            input: "true || false && true",
            expected: Object::Bool(true),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_block_statements() {
    let tests = vec![
        VmTestCase {
            input: r#"
            {
                100;
                200
            }
            "#,
            expected: Object::Integer(200),
        },
        VmTestCase {
            input: "
            let v = [];
            let a = 1;
            push(v, a);

            fn f() {
                let a = 2;
                push(v, a);
                {
                    let a = 3;
                    push(v, a);
                }
                push(v, a);
            }

            f();
            push(v, a);
            v
            ",
            expected: Object::Arr(Rc::new(Array {
                elements: RefCell::new(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(1)),
                ]),
            })),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_loop_with_break() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let a = 1;
            loop {
                if a == 10 {
                    break;
                }
                a = a + 1;
            }
            a;
            "#,
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: r#"
            let a = 1;
            loop {
                if a == 10 {
                    break;
                }
                a = a + 1;
            }

            loop {
                if a <= 0 {
                    break;
                }
                a = a - 1;
            }
            a;
            "#,
            expected: Object::Integer(0),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_while_loop() {
    let tests = vec![
        VmTestCase {
            input: r#"
            let a = 1;
            while a < 10 {
                a = a + 1;
            }
            a;
            "#,
            expected: Object::Integer(10),
        },
        VmTestCase {
            input: r#"
            let a = 1;
            while a < 10 {
                a = a + 1;
            }

            while a > 0 {
                a = a - 1;
            }
            a;
            "#,
            expected: Object::Integer(0),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_scoped_global_variables() {
    let tests: Vec<VmTestCase> = vec![
        VmTestCase {
            input: r#"
                let a = 1;
                {
                    let a = 10;
                    a
                }
                a
                "#,
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"
                let a = 1;
                {
                    let a = 10;
                    a;
                }
                "#,
            expected: Object::Integer(10),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_scoped_local_variables() {
    let tests: Vec<VmTestCase> = vec![
        VmTestCase {
            input: r#"
                let f = fn() {
                    let a = 1;
                    {
                        let a = 10;
                        a
                    }
                    a
                }()
                "#,
            expected: Object::Integer(1),
        },
        VmTestCase {
            input: r#"
                let f = fn() {
                    let a = 1;
                    {
                        let a = 10;
                        a
                    }
                }()
                "#,
            expected: Object::Integer(10),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_loop_with_break_and_continue() {
    let tests = vec![VmTestCase {
        input: r#"
            let a = 0;
            let s = 0;
            loop {
                if a % 2 == 0 {
                    a = a + 1;
                    continue;
                }
                if a > 10 {
                    puts("break: a = ", a);
                    break;
                }
                a = a + 1;
                s = s + a;
            }
            s;
            "#,
        expected: Object::Integer(30),
    }];
    run_vm_tests(&tests);
}

#[test]
fn test_while_with_continue() {
    let tests = vec![VmTestCase {
        input: r#"
            let a = 0;
            let s = 0;
            while a < 10 {
                if a % 2 == 0 {
                    a = a + 1;
                    continue;
                }
                a = a + 1;
                s = s + a;
            }
            s;
            "#,
        expected: Object::Integer(30),
    }];
    run_vm_tests(&tests);
}

#[test]
fn test_nested_loop_with_break_and_continue() {
    let tests = vec![
        VmTestCase {
            input: r#"
                let a = 1;
                let b = 0;
                let s = 0;
                a: loop {
                    if a > 3 {
                        break;
                    }
                    // skip 2
                    if a == 2 {
                        a = a + 1;
                        continue a;
                    }
                    b = 1;
                    b: loop {
                        if b > 3 {
                            break;
                        }
                        // skip a == b
                        if a == b {
                            b = b + 1;
                            continue b;
                        }
                        // count
                        s = s + 1;
                        b = b + 1;
                    }
                    a = a + 1;
                }
                s
            "#,
            // Combinations: (1,2), (1,3), (3,1), (3,2) = 4
            expected: Object::Integer(4),
        },
        VmTestCase {
            input: r#"
                let total = 0;
                let i = 1;
                let j = 0;

                loop {
                    if i > 3 {
                        break;
                    }
            
                    if i % 2 == 0 {
                        i = i + 1;
                        continue; // Skip even values of i
                    }
            
                    j = 1;
                    loop {
                        if j > 3 {
                            break;
                        }
            
                        if j % 2 == 0 {
                            j = j + 1;
                            continue; // Skip even values of j
                        }
            
                        total = total + i * j;
                        j = j + 1;
                    }
            
                    i = i + 1;
                }
                total
            "#,
            //  1*1 + 1*3 + 3*1 + 3*3 = 16
            expected: Object::Integer(16),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_nested_while_with_continue() {
    let tests = vec![
        VmTestCase {
            input: r#"
                let a = 1;
                let b = 0;
                let s = 0;
                a: while a <= 3 {
                    // skip 2
                    if a == 2 {
                        a = a + 1;
                        continue a;
                    }
                    b = 1;
                    b: while b <= 3 {
                        // skip a == b
                        if a == b {
                            b = b + 1;
                            continue b;
                        }
                        // count
                        s = s + 1;
                        b = b + 1;
                    }
                    a = a + 1;
                }
                s
            "#,
            // Combinations: (1,2), (1,3), (3,1), (3,2) = 4
            expected: Object::Integer(4),
        },
        VmTestCase {
            input: r#"
                let total = 0;
                let i = 1;
                let j = 0;

                while i <= 3 {
                    if i % 2 == 0 {
                        i = i + 1;
                        continue; // Skip even values of i
                    }

                    j = 1;
                    while j <= 3 {
                        if j % 2 == 0 {
                            j = j + 1;
                            continue; // Skip even values of j
                        }

                        total = total + i * j;
                        j = j + 1;
                    }

                    i = i + 1;
                }
                total
            "#,
            //  1*1 + 1*3 + 3*1 + 3*3 = 16
            expected: Object::Integer(16),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_match_expressions_integers() {
    let tests = vec![
        VmTestCase {
            input: r#"
                fn to_str(p) {
                    match p {
                        90..101 => "A",
                        80..90  => "B",
                        70..80  => "C",
                        60..70  => "D",
                        50..60  => "E",
                        _ => "F",
                    }
                }
                let s1 = to_str(100) + "," + to_str(80) + "," + to_str(70) + "," + to_str(60) + "," + to_str(50) + "," + to_str(40);
                let s2 = to_str(99) + "," + to_str(89) + "," + to_str(79) + "," + to_str(69) + "," + to_str(59) + "," + to_str(49);
                s1 + " " + s2
            "#,
            expected: Object::Str("A,B,C,D,E,F A,B,C,D,E,F".to_string()),
        },
        VmTestCase {
            input: r#"
                fn to_str(p) {
                    match p {
                        90..=100 => "A",
                        80..=89  => "B",
                        70..=79  => "C",
                        60..=69  => "D",
                        50..=59  => "E",
                        _ => "F",
                    }
                }
                let s1 = to_str(100) + "," + to_str(80) + "," + to_str(70) + "," + to_str(60) + "," + to_str(50) + "," + to_str(40);
                let s2 = to_str(99) + "," + to_str(89) + "," + to_str(79) + "," + to_str(69) + "," + to_str(59) + "," + to_str(49);
                s1 + " " + s2
            "#,
            expected: Object::Str("A,B,C,D,E,F A,B,C,D,E,F".to_string()),
        },
    ];
    run_vm_tests(&tests);
}

#[test]
fn test_match_expressions_strings() {
    let tests = vec![VmTestCase {
        input: r#"
                fn lang(p) {
                    match tolower(p) {
                        "a" | "e" | "i" | "o" | "u" => {  "vowel" }
                        "b"..="z" => "consonant",
                        _ => {  "others" }
                      }
                }
                [
                    lang("a"), lang("E"), lang("i"), lang("O"), lang("u"),
                    lang("b"), lang("C"), lang("d"), lang("F"), lang("z"),
                    lang("1"), lang("3"), lang("5"), lang("9"), lang("0"),
                    lang(" "), lang(","), lang(";"), lang("'"), lang("+")
                ]
            "#,
        expected: Object::Arr(Rc::new(Array {
            elements: RefCell::new(vec![
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
            ]),
        })),
    }];
    run_vm_tests(&tests);
}

#[test]
fn test_match_expressions_char() {
    let tests = vec![VmTestCase {
        // Test match expressions with char patterns and a mix of range expressions
        input: r#"
                fn lang(p) {
                    match tolower(p) {
                        'a' | 'e' | 'i' | 'o' | 'u' => {  "vowel" }
                        'b' | 'c'..='y' | 'z' => "consonant",
                        _ => {  "others" }
                      }
                }
                [
                    lang('a'), lang('E'), lang('i'), lang('O'), lang('u'),
                    lang('b'), lang('C'), lang('d'), lang('F'), lang('z'),
                    lang('1'), lang('3'), lang('5'), lang('9'), lang('0'),
                    lang(' '), lang(','), lang(';'), lang('-'), lang('+')
                ]
            "#,
        expected: Object::Arr(Rc::new(Array {
            elements: RefCell::new(vec![
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
            ]),
        })),
    }];
    run_vm_tests(&tests);
}

#[test]
fn test_match_expressions_byte() {
    let tests = vec![VmTestCase {
        // Test match expressions with byte patterns and a mix of range expressions
        input: r#"
                fn lang(p) {
                    match toupper(p) {
                        b'A' | b'E' | b'I' | b'O' | b'U' => {  "vowel" }
                        b'B' | b'C'..=b'Y' | b'Z' => "consonant",
                        _ => {  "others" }
                      }
                }
                [
                    lang(b'a'), lang(b'E'), lang(b'i'), lang(b'O'), lang(b'u'),
                    lang(b'b'), lang(b'C'), lang(b'd'), lang(b'F'), lang(b'z'),
                    lang(b'1'), lang(b'3'), lang(b'5'), lang(b'9'), lang(b'0'),
                    lang(b' '), lang(b','), lang(b';'), lang(b'-'), lang(b'+')
                ]
            "#,
        expected: Object::Arr(Rc::new(Array {
            elements: RefCell::new(vec![
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("vowel".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("consonant".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
                Rc::new(Object::Str("others".to_string())),
            ]),
        })),
    }];
    run_vm_tests(&tests);
}
