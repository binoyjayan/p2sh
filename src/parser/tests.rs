#![allow(unused_imports)]
use super::*;
use std::collections::HashMap;

#[cfg(test)]
#[derive(Clone)]
enum Literal {
    Ident(&'static str),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(&'static str),
}

#[cfg(test)]
fn parse_test_program(input: &str, num_stmts: usize) -> Program {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    check_parse_errors(&parser);

    if program.statements.len() != num_stmts {
        assert_eq!(
            num_stmts,
            program.statements.len(),
            "program.statements does not contain {} statement(s). got={}",
            num_stmts,
            program.statements.len()
        );
    }
    program
}

#[cfg(test)]
fn parse_test_program_failures(input: &str) -> Vec<String> {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let _program = parser.parse_program();
    parser.parse_errors().to_vec()
}

#[cfg(test)]
fn check_parse_errors(parser: &Parser) {
    if parser.print_errors() {
        panic!("{} parse errors", parser.parse_errors().len());
    }
}

#[cfg(test)]
fn test_integer_literal(expr: &Expression, expected: i64) {
    if let Expression::Integer(num) = expr {
        if num.value != expected {
            panic!("number.value not '{}'. got='{}'", expected, num.value);
        }
    } else {
        panic!("expr not an Integer. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_float_literal(expr: &Expression, expected: f64) {
    if let Expression::Float(num) = expr {
        if num.value != expected {
            panic!("number.value not '{}'. got='{}'", expected, num.value);
        }
    } else {
        panic!("expr not an Float. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_string_literal(expr: &Expression, expected: &str) {
    if let Expression::Str(s) = expr {
        if s.value != expected {
            panic!("string.value not '{}'. got='{}'", expected, s.value);
        }
    } else {
        panic!("expr not a String. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_boolean_literal(expr: &Expression, expected: bool) {
    if let Expression::Bool(num) = expr {
        if num.value != expected {
            panic!("number.value not '{}'. got='{}'", expected, num.value);
        }
    } else {
        panic!("expr not Boolean. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_ident_token_literal(ident: &Identifier, value: &str) {
    if ident.value == value {
        if ident.token.literal != value {
            panic!(
                "ident.token.literal not {}. got={}",
                value, ident.token.literal
            );
        }
    } else {
        panic!("ident.value not {}. got={}", value, ident);
    }
}

#[cfg(test)]
fn test_identifier(expression: &Expression, value: &str) {
    if let Expression::Ident(ident) = expression {
        test_ident_token_literal(ident, value);
    } else {
        panic!("expr not an Identifier. got={:?}", expression);
    }
}

#[cfg(test)]
fn test_literal(expression: &Expression, value: Literal) {
    match value {
        Literal::Ident(value) => {
            test_identifier(expression, value);
        }
        Literal::Integer(value) => {
            test_integer_literal(expression, value);
        }
        Literal::Float(value) => {
            test_float_literal(expression, value);
        }
        Literal::Bool(value) => {
            test_boolean_literal(expression, value);
        }
        Literal::Str(value) => {
            test_string_literal(expression, value);
        }
    }
}

// Generic prefix expression test helper that accepts a generic literal (number/string)
#[cfg(test)]
fn test_prefix_expression(expression: &Expression, operator: &str, right: Literal) {
    if let Expression::Unary(expr) = expression {
        if expr.operator != operator {
            panic!(
                "expr.operator is not '{}'. got='{}'",
                expr.operator, operator
            );
        }
        test_literal(&*expr.right, right);
    } else {
        panic!("expr not a Prefix expression. got={:?}", expression);
    }
}

// Generic infix expression test helper that accepts a generic literal (number/string)
#[cfg(test)]
fn test_infix_expression(expression: &Expression, left: Literal, operator: &str, right: Literal) {
    if let Expression::Binary(expr) = expression {
        if expr.operator != operator {
            panic!(
                "expr.operator is not '{}'. got='{}'",
                expr.operator, operator
            );
        }
        test_literal(&*expr.left, left);
        test_literal(&*expr.right, right);
    } else {
        panic!("expr not an Infix expression. got={:?}", expression);
    }
}

// Assignment expression
#[cfg(test)]
fn test_assign_expression(expression: &Expression, left: Literal, right: Literal) {
    if let Expression::Assign(expr) = expression {
        test_literal(&*expr.left, left);
        test_literal(&*expr.right, right);
    } else {
        panic!("expr not an Assign expression. got={:?}", expression);
    }
}

#[test]
fn test_let_statements() {
    struct TestLet {
        input: &'static str,
        expected_id: &'static str,
        expected_val: Literal,
    }
    let let_tests = vec![
        TestLet {
            input: "let x = 5;",
            expected_id: "x",
            expected_val: Literal::Integer(5),
        },
        TestLet {
            input: "let y = true;",
            expected_id: "y",
            expected_val: Literal::Bool(true),
        },
        TestLet {
            input: "let foobar = y;",
            expected_id: "foobar",
            expected_val: Literal::Ident("y"),
        },
    ];

    for test in let_tests {
        let program = parse_test_program(test.input, 1);

        let stmt = &program.statements[0];
        test_let_statement(&stmt, test.expected_id, test.expected_val);
    }
}

#[cfg(test)]
fn test_let_statement(stmt: &Statement, expected_id: &str, expected_val: Literal) {
    if stmt.token_literal() != "let" {
        panic!("stmt.token_literal not 'let'. got={}", stmt.token_literal());
    }

    if let Statement::Let(s) = stmt {
        if s.name.value != expected_id {
            panic!(
                "let_stmt.name.value not '{}'. got={}",
                expected_id, s.name.value
            );
        }
        test_literal(&s.value, expected_val);
    } else {
        panic!("stmt is not a 'let' statement");
    }
}

#[test]
fn test_return_statements() {
    let input = "
        return 5;
        return 10;
        return 993322;
        ";
    let program = parse_test_program(input, 3);

    let mut count = 0;
    for stmt in program.statements.iter() {
        if stmt.token_literal() != "return" {
            eprintln!(
                "stmt.token_literal() not 'return'. got={}",
                stmt.token_literal()
            );
            count += 1;
            continue;
        }
        if let Statement::Return(_stmt) = stmt {
        } else {
            count += 1;
            eprintln!("stmt is not a return statement. got={}", stmt);
        }
    }
    if count > 0 {
        panic!("{}/{} tests failed.", count, program.statements.len());
    }
}

#[test]
fn test_string_formatting() {
    let let_token = Token::new(TokenType::Let, "let", 1);
    let token_myvar1 = Token::new(TokenType::Identifier, "myvar1", 1);
    let ident_myvar1 = Identifier {
        token: token_myvar1,
        value: "myvar1".to_string(),
        access: AccessType::Get,
    };

    let token_myvar2 = Token::new(TokenType::Identifier, "myvar2", 2);
    let ident_myvar2 = Identifier {
        token: token_myvar2,
        value: "myvar2".to_string(),
        access: AccessType::Get,
    };

    let program = Program {
        statements: vec![Statement::Let(LetStmt {
            token: let_token,
            name: ident_myvar1,
            value: Expression::Ident(ident_myvar2),
        })],
    };

    let program_str = format!("{}", program);
    let expected_str = "let myvar1 = myvar2;";
    assert_eq!(
        program_str, expected_str,
        "program.string() wrong. got='{}'",
        program_str
    );
}

#[test]
fn test_identifier_expression() {
    let input = "foobar;";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if stmt.token_literal() != "foobar" {
        panic!(
            "stmt.token_literal() not 'foobar'. got={}",
            stmt.token_literal()
        );
    }
    if let Statement::Expr(stmt) = stmt {
        test_identifier(&stmt.value, "foobar");
    } else {
        panic!("stmt is not an expression statement. got={}", stmt);
    }
}

#[test]
fn test_numeric_literal_expression() {
    let input = "5;";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        test_integer_literal(&stmt.value, 5);
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_string_literal_expression() {
    let input = r#""hello world";"#;
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        test_string_literal(&stmt.value, "hello world");
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_prefix_expressions() {
    struct PrefixTest {
        input: &'static str,
        operator: &'static str,
        number: Literal,
    }
    let prefix_tests = vec![
        PrefixTest {
            input: "!5",
            operator: "!",
            number: Literal::Integer(5),
        },
        PrefixTest {
            input: "-15",
            operator: "-",
            number: Literal::Integer(15),
        },
        PrefixTest {
            input: "!5.0",
            operator: "!",
            number: Literal::Float(5.),
        },
        PrefixTest {
            input: "!true",
            operator: "!",
            number: Literal::Bool(true),
        },
        PrefixTest {
            input: "!false",
            operator: "!",
            number: Literal::Bool(false),
        },
        PrefixTest {
            input: "~5",
            operator: "~",
            number: Literal::Integer(5),
        },
    ];

    for test in prefix_tests {
        let program = parse_test_program(test.input, 1);

        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            test_prefix_expression(&stmt.value, test.operator, test.number);
        } else {
            panic!(
                "program.statements[0] is not an expression statement. got={}",
                stmt
            );
        }
    }
}

#[test]
fn test_parsing_infix_expressions() {
    struct InfixTest {
        input: &'static str,
        operator: &'static str,
        left: Literal,
        right: Literal,
    }
    let infix_tests = vec![
        InfixTest {
            input: "5 + 5;",
            operator: "+",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5.1 + 5.2;",
            operator: "+",
            left: Literal::Float(5.1),
            right: Literal::Float(5.2),
        },
        InfixTest {
            input: "5 - 5;",
            operator: "-",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 * 5;",
            operator: "*",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 / 5;",
            operator: "/",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 % 5;",
            operator: "%",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 > 5;",
            operator: ">",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 < 5;",
            operator: "<",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 == 5;",
            operator: "==",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "5 != 5;",
            operator: "!=",
            left: Literal::Integer(5),
            right: Literal::Integer(5),
        },
        InfixTest {
            input: "true == true",
            operator: "==",
            left: Literal::Bool(true),
            right: Literal::Bool(true),
        },
        InfixTest {
            input: "true != false",
            operator: "!=",
            left: Literal::Bool(true),
            right: Literal::Bool(false),
        },
        InfixTest {
            input: "false == false",
            operator: "==",
            left: Literal::Bool(false),
            right: Literal::Bool(false),
        },
        InfixTest {
            input: "5 && 2;",
            operator: "&&",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 || 2;",
            operator: "||",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 & 2;",
            operator: "&",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 | 2;",
            operator: "|",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 ^ 2;",
            operator: "^",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 << 2;",
            operator: "<<",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
        InfixTest {
            input: "5 >> 2;",
            operator: ">>",
            left: Literal::Integer(5),
            right: Literal::Integer(2),
        },
    ];

    for test in infix_tests {
        let program = parse_test_program(test.input, 1);
        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            test_infix_expression(&stmt.value, test.left, test.operator, test.right);
        } else {
            panic!(
                "program.statements[0] is not an expression statement. got={}",
                stmt
            );
        }
    }
}

#[test]
fn test_parsing_operator_precedence() {
    struct PrecedenceTest {
        input: &'static str,
        expected: &'static str,
        num_stmts: usize,
    }
    let precedence_tests = vec![
        PrecedenceTest {
            input: "",
            expected: "",
            num_stmts: 0,
        },
        PrecedenceTest {
            input: "a",
            expected: "a",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "-a",
            expected: "(-a)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "!-a",
            expected: "(!(-a))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "-a * b",
            expected: "((-a) * b)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "~a ^ b & c",
            expected: "((~a) ^ (b & c))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a + b + c",
            expected: "((a + b) + c)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a + b - c",
            expected: "((a + b) - c)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a * b * c",
            expected: "((a * b) * c)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a * b / c",
            expected: "((a * b) / c)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a + b * c + d / e - f",
            expected: "(((a + (b * c)) + (d / e)) - f)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "3 + 4;-5 * 5",
            expected: "(3 + 4)((-5) * 5)",
            num_stmts: 2,
        },
        PrecedenceTest {
            input: "5 > 4 == 3 < 4",
            expected: "(((5 > 4) == 3) < 4)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "5 < 4 != 3 > 4",
            expected: "(((5 < 4) != 3) > 4)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
            expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "true",
            expected: "true",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "false",
            expected: "false",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "3 > 5 == false",
            expected: "((3 > 5) == false)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "3 < 5 == true",
            expected: "((3 < 5) == true)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "1 + (2 + 3) + 4",
            expected: "((1 + (2 + 3)) + 4)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "(5 + 5) * 2",
            expected: "((5 + 5) * 2)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "2 / (5 + 5)",
            expected: "(2 / (5 + 5))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "20 / 2 % (5 + 5)",
            expected: "((20 / 2) % (5 + 5))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "-(5 + 5)",
            expected: "(-(5 + 5))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "!(true == true)",
            expected: "(!(true == true))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a + add(b * c) + d",
            expected: "((a + add((b * c))) + d)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
            expected: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "add(a + b + c * d / f + g)",
            expected: "add((((a + b) + ((c * d) / f)) + g))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a * [1, 2, 3, 4][b * c] * d",
            expected: "((a * ([1, 2, 3, 4][(b * c)])) * d)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "add(a * b[2], b[1], 2 * [1, 2][1])",
            expected: "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a = true",
            expected: "(a = true)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a = true",
            expected: "(a = true)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a = b = 1",
            expected: "(a = (b = 1))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "s = a + b - c",
            expected: "(s = ((a + b) - c))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a & b && c & d",
            expected: "((a & b) && (c & d))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a | b || c | d",
            expected: "((a | b) || (c | d))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a && b || c && d",
            expected: "((a && b) || (c && d))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a || b && c || d",
            expected: "((a || (b && c)) || d)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a & b ^ c | d & e",
            expected: "(((a & b) ^ c) | (d & e))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a ^ b & c | d ^ e",
            expected: "((a ^ (b & c)) | (d ^ e))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a << b >> c",
            expected: "((a << b) >> c)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a & b << c",
            expected: "(a & (b << c))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a << b + c",
            expected: "(a << (b + c))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "a && b == c || d",
            expected: "((a && (b == c)) || d)",
            num_stmts: 1,
        },
    ];

    for (i, test) in precedence_tests.iter().enumerate() {
        let program = parse_test_program(test.input, test.num_stmts);
        let actual = format!("{}", program);
        assert_eq!(actual, test.expected, "Test {}", i);
    }
}

#[test]
fn test_boolean_expressions() {
    let input = "true;false;";
    let program = parse_test_program(input, 2);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        test_boolean_literal(&stmt.value, true);
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }

    let stmt = &program.statements[1];
    if let Statement::Expr(stmt) = stmt {
        test_boolean_literal(&stmt.value, false);
    } else {
        panic!(
            "program.statements[1] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_if_then_expression() {
    let input = "if x < y { x }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::If(expr) = &stmt.value {
            test_infix_expression(
                &expr.condition,
                Literal::Ident("x"),
                "<",
                Literal::Ident("y"),
            );
            let num_stmts = expr.then_stmt.statements.len();
            assert_eq!(num_stmts, 1, "then_stmt count not 1. got={}", num_stmts);
            if let Statement::Expr(expr) = &expr.then_stmt.statements[0] {
                test_identifier(&expr.value, "x");
            } else {
                panic!(
                    "then_stmt.statements[0] is not an expression statement. got={}",
                    expr.then_stmt.statements[0]
                );
            }
            if expr.else_stmt.is_some() {
                panic!("expr.else_stmt was not nil. got={:?}", expr.else_stmt);
            }
        } else {
            panic!("stmt.expr is not an If expression. got={}", stmt.value);
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_if_then_else_expression() {
    let input = "if x < y { x } else { y }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::If(expr) = &stmt.value {
            test_infix_expression(
                &expr.condition,
                Literal::Ident("x"),
                "<",
                Literal::Ident("y"),
            );
            let num_stmts = expr.then_stmt.statements.len();
            assert_eq!(num_stmts, 1, "then_stmt count not 1. got={}", num_stmts);
            if let Statement::Expr(expr) = &expr.then_stmt.statements[0] {
                test_identifier(&expr.value, "x");
            } else {
                panic!(
                    "then_stmt.statements[0] is not an expression statement. got={}",
                    expr.then_stmt.statements[0]
                );
            }
            // If an else branch exists
            if let Some(else_stmt) = &expr.else_stmt {
                if let Statement::Expr(expr) = &else_stmt.statements[0] {
                    test_identifier(&expr.value, "y");
                } else {
                    panic!(
                        "else_stmt.statements[0] is not an expression statement. got={}",
                        else_stmt.statements[0]
                    );
                }
            }
        } else {
            panic!("stmt.expr is not an If expression. got={}", stmt.value);
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_function_literal() {
    let input = "fn(x, y) { x + y; }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Function(expr) = &stmt.value {
            assert_eq!(
                expr.params.len(),
                2,
                "FunctionLiteral params wrong. want 2, got={}",
                expr.params.len()
            );
            test_ident_token_literal(&expr.params[0], "x");
            test_ident_token_literal(&expr.params[1], "y");

            assert_eq!(
                expr.body.statements.len(),
                1,
                "function.body.statements has not one statements. got={}",
                expr.body.statements.len()
            );

            if let Statement::Expr(expr) = &expr.body.statements[0] {
                test_infix_expression(&expr.value, Literal::Ident("x"), "+", Literal::Ident("y"));
            } else {
                panic!(
                    "function.body.statements[0] is not an expression statement. got={}",
                    expr.body.statements[0]
                );
            }
        } else {
            panic!(
                "stmt.expr is not a FunctionLiteral expression. got={}",
                stmt.value
            );
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_call_expression() {
    let input = "add(1, 2 * 3, 4 + 5);";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Call(expr) = &stmt.value {
            test_identifier(&expr.func, "add");
            assert_eq!(
                expr.args.len(),
                3,
                "wrong length of arguments. got={}",
                expr.args.len()
            );
            test_literal(&expr.args[0], Literal::Integer(1));
            test_infix_expression(&expr.args[1], Literal::Integer(2), "*", Literal::Integer(3));
        } else {
            panic!("stmt.expr is not a CallExpr expression. got={}", stmt.value);
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_array_literal_expression() {
    let input = "[1, 2 * 2, 3 + 3]";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Array(expr) = &stmt.value {
            assert_eq!(
                expr.elements.len(),
                3,
                "len(array.elements) not 3. got={}",
                expr.elements.len()
            );
            // array element at index 0
            test_literal(&expr.elements[0], Literal::Integer(1));
            // array element at index 1
            test_infix_expression(
                &expr.elements[1],
                Literal::Integer(2),
                "*",
                Literal::Integer(2),
            );
            // array element at index 2
            test_infix_expression(
                &expr.elements[2],
                Literal::Integer(3),
                "+",
                Literal::Integer(3),
            );
        } else {
            panic!(
                "stmt.expr is not an ArrayLiteral expression. got={}",
                stmt.value
            );
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_array_index_expression() {
    let input = "myArray[1 + 1]";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Index(expr) = &stmt.value {
            test_identifier(&expr.left, "myArray");
            // array element at index 1
            test_infix_expression(&expr.index, Literal::Integer(1), "+", Literal::Integer(1));
        } else {
            panic!("stmt.expr is not an Array IndexExpr. got={}", stmt.value);
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_hash_literals_strings_keys() {
    let input = r#"{ "one": 1, "two": 2, "three": 3}"#;
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Hash(map_expr) = &stmt.value {
            assert_eq!(
                map_expr.pairs.len(),
                3,
                "hash.pairs has wrong length. wants=3 got={}",
                map_expr.pairs.len()
            );
            let mut map_expected: HashMap<&str, i64> = HashMap::new();
            map_expected.insert("one", 1);
            map_expected.insert("two", 2);
            map_expected.insert("three", 3);

            for (key, value) in map_expr.pairs.iter() {
                if let Expression::Str(k) = key {
                    if let Some(val) = map_expected.get(k.value.as_str()) {
                        test_integer_literal(value, *val);
                    } else {
                        panic!("key {} not found in hash", key);
                    }
                } else {
                    panic!("hash key is not a string literal. got={}", key);
                }
            }
        } else {
            panic!(
                "stmt.expr is not an HashLiteral expression. got={}",
                stmt.value
            );
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_empty_hash_literal() {
    let input = "{}";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Hash(map_expr) = &stmt.value {
            assert_eq!(
                map_expr.pairs.len(),
                0,
                "hash.pairs has wrong length. wants=0 got={}",
                map_expr.pairs.len()
            );
        } else {
            panic!(
                "stmt.expr is not an HashLiteral expression. got={}",
                stmt.value
            );
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_hash_literals_with_exprs() {
    let input = r#"{ "one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Hash(map_expr) = &stmt.value {
            assert_eq!(
                map_expr.pairs.len(),
                3,
                "hash.pairs has wrong length. wants=3 got={}",
                map_expr.pairs.len()
            );

            #[derive(Clone)]
            struct InfixTest {
                operator: &'static str,
                left: Literal,
                right: Literal,
            }

            let mut map_expected: HashMap<&str, InfixTest> = HashMap::new();
            // "one" = Infix(1  + 0)
            map_expected.insert(
                "one",
                InfixTest {
                    operator: "+",
                    left: Literal::Integer(0),
                    right: Literal::Integer(1),
                },
            );
            // "two" = Infix(10 - 8)
            map_expected.insert(
                "two",
                InfixTest {
                    operator: "-",
                    left: Literal::Integer(10),
                    right: Literal::Integer(8),
                },
            );
            // "three" = Infix(15 / 5)
            map_expected.insert(
                "three",
                InfixTest {
                    operator: "/",
                    left: Literal::Integer(15),
                    right: Literal::Integer(5),
                },
            );

            for (key, value) in map_expr.pairs.iter() {
                eprintln!("test: {} -->> {}", key, value);
                if let Expression::Str(k) = key {
                    if let Some(exp_val) = map_expected.get(k.value.as_str()) {
                        test_infix_expression(
                            value,
                            exp_val.left.clone(),
                            exp_val.operator,
                            exp_val.right.clone(),
                        );
                    } else {
                        panic!("key {} not found in hash", key);
                    }
                } else {
                    panic!("hash key is not a string literal. got={}", key);
                }
            }
        } else {
            panic!(
                "stmt.expr is not an HashLiteral expression. got={}",
                stmt.value
            );
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_function_literal_with_name() {
    let input = "let myfunc = fn() { }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Let(stmt) = stmt {
        if let Expression::Function(expr) = &stmt.value {
            assert_eq!(
                expr.params.len(),
                0,
                "FunctionLiteral params wrong. want 0, got={}",
                expr.params.len()
            );

            assert_eq!(
                expr.body.statements.len(),
                0,
                "function.body.statements has non-zero statements. got={}",
                expr.body.statements.len()
            );
            // assert_eq!(expr.name, "myfunc", "Wrong function name");
        } else {
            panic!(
                "stmt.expr is not a FunctionLiteral expression. got={}",
                stmt.value
            );
        }
        assert_eq!(
            stmt.name.token.literal, "myfunc",
            "Wrong identifier name in let statement"
        );
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_parsing_assignment_expressions() {
    struct AssignTest {
        input: &'static str,
        left: Literal,
        right: Literal,
    }
    let infix_tests = vec![
        AssignTest {
            input: "a = 10;",
            left: Literal::Ident("a"),
            right: Literal::Integer(10),
        },
        AssignTest {
            input: "a = 10.2;",
            left: Literal::Ident("a"),
            right: Literal::Float(10.2),
        },
        AssignTest {
            input: "a = true;",
            left: Literal::Ident("a"),
            right: Literal::Bool(true),
        },
        AssignTest {
            input: r#"a = "hello";"#,
            left: Literal::Ident("a"),
            right: Literal::Str("hello"),
        },
    ];

    for test in infix_tests {
        let program = parse_test_program(test.input, 1);
        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            test_assign_expression(&stmt.value, test.left, test.right);
        } else {
            panic!(
                "program.statements[0] is not an expression statement. got={}",
                stmt
            );
        }
    }
}

#[test]
fn test_parsing_assignment_expressions_negative() {
    let mut count = 0;
    let error_str = "[line 1] Invalid assignment target";
    let tests = vec![
        "1 = 1",
        "1.1 = 2.2",
        r#""a" = 1"#,
        "true = 1",
        "-a = 1",
        "a + b = 1",
        "a * b = 1",
        "(a) = 1",
    ];

    for (i, &test_input) in tests.iter().enumerate() {
        let errors = parse_test_program_failures(test_input);
        if errors.len() == 0 {
            eprintln!("[{}]: Expected error. Got none", i);
            count += 1;
        } else {
            assert_eq!(errors[0], error_str, "[{}]: Unexpected Error", i);
        }
    }
    assert_eq!(0, count, "tests failed");
}
