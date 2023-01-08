use super::*;

#[cfg(test)]
fn parse_test_program(input: &str, num_stmts: usize) -> Program {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();

    if program.statements.len() != num_stmts {
        panic!(
            "program.statements does not contain {} statement(s). got={}",
            num_stmts,
            program.statements.len()
        );
    }

    check_parse_errors(&parser);
    program
}

#[cfg(test)]
fn check_parse_errors(parser: &Parser) {
    if parser.errors.is_empty() {
        return;
    }
    eprintln!("parser has {} errors", parser.parse_errors().len());
    for msg in parser.parse_errors() {
        eprintln!("parser error: {}", msg);
    }
    panic!();
}

#[cfg(test)]
fn test_numeric_literal(expr: &Expression, expected: f64) {
    if let Expression::Number(num) = expr {
        if num.value != expected {
            panic!("number.value not '{}'. got='{}'", expected, num.value);
        }
    } else {
        panic!("expr not an Number. got={:?}", expr);
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
fn test_identifier(expression: &Expression, value: String) {
    if let Expression::Ident(ident) = expression {
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
    } else {
        panic!("expr not an Identifier. got={:?}", expression);
    }
}

#[test]
fn test_let_statements() {
    let input = "
        let x = 5;
        let y = 10;
        let foobar = 838383;
        ";
    let identifiers = ["x", "y", "foobar"];

    let program = parse_test_program(input, 3);
    for (i, tt) in identifiers.iter().enumerate() {
        let stmt = &program.statements[i];
        test_let_statement(stmt, tt);
    }
}

fn test_let_statement(stmt: &Statement, name: &str) {
    if stmt.token_literal() != "let" {
        panic!("stmt.token_literal not 'let'. got={}", stmt.token_literal());
    }

    if let Statement::Let(s) = stmt {
        if s.name.value != name {
            panic!("let_stmt.name.value not '{}'. got={}", name, s.name.value);
        }
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
    };

    let token_myvar2 = Token::new(TokenType::Identifier, "myvar2", 2);
    let ident_myvar2 = Identifier {
        token: token_myvar2,
        value: "myvar2".to_string(),
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
        test_identifier(&stmt.value, "foobar".to_string());
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
        test_numeric_literal(&stmt.value, 5.);
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
        number: f64,
    }
    let prefix_tests = vec![
        PrefixTest {
            input: "!5",
            operator: "!",
            number: 5.,
        },
        PrefixTest {
            input: "-15",
            operator: "-",
            number: 15.,
        },
    ];

    for test in prefix_tests {
        let program = parse_test_program(test.input, 1);

        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            if let Expression::Unary(expr) = &stmt.value {
                if expr.operator != test.operator {
                    panic!(
                        "expr.operator is not '{}'. got='{}'",
                        expr.operator, test.operator
                    );
                }
                test_numeric_literal(&*expr.right, test.number);
            } else {
                panic!("expr not an Prefix expression. got={:?}", stmt);
            }
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
        left: f64,
        right: f64,
    }
    let infix_tests = vec![
        InfixTest {
            input: "5 + 5;",
            operator: "+",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 - 5;",
            operator: "-",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 * 5;",
            operator: "*",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 / 5;",
            operator: "/",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 > 5;",
            operator: ">",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 < 5;",
            operator: "<",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 == 5;",
            operator: "==",
            left: 5.,
            right: 5.,
        },
        InfixTest {
            input: "5 != 5;",
            operator: "!=",
            left: 5.,
            right: 5.,
        },
    ];

    for test in infix_tests {
        let program = parse_test_program(test.input, 1);

        let stmt = &program.statements[0];
        if let Statement::Expr(stmt) = stmt {
            if let Expression::Binary(expr) = &stmt.value {
                if expr.operator != test.operator {
                    panic!(
                        "expr.operator is not '{}'. got='{}'",
                        expr.operator, test.operator
                    );
                }
                test_numeric_literal(&*expr.left, test.left);
                test_numeric_literal(&*expr.right, test.right);
            } else {
                panic!("expr not an Infix expression. got={:?}", stmt);
            }
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
            expected: "((5 > 4) == (3 < 4))",
            num_stmts: 1,
        },
        PrecedenceTest {
            input: "5 < 4 != 3 > 4",
            expected: "((5 < 4) != (3 > 4))",
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
    ];

    for test in precedence_tests {
        let program = parse_test_program(test.input, test.num_stmts);
        let actual = format!("{}", program);
        assert_eq!(actual, test.expected);
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
