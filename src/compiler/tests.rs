#![allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::code::definitions;
use crate::code::opcode::*;
use crate::common::error::*;
use crate::common::object::*;
use crate::parser::*;
use crate::scanner::*;

#[cfg(test)]
struct CompilerTestCase {
    input: &'static str,
    expected_constants: Vec<Object>,
    expected_instructions: Vec<Instructions>,
}

#[cfg(test)]
struct CompilerTestCaseErrors {
    input: &'static str,
    error: &'static str,
}

#[cfg(test)]
fn parse_program(input: &str) -> Program {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    if parser.print_errors() {
        panic!("{} parse errors", parser.parse_errors().len());
    }
    program
}

#[cfg(test)]
pub fn test_constants(expected: &Vec<Object>, actual: &Vec<Rc<Object>>) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "Wrong nmber of constants. got={}, want={}",
        actual.len(),
        expected.len()
    );
    for (exp, got) in expected.iter().zip(actual) {
        match exp {
            Object::Bool(e) => test_boolean_object(got.clone(), e.clone()),
            Object::Integer(e) => test_integer_object(got.clone(), e.clone()),
            Object::Float(e) => test_float_object(got.clone(), e.clone()),
            Object::Str(s) => test_string_object(got, &s.clone()),
            Object::Func(func) => test_function_object(&got.clone(), &func),
            _ => {}
        }
    }
}

#[cfg(test)]
fn test_boolean_object(actual: Rc<Object>, exp: bool) {
    if let Object::Bool(act) = *actual.clone() {
        assert_eq!(
            act, exp,
            "object has wrong value. got={}, want={}",
            act, exp
        );
    } else {
        panic!("object is not boolean. got={}", actual);
    }
}

#[cfg(test)]
fn test_integer_object(actual: Rc<Object>, exp: i64) {
    if let Object::Integer(act) = *actual.clone() {
        assert_eq!(
            act, exp,
            "object has wrong value. got={}, want={}",
            act, exp
        );
    } else {
        panic!("object is not an integer. got={}", actual);
    }
}

#[cfg(test)]
fn test_float_object(actual: Rc<Object>, exp: f64) {
    if let Object::Float(act) = *actual.clone() {
        assert_eq!(
            act, exp,
            "object has wrong value. got={}, want={}",
            act, exp
        );
    } else {
        panic!("object is not a float. got={}", actual);
    }
}

#[cfg(test)]
fn test_string_object(actual: &Object, expected: &str) {
    if let Object::Str(result) = actual {
        assert_eq!(
            result, expected,
            "object has wrong value. got={}, want={}",
            result, expected
        );
    } else {
        panic!("object is not Str. got={:?}", actual);
    }
}

#[cfg(test)]
fn test_function_object(actual_obj: &Object, expected: &CompiledFunction) {
    if let Object::Func(actual) = actual_obj {
        test_instructions(
            &vec![(&*expected.instructions).clone()],
            &actual.instructions,
        );
    } else {
        panic!("object is not a compiled function. got={:?}", actual_obj);
    }
}

/*
 * concat_instructions is needed because the expected_instructions field in
 * CompilerTestCase is not just a slice of bytes, but a slice of slices of
 * bytes. And that’s because 'make' is used to generate the expected_instructions,
 * which produces a [u8]. So in order to compare the expected_instructions with
 * the actual instructions, we need to turn the slice of slices into a flattened
 * slice by concatenating the instructions.
 */
#[cfg(test)]
fn concat_instructions(s: &[Instructions]) -> Instructions {
    let mut out = Instructions::default();
    for ins in s {
        out.code.extend_from_slice(&ins.code);
        out.lines.extend_from_slice(&ins.lines);
    }
    out
}

#[cfg(test)]
fn test_instructions(expected: &[Instructions], actual: &Instructions) {
    let concatted = concat_instructions(expected);

    assert_eq!(
        concatted.len(),
        actual.len(),
        "Wrong number of instructions. want={}, got={}",
        concatted,
        actual,
    );
    for i in 0..concatted.len() {
        if actual.get(i) != concatted.get(i) {
            panic!(
                "wrong instruction at index {}.\nwant={}\ngot ={}",
                i, concatted, actual
            );
        }
    }
}

#[cfg(test)]
fn run_compiler_tests(tests: &[CompilerTestCase]) {
    for (n, t) in tests.iter().enumerate() {
        let program = parse_program(&t.input);
        let mut compiler = Compiler::new();
        let result = compiler.compile(program);
        if let Err(err) = result {
            panic!("[{}] {}", n, err);
        }
        println!("[{}] Compiler Test", n);
        let bytecode = compiler.bytecode();
        test_instructions(&t.expected_instructions, &bytecode.instructions);
        test_constants(&t.expected_constants, &bytecode.constants);
    }
}

#[cfg(test)]
fn run_compiler_failed_tests(tests: &[CompilerTestCaseErrors]) {
    for t in tests.iter() {
        let program = parse_program(&t.input);
        let mut compiler = Compiler::new();
        let result = compiler.compile(program);
        if let Err(err) = result {
            let serr = format!("{}", err);
            assert_eq!(serr, t.error);
        }
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        CompilerTestCase {
            input: "1 + 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 - 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Sub, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 * 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Mul, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "2 / 1",
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Div, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "2 % 1",
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Mod, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1; 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_unary_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "-1",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Minus, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "!10",
            expected_constants: vec![Object::Integer(10)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Bang, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "~10",
            expected_constants: vec![Object::Integer(10)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Not, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "!null",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::Null, &[], 1),
                definitions::make(Opcode::Bang, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_bitwise_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "1 & 3",
            expected_constants: vec![Object::Integer(1), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::And, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 | 3",
            expected_constants: vec![Object::Integer(1), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Or, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 ^ 3",
            expected_constants: vec![Object::Integer(1), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Xor, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 << 3",
            expected_constants: vec![Object::Integer(1), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::ShiftLeft, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 >> 3",
            expected_constants: vec![Object::Integer(1), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::ShiftRight, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_boolean_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "true",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::True, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::False, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 > 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 >= 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::GreaterEq, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1.1 > 2.2",
            expected_constants: vec![Object::Float(1.1), Object::Float(2.2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 < 2",
            // Constants are in reverse order: '1 < 2' is '2 > 1'
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 <= 2",
            // Constants are in reverse order: '1 < 2' is '2 > 1'
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::GreaterEq, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1.1 < 2.2",
            expected_constants: vec![Object::Float(2.2), Object::Float(1.1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 == 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Equal, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "1 != 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::NotEqual, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "true == false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::True, &[], 1),
                definitions::make(Opcode::False, &[], 1),
                definitions::make(Opcode::Equal, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "true != false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::True, &[], 1),
                definitions::make(Opcode::False, &[], 1),
                definitions::make(Opcode::NotEqual, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "null == false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::Null, &[], 1),
                definitions::make(Opcode::False, &[], 1),
                definitions::make(Opcode::Equal, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "!false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::False, &[], 1),
                definitions::make(Opcode::Bang, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "!!null",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::Null, &[], 1),
                definitions::make(Opcode::Bang, &[], 1),
                definitions::make(Opcode::Bang, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_conditional() {
    let tests = vec![
        CompilerTestCase {
            input: "if true { }; 3333;",
            expected_constants: vec![Object::Integer(3333)],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump to the 'Null' instruction following 'then_stmt'
                definitions::make(Opcode::JumpIfFalse, &[8], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Null, &[], 1),
                // 0005 : To Jump over the 'else_stmt' to the end of else statement
                definitions::make(Opcode::Jump, &[9], 1),
                // 0008 : The 'else_stmt' (it is a Null)
                definitions::make(Opcode::Null, &[], 1),
                // 0009 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[], 1),
                // 0010 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[0], 1),
                // 0013 : Pop Constant '3333'
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "if true { } else { }; 3333;",
            expected_constants: vec![Object::Integer(3333)],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump to the 'Null' instruction following 'then_stmt'
                definitions::make(Opcode::JumpIfFalse, &[8], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Null, &[], 1),
                // 0005 : To Jump over the 'else_stmt' to the end of else statement
                definitions::make(Opcode::Jump, &[9], 1),
                // 0008 : The 'else_stmt' (it is a Null)
                definitions::make(Opcode::Null, &[], 1),
                // 0009 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[], 1),
                // 0010 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[0], 1),
                // 0013 : Pop Constant '3333'
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "if true { 10 }; 3333;",
            expected_constants: vec![Object::Integer(10), Object::Integer(3333)],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump to the 'Null' instruction following 'then_stmt'
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : To Jump over the 'else_stmt' to the end of else statement
                definitions::make(Opcode::Jump, &[11], 1),
                // 0010 : The 'else_stmt' (it is a Null)
                definitions::make(Opcode::Null, &[], 1),
                // 0011 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[], 1),
                // 0012 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[1], 1),
                // 0015 : Pop Constant '3333'
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "if true { 10 } else { 20 } ; 3333;",
            expected_constants: vec![
                Object::Integer(10),
                Object::Integer(20),
                Object::Integer(3333),
            ],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[], 1),
                // 0001: Jump to 'else_stmt' if condition is false
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : Jump to the instruction following the 'if' expression
                definitions::make(Opcode::Jump, &[13], 1),
                // 0010 : The 'else_stmt'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0013 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[], 1),
                // 0014 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[2], 1),
                // 0017 : Pop Constant '3333'
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_global_let_statements() {
    let tests = vec![
        CompilerTestCase {
            input: "let one = 1;let two = 2;",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::DefineGlobal, &[1], 1),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;one;",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;let two = one;two;",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::DefineGlobal, &[1], 1),
                definitions::make(Opcode::GetGlobal, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_global_get_expressions() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "undefined_var",
            error: "[line 1] compile error: undefined identifier 'undefined_var'",
        },
        CompilerTestCaseErrors {
            input: "undefined_function()",
            error: "[line 1] compile error: undefined identifier 'undefined_function'",
        },
    ];
    run_compiler_failed_tests(&tests);
}

#[test]
fn test_string_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: r#""p2sh""#,
            expected_constants: vec![Object::Str(String::from("p2sh"))],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#""p2" + "sh""#,
            expected_constants: vec![
                Object::Str(String::from("p2")),
                Object::Str(String::from("sh")),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_array_literals() {
    // An array literal expression involves building 'n' elements on the stack
    // followed by an OpArray instruction
    let tests = vec![
        CompilerTestCase {
            input: "[]",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::Array, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "[1, 2, 3]",
            expected_constants: vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "[1 + 2, 3 - 4, 5 * 6]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Sub, &[], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Mul, &[], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_hash_literals() {
    let tests = vec![
        CompilerTestCase {
            input: "{}",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::Map, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2, 3: 4, 5: 6}",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Map, &[6], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2 + 3, 4: 5 * 6}",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Mul, &[], 1),
                definitions::make(Opcode::Map, &[4], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_get_index_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "[1, 2, 3][1 + 1]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::GetIndex, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2}[2 - 1]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(2),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Map, &[2], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Sub, &[], 1),
                definitions::make(Opcode::GetIndex, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_set_index_expressions() {
    let tests = vec![
        CompilerTestCase {
            // The assignment expressions are right associative
            input: "[1, 2, 3][1] = 42",
            expected_constants: vec![
                Object::Integer(42),
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                // RHS of the assignment
                definitions::make(Opcode::Constant, &[0], 1),
                // LHS of the assignment
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::SetIndex, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "[1, 2, 3][1 + 1] = 2 * 3",
            expected_constants: vec![
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                // RHS of the assignment
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Mul, &[], 1),
                // LHS of the assignment
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Constant, &[6], 1),
                definitions::make(Opcode::Add, &[], 1),
                definitions::make(Opcode::SetIndex, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2}[2 - 1] = 2 * 3",
            expected_constants: vec![
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(2),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                // RHS of the assignment
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Mul, &[], 1),
                // LHS of the assignment
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Map, &[2], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Sub, &[], 1),
                definitions::make(Opcode::SetIndex, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_function_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { return }",
            expected_constants: vec![Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::Null, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                0,
                0,
            )))],
            expected_instructions: vec![
                // Number of free variables is '0'
                definitions::make(Opcode::Closure, &[0, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { return 5 + 10 }",
            expected_constants: vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // Number of free variables is '0'
                definitions::make(Opcode::Closure, &[2, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { 5 + 10 }",
            expected_constants: vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // Number of free variables is '0'
                definitions::make(Opcode::Closure, &[2, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { 1; 2 }",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        // Pop the first value
                        definitions::make(Opcode::Pop, &[], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        // The pop is replaced by the implicit return
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[2, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_function_statement() {
    let tests = vec![
        CompilerTestCase {
            input: "fn f1() { return 5 + 10 }",
            expected_constants: vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // Number of free variables is '0'
                definitions::make(Opcode::Closure, &[2, 0], 1),
                // Define the function
                definitions::make(Opcode::DefineGlobal, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "fn f2() { 5 + 10 }",
            expected_constants: vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // Number of free variables is '0'
                definitions::make(Opcode::Closure, &[2, 0], 1),
                // Define the function
                definitions::make(Opcode::DefineGlobal, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "fn f3() { 1; 2 }",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        // Pop the first value
                        definitions::make(Opcode::Pop, &[], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        // The pop is replaced by the implicit return
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[2, 0], 1),
                // Define the function
                definitions::make(Opcode::DefineGlobal, &[0], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_functions_without_return_value() {
    let tests = vec![CompilerTestCase {
        input: "fn() { }",
        expected_constants: vec![Object::Func(Rc::new(CompiledFunction::new(
            definitions::make(Opcode::Return, &[], 1),
            0,
            0,
        )))],
        expected_instructions: vec![
            definitions::make(Opcode::Closure, &[0, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_return_from_non_functions() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "return 1",
            error: "[line 1] compile error: return statement outside of function",
        },
        CompilerTestCaseErrors {
            input: "if true { return 1; }",
            error: "[line 1] compile error: return statement outside of function",
        },
    ];

    run_compiler_failed_tests(&tests);
}

#[test]
fn test_compiler_scopes() {
    let mut compiler = Compiler::new();
    assert_eq!(compiler.scope_index, 0);

    let global_symbol_table = compiler.symtab.clone();
    compiler.emit(Opcode::Mul, &[0], 1);

    compiler.enter_scope();
    assert_eq!(compiler.scope_index, 1);

    compiler.emit(Opcode::Sub, &[0], 1);
    assert_eq!(compiler.scopes[compiler.scope_index].instructions.len(), 1);
    let last = &compiler.scopes[compiler.scope_index].last_ins;
    assert_eq!(last.opcode, Opcode::Sub);

    if let Some(outer) = compiler.symtab.outer.clone() {
        assert_eq!(outer.as_ref().clone(), global_symbol_table);
    } else {
        panic!("compiler did not enclose symbol table");
    }

    compiler.leave_scope();
    assert_eq!(compiler.scope_index, 0, "scope index wrong");

    assert_eq!(
        compiler.symtab, global_symbol_table,
        "compiler did not restore symbol table"
    );
    assert!(
        compiler.symtab.outer.is_none(),
        "compiler modified global symbol table incorrectly"
    );

    compiler.emit(Opcode::Add, &[0], 1);

    assert_eq!(compiler.scopes[compiler.scope_index].instructions.len(), 2);
    let last = &compiler.scopes[compiler.scope_index].last_ins;
    assert_eq!(last.opcode, Opcode::Add);

    let previous = &compiler.scopes[compiler.scope_index].prev_ins;
    assert_eq!(previous.opcode, Opcode::Mul);
}

#[test]
fn test_function_calls() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { 24 }()",
            expected_constants: vec![
                Object::Integer(24),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // The literal '24'
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // The compiled function (closure)
                definitions::make(Opcode::Closure, &[1, 0], 1),
                definitions::make(Opcode::Call, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            // Function is bound to a name here
            input: "let noArg = fn() { 24 }; noArg();",
            expected_constants: vec![
                Object::Integer(24),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // The literal '24'
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // The compiled function
                definitions::make(Opcode::Closure, &[1, 0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Call, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "
                let oneArg = fn(a) { a };
                oneArg(24);
            ",
            expected_constants: vec![
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    1,
                ))),
                Object::Integer(24),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[0, 0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Call, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "
                let manyArg = fn(a, b, c) { a; b; c; };
                manyArg(24, 25, 26);
            ",
            expected_constants: vec![
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // References to the arguments to the function
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::Pop, &[], 1),
                        definitions::make(Opcode::GetLocal, &[1], 1),
                        definitions::make(Opcode::Pop, &[], 1),
                        definitions::make(Opcode::GetLocal, &[2], 1),
                        // returning the last reference
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    3,
                ))),
                Object::Integer(24),
                Object::Integer(25),
                Object::Integer(26),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[0, 0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Call, &[3], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_let_statement_scopes() {
    let tests = vec![
        CompilerTestCase {
            input: "let num = 55; fn() { num }",
            expected_constants: vec![
                Object::Integer(55),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // push the value of global variable 'num'
                        definitions::make(Opcode::GetGlobal, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // constant - number 55
                definitions::make(Opcode::Constant, &[0], 1),
                // set the global variable 'num'
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                // constant - compiled function (closure)
                definitions::make(Opcode::Closure, &[1, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { let num = 55; num }",
            expected_constants: vec![
                Object::Integer(55),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // constant - number 55
                        definitions::make(Opcode::Constant, &[0], 1),
                        // set the global variable 'num'
                        definitions::make(Opcode::DefineLocal, &[0], 1),
                        // push the value of global variable 'num'
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    1,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // constant - compiled function (closure)
                definitions::make(Opcode::Closure, &[1, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "
                fn() {
                    let a = 55;
                    let b = 77;
                    a + b
                }
            ",
            expected_constants: vec![
                Object::Integer(55),
                Object::Integer(77),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),    // 55
                        definitions::make(Opcode::DefineLocal, &[0], 1), // 'a'
                        definitions::make(Opcode::Constant, &[1], 1),    // 77
                        definitions::make(Opcode::DefineLocal, &[1], 1), // 'b'
                        definitions::make(Opcode::GetLocal, &[0], 1),    // 'a'
                        definitions::make(Opcode::GetLocal, &[1], 1),    // 'b'
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    2,
                    0,
                ))),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[2, 0], 1), // compiled fn (closure)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_local_get_expressions() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "fn() { undefined_var }",
            error: "[line 1] compile error: undefined identifier 'undefined_var'",
        },
        CompilerTestCaseErrors {
            input: "fn() { undefined_function() }",
            error: "[line 1] compile error: undefined identifier 'undefined_function'",
        },
    ];
    run_compiler_failed_tests(&tests);
}

#[test]
fn test_builtins() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                len([]);
                push([], 1);
            "#,
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::GetBuiltinFn, &[0], 1),
                definitions::make(Opcode::Array, &[0], 1),
                // call built-in fn 'len' with one argument
                definitions::make(Opcode::Call, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
                definitions::make(Opcode::GetBuiltinFn, &[5], 1),
                definitions::make(Opcode::Array, &[0], 1),
                definitions::make(Opcode::Constant, &[0], 1),
                // call built-in fn 'push' with two arguments
                definitions::make(Opcode::Call, &[2], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { len([]) }",
            expected_constants: vec![Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::GetBuiltinFn, &[0], 1),
                    definitions::make(Opcode::Array, &[0], 1),
                    // call built-in fn 'len' with one argument
                    definitions::make(Opcode::Call, &[1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            )))],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[0, 0], 1), // compiled fn (closure)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                len(argv);
                push(argv, 1);
            "#,
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                definitions::make(Opcode::GetBuiltinFn, &[0], 1),
                definitions::make(Opcode::GetBuiltinVar, &[0], 1),
                // call built-in fn 'len' with one argument
                definitions::make(Opcode::Call, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
                // built-in function push
                definitions::make(Opcode::GetBuiltinFn, &[5], 1),
                definitions::make(Opcode::GetBuiltinVar, &[0], 1),
                definitions::make(Opcode::Constant, &[0], 1),
                // call built-in fn 'push' with two arguments
                definitions::make(Opcode::Call, &[2], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { len(argv) }",
            expected_constants: vec![Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::GetBuiltinFn, &[0], 1),
                    definitions::make(Opcode::GetBuiltinVar, &[0], 1),
                    // call built-in fn 'len' with one argument
                    definitions::make(Opcode::Call, &[1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            )))],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[0, 0], 1), // compiled fn (closure)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_closures() {
    let tests = vec![CompilerTestCase {
        input: "
                fn(a) {
                    fn(b) {
                        a + b
                    }
                }
            ",

        expected_constants: vec![
            Object::Func(Rc::new(CompiledFunction::new(
                // the real closure
                concat_instructions(&[
                    // variable 'a' defined in the enclosing scope is a
                    // 'free' variable for this scope
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    // #free-vars is 1 as there is one free variable on the stack
                    // that needs to be saved into the free field of the closure
                    definitions::make(Opcode::Closure, &[0, 1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
        ],

        expected_instructions: vec![
            definitions::make(Opcode::Closure, &[1, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_closures_with_depth() {
    let tests = vec![CompilerTestCase {
        input: "
                fn(a) {
                    if true {
                        let a = 1;
                    }
                    fn(b) {
                        a + b
                    }
                }
            ",

        expected_constants: vec![
            Object::Integer(1),
            Object::Func(Rc::new(CompiledFunction::new(
                // the real closure
                concat_instructions(&[
                    // variable 'a' defined in the enclosing scope is a
                    // 'free' variable for this scope
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::True, &[], 1),
                    definitions::make(Opcode::JumpIfFalse, &[12], 1),
                    definitions::make(Opcode::Constant, &[0], 1),
                    definitions::make(Opcode::DefineLocal, &[1], 1),
                    definitions::make(Opcode::Jump, &[13], 1),
                    definitions::make(Opcode::Null, &[], 1),
                    definitions::make(Opcode::Pop, &[], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    // #free-vars is 1 as there is one free variable on the stack
                    // that needs to be saved into the free field of the closure
                    definitions::make(Opcode::Closure, &[1, 1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
        ],

        expected_instructions: vec![
            definitions::make(Opcode::Closure, &[2, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

// There are three nested functions. The innermost function, the one with the
// c parameter, references two free variables: a and b. b is defined in the
// immediate enclosing scope, but a is defined in the outermost function, two
// scopes removed. The middle function is expected to contain an OpClosure
// instruction that turns the innermost function into a closure. Since the
// second operand is 2, there are supposed to be two free variables sitting
// on the stack when the VM executes it. From the perspective of the middle
// function, a is also a free variable. It is neither defined in scope nor
// as a parameter.
#[test]
fn test_nested_closures() {
    let tests = vec![CompilerTestCase {
        input: "
                fn(a) {
                    fn(b) {
                        fn(c) {
                            a + b + c
                        }
                    }
                }
            ",
        expected_constants: vec![
            Object::Func(Rc::new(CompiledFunction::new(
                // This is the defintion of the inner-most function that has
                // two free variables 'a' and 'b' but defines no closure(s)
                // within itself (via the OpClosure instruction).
                concat_instructions(&[
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::GetFree, &[1], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                // middle function has one free variable 'a' and defines the
                // inner-most function as a closure that has two free variables
                // The number of free variables is passed as the second arg.
                concat_instructions(&[
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    // two free variables on stack
                    definitions::make(Opcode::Closure, &[0, 2], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                // outer-most function has no free variables but compiles
                // the middle closure that has a single free variable
                concat_instructions(&[
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Closure, &[1, 1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
        ],
        expected_instructions: vec![
            definitions::make(Opcode::Closure, &[2, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_closures_with_scopes() {
    let tests = vec![CompilerTestCase {
        input: "
                let global = 55;
                fn() {
                    let a = 66;
                    fn() {
                        let b = 77;
                        fn() {
                            let c = 88;
                            global + a + b + c;
                        }
                    }
                }
            ",
        expected_constants: vec![
            Object::Integer(55),
            Object::Integer(66),
            Object::Integer(77),
            Object::Integer(88),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::Constant, &[3], 1),
                    definitions::make(Opcode::DefineLocal, &[0], 1),
                    definitions::make(Opcode::GetGlobal, &[0], 1),
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::GetFree, &[1], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::Constant, &[2], 1),
                    definitions::make(Opcode::DefineLocal, &[0], 1),
                    definitions::make(Opcode::GetFree, &[0], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Closure, &[4, 2], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::Constant, &[1], 1),
                    definitions::make(Opcode::DefineLocal, &[0], 1),
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    definitions::make(Opcode::Closure, &[5, 1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
        ],
        expected_instructions: vec![
            definitions::make(Opcode::Constant, &[0], 1),
            definitions::make(Opcode::DefineGlobal, &[0], 1),
            definitions::make(Opcode::Closure, &[6, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_recursive_functions() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                let countDown = fn(x) { countDown(x - 1); };
                countDown(1);
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // first load callee, the the args and then the OpCall
                        // Here, the callee is also the current closure
                        definitions::make(Opcode::CurrClosure, &[], 1),
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Sub, &[], 1),
                        definitions::make(Opcode::Call, &[1], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    1,
                    0,
                ))),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[1, 0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Call, &[1], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
            let wrapper = fn() {
                let countDown = fn(x) { countDown(x - 1); };
                countDown(1);
            };
            wrapper();
        "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::CurrClosure, &[], 1),
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Sub, &[], 1),
                        definitions::make(Opcode::Call, &[1], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    1,
                    0,
                ))),
                Object::Integer(1),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Closure, &[1, 0], 1),
                        definitions::make(Opcode::DefineLocal, &[0], 1),
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::Constant, &[2], 1),
                        definitions::make(Opcode::Call, &[1], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    1,
                    0,
                ))),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[3, 0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Call, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_assignment_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "let a = 123; a = 456",
            expected_constants: vec![Object::Integer(123), Object::Integer(456)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "let a = [123]; a = [456, 789]",
            expected_constants: vec![
                Object::Integer(123),
                Object::Integer(456),
                Object::Integer(789),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Array, &[1], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Array, &[2], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "let a = [1, 2]; a = [3, 4]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Array, &[2], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Array, &[2], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"let m = {"a": 1}; m = {"a": 2}"#,
            expected_constants: vec![
                Object::Str("a".to_string()),
                Object::Integer(1),
                Object::Str("a".to_string()),
                Object::Integer(2),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Map, &[2], 1),
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Map, &[2], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_assignment_expressions_negative() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "puts = 1",
            error: "[line 1] compile error: Invalid lvalue",
        },
        CompilerTestCaseErrors {
            input: "argv = 1",
            error: "[line 1] compile error: Invalid lvalue",
        },
    ];

    run_compiler_failed_tests(&tests);
}

#[test]
fn test_assignment_expression_scopes() {
    let tests = vec![
        CompilerTestCase {
            input: "let num = 55; fn() { num = 66; num }",
            expected_constants: vec![
                Object::Integer(55),
                Object::Integer(66),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // num = 66
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::SetGlobal, &[0], 1),
                        definitions::make(Opcode::Pop, &[], 1),
                        // push the value of global variable 'num'
                        definitions::make(Opcode::GetGlobal, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    0,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // constant - number 55
                definitions::make(Opcode::Constant, &[0], 1),
                // set the global variable 'num'
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                // constant - compiled function (closure)
                definitions::make(Opcode::Closure, &[2, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { let num = 55; num = 66; num }",
            expected_constants: vec![
                Object::Integer(55),
                Object::Integer(66),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        // constant - number 55
                        definitions::make(Opcode::Constant, &[0], 1),
                        // define the global variable 'num'
                        definitions::make(Opcode::DefineLocal, &[0], 1),
                        // num = 66
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::SetLocal, &[0], 1),
                        definitions::make(Opcode::Pop, &[], 1),
                        // push the value of global variable 'num'
                        definitions::make(Opcode::GetLocal, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    1,
                    0,
                ))),
            ],
            expected_instructions: vec![
                // constant - compiled function (closure)
                definitions::make(Opcode::Closure, &[2, 0], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "
                fn() {
                    let a = 55;
                    let b = 77;
                    a = 66;
                    b = 88;
                    a + b
                }
            ",
            expected_constants: vec![
                Object::Integer(55),
                Object::Integer(77),
                Object::Integer(66),
                Object::Integer(88),
                Object::Func(Rc::new(CompiledFunction::new(
                    concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),    // 55
                        definitions::make(Opcode::DefineLocal, &[0], 1), // 'a'
                        definitions::make(Opcode::Constant, &[1], 1),    // 77
                        definitions::make(Opcode::DefineLocal, &[1], 1), // 'b'
                        // a = 66
                        definitions::make(Opcode::Constant, &[2], 1), // 66
                        definitions::make(Opcode::SetLocal, &[0], 1), // 'a'
                        definitions::make(Opcode::Pop, &[], 1),
                        // b = 88
                        definitions::make(Opcode::Constant, &[3], 1), // 88
                        definitions::make(Opcode::SetLocal, &[1], 1), // 'b'
                        definitions::make(Opcode::Pop, &[], 1),
                        definitions::make(Opcode::GetLocal, &[0], 1), // 'a'
                        definitions::make(Opcode::GetLocal, &[1], 1), // 'b'
                        definitions::make(Opcode::Add, &[], 1),
                        definitions::make(Opcode::ReturnValue, &[], 1),
                    ]),
                    2,
                    0,
                ))),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Closure, &[4, 0], 1), // compiled fn (closure)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_assignment_expressions_free_variables() {
    let tests = vec![CompilerTestCase {
        input: "
                fn(a) {
                    fn(b) {
                        a = 99;
                        a + b
                    }
                }
            ",

        expected_constants: vec![
            Object::Integer(99),
            Object::Func(Rc::new(CompiledFunction::new(
                // the real closure
                concat_instructions(&[
                    // a = 66
                    definitions::make(Opcode::Constant, &[0], 1), // 99
                    definitions::make(Opcode::SetFree, &[0], 1),  // 'a'
                    definitions::make(Opcode::Pop, &[], 1),
                    definitions::make(Opcode::GetFree, &[0], 1), // 'a'
                    definitions::make(Opcode::GetLocal, &[0], 1), // 'b'
                    definitions::make(Opcode::Add, &[], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
            Object::Func(Rc::new(CompiledFunction::new(
                concat_instructions(&[
                    definitions::make(Opcode::GetLocal, &[0], 1),
                    // #free-vars is 1 as there is one free variable on the stack
                    // that needs to be saved into the free field of the closure
                    definitions::make(Opcode::Closure, &[1, 1], 1),
                    definitions::make(Opcode::ReturnValue, &[], 1),
                ]),
                1,
                0,
            ))),
        ],

        expected_instructions: vec![
            definitions::make(Opcode::Closure, &[2, 0], 1),
            definitions::make(Opcode::Pop, &[], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_logical_and_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: r#"true && "hello""#,
            expected_constants: vec![Object::Str("hello".into())],
            expected_instructions: vec![
                // 0000 : The left-hand side expression
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalseNoPop, &[8], 1),
                // 0004 : Pop the result of the lhs expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0005 : The rhs expression
                definitions::make(Opcode::Constant, &[0], 1),
                // 0008 : Pop the result of the expression (outside the && expression)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"1 && "hello""#,
            expected_constants: vec![Object::Integer(1), Object::Str("hello".into())],
            expected_instructions: vec![
                // 0000 : The left-hand side expression
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalseNoPop, &[10], 1),
                // 0006 : Pop the result of the lhs expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0007 : The rhs expression
                definitions::make(Opcode::Constant, &[1], 1),
                // 0010 : Pop the result of the expression (outside the && expression)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_logical_or_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: r#"0 || "hello""#,
            expected_constants: vec![Object::Integer(0), Object::Str("hello".into())],
            expected_instructions: vec![
                // 0000 : The left-hand side expression
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalseNoPop, &[9], 1),
                // 0006 : Pop the result of the lhs expression
                definitions::make(Opcode::Jump, &[13], 1),
                // 0009 : Pop the result of the lhs expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0010 : The rhs expression
                definitions::make(Opcode::Constant, &[1], 1),
                // 0013 : Pop the result of the expression (outside the && expression)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"1 || "hello""#,
            expected_constants: vec![Object::Integer(1), Object::Str("hello".into())],
            expected_instructions: vec![
                // 0000 : The left-hand side expression
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalseNoPop, &[9], 1),
                // 0006 : Pop the result of the lhs expression
                definitions::make(Opcode::Jump, &[13], 1),
                // 0009 : Pop the result of the lhs expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0010 : The rhs expression
                definitions::make(Opcode::Constant, &[1], 1),
                // 0013 : Pop the result of the expression (outside the || expression)
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_loop_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                loop {
                    break;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000 : The Jump instruction for the break
                definitions::make(Opcode::Jump, &[6], 1),
                // 0003 : The Jump instruction for the loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0006 : The constant 1111
                definitions::make(Opcode::Constant, &[0], 1),
                // 0009 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                let a = 1;
                loop {
                    if a == 10 {
                        break;
                    }
                }
                1111;
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(10),
                Object::Integer(1111),
            ],
            expected_instructions: vec![
                // 0000 : The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Define the global variable 'a'
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                // 0006 : Start of loop; get the value of 'a'
                definitions::make(Opcode::GetGlobal, &[0], 3),
                // 0009 : The constant '10'
                definitions::make(Opcode::Constant, &[1], 3),
                // 0012 : Instruction to compare 'a' and 10
                definitions::make(Opcode::Equal, &[], 3),
                // 0013 : Jump over the 'then' statement if condition is false
                definitions::make(Opcode::JumpIfFalse, &[22], 3),
                // 0016 : Jump for the break instruction
                definitions::make(Opcode::Jump, &[27], 4),
                // 0019 : Jump to the end of the 'if' expression
                definitions::make(Opcode::Jump, &[23], 3),
                // 0022 : Null else case
                definitions::make(Opcode::Null, &[], 3),
                // 0023 : Pop the result of the 'if' expression
                definitions::make(Opcode::Pop, &[], 3),
                // 0024 : Jump to the start of the loop
                definitions::make(Opcode::Jump, &[6], 5),
                // 0027 : The constant '1111'
                definitions::make(Opcode::Constant, &[2], 7),
                // 0030 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 7),
            ],
        },
        CompilerTestCase {
            input: r#"
                loop {
                    break;
                }
                loop {
                    break;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000 : Loop 1: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[6], 1),
                // 0003 : The Jump instruction for the first loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0006 : Loop 2: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[12], 1),
                // 0009 : The Jump instruction for the second loop
                definitions::make(Opcode::Jump, &[6], 1),
                // 0012 : The constant 1111
                definitions::make(Opcode::Constant, &[0], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_while_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                while true {
                    break;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000 : The constant 'true'
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004: Jump for the break
                definitions::make(Opcode::Jump, &[10], 1),
                // 0007 : Jump to the beginning of the while loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0010 : The constant 1111
                definitions::make(Opcode::Constant, &[0], 1),
                // 0013 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                let a = 1;
                while a < 10 {
                }
                1111;
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(10),
                Object::Integer(1111),
            ],
            expected_instructions: vec![
                // 0000 : The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Define the global variable 'a'
                definitions::make(Opcode::DefineGlobal, &[0], 1),
                // 0006 : Start of while loop; The constant '10'
                definitions::make(Opcode::Constant, &[1], 3),
                // 0009 : Get the value of 'a'
                definitions::make(Opcode::GetGlobal, &[0], 3),
                // 0012 : Instruction to compare 'a' and 10
                definitions::make(Opcode::Greater, &[], 3),
                // 0013 : Jump to end of loop if false
                definitions::make(Opcode::JumpIfFalse, &[19], 3),
                // 0016 : Jump to beginning of loop
                definitions::make(Opcode::Jump, &[6], 3),
                // 0019 : The constant '1111'
                definitions::make(Opcode::Constant, &[2], 7),
                // 0022 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 7),
            ],
        },
        CompilerTestCase {
            input: r#"
                while true {
                    break;
                }
                while false {
                    continue;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000 : Loop 1: The constant 'true'
                definitions::make(Opcode::True, &[], 1),
                // 0001 : Jump to end of loop if false
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004: Jump for the break
                definitions::make(Opcode::Jump, &[10], 1),
                // 0007 : Jump to the beginning of the while loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0010 : Loop 2: The constant 'false'
                definitions::make(Opcode::False, &[], 1),
                // 0011 : Jump over the rhs expression if the lhs is false
                definitions::make(Opcode::JumpIfFalse, &[20], 1),
                // 0014: Jump for the continue
                definitions::make(Opcode::Jump, &[10], 1),
                // 0017 : Jump to the beginning of the while loop
                definitions::make(Opcode::Jump, &[10], 1),
                // 0020 : The constant 1111
                definitions::make(Opcode::Constant, &[0], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_break_outside_loop() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "break;",
            error: "[line 1] compile error: break statement outside of loop",
        },
        CompilerTestCaseErrors {
            input: "loop {} break; loop {}",
            error: "[line 1] compile error: break statement outside of loop",
        },
    ];
    run_compiler_failed_tests(&tests);
}

#[test]
fn test_continue_outside_loop() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: "continue;",
            error: "[line 1] compile error: continue statement outside of loop",
        },
        CompilerTestCaseErrors {
            input: "loop {} continue; loop {}",
            error: "[line 1] compile error: continue statement outside of loop",
        },
    ];
    run_compiler_failed_tests(&tests);
}

#[test]
fn test_nested_loop_with_break_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                loop {
                    1;
                    loop {
                        2;
                        break;
                    }
                    break;
                }
                1111;
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(1111),
            ],
            expected_instructions: vec![
                // 0000 : Loop 1: The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Loop 1: Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0004: Loop 2: The constant '2'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0007 : Loop 2: Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0008 : Loop 2: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[14], 1),
                // 0011 : Loop 2: Jump to beginning of (inner) loop 2
                definitions::make(Opcode::Jump, &[4], 1),
                // 0014 : Loop 1: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[20], 1),
                // 0017 : Loop 1: Jump to beginning of (outer) loop 1
                definitions::make(Opcode::Jump, &[0], 1),
                // 0020 : The constant '1111'
                definitions::make(Opcode::Constant, &[2], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: loop {
                    b: loop {
                        break b;
                    }
                    break a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0001 : Loop b: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[6], 1),
                // 0003 : Loop b: Jump to beginning of (inner) loop b
                definitions::make(Opcode::Jump, &[0], 1),
                // 0006 : Loop a: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[12], 1),
                // 0009 : Loop a: Jump to beginning of (outer) loop a
                definitions::make(Opcode::Jump, &[0], 1),
                // 0012 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: loop {
                    b: loop {
                        break a;
                    }
                    break a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0001 : Loop b: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[12], 1),
                // 0003 : Loop b: Jump to beginning of (inner) loop b
                definitions::make(Opcode::Jump, &[0], 1),
                // 0006 : Loop a: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[12], 1),
                // 0009 : Loop a: Jump to beginning of (outer) loop a
                definitions::make(Opcode::Jump, &[0], 1),
                // 0012 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_nested_while_with_break_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                while true {
                    1;
                    while false {
                        2;
                        break;
                    }
                    break;
                }
                1111;
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(1111),
            ],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::True, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[28], 1),
                // 0004 : The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0008: Loop 2: condition
                definitions::make(Opcode::False, &[], 1),
                // 0009 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[22], 1),
                // 0012 : The constant '2'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0016 : Loop 2: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[22], 1),
                // 0019 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[8], 1),
                // 0022 : Loop 1: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[28], 1),
                // 0025 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0028 : The constant '1111'
                definitions::make(Opcode::Constant, &[2], 1),
                // 0031 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: while false {
                    b: while true {
                        break b;
                    }
                    break a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::False, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[20], 1),
                // 0004: Loop 2: condition
                definitions::make(Opcode::True, &[], 1),
                // 0005 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[14], 1),
                // 0008 : Loop 2: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[14], 1),
                // 0011 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[4], 1),
                // 0014 : Loop 1: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[20], 1),
                // 0017 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0020 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: while false {
                    b: while false {
                        break a;
                    }
                    break a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::False, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[20], 1),
                // 0004: Loop 2: condition
                definitions::make(Opcode::False, &[], 1),
                // 0005 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[14], 1),
                // 0008 : Loop 2: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[20], 1),
                // 0011 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[4], 1),
                // 0014 : Loop 1: The Jump instruction for the break
                definitions::make(Opcode::Jump, &[20], 1),
                // 0017 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0020 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_nested_loop_with_continue_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                a: loop {
                    1;
                    b: loop {
                        continue b;
                    }
                    continue a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1), Object::Integer(1111)],
            expected_instructions: vec![
                // 0000 : Loop 1: The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0003 : Loop 1: Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0004 : Loop b: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[4], 1),
                // 0007 : Loop b: Jump to beginning of (inner) loop b
                definitions::make(Opcode::Jump, &[4], 1),
                // 0010 : Loop a: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0013 : Loop a: Jump to beginning of (outer) loop a
                definitions::make(Opcode::Jump, &[0], 1),
                // 0016 : The constant '1111'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0019 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: loop {
                    b: loop {
                        continue a;
                    }
                    continue a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0001 : Loop b: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0003 : Loop b: Jump to beginning of (inner) loop b
                definitions::make(Opcode::Jump, &[0], 1),
                // 0006 : Loop a: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0009 : Loop a: Jump to beginning of (outer) loop a
                definitions::make(Opcode::Jump, &[0], 1),
                // 0012 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_nested_while_with_continue_statements() {
    let tests = vec![
        CompilerTestCase {
            input: r#"
                while true {
                    1;
                    while false {
                        2;
                        continue;
                    }
                    continue;
                }
                1111;
            "#,
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(1111),
            ],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::True, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[28], 1),
                // 0004 : The constant '1'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0008: Loop 2: condition
                definitions::make(Opcode::False, &[], 1),
                // 0009 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[22], 1),
                // 0012 : The constant '2'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0015 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
                // 0016 : Loop 2: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[8], 1),
                // 0019 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[8], 1),
                // 0022 : Loop 1: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0025 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0028 : The constant '1111'
                definitions::make(Opcode::Constant, &[2], 1),
                // 0031 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: while false {
                    b: while true {
                        continue b;
                    }
                    continue a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::False, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[20], 1),
                // 0004: Loop 2: condition
                definitions::make(Opcode::True, &[], 1),
                // 0005 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[14], 1),
                // 0008 : Loop 2: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[4], 1),
                // 0011 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[4], 1),
                // 0014 : Loop 1: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0017 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0020 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: r#"
                a: while false {
                    b: while false {
                        continue a;
                    }
                    continue a;
                }
                1111;
            "#,
            expected_constants: vec![Object::Integer(1111)],
            expected_instructions: vec![
                // 0000: Loop 1: condition
                definitions::make(Opcode::False, &[], 1),
                // 0001 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[20], 1),
                // 0004: Loop 2: condition
                definitions::make(Opcode::False, &[], 1),
                // 0005 : The Jump to end if false
                definitions::make(Opcode::JumpIfFalse, &[14], 1),
                // 0008 : Loop 2: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0011 : Loop 2: Jump to beginning of (inner) loop
                definitions::make(Opcode::Jump, &[4], 1),
                // 0014 : Loop 1: The Jump instruction for the continue
                definitions::make(Opcode::Jump, &[0], 1),
                // 0017 : Loop 1: Jump to beginning of (outer) loop
                definitions::make(Opcode::Jump, &[0], 1),
                // 0020 : The constant '1111'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0023 : Pop the result of the expression
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_unknown_loop_label() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: r#"
                loop {
                    loop {
                        continue a;
                    }
                    continue a;
                }
                1111;
            "#,
            error: "[line 4] compile error: unknown loop label 'a'",
        },
        CompilerTestCaseErrors {
            input: r#"
                a: loop {
                    b: loop {
                        continue b;
                    }
                    continue b;
                }
                1111;
            "#,
            error: "[line 6] compile error: unknown loop label 'b'",
        },
    ];
    run_compiler_failed_tests(&tests);
}

#[test]
fn test_unknown_while_loop_label() {
    let tests = vec![
        CompilerTestCaseErrors {
            input: r#"
                while true {
                    while false {
                        continue a;
                    }
                    continue a;
                }
                1111;
            "#,
            error: "[line 4] compile error: unknown loop label 'a'",
        },
        CompilerTestCaseErrors {
            input: r#"
                a: while false {
                    b: while true {
                        continue b;
                    }
                    continue b;
                }
                1111;
            "#,
            error: "[line 6] compile error: unknown loop label 'b'",
        },
    ];
    run_compiler_failed_tests(&tests);
}
