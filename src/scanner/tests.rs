#![allow(unused_imports)]
use super::*;

#[cfg(test)]
struct ExpectedToken<'a>(TokenType, &'a str);

#[cfg(test)]
fn run_scanner_tests(input: &str, tests: Vec<ExpectedToken>) {
    let mut scanner = Scanner::new(input);
    for (n, tt) in tests.iter().enumerate() {
        println!("[{}] Scanner Test", n);
        let token = scanner.next_token();
        if token.ttype != tt.0 {
            panic!(
                "tests[{}] - tokentype wrong. expected='{}', got='{}[{}]'",
                n, tt.0, token.ttype, token.literal
            );
        }
        if token.literal != tt.1 {
            panic!(
                "tests[{}] - literal wrong. expected='{}', got='{}'",
                n, tt.1, token.literal
            );
        }
    }
}

#[test]
fn test_tokens_constants() {
    let input = r#"
        "foobar"
        "foo bar"
        [1, 2];
        map {"foo": "bar"}
    "#;
    let tests = vec![
        // "foobar"
        ExpectedToken(TokenType::Str, "foobar"),
        // "foo bar"
        ExpectedToken(TokenType::Str, "foo bar"),
        // [1, 2];
        ExpectedToken(TokenType::LeftBracket, "["),
        ExpectedToken(TokenType::Integer, "1"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::RightBracket, "]"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // map {"foo": "bar"}
        ExpectedToken(TokenType::Map, "map"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Str, "foo"),
        ExpectedToken(TokenType::Colon, ":"),
        ExpectedToken(TokenType::Str, "bar"),
        ExpectedToken(TokenType::RightBrace, "}"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_let() {
    let input = r#"
        let none = null;
        let five = 5;
        let ten = 10;
        let ch = 'a';
        let byte = b'a';
    "#;

    let tests = vec![
        // let none = null;
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "none"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Null, "null"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // let five = 5;
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "five"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // let ten = 10;
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "ten"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // let ch = 'a';
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "ch"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Char, "a"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // let byte = b'a';
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "byte"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Byte, "a"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];

    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_arithmetic() {
    let input = r#"
        !-/*5%2;
        5. + .1;
        10.0 - 30e1;
        40e+1 * 50e-1;
    "#;
    let tests = vec![
        // !-/*5%2;
        ExpectedToken(TokenType::Bang, "!"),
        ExpectedToken(TokenType::Minus, "-"),
        ExpectedToken(TokenType::Slash, "/"),
        ExpectedToken(TokenType::Asterisk, "*"),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Modulo, "%"),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 5. + .1;
        ExpectedToken(TokenType::Float, "5."),
        ExpectedToken(TokenType::Plus, "+"),
        ExpectedToken(TokenType::Float, ".1"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 10.0 - 30e1;
        ExpectedToken(TokenType::Float, "10.0"),
        ExpectedToken(TokenType::Minus, "-"),
        ExpectedToken(TokenType::Float, "30e1"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 40e+1 * 50e-1;
        ExpectedToken(TokenType::Float, "40e+1"),
        ExpectedToken(TokenType::Asterisk, "*"),
        ExpectedToken(TokenType::Float, "50e-1"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_relational() {
    let input = r#"
        10 == 10;
        10 != 9;
        5 < 10 > 5;
        5 <= 10 >= 5;
        a == b && c != d || e <= f;
    "#;
    let tests = vec![
        // 10 == 10;
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::Equal, "=="),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 10 != 9;
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::BangEqual, "!="),
        ExpectedToken(TokenType::Integer, "9"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 5 < 10 > 5;
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Less, "<"),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::Greater, ">"),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 5 <= 10 >= 5;
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::LessEqual, "<="),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::GreaterEqual, ">="),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // a == b && c != d || e <= f;
        ExpectedToken(TokenType::Identifier, "a"),
        ExpectedToken(TokenType::Equal, "=="),
        ExpectedToken(TokenType::Identifier, "b"),
        ExpectedToken(TokenType::LogicalAnd, "&&"),
        ExpectedToken(TokenType::Identifier, "c"),
        ExpectedToken(TokenType::BangEqual, "!="),
        ExpectedToken(TokenType::Identifier, "d"),
        ExpectedToken(TokenType::LogicalOr, "||"),
        ExpectedToken(TokenType::Identifier, "e"),
        ExpectedToken(TokenType::LessEqual, "<="),
        ExpectedToken(TokenType::Identifier, "f"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_bitwise() {
    let input = r#"
        a & b | c ^ d;
        ~a | b << 1 >> 2;
    "#;
    let tests = vec![
        // a & b | c ^ d;
        ExpectedToken(TokenType::Identifier, "a"),
        ExpectedToken(TokenType::BitwiseAnd, "&"),
        ExpectedToken(TokenType::Identifier, "b"),
        ExpectedToken(TokenType::BitwiseOr, "|"),
        ExpectedToken(TokenType::Identifier, "c"),
        ExpectedToken(TokenType::BitwiseXor, "^"),
        ExpectedToken(TokenType::Identifier, "d"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // ~a | b << 1 >> 2;
        ExpectedToken(TokenType::BitwiseNot, "~"),
        ExpectedToken(TokenType::Identifier, "a"),
        ExpectedToken(TokenType::BitwiseOr, "|"),
        ExpectedToken(TokenType::Identifier, "b"),
        ExpectedToken(TokenType::LeftShift, "<<"),
        ExpectedToken(TokenType::Integer, "1"),
        ExpectedToken(TokenType::RightShift, ">>"),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_ranges() {
    let input = r#"
        1..2 == 1 ..= 2;
    "#;
    let tests = vec![
        // 1..2 == 1..=2;
        ExpectedToken(TokenType::Integer, "1"),
        ExpectedToken(TokenType::RangeEx, ".."),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::Equal, "=="),
        ExpectedToken(TokenType::Integer, "1"),
        ExpectedToken(TokenType::RangeInc, "..="),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_conditionals() {
    let input = r#"
        if (5 < 10) {
            return true;
        } else {
            return false;
        }
        match x { 0 => 1, _ => 0, }
    "#;
    let tests = vec![
        // if (5 < 10) { return true; } else { return false; }
        ExpectedToken(TokenType::If, "if"),
        ExpectedToken(TokenType::LeftParen, "("),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Less, "<"),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::RightParen, ")"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Return, "return"),
        ExpectedToken(TokenType::True, "true"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::RightBrace, "}"),
        ExpectedToken(TokenType::Else, "else"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Return, "return"),
        ExpectedToken(TokenType::False, "false"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::RightBrace, "}"),
        // match x { 0 => 1, _ => 0, }
        ExpectedToken(TokenType::Match, "match"),
        ExpectedToken(TokenType::Identifier, "x"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Integer, "0"),
        ExpectedToken(TokenType::MatchArm, "=>"),
        ExpectedToken(TokenType::Integer, "1"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Underscore, "_"),
        ExpectedToken(TokenType::MatchArm, "=>"),
        ExpectedToken(TokenType::Integer, "0"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::RightBrace, "}"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_functions() {
    let input = r#"
        fn add(x, y) {
            x + y;
        }
        let result = add(five, ten);
    "#;
    let tests = vec![
        // fn add(x, y) { x + y; }
        ExpectedToken(TokenType::Function, "fn"),
        ExpectedToken(TokenType::Identifier, "add"),
        ExpectedToken(TokenType::LeftParen, "("),
        ExpectedToken(TokenType::Identifier, "x"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Identifier, "y"),
        ExpectedToken(TokenType::RightParen, ")"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Identifier, "x"),
        ExpectedToken(TokenType::Plus, "+"),
        ExpectedToken(TokenType::Identifier, "y"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::RightBrace, "}"),
        // let result = add(five, ten);
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "result"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Identifier, "add"),
        ExpectedToken(TokenType::LeftParen, "("),
        ExpectedToken(TokenType::Identifier, "five"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Identifier, "ten"),
        ExpectedToken(TokenType::RightParen, ")"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_loops() {
    let input = r#"
        loop { break; while { continue; } }
    "#;
    let tests = vec![
        // loop { break; while { continue; } }
        ExpectedToken(TokenType::Loop, "loop"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Break, "break"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::While, "while"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Continue, "continue"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::RightBrace, "}"),
        ExpectedToken(TokenType::RightBrace, "}"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_struct() {
    let input = r#"
        struct { x, y }
    "#;

    let tests = vec![
        // struct { x, y }
        ExpectedToken(TokenType::Struct, "struct"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Identifier, "x"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Identifier, "y"),
        ExpectedToken(TokenType::RightBrace, "}"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];

    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_std_files() {
    let input = r#"
        stdin;
        stdout;
        stderr;
    "#;
    let tests = vec![
        ExpectedToken(TokenType::Stdin, "stdin"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Stdout, "stdout"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Stderr, "stderr"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}

#[test]
fn test_tokens_ethernet_fields() {
    let input = r#"
        p.dest;
        p.src;
    "#;
    let tests = vec![
        ExpectedToken(TokenType::Identifier, "p"),
        ExpectedToken(TokenType::Dot, "."),
        ExpectedToken(TokenType::Identifier, "dest"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Identifier, "p"),
        ExpectedToken(TokenType::Dot, "."),
        ExpectedToken(TokenType::Identifier, "src"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // EOF
        ExpectedToken(TokenType::Eof, ""),
    ];
    run_scanner_tests(input, tests);
}
