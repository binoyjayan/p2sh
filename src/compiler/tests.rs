#![allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::code::definitions;
use crate::code::opcode::*;
use crate::common::environment::*;
use crate::common::error::*;
use crate::common::object::*;
use crate::evaluator::Evaluator;
use crate::parser::*;
use crate::scanner::*;

#[cfg(test)]
struct CompilerTestCase {
    input: &'static str,
    expected_constants: Vec<Object>,
    expected_instructions: Vec<Instructions>,
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
pub fn test_constants(expected: &Vec<Object>, actual: &Vec<Object>) {
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
            Object::Number(e) => test_numeric_object(got.clone(), e.clone()),
            Object::Str(s) => test_string_object(&got.clone(), &s.clone()),
            Object::CompiledFunc(func) => test_function_object(&got.clone(), &func),
            _ => {}
        }
    }
}

#[cfg(test)]
fn test_boolean_object(actual: Object, exp: bool) {
    if let Object::Bool(act) = actual.clone() {
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
fn test_numeric_object(actual: Object, exp: f64) {
    if let Object::Number(act) = actual.clone() {
        assert_eq!(
            act, exp,
            "object has wrong value. got={}, want={}",
            act, exp
        );
    } else {
        panic!("object is not numeric. got={}", actual);
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
    if let Object::CompiledFunc(actual) = actual_obj {
        test_instructions(&vec![expected.instructions.clone()], &actual.instructions);
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
        // println!("[{}] Compiler Test", n);
        let bytecode = compiler.bytecode();
        test_instructions(&t.expected_instructions, &bytecode.instructions);
        test_constants(&t.expected_constants, &bytecode.constants);
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        CompilerTestCase {
            input: "1 + 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Add, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 - 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Sub, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 * 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Mul, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "2 / 1",
            expected_constants: vec![Object::Number(2.), Object::Number(1.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Div, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1; 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "-1",
            expected_constants: vec![Object::Number(1.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Minus, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
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
                definitions::make(Opcode::True, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::False, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 > 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 < 2",
            // Constants are in reverse order: '1 < 2' is '2 > 1'
            expected_constants: vec![Object::Number(2.), Object::Number(1.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Greater, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 == 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Equal, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "1 != 2",
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::NotEqual, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "true == false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::True, &[0], 1),
                definitions::make(Opcode::False, &[0], 1),
                definitions::make(Opcode::Equal, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "true != false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::True, &[0], 1),
                definitions::make(Opcode::False, &[0], 1),
                definitions::make(Opcode::NotEqual, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "!false",
            expected_constants: vec![],
            expected_instructions: vec![
                definitions::make(Opcode::False, &[0], 1),
                definitions::make(Opcode::Bang, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_conditional() {
    let tests = vec![
        CompilerTestCase {
            input: "if (true) { 10 }; 3333;",
            expected_constants: vec![Object::Number(10.), Object::Number(3333.)],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[0], 1),
                // 0001 : Jump to the 'Nil' instruction following 'then_stmt'
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : To Jump over the 'else_stmt' which is 'Nil' here.
                definitions::make(Opcode::Jump, &[11], 1),
                // 0010
                definitions::make(Opcode::Nil, &[0], 1),
                // 0011 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[0], 1),
                // 0012 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[1], 1),
                // 0015
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "if (true) { 10 } else { 20 } ; 3333;",
            expected_constants: vec![
                Object::Number(10.),
                Object::Number(20.),
                Object::Number(3333.),
            ],
            expected_instructions: vec![
                // 0000 : The condition
                definitions::make(Opcode::True, &[0], 1),
                // 0001: Jump to 'else_stmt' if condition is false
                definitions::make(Opcode::JumpIfFalse, &[10], 1),
                // 0004 : The 'then_stmt'
                definitions::make(Opcode::Constant, &[0], 1),
                // 0007 : Jump to the instruction following the 'if' expression
                definitions::make(Opcode::Jump, &[13], 1),
                // 0010 : The 'else_stmt'
                definitions::make(Opcode::Constant, &[1], 1),
                // 0013 : [ Not part of the if expr - Pop its result ]
                definitions::make(Opcode::Pop, &[0], 1),
                // 0014 : The instruction following the if expr
                definitions::make(Opcode::Constant, &[2], 1),
                // 0017
                definitions::make(Opcode::Pop, &[0], 1),
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
            expected_constants: vec![Object::Number(1.), Object::Number(2.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::SetGlobal, &[1], 1),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;one;",
            expected_constants: vec![Object::Number(1.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;let two = one;two;",
            expected_constants: vec![Object::Number(1.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::SetGlobal, &[1], 1),
                definitions::make(Opcode::GetGlobal, &[1], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_string_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: r#""p2sh""#,
            expected_constants: vec![Object::Str(String::from("p2sh"))],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
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
                definitions::make(Opcode::Add, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
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
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "[1, 2, 3]",
            expected_constants: vec![Object::Number(1.), Object::Number(2.), Object::Number(3.)],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "[1 + 2, 3 - 4, 5 * 6]",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::Number(3.),
                Object::Number(4.),
                Object::Number(5.),
                Object::Number(6.),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Add, &[0], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Sub, &[0], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Mul, &[0], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Pop, &[0], 1),
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
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2, 3: 4, 5: 6}",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::Number(3.),
                Object::Number(4.),
                Object::Number(5.),
                Object::Number(6.),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Map, &[6], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2 + 3, 4: 5 * 6}",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::Number(3.),
                Object::Number(4.),
                Object::Number(5.),
                Object::Number(6.),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Add, &[0], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Constant, &[5], 1),
                definitions::make(Opcode::Mul, &[0], 1),
                definitions::make(Opcode::Map, &[4], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "[1, 2, 3][1 + 1]",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::Number(3.),
                Object::Number(1.),
                Object::Number(1.),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Array, &[3], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Constant, &[4], 1),
                definitions::make(Opcode::Add, &[0], 1),
                definitions::make(Opcode::Index, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
        CompilerTestCase {
            input: "{1: 2}[2 - 1]",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::Number(2.),
                Object::Number(1.),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[0], 1),
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Map, &[2], 1),
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Constant, &[3], 1),
                definitions::make(Opcode::Sub, &[0], 1),
                definitions::make(Opcode::Index, &[], 1),
                definitions::make(Opcode::Pop, &[], 1),
            ],
        },
    ];

    run_compiler_tests(&tests);
}

#[test]
fn test_functions() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { return 5 + 10 }",
            expected_constants: vec![
                Object::Number(5.),
                Object::Number(10.),
                Object::CompiledFunc(Rc::new(CompiledFunction {
                    instructions: concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        definitions::make(Opcode::Add, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[0], 1),
                    ]),
                })),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            input: "fn() { 1; 2 }",
            expected_constants: vec![
                Object::Number(1.),
                Object::Number(2.),
                Object::CompiledFunc(Rc::new(CompiledFunction {
                    instructions: concat_instructions(&[
                        definitions::make(Opcode::Constant, &[0], 1),
                        // Pop the first value
                        definitions::make(Opcode::Pop, &[0], 1),
                        definitions::make(Opcode::Constant, &[1], 1),
                        // The pop is replaced by the implicit return
                        definitions::make(Opcode::ReturnValue, &[0], 1),
                    ]),
                })),
            ],
            expected_instructions: vec![
                definitions::make(Opcode::Constant, &[2], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}

#[test]
fn test_functions_without_return_value() {
    let tests = vec![CompilerTestCase {
        input: "fn() { }",
        expected_constants: vec![Object::CompiledFunc(Rc::new(CompiledFunction {
            instructions: definitions::make(Opcode::Return, &[0], 1),
        }))],
        expected_instructions: vec![
            definitions::make(Opcode::Constant, &[0], 1),
            definitions::make(Opcode::Pop, &[0], 1),
        ],
    }];
    run_compiler_tests(&tests);
}

#[test]
fn test_compiler_scopes() {
    let mut compiler = Compiler::new();
    assert_eq!(compiler.scope_index, 0);
    compiler.emit(Opcode::Mul, &[0], 1);

    compiler.enter_scope();
    assert_eq!(compiler.scope_index, 1);
    compiler.emit(Opcode::Sub, &[0], 1);
    let scope_instructions_len = compiler.scopes[compiler.scope_index].instructions.len();
    assert_eq!(scope_instructions_len, 1);

    let last_instruction = compiler.scopes[compiler.scope_index].last_ins.clone();
    assert_eq!(last_instruction.opcode, Opcode::Sub);

    compiler.leave_scope();
    assert_eq!(compiler.scope_index, 0);

    compiler.emit(Opcode::Add, &[0], 1);
    let scope_instructions_len = compiler.scopes[compiler.scope_index].instructions.len();
    assert_eq!(scope_instructions_len, 2);

    let last_instruction = compiler.scopes[compiler.scope_index].last_ins.clone();
    assert_eq!(last_instruction.opcode, Opcode::Add);
}

#[test]
fn test_function_calls() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { 24 }()",
            expected_constants: vec![
                Object::Number(24.),
                Object::CompiledFunc(Rc::new(CompiledFunction {
                    instructions: concat_instructions(&[
                        // The literal '24'
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[0], 1),
                    ]),
                })),
            ],
            expected_instructions: vec![
                // The compiled function
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::Call, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
        CompilerTestCase {
            // Function is bound to a name here
            input: "let noArg = fn() { 24 }; noArg();",
            expected_constants: vec![
                Object::Number(24.),
                Object::CompiledFunc(Rc::new(CompiledFunction {
                    instructions: concat_instructions(&[
                        // The literal '24'
                        definitions::make(Opcode::Constant, &[0], 1),
                        definitions::make(Opcode::ReturnValue, &[0], 1),
                    ]),
                })),
            ],
            expected_instructions: vec![
                // The compiled function
                definitions::make(Opcode::Constant, &[1], 1),
                definitions::make(Opcode::SetGlobal, &[0], 1),
                definitions::make(Opcode::GetGlobal, &[0], 1),
                definitions::make(Opcode::Call, &[0], 1),
                definitions::make(Opcode::Pop, &[0], 1),
            ],
        },
    ];
    run_compiler_tests(&tests);
}
