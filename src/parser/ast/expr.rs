use std::fmt;

use crate::scanner::token::*;

use super::stmt::*;

#[derive(Clone, Debug)]
pub enum Expression {
    Ident(Identifier),
    Number(NumberLiteral),
    Str(StringLiteral),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Bool(BooleanExpr),
    If(IfExpr),
    Function(FunctionLiteral),
    Call(CallExpr),
    Array(ArrayLiteral),
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
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

impl fmt::Display for StringLiteral {
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

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub token: Token, // if token
    pub condition: Box<Expression>,
    pub then_stmt: BlockStatement,
    pub else_stmt: Option<BlockStatement>,
}

impl fmt::Display for IfExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if ({}) {{ {} }}", self.condition, self.then_stmt)?;
        if let Some(else_stmt) = &self.else_stmt {
            write!(f, " else {{ {} }}", else_stmt)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
}

impl fmt::Display for FunctionLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params_str = self
            .params
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let params_str = params_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{} ({}) {}", self.token, params_str, self.body)
    }
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub token: Token,          // The '(' Token
    pub func: Box<Expression>, // Identifier or FunctionLiteral
    pub args: Vec<Expression>,
}

impl fmt::Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args_str = self
            .args
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let args_str = args_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{}({})", self.func, args_str)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}

impl fmt::Display for ArrayLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elements_str = self
            .elements
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let elements_str = elements_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "[{}]", elements_str)
    }
}

impl Expression {
    #[allow(dead_code)]
    fn token_literal(&self) -> String {
        match &self {
            Expression::Ident(ident) => ident.token.literal.clone(),
            Expression::Number(num) => num.token.literal.clone(),
            Expression::Str(s) => s.token.literal.clone(),
            Expression::Unary(unary) => unary.token.literal.clone(),
            Expression::Binary(binary) => binary.token.literal.clone(),
            Expression::Bool(b) => b.token.literal.clone(),
            Expression::If(i) => i.token.literal.clone(),
            Expression::Function(f) => f.token.literal.clone(),
            Expression::Call(c) => c.token.literal.clone(),
            Expression::Array(s) => s.token.literal.clone(),
            Expression::Nil => "nil".to_string(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expression::Ident(ident) => write!(f, "{}", ident),
            Expression::Number(num) => write!(f, "{}", num),
            Expression::Str(s) => write!(f, "{}", s),
            Expression::Unary(prefix) => write!(f, "{}", prefix),
            Expression::Binary(binary) => write!(f, "{}", binary),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::If(i) => write!(f, "{}", i),
            Expression::Function(fun) => write!(f, "{}", fun),
            Expression::Call(c) => write!(f, "{}", c),
            Expression::Array(s) => write!(f, "{}", s),
            Expression::Nil => write!(f, "nil"),
        }
    }
}
