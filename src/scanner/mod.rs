pub mod tests;
pub mod token;

use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::scanner::token::*;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("_".into(), TokenType::Underscore);
        m.insert("let".into(), TokenType::Let);
        m.insert("fn".into(), TokenType::Function);
        m.insert("true".into(), TokenType::True);
        m.insert("false".into(), TokenType::False);
        m.insert("if".into(), TokenType::If);
        m.insert("else".into(), TokenType::Else);
        m.insert("return".into(), TokenType::Return);
        m.insert("null".into(), TokenType::Null);
        m.insert("loop".into(), TokenType::Loop);
        m.insert("while".into(), TokenType::While);
        m.insert("break".into(), TokenType::Break);
        m.insert("continue".into(), TokenType::Continue);
        m.insert("match".into(), TokenType::Match);
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
    fn peek_char(&mut self) -> char {
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
        self.skip_comments();

        let token = match self.ch {
            '\0' => self.make_token(TokenType::Eof, ""),
            ';' => self.make_token_ch(TokenType::Semicolon),
            ',' => self.make_token_ch(TokenType::Comma),
            ':' => self.make_token_ch(TokenType::Colon),
            '(' => self.make_token_ch(TokenType::LeftParen),
            ')' => self.make_token_ch(TokenType::RightParen),
            '{' => self.make_token_ch(TokenType::LeftBrace),
            '}' => self.make_token_ch(TokenType::RightBrace),
            '[' => self.make_token_ch(TokenType::LeftBracket),
            ']' => self.make_token_ch(TokenType::RightBracket),
            '+' => self.make_token_ch(TokenType::Plus),
            '-' => self.make_token_ch(TokenType::Minus),
            '*' => self.make_token_ch(TokenType::Asterisk),
            '/' => self.make_token_ch(TokenType::Slash),
            '%' => self.make_token_ch(TokenType::Modulo),
            '^' => self.make_token_ch(TokenType::BitwiseXor),
            '~' => self.make_token_ch(TokenType::BitwiseNot),
            '!' => self.make_token_twin(TokenType::Bang, &[('=', TokenType::BangEqual)]),
            '&' => self.make_token_twin(TokenType::BitwiseAnd, &[('&', TokenType::LogicalAnd)]),
            '|' => self.make_token_twin(TokenType::BitwiseOr, &[('|', TokenType::LogicalOr)]),
            '=' => self.make_token_twin(
                TokenType::Assign,
                &[('=', TokenType::Equal), ('>', TokenType::MatchArm)],
            ),
            '<' => self.make_token_twin(
                TokenType::Less,
                &[('=', TokenType::LessEqual), ('<', TokenType::LeftShift)],
            ),
            '>' => self.make_token_twin(
                TokenType::Greater,
                &[('=', TokenType::GreaterEqual), ('>', TokenType::RightShift)],
            ),
            '"' => self.read_string(),
            '\'' => self.read_char_token(),
            _ => {
                if Self::is_identifier_first(self.ch) {
                    return self.read_identifier();
                } else if self.ch == '.' && self.peek_char() == '.' {
                    return self.read_dot();
                } else if self.ch == '.' || self.ch.is_ascii_digit() {
                    return self.read_number();
                }
                self.make_token(TokenType::Illegal, &self.ch.to_string())
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
    // If the next character in the input matches the characters in 'next'
    // then make a token with the two characters (single, next[n].0), otherwise
    // make a token of type 'single' with the first character.
    fn make_token_twin(&mut self, single: TokenType, next: &[(char, TokenType)]) -> Token {
        let curr = self.ch;
        if self.peek_char() == next[0].0 {
            self.read_char();
            self.make_token(next[0].1, &format!("{}{}", curr, next[0].0))
        } else if next.len() > 1 && self.peek_char() == next[1].0 {
            self.read_char();
            self.make_token(next[1].1, &format!("{}{}", curr, next[1].0))
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
        // Check for a byte literal
        if self.ch == '\'' && identifier == "b" {
            self.read_char();
            let the_byte = self.input[self.position];
            // Consume ending quote (')
            self.read_char();
            if self.ch == '\'' {
                // advance
                self.read_char();
                // if the character is ascii, return a byte token
                if the_byte.is_ascii() {
                    return self.make_token(TokenType::Byte, &the_byte.to_string());
                }
            }
            // If no immediate ending quote (') was found, return an illegal token
            while self.ch != '\'' && self.ch != '\0' {
                self.read_char();
            }
            if self.ch == '\'' {
                self.read_char();
            }
            let tok: String = self.input[position..self.position].iter().collect();
            return self.make_token(TokenType::Illegal, &tok);
        }

        // Proceed to process identifier
        let ttype = Self::lookup_identifier(identifier.clone());
        self.make_token(ttype, &identifier)
    }

    fn read_number(&mut self) -> Token {
        let mut is_float = false;
        let position = self.position;

        // digits before the period(.)
        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        // Check for a decimal point but not a range operator (..)
        if self.ch == '.' && self.peek_char() != '.' {
            is_float = true;
            self.read_char(); // Consume the '.'
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
        }
        // Check for an exponent (scientific notation)
        if self.ch == 'e' || self.ch == 'E' {
            is_float = true;
            self.read_char(); // Consume 'e' or 'E'

            // 'e' without an exponent is illegal
            if self.ch != '-' && self.ch != '+' && !self.ch.is_ascii_digit() {
                let number: String = self.input[position..self.position].iter().collect();
                return self.make_token(TokenType::Illegal, &number);
            }

            if self.ch == '-' || self.ch == '+' {
                self.read_char(); // Consume '-' or '+'
            }
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
        }

        let number: String = self.input[position..self.position].iter().collect();
        let token_type = if is_float {
            TokenType::Float
        } else {
            TokenType::Integer
        };

        self.make_token(token_type, &number)
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
        if self.ch == '"' {
            self.make_token(TokenType::Str, &the_str)
        } else {
            // unterminated string
            self.make_token(TokenType::Illegal, &the_str)
        }
    }

    fn read_char_token(&mut self) -> Token {
        let position = self.position;
        // move past the opening quote (') character
        self.read_char();
        let the_char = self.input[self.position].to_string();
        self.read_char();
        if self.ch == '\'' {
            return self.make_token(TokenType::Char, &the_char);
        }
        // If no immediate ending quote (') was found, return an illegal token
        while self.ch != '\'' && self.ch != '\0' {
            self.read_char();
        }
        if self.ch == '\'' {
            self.read_char();
        }
        let tok: String = self.input[position..self.position].iter().collect();
        return self.make_token(TokenType::Illegal, &tok);
    }

    // If current character is a dot, read next to see if it is
    // a exclusive (..) or an inclusive (..=) range operator
    fn read_dot(&mut self) -> Token {
        self.read_char();
        self.read_char();
        if self.ch == '=' {
            self.read_char();
            self.make_token(TokenType::RangeInc, "..=")
        } else {
            self.make_token(TokenType::RangeEx, "..")
        }
    }

    // Identifiers can start with a letter or underscore
    fn is_identifier_first(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    // Identifiers can contain letters, numbers or underscores
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

    // skip single line comments
    fn skip_comments(&mut self) {
        loop {
            if self.ch == '#' || self.ch == '/' && self.peek_char() == '/' {
                loop {
                    self.read_char();
                    if self.ch == '\n' || self.ch == '\0' {
                        break;
                    }
                }
                self.skip_whitespace();
            } else {
                break;
            }
        }
    }
}
