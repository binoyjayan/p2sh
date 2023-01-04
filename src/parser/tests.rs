use super::*;
use crate::scanner::*;

fn parse_test_program(input: &str) -> Program {
    let scanner = Scanner::new(input);
    let mut parser = Parser::new(scanner);
    let program = parser.parse_program();
    if program.statements.is_empty() {
        panic!("program.statements is empty");
    }
    check_parse_errors(&parser);
    program
}

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

#[test]
fn test_let_statements() {
    let input = "
        let x = 5;
        let y = 10;
        let foobar = 838383;
        ";
    let program = parse_test_program(input);

    let let_tests = ["x", "y", "foobar"];
    for (i, tt) in let_tests.iter().enumerate() {
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
    let program = parse_test_program(input);

    if program.statements.len() != 3 {
        panic!(
            "program.statements does not contain 3 statements. got={}",
            program.statements.len()
        );
    }

    for stmt in program.statements.iter() {
        if stmt.token_literal() != "return" {
            eprintln!(
                "stmt.token_literal() not 'return'. got={}",
                stmt.token_literal()
            );
            continue;
        }
        if let Statement::Return(_stmt) = stmt {
        } else {
            eprintln!("stmt is not a return statement. got={}", stmt);
        }
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
