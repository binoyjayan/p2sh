#![allow(unused_imports)]
use super::*;

#[test]
fn test_next_token() {
    // expected-type, expected-literal
    struct ExpectedToken<'a>(TokenType, &'a str);
    let input = r#"
            let none = null;
            let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            }
            let result = add(five, ten);

            !-/*5%2;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
            "foobar"
            "foo bar"
            [1, 2];
            {"foo": "bar"}
            5. + .1;
            10.0 - 30e1;
            40e+1 * 50e-1;
            a == b && c != d || e <= f;
            a & b | c ^ d;
            ~a | b << 1 >> 2;
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
        // let add = fn(x, y) { x + y; }
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "add"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Function, "fn"),
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
        // !-/*5%2;
        ExpectedToken(TokenType::Bang, "!"),
        ExpectedToken(TokenType::Minus, "-"),
        ExpectedToken(TokenType::Slash, "/"),
        ExpectedToken(TokenType::Asterisk, "*"),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Modulo, "%"),
        ExpectedToken(TokenType::Integer, "2"),
        ExpectedToken(TokenType::Semicolon, ";"),
        // 5 < 10 > 5;
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Less, "<"),
        ExpectedToken(TokenType::Integer, "10"),
        ExpectedToken(TokenType::Greater, ">"),
        ExpectedToken(TokenType::Integer, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
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
        // {"foo": "bar"}
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Str, "foo"),
        ExpectedToken(TokenType::Colon, ":"),
        ExpectedToken(TokenType::Str, "bar"),
        ExpectedToken(TokenType::RightBrace, "}"),
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

    let mut scanner = Scanner::new(input);

    for (i, tt) in tests.iter().enumerate() {
        let token = scanner.next_token();
        if token.ttype != tt.0 {
            panic!(
                "tests[{}] - tokentype wrong. expected='{}', got='{}'",
                i, tt.0, token.ttype
            );
        }
        if token.literal != tt.1 {
            panic!(
                "tests[{}] - literal wrong. expected='{}', got='{}'",
                i, tt.1, token.literal
            );
        }
    }
}
