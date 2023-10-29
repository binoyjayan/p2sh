use std::fmt;

#[derive(Eq, PartialEq, Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub literal: String,
    pub line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, literal: &str, line: usize) -> Self {
        Self {
            ttype,
            literal: literal.to_string(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self::new(self.ttype, &self.literal, self.line)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new(TokenType::Illegal, "", 0)
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    Illegal = 0,
    Eof,
    // Identifiers + literals
    Identifier,
    Integer,
    Float,
    Str,
    // Operators
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Modulo,
    Bang,
    LogicalAnd,
    LogicalOr,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    BangEqual,
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    LeftShift,
    RightShift,
    // Delimiters
    Comma,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    NumberOfTokens,
}

impl From<TokenType> for &'static str {
    fn from(ttype: TokenType) -> &'static str {
        match ttype {
            TokenType::Illegal => "ILLEGAL",
            TokenType::Eof => "EOF",
            TokenType::Identifier => "IDENT",
            TokenType::Integer => "INT",
            TokenType::Float => "FLOAT",
            TokenType::Str => "STRING",
            TokenType::Assign => "=",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Asterisk => "*",
            TokenType::Slash => "/",
            TokenType::Modulo => "%",
            TokenType::Bang => "!",
            TokenType::LogicalAnd => "&&",
            TokenType::LogicalOr => "||",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Equal => "==",
            TokenType::BangEqual => "!=",
            TokenType::BitwiseAnd => "&",
            TokenType::BitwiseOr => "!",
            TokenType::BitwiseXor => "^",
            TokenType::BitwiseNot => "~",
            TokenType::LeftShift => "<<",
            TokenType::RightShift => "<<",
            TokenType::Comma => ",",
            TokenType::Colon => ":",
            TokenType::Semicolon => ";",
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::LeftBracket => "[",
            TokenType::RightBracket => "]",
            TokenType::Function => "FUNCTION",
            TokenType::Let => "LET",
            TokenType::True => "TRUE",
            TokenType::False => "FALSE",
            TokenType::If => "IF",
            TokenType::Else => "ELSE",
            TokenType::Return => "RETURN",
            TokenType::NumberOfTokens => "",
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s: &'static str = (*self).into();
        write!(f, "{}", s)
    }
}
