use std::fmt;

use crate::token::*;

#[derive(Clone, Debug)]
pub enum Expression {
    Ident(Identifier),
    Number(NumberLiteral),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Bool(BooleanExpr),
    Nil,
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct NumberLiteral {
    pub token: Token,
    pub value: f64,
}

impl fmt::Display for NumberLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub token: Token, //operator token
    pub operator: String,
    pub right: Box<Expression>,
}

impl fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}{})", self.token, self.right)
    }
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub token: Token, //operator token
    pub operator: String,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.left, self.token, self.right)
    }
}

#[derive(Clone, Debug)]
pub struct BooleanExpr {
    pub token: Token,
    pub value: bool,
}

impl fmt::Display for BooleanExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token.literal)
    }
}

impl Expression {
    fn token_literal(&self) -> String {
        match &self {
            Expression::Ident(ident) => ident.token.literal.clone(),
            Expression::Number(num) => num.token.literal.clone(),
            Expression::Unary(unary) => unary.token.literal.clone(),
            Expression::Binary(binary) => binary.token.literal.clone(),
            Expression::Bool(b) => b.token.literal.clone(),
            Expression::Nil => "nil".to_string(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expression::Ident(ident) => write!(f, "{}", ident),
            Expression::Number(num) => write!(f, "{}", num),
            Expression::Unary(prefix) => write!(f, "{}", prefix),
            Expression::Binary(binary) => write!(f, "{}", binary),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::Nil => write!(f, "let"),
        }
    }
}
