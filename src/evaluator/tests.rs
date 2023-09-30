use std::cell::RefCell;
use std::rc::Rc;

use super::object::*;
use super::*;
use crate::evaluator::error::RTError;
use crate::parser::*;
use crate::scanner::*;

#[cfg(test)]
fn check_parse_errors(parser: &Parser) {
    if parser.print_errors() {
        panic!("{} parse errors", parser.parse_errors().len());
    }
}

#[cfg(test)]
fn test_numeric_object(evaluated: Object, expected: f64) {
    if let Object::Number(num) = evaluated {
        assert_eq!(
            num, expected,
            "object has wrong value. got={}, want={}",
            num, expected
        );
    } else {
        panic!("object is not numeric. got={}", evaluated);
    }
}

#[cfg(test)]
fn test_boolean_object(evaluated: Object, expected: bool) {
    if let Object::Bool(b) = evaluated {
        assert_eq!(
            b, expected,
            "object has wrong value. got={}, want={}",
            b, expected
        );
    } else {
        panic!("object is not boolean. got={}", evaluated);
    }
}

#[cfg(test)]
fn test_nil_object(evaluated: Object) {
    if let Object::Nil = evaluated {
    } else {
        panic!("object is not nil. got={}", evaluated);
    }
}

#[cfg(test)]
fn test_eval(input: &str) -> Result<Object, RTError> {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    check_parse_errors(&parser);
    let environment = Rc::new(RefCell::new(Environment::new()));
    let mut evaluator = Evaluator::new();
    evaluator.eval_program(&environment, program)
}

#[test]
fn test_eval_numeric_expr() {
    struct NumericObj {
        input: &'static str,
        expected: f64,
    }
    let numeric_tests = vec![
        NumericObj {
            input: "5",
            expected: 5.,
        },
        NumericObj {
            input: "10",
            expected: 10.,
        },
        NumericObj {
            input: "-5",
            expected: -5.,
        },
        NumericObj {
            input: "-10",
            expected: -10.,
        },
        NumericObj {
            input: "5 + 5 + 5 + 5 - 10",
            expected: 10.,
        },
        NumericObj {
            input: "2 * 2 * 2 * 2 * 2",
            expected: 32.,
        },
        NumericObj {
            input: "-50 + 100 + -50",
            expected: 0.,
        },
        NumericObj {
            input: "5 * 2 + 10",
            expected: 20.,
        },
        NumericObj {
            input: "5 + 2 * 10",
            expected: 25.,
        },
        NumericObj {
            input: "20 + 2 * -10",
            expected: 0.,
        },
        NumericObj {
            input: "50 / 2 * 2 + 10",
            expected: 60.,
        },
        NumericObj {
            input: "2 * (5 + 10)",
            expected: 30.,
        },
        NumericObj {
            input: "3 * 3 * 3 + 10",
            expected: 37.,
        },
        NumericObj {
            input: "3 * (3 * 3) + 10",
            expected: 37.,
        },
        NumericObj {
            input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
            expected: 50.,
        },
    ];
    for test in numeric_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => test_numeric_object(evaluated, test.expected),
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_eval_boolean_expr() {
    struct BooleanObj {
        input: &'static str,
        expected: bool,
    }
    let boolean_tests = vec![
        BooleanObj {
            input: "true",
            expected: true,
        },
        BooleanObj {
            input: "false",
            expected: false,
        },
        BooleanObj {
            input: "1 < 2",
            expected: true,
        },
        BooleanObj {
            input: "1 > 2",
            expected: false,
        },
        BooleanObj {
            input: "1 < 1",
            expected: false,
        },
        BooleanObj {
            input: "1 > 1",
            expected: false,
        },
        BooleanObj {
            input: "1 == 1",
            expected: true,
        },
        BooleanObj {
            input: "1 != 1",
            expected: false,
        },
        BooleanObj {
            input: "1 == 2",
            expected: false,
        },
        BooleanObj {
            input: "1 != 2",
            expected: true,
        },
        BooleanObj {
            input: "true == true",
            expected: true,
        },
        BooleanObj {
            input: "false == false",
            expected: true,
        },
        BooleanObj {
            input: "true == false",
            expected: false,
        },
        BooleanObj {
            input: "true != false",
            expected: true,
        },
        BooleanObj {
            input: "false != true",
            expected: true,
        },
        BooleanObj {
            input: "(1 < 2) == true",
            expected: true,
        },
        BooleanObj {
            input: "(1 < 2) == false",
            expected: false,
        },
        BooleanObj {
            input: "(1 > 2) == true",
            expected: false,
        },
        BooleanObj {
            input: "(1 > 2) == false",
            expected: true,
        },
    ];
    for test in boolean_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => test_boolean_object(evaluated, test.expected),
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_eval_bang_operator() {
    struct BangExpr {
        input: &'static str,
        expected: bool,
    }
    let boolean_tests = vec![
        BangExpr {
            input: "!true",
            expected: false,
        },
        BangExpr {
            input: "!false",
            expected: true,
        },
        BangExpr {
            input: "!5",
            expected: false,
        },
        BangExpr {
            input: "!!true",
            expected: true,
        },
        BangExpr {
            input: "!!false",
            expected: false,
        },
    ];
    for test in boolean_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => test_boolean_object(evaluated, test.expected),
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_if_else_expr() {
    struct IfElseExpr {
        input: &'static str,
        expected: Object,
    }
    let if_else_tests = vec![
        IfElseExpr {
            input: "if (true) { 10 }",
            expected: Object::Number(10.),
        },
        IfElseExpr {
            input: "if (false) { 10 }",
            expected: Object::Nil,
        },
        IfElseExpr {
            input: "if (1) { 10 }",
            expected: Object::Number(10.),
        },
        IfElseExpr {
            input: "if (1 < 2) { 10 }",
            expected: Object::Number(10.),
        },
        IfElseExpr {
            input: "if (1 > 2) { 10 }",
            expected: Object::Nil,
        },
        IfElseExpr {
            input: "if (1 < 2) { 10 } else { 20 }",
            expected: Object::Number(10.),
        },
        IfElseExpr {
            input: "if (1 > 2) { 10 } else { 20 }",
            expected: Object::Number(20.),
        },
    ];
    for test in if_else_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => match test.expected {
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                Object::Nil => test_nil_object(evaluated),
                _ => panic!("Invalid expected object"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_return_stmt() {
    struct ReturnTest {
        input: &'static str,
        expected: Object,
    }
    let if_else_tests = vec![
        ReturnTest {
            input: "return 10;",
            expected: Object::Number(10.),
        },
        ReturnTest {
            input: "return 10; 9;",
            expected: Object::Number(10.),
        },
        ReturnTest {
            input: "return 2 * 5; 9;",
            expected: Object::Number(10.),
        },
        ReturnTest {
            input: "9; return 2 * 5; 9;",
            expected: Object::Number(10.),
        },
        ReturnTest {
            input: "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
            expected: Object::Number(10.),
        },
    ];
    for test in if_else_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => match test.expected {
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                _ => panic!("Invalid expected object"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_error_handling() {
    struct ErrorTest {
        input: &'static str,
        expected: RTError,
    }
    let error_tests = vec![
        ErrorTest {
            input: "5 + true;",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "5 + true; 5;",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "-true",
            expected: RTError::new("invalid unary operation", 1),
        },
        ErrorTest {
            input: "true + false;",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "5; true + false; 5",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "if (10 > 1) { true + false; }",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
            expected: RTError::new("invalid binary operation", 1),
        },
        ErrorTest {
            input: "foobar",
            expected: RTError::new("Undefined identifier: 'foobar'", 1),
        },
    ];
    for (i, test) in error_tests.iter().enumerate() {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(obj) => {
                panic!("[{}] No error object returned. got={:?}", i, obj);
            }
            Err(err) => {
                assert_eq!(err.msg, test.expected.msg, "[{}] wrong error message", i);
            }
        }
    }
}

#[test]
fn test_let_statement() {
    struct LetTest {
        input: &'static str,
        expected: f64,
    }
    let error_tests = vec![
        LetTest {
            input: "let a = 5; a;",
            expected: 5.,
        },
        LetTest {
            input: "let a = 5 * 5; a;",
            expected: 25.,
        },
        LetTest {
            input: "let a = 5; let b = a; b;",
            expected: 5.,
        },
        LetTest {
            input: "let a = 5; let b = a; let c = a + b + 5; c;",
            expected: 15.,
        },
    ];
    for test in error_tests.iter() {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(obj) => test_numeric_object(obj, test.expected),
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_function_object() {
    let input = "fn(x) { x + 2; };";
    let expected_body = "(x + 2)";
    let evaluated = test_eval(input);
    match evaluated {
        Ok(obj) => {
            if let Object::Func(fun) = obj {
                if fun.params.len() != 1 {
                    panic!("functon has wrong #paramters. got={:?}", fun.params.len());
                }
                if fun.params[0].to_string() != "x" {
                    panic!("parameter is not 'x'. got={}", fun.params[0]);
                }
                assert_eq!(fun.body.to_string(), expected_body);
            } else {
                panic!("object is not a function. got={:?}", obj);
            }
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_function_calls() {
    struct FunTest {
        input: &'static str,
        expected: f64,
    }
    let fun_tests = vec![
        FunTest {
            input: "let identity = fn(x) { x; }; identity(5);",
            expected: 5.,
        },
        FunTest {
            input: "let identity = fn(x) { return x; }; identity(5);",
            expected: 5.,
        },
        FunTest {
            input: "let double = fn(x) { return x * 2; }; double(5);",
            expected: 10.,
        },
        FunTest {
            input: "let add = fn(x, y) { return x + y; }; add(5 + 5, add(5, 5));",
            expected: 20.,
        },
        FunTest {
            input: "fn(x) { x; }(5)",
            expected: 5.,
        },
    ];
    for test in fun_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(obj) => test_numeric_object(obj, test.expected),
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_closures() {
    struct ClosureTest {
        input: &'static str,
        expected: Object,
    }
    let closure_tests = vec![
        ClosureTest {
            input: "fn(x) { x == 10 } (5)",
            expected: Object::Bool(false),
        },
        ClosureTest {
            input: "fn(x) { x == 10 } (10)",
            expected: Object::Bool(true),
        },
        ClosureTest {
            input: "
            let newAdder = fn(x) {
                fn(y) {x + y};
            }
            let addTwo = newAdder(2);
            addTwo(2);
            ",
            expected: Object::Number(4.),
        },
        ClosureTest {
            input: "
            let add = fn(a, b) { a + b;}
            let sub = fn(a, b) { a - b;}
            let applyFunc = fn(a, b, func) { func(a, b) };
            applyFunc(2, 2, add);
            ",
            expected: Object::Number(4.),
        },
    ];
    for test in closure_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => match test.expected {
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                Object::Bool(expected) => test_boolean_object(evaluated, expected),
                _ => panic!("Invalid expected object"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}
