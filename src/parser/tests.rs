#![allow(unused_imports)]
use super::*;
use crate::code::prop::PacketPropType;
use std::collections::HashMap;

#[cfg(test)]
#[derive(Clone)]
enum Literal {
    Ident(&'static str),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(&'static str),
    Char(char),
    Byte(u8),
    Range(i64, i64),
    Prop(PacketPropType),
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
fn test_char_literal(expr: &Expression, expected: char) {
    if let Expression::Char(ch) = expr {
        if ch.value != expected {
            panic!("char.value not '{}'. got='{}'", expected, ch.value);
        }
    } else {
        panic!("expr not a char. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_byte_literal(expr: &Expression, expected: u8) {
    if let Expression::Byte(b) = expr {
        if b.value != expected {
            panic!("byte.value not '{}'. got='{}'", expected, b.value);
        }
    } else {
        panic!("expr not a byte. got={:?}", expr);
    }
}

#[cfg(test)]
fn test_prop_literal(expr: &Expression, expected: PacketPropType) {
    if let Expression::Prop(p) = expr {
        if p.value != expected {
            panic!("prop.value not '{}'. got='{}'", expected, p.value);
        }
    } else {
        panic!("expr not a property. got={:?}", expr);
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
        Literal::Char(value) => {
            test_char_literal(expression, value);
        }
        Literal::Byte(value) => {
            test_byte_literal(expression, value);
        }
        Literal::Range(begin, end) => {
            test_integer_literal(expression, begin);
            test_integer_literal(expression, end);
        }
        Literal::Prop(value) => {
            test_prop_literal(expression, value);
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

// Generic dot expression test helper
#[cfg(test)]
fn test_dot_expression(expression: &Expression, left: Literal, right: Literal) {
    if let Expression::Dot(expr) = expression {
        test_literal(&*expr.left, left);
        test_literal(&*&expr.property, right);
    } else {
        panic!("expr not an dot expression. got={:?}", expression);
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
        TestLet {
            input: "let s = \"hello\";",
            expected_id: "s",
            expected_val: Literal::Str("hello"),
        },
        TestLet {
            input: "let ch = 'c';",
            expected_id: "ch",
            expected_val: Literal::Char('c'),
        },
        TestLet {
            input: "let byte = b'c';",
            expected_id: "byte",
            expected_val: Literal::Byte('c' as u8),
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
        return;
        return 5;
        return 10;
        return 993322;
        ";
    let program = parse_test_program(input, 4);

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
        context: ParseContext {
            access: AccessType::Get,
        },
    };

    let token_myvar2 = Token::new(TokenType::Identifier, "myvar2", 2);
    let ident_myvar2 = Identifier {
        token: token_myvar2,
        value: "myvar2".to_string(),
        context: ParseContext {
            access: AccessType::Get,
        },
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
fn test_parsing_constant_expressions() {
    struct ConstantTest {
        input: &'static str,
        number: Literal,
    }
    let prefix_tests = vec![
        ConstantTest {
            input: "0b1010",
            number: Literal::Integer(10),
        },
        ConstantTest {
            input: "0o10",
            number: Literal::Integer(8),
        },
        ConstantTest {
            input: "10",
            number: Literal::Integer(10),
        },
        ConstantTest {
            input: "0xFF",
            number: Literal::Integer(255),
        },
        ConstantTest {
            input: "5.0",
            number: Literal::Float(5.),
        },
        ConstantTest {
            input: "5e1",
            number: Literal::Float(50.),
        },
    ];

    for test in prefix_tests {
        let program = parse_test_program(test.input, 1);

        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            test_literal(&stmt.value, test.number);
        } else {
            panic!(
                "program.statements[0] is not an expression statement. got={}",
                stmt
            );
        }
    }
}

#[test]
fn test_parsing_constant_expressions_negative() {
    struct ConstantTest {
        input: &'static str,
        errors: Vec<&'static str>,
    }
    let tests = vec![
        ConstantTest {
            input: "0b102",
            errors: vec!["[line 1] could not parse '0b102' as a binary integer"],
        },
        ConstantTest {
            input: "0o108",
            errors: vec!["[line 1] could not parse '0o108' as an octal integer"],
        },
        ConstantTest {
            input: "11FF",
            errors: vec!["[line 1] could not parse '11FF' as an integer"],
        },
        ConstantTest {
            input: "0xFAN",
            errors: vec!["[line 1] could not parse '0xFAN' as a hexadecimal integer"],
        },
    ];

    for (i, test) in tests.iter().enumerate() {
        let errors = parse_test_program_failures(test.input);
        assert_eq!(
            errors.len(),
            test.errors.len(),
            "[{}] Error count mismatch",
            i
        );
        for (j, error) in errors.iter().enumerate() {
            assert_eq!(error, test.errors[j], "[{}][{}] Error mismatch", i, j);
        }
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
        PrefixTest {
            input: "!'a'",
            operator: "!",
            number: Literal::Char('a'),
        },
        PrefixTest {
            input: "!b'a'",
            operator: "!",
            number: Literal::Byte(b'a'),
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
fn test_parsing_bitwise_infix_expressions() {
    struct InfixTest {
        input: &'static str,
        operator: &'static str,
        left: Literal,
        right: Literal,
    }

    let infix_tests = vec![
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
            input: "3 >= 5 == false",
            expected: "((3 >= 5) == false)",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "3 <= 5 == true",
            expected: "((3 <= 5) == true)",
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
        PrecedenceTest {
            input: "match x { 1..2 | 5..=6 => {} }",
            expected: "match x { (1..2) | (5..=6) | => {  } _ | => { null; }}",
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
            match expr.else_if {
                ElseIfExpr::Empty => {} // Do nothing
                _ => panic!(
                    "expr.else_if was not of type ElseIfExpr::Empty. got={:?}",
                    expr.else_if
                ),
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
            if let ElseIfExpr::Else(else_stmt) = &expr.else_if {
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
fn test_if_then_else_if_expression() {
    let input = "if a == 1 { 10 } else if a == 2 { 20 } else { 99 }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::If(expr) = &stmt.value {
            test_infix_expression(
                &expr.condition,
                Literal::Ident("a"),
                "==",
                Literal::Integer(1),
            );
            let num_stmts = expr.then_stmt.statements.len();
            assert_eq!(num_stmts, 1, "then_stmt count not 1. got={}", num_stmts);
            if let Statement::Expr(expr) = &expr.then_stmt.statements[0] {
                test_integer_literal(&expr.value, 10);
            } else {
                panic!(
                    "then_stmt.statements[0] is not an expression statement. got={}",
                    expr.then_stmt.statements[0]
                );
            }
            // else if branch
            if let ElseIfExpr::ElseIf(else_if) = &expr.else_if {
                if let Expression::If(expr) = &**else_if {
                    test_infix_expression(
                        &expr.condition,
                        Literal::Ident("a"),
                        "==",
                        Literal::Integer(2),
                    );
                    let num_stmts = expr.then_stmt.statements.len();
                    assert_eq!(num_stmts, 1, "then_stmt count not 1. got={}", num_stmts);
                    if let Statement::Expr(expr) = &expr.then_stmt.statements[0] {
                        test_integer_literal(&expr.value, 20);
                    } else {
                        panic!(
                            "then_stmt.statements[0] is not an expression statement. got={}",
                            expr.then_stmt.statements[0]
                        );
                    }

                    // else branch
                    if let ElseIfExpr::Else(else_stmt) = &expr.else_if {
                        if let Statement::Expr(expr) = &else_stmt.statements[0] {
                            test_integer_literal(&expr.value, 99);
                        } else {
                            panic!(
                                "else_stmt.statements[0] is not an expression statement. got={}",
                                else_stmt.statements[0]
                            );
                        }
                    }
                } else {
                    panic!("else_if is not an If expression. got={}", else_if);
                }
            } else {
                panic!(
                    "expr.else_if is not an ElseIfExpr::ElseIf. got={:?}",
                    expr.else_if
                );
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
    let input = r#"map { "one": 1, "two": 2, "three": 3}"#;
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
    let input = "map {}";
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
    let input = r#"map { "one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
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
        assert_eq!(stmt.name.value, "myfunc", "Wrong function name");
        assert_eq!(
            stmt.name.token.literal, "myfunc",
            "Wrong identifier name in let statement"
        );
        if let Expression::Function(expr) = &stmt.value {
            assert_eq!(expr.token.literal, "fn", "Wrong function keyword");
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
            assert_eq!(expr.name, "myfunc", "Wrong function name");
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
fn test_function_literal_with_no_name() {
    let input = "fn() { }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        assert_eq!(stmt.token.literal, "fn", "Wrong function keyword");
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
            assert_eq!(expr.name, "", "Non empty function name");
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
fn test_function_statement() {
    let input = "fn myfunc() { }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Function(stmt) = stmt {
        assert_eq!(stmt.token.literal, "fn", "Wrong function keyword");
        assert_eq!(stmt.name, "myfunc", "Wrong function name");
        assert_eq!(
            stmt.params.len(),
            0,
            "FunctionLiteral params wrong. want 0, got={}",
            stmt.params.len()
        );

        assert_eq!(
            stmt.body.statements.len(),
            0,
            "function.body.statements has non-zero statements. got={}",
            stmt.body.statements.len()
        );
    } else {
        panic!(
            "program.statements[0] is not an function statement. got={}",
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

#[test]
fn test_block_statement_empty() {
    let input = "{ }";
    let program = parse_test_program(input, 1);
    let stmt = &program.statements[0];
    if let Statement::Block(stmt) = stmt {
        let len = stmt.statements.len();
        assert_eq!(len, 0, "found number of statements. got={}", len);
    } else {
        panic!(
            "program.statements[0] is not a block statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_block_statements() {
    let input = "{ 1; 2 }";
    let program = parse_test_program(input, 1);
    let stmt = &program.statements[0];
    if let Statement::Block(stmt) = stmt {
        let len = stmt.statements.len();
        assert_eq!(len, 2, "found number of statements. got={}", len);
        if let Statement::Expr(expr) = &stmt.statements[0] {
            test_literal(&expr.value, Literal::Integer(1));
        } else {
            panic!(
                "stmt.statements[0] is not an expression statement. got={}",
                stmt.statements[0]
            );
        }
        if let Statement::Expr(expr) = &stmt.statements[1] {
            test_literal(&expr.value, Literal::Integer(2));
        } else {
            panic!(
                "stmt.statements[1] is not an expression statement. got={}",
                stmt.statements[0]
            );
        }
    } else {
        panic!(
            "program.statements[0] is not a block statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_loop_with_break_statement() {
    let input = "loop { break; }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Loop(stmt) = stmt {
        let len = stmt.body.statements.len();
        assert_eq!(
            len, 1,
            "loop body has wrong number of statements. got={}",
            len
        );
        // The loop does not have a label
        if let Some(label) = stmt.label.clone() {
            panic!("unexpected loop label: got={}", label);
        }
        if let Statement::Break(stmt) = &stmt.body.statements[0] {
            assert_eq!(stmt.token.literal, "break");
        } else {
            panic!(
                "loop body statement is not a break statement. got={}",
                stmt.body.statements[0]
            );
        }
    } else {
        panic!(
            "program.statements[0] is not a loop statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_loop_with_label_statement() {
    let input = "empty: loop { }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Loop(stmt) = stmt {
        let len = stmt.body.statements.len();
        assert_eq!(
            len, 0,
            "loop body has wrong number of statements. got={}",
            len
        );
        if let Some(label) = stmt.label.clone() {
            assert_eq!(label.literal, "empty", "wrong loop label. got={}", len);
        } else {
            panic!("loop statement has no label");
        }
    } else {
        panic!(
            "program.statements[0] is not a loop statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_while_with_break_statement() {
    let input = "while true { break; }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::While(stmt) = stmt {
        let len = stmt.body.statements.len();
        assert_eq!(
            len, 1,
            "loop body has wrong number of statements. got={}",
            len
        );
        // assert condition
        test_boolean_literal(&stmt.condition, true);
        // The loop does not have a label
        if let Some(label) = stmt.label.clone() {
            panic!("unexpected loop label: got={}", label);
        }
        if let Statement::Break(stmt) = &stmt.body.statements[0] {
            assert_eq!(stmt.token.literal, "break");
        } else {
            panic!(
                "loop body statement is not a break statement. got={}",
                stmt.body.statements[0]
            );
        }
    } else {
        panic!(
            "program.statements[0] is not a loop statement. got={}",
            stmt
        );
    }
}

#[test]
fn test_while_with_label_statement() {
    let input = "empty: while false { }";
    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::While(stmt) = stmt {
        let len = stmt.body.statements.len();
        assert_eq!(
            len, 0,
            "loop body has wrong number of statements. got={}",
            len
        );
        // assert condition
        test_boolean_literal(&stmt.condition, false);
        if let Some(label) = stmt.label.clone() {
            assert_eq!(label.literal, "empty", "wrong loop label. got={}", len);
        } else {
            panic!("loop statement has no label");
        }
    } else {
        panic!(
            "program.statements[0] is not a loop statement. got={}",
            stmt
        );
    }
}

#[cfg(test)]
fn test_match_arm(arm: MatchArm, expected_pattern: Vec<Literal>, expected_body: Literal) {
    assert_eq!(
        arm.patterns.len(),
        expected_pattern.len(),
        "wrong number of patterns. got={}",
        arm.patterns.len()
    );
    for (i, pattern) in arm.patterns.iter().enumerate() {
        let exp = expected_pattern[i].clone();
        match (pattern, exp) {
            (MatchPattern::Integer(m), Literal::Integer(n)) => {
                assert_eq!(m.value, n, "[{}] wrong integer literal. got={}", i, m)
            }
            (MatchPattern::Str(s1), Literal::Str(s2)) => {
                assert_eq!(s1.value, s2, "[{}] wrong integer string. got={}", i, s1)
            }
            (MatchPattern::Default(d), Literal::Str(s)) => assert_eq!(d.value, s),
            (MatchPattern::Range(r), Literal::Range(b, e)) => {
                test_literal(&r.begin, Literal::Integer(b));
                test_literal(&r.end, Literal::Integer(e));
            }
            _ => panic!("[{}] unexpected pattern", i),
        }
    }
    if let Statement::Expr(expr) = &arm.body.statements[0] {
        test_literal(&expr.value, expected_body);
    } else {
        panic!(
            "match arm body is not an expression statement. got={}",
            arm.body,
        );
    }
}

// Test match expressions
#[test]
fn test_match_expression() {
    let input = r#"
        match x {
            1 => "one"
            1 | 2 => { "one-or-two" },
            1..10 | 10..=20 => { "range" },
            _ => "default"
        }
    "#;
    let expected: Vec<(Vec<Literal>, Literal)> = vec![
        (vec![Literal::Integer(1)], Literal::Str("one")),
        (
            vec![Literal::Integer(1), Literal::Integer(2)],
            Literal::Str("one-or-two"),
        ),
        (
            vec![Literal::Range(1, 10), Literal::Range(10, 20)],
            Literal::Str("range"),
        ),
        (vec![Literal::Str("_")], Literal::Str("default")),
    ];

    let program = parse_test_program(input, 1);

    let stmt = &program.statements[0];
    if let Statement::Expr(stmt) = stmt {
        if let Expression::Match(expr) = &stmt.value {
            test_identifier(&expr.expr, "x");
            assert_eq!(
                expr.arms.len(),
                4,
                "match expression has wrong number of arms. got={}",
                expr.arms.len()
            );

            // Test match arms
            for (i, arm) in expr.arms.iter().enumerate() {
                test_match_arm(arm.clone(), expected[i].0.clone(), expected[i].1.clone());
            }
        } else {
            panic!("stmt.expr is not a Match expression. got={}", stmt.value);
        }
    } else {
        panic!(
            "program.statements[0] is not an expression statement. got={}",
            stmt
        )
    }
}

#[test]
fn test_match_expressions_negative() {
    struct MatchTest {
        input: &'static str,
        errors: Vec<&'static str>,
    }
    let tests = vec![
        MatchTest {
            input: r#"
                match x {
                    a => { "id" }
                }
            "#,
            errors: vec!["[line 3] invalid pattern in match arm 'a'"],
        },
        MatchTest {
            input: r#"
                match x {
                    1 || 2 => { "logical" }
                }
            "#,
            errors: vec!["[line 3] invalid operation in match pattern '||'"],
        },
        MatchTest {
            input: r#"
                match x {
                    _ => { "default1" }
                    _ => { "default2" }
                }
            "#,
            errors: vec!["[line 4] multiple default arms in match expression"],
        },
        MatchTest {
            input: r#"
                match x {
                    _ | _ => { "default" }
                }
            "#,
            errors: vec!["[line 3] multiple default patterns in match arm"],
        },
        MatchTest {
            input: r#"
                match x {
                    1 | _ => { "default" }
                }
            "#,
            errors: vec!["[line 3] default pattern cannot be used with other patterns"],
        },
        MatchTest {
            input: r#"
                match x {
                    _ => { "default" }
                    1 => { "one" }
                }
            "#,
            errors: vec!["[line 4] unreachable pattern"],
        },
        MatchTest {
            // This error is reported in the compiler
            input: r#"
                match x {
                    a..b => { "id-range" }
                }
            "#,
            errors: vec![],
        },
    ];

    for (i, test) in tests.iter().enumerate() {
        let errors = parse_test_program_failures(test.input);
        assert_eq!(
            errors.len(),
            test.errors.len(),
            "[{}] Error count mismatch",
            i
        );
        for (j, error) in errors.iter().enumerate() {
            assert_eq!(error, test.errors[j], "[{}][{}] Error mismatch", i, j);
        }
    }
}

#[test]
fn test_parsing_dot_expressions() {
    struct DotExprTest {
        input: &'static str,
        left: Literal,
        prop: Literal,
    }
    let infix_tests = vec![
        DotExprTest {
            input: "pcap.magic",
            left: Literal::Ident("pcap"),
            prop: Literal::Prop(PacketPropType::Magic),
        },
        DotExprTest {
            input: "eth.src",
            left: Literal::Ident("eth"),
            prop: Literal::Prop(PacketPropType::Src),
        },
        DotExprTest {
            input: "eth.dst",
            left: Literal::Ident("eth"),
            prop: Literal::Prop(PacketPropType::Dst),
        },
        DotExprTest {
            input: "eth.type",
            left: Literal::Ident("eth"),
            prop: Literal::Prop(PacketPropType::EtherType),
        },
        DotExprTest {
            input: "vlan.id",
            left: Literal::Ident("vlan"),
            prop: Literal::Prop(PacketPropType::Id),
        },
        DotExprTest {
            input: "vlan.type",
            left: Literal::Ident("vlan"),
            prop: Literal::Prop(PacketPropType::EtherType),
        },
    ];

    for test in infix_tests {
        let program = parse_test_program(test.input, 1);
        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            test_dot_expression(&stmt.value, test.left, test.prop);
        } else {
            panic!(
                "program.statements[0] is not an expression statement. got={}",
                stmt
            );
        }
    }
}

#[test]
fn test_parsing_dot_expressions_negative() {
    struct DotExprTest {
        input: &'static str,
        errors: Vec<&'static str>,
    }
    let tests = vec![
        DotExprTest {
            input: "eth.unknown",
            errors: vec!["[line 1] invalid property 'unknown'"],
        },
        DotExprTest {
            input: "vlan.unknown",
            errors: vec!["[line 1] invalid property 'unknown'"],
        },
    ];

    for (i, test) in tests.iter().enumerate() {
        let errors = parse_test_program_failures(test.input);
        assert_eq!(
            errors.len(),
            test.errors.len(),
            "[{}] Error count mismatch",
            i
        );
        for (j, error) in errors.iter().enumerate() {
            assert_eq!(error, test.errors[j], "[{}][{}] Error mismatch", i, j);
        }
    }
}
