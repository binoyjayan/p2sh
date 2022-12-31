use std::fmt;

use crate::token::*;

pub enum Node {
    Stmt(Statement),
    Expr(Expression),
}

#[derive(Debug)]
pub enum Expression {
    Ident(Identifier),
    Nil,
}

pub enum Statement {
    Let(LetStmt),
    Return(ReturnStmt),
}

pub struct LetStmt {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub token: Token,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Statement {
    pub fn token_literal(&self) -> String {
        match &self {
            Statement::Let(stmt) => stmt.token.literal.clone(),
            Statement::Return(stmt) => stmt.token.literal.clone(),
            _ => "".to_string(),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Statement::Let(_) => write!(f, "let"),
            Statement::Return(_) => write!(f, "return"),
        }
    }
}

impl Expression {
    fn token_literal(&self) -> String {
        match &self {
            Expression::Ident(ident) => ident.token.literal.clone(),
            Expression::Nil => "nil".to_string(),
        }
    }
}

#[derive(Default)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    fn token_literal(&self) -> String {
        if self.statements.is_empty() {
            "".to_string()
        } else {
            self.statements[0].token_literal()
        }
    }
}
