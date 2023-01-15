use super::environment::*;
use super::error::RTError;
use super::object::*;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub struct Evaluator {
    environment: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<Object, RTError> {
        self.eval_statements(program.statements)
    }

    // Unwrap return values here since this is the outer most block
    fn eval_statements(&mut self, statements: Vec<Statement>) -> Result<Object, RTError> {
        let mut result = Object::Nil;
        for stmt in statements {
            result = self.eval_statement(stmt)?;
            if let Object::Return(retval) = result {
                return Ok(*retval);
            }
        }
        Ok(result)
    }

    // While evaluating block statements, do not unwrap return value.
    // Only check if it is a return value and if so, return the
    // Object::Return(val) object. This is so that a nested block
    // statement can return value correctly. This helps in the outer
    // block also return the wrapped return i.e. Object::Return(val)
    // Unwrapping only happens while executing the outer most block
    // statement which is a statement one level down the program.
    fn eval_block_statement(&mut self, stmt: BlockStatement) -> Result<Object, RTError> {
        let mut result = Object::Nil;
        for stmt in stmt.statements {
            result = self.eval_statement(stmt)?;
            if let Object::Return(_) = result {
                return Ok(result);
            }
        }
        Ok(result)
    }

    // Wrap the return value in a Return object
    fn eval_return_stmt(&mut self, expr: Expression) -> Result<Object, RTError> {
        let value = self.eval_expression(expr)?;
        Ok(Object::Return(Box::new(value)))
    }

    fn eval_expression(&mut self, expr: Expression) -> Result<Object, RTError> {
        match expr {
            Expression::Number(num) => Ok(Object::Number(num.value)),
            Expression::Bool(num) => Ok(Object::Bool(num.value)),
            Expression::Unary(unary) => {
                let right = self.eval_expression(*unary.right)?;
                self.eval_prefix_expr(&unary.operator, right, unary.token.line)
            }
            Expression::Binary(binary) => {
                let left = self.eval_expression(*binary.left)?;
                let right = self.eval_expression(*binary.right)?;
                self.eval_infix_expr(&binary.operator, left, right, binary.token.line)
            }
            Expression::If(expr) => {
                let condition = self.eval_expression(*expr.condition)?;
                if Self::is_truthy(condition) {
                    return self.eval_block_statement(expr.then_stmt);
                } else {
                    if let Some(else_stmt) = expr.else_stmt {
                        return self.eval_block_statement(else_stmt);
                    }
                }
                Ok(Object::Nil)
            }
            Expression::Ident(expr) => Ok(self.environment.get(&expr.token)?),
            _ => Ok(Object::Nil),
        }
    }

    fn eval_let_stmt(&mut self, name: &Identifier, expr: Expression) -> Result<Object, RTError> {
        let value = self.eval_expression(expr)?;
        let name = name.token.clone();
        self.environment.set(&name, value)?;
        Ok(Object::Nil)
    }

    fn eval_statement(&mut self, stmt: Statement) -> Result<Object, RTError> {
        match stmt {
            Statement::Expr(stmt) => self.eval_expression(stmt.value),
            Statement::Return(stmt) => self.eval_return_stmt(stmt.value),
            Statement::Let(stmt) => self.eval_let_stmt(&stmt.name, stmt.value),
            _ => Ok(Object::Nil),
        }
    }

    fn is_truthy(obj: Object) -> bool {
        match obj {
            Object::Nil => false,
            Object::Bool(b) => b,
            Object::Number(n) => n != 0.,
            _ => true,
        }
    }

    fn eval_prefix_expr(
        &self,
        operator: &str,
        right: Object,
        line: usize,
    ) -> Result<Object, RTError> {
        match operator {
            "!" => Ok(self.eval_bang_operator_expr(right)),
            "-" => self.eval_minus_operator_expr(right, line),
            _ => Err(RTError::new("invalid prefix operator", line)),
        }
    }

    // Does not return runtime error
    fn eval_bang_operator_expr(&self, right: Object) -> Object {
        Object::Bool(right.is_falsey())
    }

    fn eval_minus_operator_expr(&self, right: Object, line: usize) -> Result<Object, RTError> {
        match right {
            Object::Number(num) => Ok(Object::Number(-num)),
            _ => Err(RTError::new("invalid unary operation", line)),
        }
    }

    fn eval_infix_expr(
        &self,
        operator: &str,
        left: Object,
        right: Object,
        line: usize,
    ) -> Result<Object, RTError> {
        match (left, right) {
            (Object::Number(left), Object::Number(right)) => match operator {
                "+" => Ok(Object::Number(left + right)),
                "-" => Ok(Object::Number(left - right)),
                "*" => Ok(Object::Number(left * right)),
                "/" => Ok(Object::Number(left / right)),
                "<" => Ok(Object::Bool(left < right)),
                ">" => Ok(Object::Bool(left > right)),
                "==" => Ok(Object::Bool(left == right)),
                "!=" => Ok(Object::Bool(left != right)),
                _ => Err(RTError::new("invalid binary operator", line)),
            },
            (Object::Bool(left), Object::Bool(right)) => match operator {
                "==" => Ok(Object::Bool(left == right)),
                "!=" => Ok(Object::Bool(left != right)),
                _ => Err(RTError::new("invalid binary operation", line)),
            },
            _ => Err(RTError::new("invalid binary operation", line)),
        }
    }
}
