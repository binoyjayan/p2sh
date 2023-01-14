use super::object::*;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval_program(&self, program: Program) -> Object {
        self.eval_statements(program.statements)
    }

    // Unwrap return values here since this is the outer most block
    fn eval_statements(&self, statements: Vec<Statement>) -> Object {
        let mut result = Object::Nil;
        for stmt in statements {
            result = self.eval_statement(stmt);
            if let Object::Return(retval) = result {
                return *retval;
            }
        }
        result
    }

    // While evaluating block statements, do not unwrap return value.
    // Only check if it is a return value and if so, return the
    // Object::Return(val) object. This is so that a nested block
    // statement can return value correctly. This helps in the outer
    // block also return the wrapped return i.e. Object::Return(val)
    // Unwrapping only happens while executing the outer most block
    // statement which is a statement one level down the program.
    fn eval_block_statement(&self, stmt: BlockStatement) -> Object {
        let mut result = Object::Nil;
        for stmt in stmt.statements {
            result = self.eval_statement(stmt);
            if let Object::Return(_) = result {
                return result;
            }
        }
        result
    }

    // Wrap the return value in a Return object
    fn eval_return_stmt(&self, expr: Expression) -> Object {
        let value = self.eval_expression(expr);
        Object::Return(Box::new(value))
    }

    fn eval_expression(&self, expr: Expression) -> Object {
        match expr {
            Expression::Number(num) => Object::Number(num.value),
            Expression::Bool(num) => Object::Bool(num.value),
            Expression::Unary(unary) => {
                let right = self.eval_expression(*unary.right);
                self.eval_prefix_expr(&unary.operator, right)
            }
            Expression::Binary(binary) => {
                let left = self.eval_expression(*binary.left);
                let right = self.eval_expression(*binary.right);
                self.eval_infix_expr(&binary.operator, left, right)
            }
            Expression::If(expr) => {
                let condition = self.eval_expression(*expr.condition);
                if Self::is_truthy(condition) {
                    return self.eval_block_statement(expr.then_stmt);
                } else {
                    if let Some(else_stmt) = expr.else_stmt {
                        return self.eval_block_statement(else_stmt);
                    }
                }
                Object::Nil
            }
            _ => Object::Nil,
        }
    }

    fn eval_statement(&self, stmt: Statement) -> Object {
        match stmt {
            Statement::Expr(stmt) => self.eval_expression(stmt.value),
            Statement::Return(stmt) => self.eval_return_stmt(stmt.value),
            _ => Object::Nil,
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

    fn eval_prefix_expr(&self, operator: &str, right: Object) -> Object {
        match operator {
            "!" => self.eval_bang_operator_expr(right),
            "-" => self.eval_minus_operator_expr(right),
            _ => Object::Nil,
        }
    }

    fn eval_bang_operator_expr(&self, right: Object) -> Object {
        match right {
            Object::Bool(true) => Object::Bool(false),
            Object::Bool(false) => Object::Bool(true),
            Object::Nil => Object::Bool(true),
            _ => Object::Bool(false),
        }
    }

    fn eval_minus_operator_expr(&self, right: Object) -> Object {
        match right {
            Object::Number(num) => Object::Number(-num),
            _ => Object::Nil,
        }
    }

    fn eval_infix_expr(&self, operator: &str, left: Object, right: Object) -> Object {
        match (left, right) {
            (Object::Number(left), Object::Number(right)) => match operator {
                "+" => Object::Number(left + right),
                "-" => Object::Number(left - right),
                "*" => Object::Number(left * right),
                "/" => Object::Number(left / right),
                "<" => Object::Bool(left < right),
                ">" => Object::Bool(left > right),
                "==" => Object::Bool(left == right),
                "!=" => Object::Bool(left != right),
                _ => Object::Nil,
            },
            (Object::Bool(left), Object::Bool(right)) => match operator {
                "==" => Object::Bool(left == right),
                "!=" => Object::Bool(left != right),
                _ => Object::Nil,
            },
            _ => Object::Nil,
        }
    }
}
