#![allow(unused_imports)]
use crate::common::environment::Environment;
use crate::common::error::RTError;
use crate::common::object::*;
use crate::evaluator::Evaluator;
use crate::parser::*;
use crate::scanner::*;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;

#[cfg(test)]
fn check_parse_errors(parser: &Parser) {
    if parser.print_errors() {
        panic!("{} parse errors", parser.parse_errors().len());
    }
}

#[cfg(test)]
fn test_numeric_object(evaluated: Rc<Object>, expected: f64) {
    if let Object::Number(num) = *evaluated {
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
fn test_string_object(evaluated: Rc<Object>, expected: &str) {
    if let Object::Str(s) = &*evaluated {
        assert_eq!(
            s, expected,
            "object has wrong value. got={}, want={}",
            s, expected
        );
    } else {
        panic!("object is not a string. got={}", evaluated);
    }
}

#[cfg(test)]
fn test_boolean_object(evaluated: Rc<Object>, expected: bool) {
    if let Object::Bool(b) = *evaluated {
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
fn test_nil_object(evaluated: Rc<Object>) {
    if let Object::Nil = *evaluated {
    } else {
        panic!("object is not nil. got={}", evaluated);
    }
}

#[cfg(test)]
fn test_eval(input: &str) -> Result<Rc<Object>, RTError> {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    check_parse_errors(&parser);
    let environment = Rc::new(RefCell::new(Environment::default()));
    let mut evaluator = Evaluator::new();
    evaluator.eval_program(&environment, program)
}

#[test]
fn test_string_literal() {
    let input = r#""Hello, World""#;
    let expected = "Hello, World";
    let evaluated = test_eval(input);
    match evaluated {
        Ok(evaluated) => test_string_object(evaluated, expected),
        Err(e) => panic!("{}", e),
    }
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
fn test_eval_string_expr() {
    struct StringObj {
        input: &'static str,
        expected: &'static str,
    }
    let numeric_tests = vec![
        StringObj {
            input: r#""Hello, World""#,
            expected: "Hello, World",
        },
        StringObj {
            input: r#""Hello, " + "World!!""#,
            expected: "Hello, World!!",
        },
        StringObj {
            input: r#""*" * 30"#,
            expected: "******************************",
        },
        StringObj {
            input: r#"30 * "*" "#,
            expected: "******************************",
        },
    ];
    for test in numeric_tests {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => test_string_object(evaluated, test.expected),
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
        ErrorTest {
            input: r#""foo" - "bar""#,
            expected: RTError::new("invalid binary operator", 1),
        },
        ErrorTest {
            input: "1[1]",
            expected: RTError::new("index operator not supported", 1),
        },
        ErrorTest {
            input: "true[1]",
            expected: RTError::new("index operator not supported", 1),
        },
        ErrorTest {
            input: r#""not_array"[1]"#,
            expected: RTError::new("index operator not supported", 1),
        },
        ErrorTest {
            input: "fn(x) {x}[1]",
            expected: RTError::new("index operator not supported", 1),
        },
        ErrorTest {
            input: "[1, 2, 3] [fn(x) {x}]",
            expected: RTError::new("invalid index to array object", 1),
        },
        ErrorTest {
            input: r#"{"name": "p2sh"} [fn(x) {x}]"#,
            expected: RTError::new("hash key should be a numeric, a string or a boolean", 1),
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
            if let Object::Func(fun) = &*obj.clone() {
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

#[test]
fn test_builtin_functions() {
    let mut failed = 0;
    struct BuiltinTest {
        input: &'static str,
        expected: Object,
    }
    let builtin_tests = vec![
        BuiltinTest {
            input: r#"len("")"#,
            expected: Object::Number(0.),
        },
        BuiltinTest {
            input: r#"len("four")"#,
            expected: Object::Number(4.),
        },
        BuiltinTest {
            input: r#"len("hello world")"#,
            expected: Object::Number(11.),
        },
        BuiltinTest {
            input: "len([])",
            expected: Object::Number(0.),
        },
        BuiltinTest {
            input: "len([1, 2, 3])",
            expected: Object::Number(3.),
        },
        BuiltinTest {
            input: "first([])",
            expected: Object::Nil,
        },
        BuiltinTest {
            input: "last([])",
            expected: Object::Nil,
        },
        BuiltinTest {
            input: "rest([])",
            expected: Object::Nil,
        },
        BuiltinTest {
            input: "first([1, 2, 3])",
            expected: Object::Number(1.),
        },
        BuiltinTest {
            input: "last([1, 2, 3])",
            expected: Object::Number(3.),
        },
        BuiltinTest {
            input: "rest([1, 2, 3])",
            expected: Object::Arr(Array {
                elements: vec![Rc::new(Object::Number(2.)), Rc::new(Object::Number(3.))],
            }),
        },
        BuiltinTest {
            input: "let a = [1, 2, 3, 4]; rest(rest(rest(a)))",
            expected: Object::Arr(Array {
                elements: vec![Rc::new(Object::Number(4.))],
            }),
        },
        BuiltinTest {
            input: r#"puts("Hello")"#,
            expected: Object::Nil,
        },
    ];
    for (i, test) in builtin_tests.iter().enumerate() {
        let result = test_eval(test.input);
        match result {
            Ok(evaluated) => match test.expected.clone() {
                Object::Arr(arr_exp) => {
                    if let Object::Arr(arr) = &*evaluated.clone() {
                        for (i, item) in arr_exp.elements.iter().enumerate() {
                            if let Object::Number(exp) = *arr.elements[i] {
                                test_numeric_object((*item).clone(), exp)
                            }
                        }
                    } else {
                        eprintln!("[{}] failed built-in test - array expected", i);
                        failed += 1;
                    }
                }
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                Object::Nil => test_nil_object(evaluated),
                _ => {
                    eprintln!("[{}] unhandled expected value", i);
                    failed += 1;
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                failed += 1;
            }
        }
        if failed != 0 {
            panic!("{} built-in tests failed", failed);
        }
    }
    struct ErrorTest {
        input: &'static str,
        expected: RTError,
    }
    let error_tests = vec![
        ErrorTest {
            input: "len(1)",
            expected: RTError::new("len: unsupported argument", 1),
        },
        ErrorTest {
            input: r#"len("one", "two")"#,
            expected: RTError::new("len: takes one argument. got=2", 1),
        },
    ];

    for (i, test) in error_tests.iter().enumerate() {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(obj) => {
                panic!("[{}] No error object returned. got={:?}", i, obj);
            }
            Err(err) => {
                if err.msg != test.expected.msg {
                    eprintln!(
                        "[{}] wrong error message. expected='{}', got='{}'",
                        i, err.msg, test.expected.msg
                    );
                    failed += 1;
                }
            }
        }
    }
    if failed != 0 {
        panic!("{} builtin function tests failed", failed);
    }
}

#[test]
fn test_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let evaluated = test_eval(input);

    match evaluated {
        Ok(obj) => match &*obj {
            Object::Arr(arr) => {
                test_numeric_object(arr.elements[0].clone(), 1.);
                test_numeric_object(arr.elements[1].clone(), 4.);
                test_numeric_object(arr.elements[2].clone(), 6.);
            }
            _ => panic!("object is not an array. got={:?}", *obj),
        },
        _ => panic!("object is not an array. got={:?}", evaluated),
    }
}

#[test]
fn test_array_index_expressions() {
    struct ArrayIndexExpr {
        input: &'static str,
        expected: Object,
    }
    let index_exprs = vec![
        ArrayIndexExpr {
            input: "[1, 2, 3][0]",
            expected: Object::Number(1.),
        },
        ArrayIndexExpr {
            input: "[1, 2, 3][1]",
            expected: Object::Number(2.),
        },
        ArrayIndexExpr {
            input: "[1, 2, 3][2]",
            expected: Object::Number(3.),
        },
        ArrayIndexExpr {
            input: "let i = 0; [1][i]",
            expected: Object::Number(1.),
        },
        ArrayIndexExpr {
            input: "let my_arr = [1, 2, 3]; my_arr[0] + my_arr[1] + my_arr[2];",
            expected: Object::Number(6.),
        },
        ArrayIndexExpr {
            input: "let my_arr = [1, 2, 3]; let i = my_arr[0]; my_arr[i];",
            expected: Object::Number(2.),
        },
        ArrayIndexExpr {
            input: "[1, 2, 3][3]",
            expected: Object::Nil,
        },
        ArrayIndexExpr {
            input: "[1, 2, 3][-1]",
            expected: Object::Nil,
        },
    ];

    for test in index_exprs {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => match test.expected {
                Object::Number(expect) => test_numeric_object(evaluated, expect),
                Object::Nil => test_nil_object(evaluated),
                _ => panic!("Invalid expected object"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}

#[cfg(test)]
fn make_object_hash(obj: Object) -> u64 {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_hash_key() {
    let hash_s1 = make_object_hash(Object::Str("Hello, World!".into()));
    let hash_s2 = make_object_hash(Object::Str("Hello, World!".into()));
    let hash_s3 = make_object_hash(Object::Str("Hello  World!".into()));
    assert_eq!(
        hash_s1, hash_s2,
        "strings with the same content have different hash keys"
    );
    assert_ne!(
        hash_s1, hash_s3,
        "strings with different content have the same hash keys"
    );

    let hash_n1 = make_object_hash(Object::Number(11111.11));
    let hash_n2 = make_object_hash(Object::Number(11111.11));
    let hash_n3 = make_object_hash(Object::Number(22222.22));

    assert_eq!(
        hash_n1, hash_n2,
        "numbers with the same content have different hash keys"
    );
    assert_ne!(
        hash_n1, hash_n3,
        "numbers with different content have the same hash keys"
    );

    let hash_b1 = make_object_hash(Object::Bool(true));
    let hash_b2 = make_object_hash(Object::Bool(true));
    let hash_b3 = make_object_hash(Object::Bool(false));

    assert_eq!(
        hash_b1, hash_b2,
        "booleans with the same content have different hash keys"
    );
    assert_ne!(
        hash_b1, hash_b3,
        "booleans with different content have the same hash keys"
    );
}

#[test]
fn test_hash_literals() {
    let input = r#"let two = "two";
    {
        "one": 10 - 9,
        two: 1 + 1,
        "three": 6 / 2,
        4: 4,
        true: 5,
        false: 6
    }"#;
    let evaluated = test_eval(input);
    if let Ok(obj) = evaluated {
        match &*obj {
            Object::Map(map) => {
                test_numeric_object(map.pairs[&Object::Str("one".into())].clone(), 1.);
                test_numeric_object(map.pairs[&Object::Str("two".into())].clone(), 2.);
                test_numeric_object(map.pairs[&Object::Str("three".into())].clone(), 3.);
                test_numeric_object(map.pairs[&Object::Number(4.)].clone(), 4.);
                test_numeric_object(map.pairs[&Object::Bool(true)].clone(), 5.);
                test_numeric_object(map.pairs[&Object::Bool(false)].clone(), 6.);
            }
            _ => panic!("object is not an hash literal. got={:?}", *obj),
        }
    } else {
        panic!("object is not an hash literal. got={:?}", evaluated);
    }
}

#[test]
fn test_hash_index_expressions() {
    struct IndexExpression {
        input: &'static str,
        expected: Object,
    }
    let index_exprs = vec![
        IndexExpression {
            input: r#"{"foo": 5}["foo"]"#,
            expected: Object::Number(5.),
        },
        IndexExpression {
            input: r#"{"foo": 5}["bar"]"#,
            expected: Object::Nil,
        },
        IndexExpression {
            input: r#"{}["foo"]"#,
            expected: Object::Nil,
        },
        IndexExpression {
            input: "{5: 5}[5]",
            expected: Object::Number(5.),
        },
        IndexExpression {
            input: "{true: 1}[true]",
            expected: Object::Number(1.),
        },
        IndexExpression {
            input: "{false: 0}[false]",
            expected: Object::Number(0.),
        },
        IndexExpression {
            input: r#"{true: "true"}[true]"#,
            expected: Object::Str("true".to_string()),
        },
        IndexExpression {
            input: r#"[{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}][0]["name"]"#,
            expected: Object::Str("Alice".to_string()),
        },
        IndexExpression {
            input: r#"[{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}][1]["name"]"#,
            expected: Object::Str("Anna".to_string()),
        },
    ];

    for test in index_exprs {
        let evaluated = test_eval(test.input);
        match evaluated {
            Ok(evaluated) => match test.expected {
                Object::Number(expect) => test_numeric_object(evaluated, expect),
                Object::Nil => test_nil_object(evaluated),
                Object::Str(s) => test_string_object(evaluated, &s),
                _ => panic!("Invalid expected object"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}

#[test]
fn test_recursive_fibonacci() {
    let input = r#"
        let fibonacci = fn(x) {
            if (x == 0) {
                return 0;
            } else {
                if (x == 1) {
                    return 1;
                } else {
                    fibonacci(x - 1) + fibonacci(x - 2);
                }
            }
        };
        fibonacci(15);
        "#;
    let expected = Object::Number(610.);

    let evaluated = test_eval(input);
    match evaluated {
        Ok(evaluated) => match expected {
            Object::Number(expect) => test_numeric_object(evaluated, expect),
            Object::Nil => test_nil_object(evaluated),
            Object::Str(s) => test_string_object(evaluated, &s),
            _ => panic!("Invalid expected object"),
        },
        Err(e) => panic!("{}", e),
    }
}
