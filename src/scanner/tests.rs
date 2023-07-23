#![allow(unused_imports)]
use super::*;

#[test]
fn test_next_token() {
    // expected-type, expected-literal
    struct ExpectedToken<'a>(TokenType, &'a str);
    let input = "
            let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            }
            let result = add(five, ten);

            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
            \"foobar\"
            \"foo bar\"
            [1, 2];
            {\"foo\": \"bar\"}
        ";

    let tests = vec![
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "five"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Number, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Let, "let"),
        ExpectedToken(TokenType::Identifier, "ten"),
        ExpectedToken(TokenType::Assign, "="),
        ExpectedToken(TokenType::Number, "10"),
        ExpectedToken(TokenType::Semicolon, ";"),
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
        ExpectedToken(TokenType::Bang, "!"),
        ExpectedToken(TokenType::Minus, "-"),
        ExpectedToken(TokenType::Slash, "/"),
        ExpectedToken(TokenType::Asterisk, "*"),
        ExpectedToken(TokenType::Number, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Number, "5"),
        ExpectedToken(TokenType::Less, "<"),
        ExpectedToken(TokenType::Number, "10"),
        ExpectedToken(TokenType::Greater, ">"),
        ExpectedToken(TokenType::Number, "5"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::If, "if"),
        ExpectedToken(TokenType::LeftParen, "("),
        ExpectedToken(TokenType::Number, "5"),
        ExpectedToken(TokenType::Less, "<"),
        ExpectedToken(TokenType::Number, "10"),
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
        ExpectedToken(TokenType::Number, "10"),
        ExpectedToken(TokenType::Equal, "=="),
        ExpectedToken(TokenType::Number, "10"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Number, "10"),
        ExpectedToken(TokenType::BangEqual, "!="),
        ExpectedToken(TokenType::Number, "9"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::Str, "foobar"),
        ExpectedToken(TokenType::Str, "foo bar"),
        ExpectedToken(TokenType::LeftBracket, "["),
        ExpectedToken(TokenType::Number, "1"),
        ExpectedToken(TokenType::Comma, ","),
        ExpectedToken(TokenType::Number, "2"),
        ExpectedToken(TokenType::RightBracket, "]"),
        ExpectedToken(TokenType::Semicolon, ";"),
        ExpectedToken(TokenType::LeftBrace, "{"),
        ExpectedToken(TokenType::Str, "foo"),
        ExpectedToken(TokenType::Colon, ":"),
        ExpectedToken(TokenType::Str, "bar"),
        ExpectedToken(TokenType::RightBrace, "}"),
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
