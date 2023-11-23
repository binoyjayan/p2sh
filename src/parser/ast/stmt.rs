use std::fmt;

use super::expr::*;
use crate::scanner::token::*;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStmt),
    Return(ReturnStmt),
    Expr(ExpressionStmt),
    Loop(LoopStmt),
    While(WhileStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Function(FunctionLiteral),
    Invalid,
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
    pub value: Option<Expression>,
}

impl fmt::Display for ReturnStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            Some(ref v) => write!(f, "return {};", v),
            None => write!(f, "return;"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub token: Token,
    pub label: Option<Token>,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub token: Token,
    pub label: Option<Token>,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub token: Token, // loop token
    pub label: Option<Token>,
    pub body: BlockStatement,
}

impl fmt::Display for LoopStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "loop {{ {} }}", self.body)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub token: Token, // while token
    pub label: Option<Token>,
    pub condition: Expression,
    pub body: BlockStatement,
}

impl fmt::Display for WhileStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "while {{ {} }}", self.body)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt {
    pub token: Token,
    pub value: Expression,
    pub is_assign: bool,
}

#[derive(Clone, Debug)]
pub struct BlockStatement {
    pub token: Token, // '{'
    pub statements: Vec<Statement>,
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}; ", stmt)?;
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
            Statement::Loop(stmt) => stmt.token.literal.clone(),
            Statement::While(stmt) => stmt.token.literal.clone(),
            Statement::Break(brk) => brk.token.literal.clone(),
            Statement::Continue(con) => con.token.literal.clone(),
            Statement::Function(stmt) => stmt.token.literal.clone(),
            Statement::Invalid => "null".to_string(),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Statement::Let(l) => write!(f, "let {} = {};", l.name, l.value),
            Statement::Return(r) => write!(f, "{}", r),
            Statement::Expr(e) => write!(f, "{}", e.value),
            Statement::Loop(l) => write!(f, "{}", l),
            Statement::While(w) => write!(f, "{}", w),
            Statement::Break(_) => write!(f, "break"),
            Statement::Continue(_) => write!(f, "continue"),
            Statement::Function(fun) => write!(f, "{}", fun),
            Statement::Invalid => write!(f, "invalid"),
        }
    }
}
