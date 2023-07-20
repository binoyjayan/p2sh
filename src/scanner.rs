use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::token::*;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("let".into(), TokenType::Let);
        m.insert("fn".into(), TokenType::Function);
        m.insert("true".into(), TokenType::True);
        m.insert("false".into(), TokenType::False);
        m.insert("if".into(), TokenType::If);
        m.insert("else".into(), TokenType::Else);
        m.insert("return".into(), TokenType::Return);
        m
    };
}

#[derive(Default)]
pub struct Scanner {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        let mut scanner = Self {
            input: source.chars().collect::<Vec<char>>(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
        };
        scanner.read_char();
        scanner
    }

    /// Read the next character and advance the position in the input
    /// position points to the position where a character was last read from.
    /// read_position always points to the next position.
    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    // peek_char() does a lookahead in the input for the next character
    pub fn peek_char(&mut self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.ch {
            '\0' => self.make_token(TokenType::Eof, ""),
            ';' => self.make_token_ch(TokenType::Semicolon),
            ',' => self.make_token_ch(TokenType::Comma),
            '(' => self.make_token_ch(TokenType::LeftParen),
            ')' => self.make_token_ch(TokenType::RightParen),
            '{' => self.make_token_ch(TokenType::LeftBrace),
            '}' => self.make_token_ch(TokenType::RightBrace),
            '+' => self.make_token_ch(TokenType::Plus),
            '-' => self.make_token_ch(TokenType::Minus),
            '*' => self.make_token_ch(TokenType::Asterisk),
            '/' => self.make_token_ch(TokenType::Slash),
            '=' => self.make_token_twin('=', TokenType::Assign, TokenType::Equal),
            '!' => self.make_token_twin('=', TokenType::Bang, TokenType::BangEqual),
            '<' => self.make_token_twin('=', TokenType::Less, TokenType::LessEqual),
            '>' => self.make_token_twin('=', TokenType::Greater, TokenType::GreaterEqual),
            '"' => self.read_string(),
            _ => {
                if Self::is_identifier_first(self.ch) {
                    return self.read_identifier();
                } else if self.ch.is_ascii_digit() {
                    return self.read_number();
                }
                self.make_token(TokenType::Illegal, "")
            }
        };
        self.read_char();
        token
    }

    fn make_token(&self, ttype: TokenType, literal: &str) -> Token {
        Token::new(ttype, literal, self.line)
    }

    // Handle single character tokens
    fn make_token_ch(&self, ttype: TokenType) -> Token {
        self.make_token(ttype, &self.ch.to_string())
    }

    // Handle two character tokens by looking ahead one more character.
    // If the next character in the input matches 'next', then it is a token
    // of type 'twin', otherwise it is a token of type 'single'
    fn make_token_twin(&mut self, next: char, single: TokenType, twin: TokenType) -> Token {
        let curr = self.ch;
        if self.peek_char() == next {
            self.read_char();
            self.make_token(twin, &format!("{}{}", curr, next))
        } else {
            self.make_token_ch(single)
        }
    }

    fn read_identifier(&mut self) -> Token {
        let position = self.position;
        while Self::is_identifier_remaining(self.ch) {
            self.read_char();
        }
        let identifier: String = self.input[position..self.position].iter().collect();
        let ttype = Self::lookup_identifier(identifier.clone());
        self.make_token(ttype, &identifier)
    }

    fn read_number(&mut self) -> Token {
        let position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        let number: String = self.input[position..self.position].iter().collect();
        self.make_token(TokenType::Number, &number)
    }

    fn read_string(&mut self) -> Token {
        // move past the opening quotes (") character
        let position = self.position + 1;
        loop {
            self.read_char();

            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }
        let the_str: String = self.input[position..self.position].iter().collect();
        self.make_token(TokenType::Str, &the_str)
    }

    fn is_identifier_first(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn is_identifier_remaining(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn lookup_identifier(identifier: String) -> TokenType {
        match KEYWORDS.get(&identifier) {
            Some(kw_ttype) => *kw_ttype,
            None => TokenType::Identifier,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.ch {
                ' ' | '\t' => {
                    self.read_char();
                }
                '\n' | '\r' => {
                    self.line += 1;
                    self.read_char();
                }
                _ => {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
