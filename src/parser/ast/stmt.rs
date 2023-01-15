use std::fmt;

use super::expr::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStmt),
    Return(ReturnStmt),
    Expr(ExpressionStmt),
    Nil,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub token: Token,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt {
    pub token: Token,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl Statement {
    pub fn token_literal(&self) -> String {
        match &self {
            Statement::Let(stmt) => stmt.token.literal.clone(),
            Statement::Return(stmt) => stmt.token.literal.clone(),
            Statement::Expr(stmt) => stmt.token.literal.clone(),
            Statement::Nil => "nil".to_string(),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Statement::Let(l) => write!(f, "let {} = {};", l.name, l.value),
            Statement::Return(r) => write!(f, "return {};", r.value),
            Statement::Expr(e) => write!(f, "{}", e.value),
            Statement::Nil => write!(f, "nil"),
        }
    }
}
