use crate::object::*;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;
use crate::parser::*;
use crate::scanner::*;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    // Unwrap return values here since this is the outer most block
    pub fn eval_program(&self, program: Program) -> Object {
        let mut result = Object::Nil;
        for stmt in program.statements {
            result = self.eval_statement(stmt);
            if let Object::Return(retval) = result {
                return *retval;
            }
        }
        result
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    fn check_parse_errors(parser: &Parser) {
        if parser.print_errors() {
            panic!("{} parse errors", parser.parse_errors().len());
        }
    }

    fn test_numeric_object(evaluated: Object, expected: f64) {
        if let Object::Number(num) = evaluated {
            assert_eq!(
                num, expected,
                "object has wrong value. got={}, want={}",
                num, expected
            );
        } else {
            panic!("object is not numeric. got={}", evaluated);
        }
    }

    fn test_boolean_object(evaluated: Object, expected: bool) {
        if let Object::Bool(b) = evaluated {
            assert_eq!(
                b, expected,
                "object has wrong value. got={}, want={}",
                b, expected
            );
        } else {
            panic!("object is not boolean. got={}", evaluated);
        }
    }

    fn test_nil_object(evaluated: Object) {
        if let Object::Nil = evaluated {
        } else {
            panic!("object is not nil. got={}", evaluated);
        }
    }

    fn test_eval(input: &str) -> Object {
        let scanner = Scanner::new(input);
        let mut parser = Parser::new(scanner);
        let program = parser.parse_program();
        check_parse_errors(&parser);
        let evaluator = Evaluator::new();
        evaluator.eval_program(program)
    }

    #[test]
    fn test_eval_numeric_expr() {
        struct NumericObj {
            input: &'static str,
            expected: f64,
        }
        let numeric_tests = vec![
            NumericObj {
                input: "5",
                expected: 5.,
            },
            NumericObj {
                input: "10",
                expected: 10.,
            },
            NumericObj {
                input: "-5",
                expected: -5.,
            },
            NumericObj {
                input: "-10",
                expected: -10.,
            },
            NumericObj {
                input: "5 + 5 + 5 + 5 - 10",
                expected: 10.,
            },
            NumericObj {
                input: "2 * 2 * 2 * 2 * 2",
                expected: 32.,
            },
            NumericObj {
                input: "-50 + 100 + -50",
                expected: 0.,
            },
            NumericObj {
                input: "5 * 2 + 10",
                expected: 20.,
            },
            NumericObj {
                input: "5 + 2 * 10",
                expected: 25.,
            },
            NumericObj {
                input: "20 + 2 * -10",
                expected: 0.,
            },
            NumericObj {
                input: "50 / 2 * 2 + 10",
                expected: 60.,
            },
            NumericObj {
                input: "2 * (5 + 10)",
                expected: 30.,
            },
            NumericObj {
                input: "3 * 3 * 3 + 10",
                expected: 37.,
            },
            NumericObj {
                input: "3 * (3 * 3) + 10",
                expected: 37.,
            },
            NumericObj {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
                expected: 50.,
            },
        ];
        for test in numeric_tests {
            let evaluated = test_eval(test.input);
            test_numeric_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_eval_boolean_expr() {
        struct BooleanObj {
            input: &'static str,
            expected: bool,
        }
        let boolean_tests = vec![
            BooleanObj {
                input: "true",
                expected: true,
            },
            BooleanObj {
                input: "false",
                expected: false,
            },
            BooleanObj {
                input: "1 < 2",
                expected: true,
            },
            BooleanObj {
                input: "1 > 2",
                expected: false,
            },
            BooleanObj {
                input: "1 < 1",
                expected: false,
            },
            BooleanObj {
                input: "1 > 1",
                expected: false,
            },
            BooleanObj {
                input: "1 == 1",
                expected: true,
            },
            BooleanObj {
                input: "1 != 1",
                expected: false,
            },
            BooleanObj {
                input: "1 == 2",
                expected: false,
            },
            BooleanObj {
                input: "1 != 2",
                expected: true,
            },
            BooleanObj {
                input: "true == true",
                expected: true,
            },
            BooleanObj {
                input: "false == false",
                expected: true,
            },
            BooleanObj {
                input: "true == false",
                expected: false,
            },
            BooleanObj {
                input: "true != false",
                expected: true,
            },
            BooleanObj {
                input: "false != true",
                expected: true,
            },
            BooleanObj {
                input: "(1 < 2) == true",
                expected: true,
            },
            BooleanObj {
                input: "(1 < 2) == false",
                expected: false,
            },
            BooleanObj {
                input: "(1 > 2) == true",
                expected: false,
            },
            BooleanObj {
                input: "(1 > 2) == false",
                expected: true,
            },
        ];
        for test in boolean_tests {
            let evaluated = test_eval(test.input);
            test_boolean_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_eval_bang_operator() {
        struct BangExpr {
            input: &'static str,
            expected: bool,
        }
        let boolean_tests = vec![
            BangExpr {
                input: "!true",
                expected: false,
            },
            BangExpr {
                input: "!false",
                expected: true,
            },
            BangExpr {
                input: "!5",
                expected: false,
            },
            BangExpr {
                input: "!!true",
                expected: true,
            },
            BangExpr {
                input: "!!false",
                expected: false,
            },
        ];
        for test in boolean_tests {
            let evaluated = test_eval(test.input);
            test_boolean_object(evaluated, test.expected);
        }
    }

    #[test]
    fn test_if_else_expr() {
        struct IfElseExpr {
            input: &'static str,
            expected: Object,
        }
        let if_else_tests = vec![
            IfElseExpr {
                input: "if (true) { 10 }",
                expected: Object::Number(10.),
            },
            IfElseExpr {
                input: "if (false) { 10 }",
                expected: Object::Nil,
            },
            IfElseExpr {
                input: "if (1) { 10 }",
                expected: Object::Number(10.),
            },
            IfElseExpr {
                input: "if (1 < 2) { 10 }",
                expected: Object::Number(10.),
            },
            IfElseExpr {
                input: "if (1 > 2) { 10 }",
                expected: Object::Nil,
            },
            IfElseExpr {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: Object::Number(10.),
            },
            IfElseExpr {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: Object::Number(20.),
            },
        ];
        for test in if_else_tests {
            let evaluated = test_eval(test.input);
            match test.expected {
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                Object::Nil => test_nil_object(evaluated),
                _ => panic!("Invalid expected object"),
            }
        }
    }

    #[test]
    fn test_return_stmt() {
        struct ReturnTest {
            input: &'static str,
            expected: Object,
        }
        let if_else_tests = vec![
            ReturnTest {
                input: "return 10;",
                expected: Object::Number(10.),
            },
            ReturnTest {
                input: "return 10; 9;",
                expected: Object::Number(10.),
            },
            ReturnTest {
                input: "return 2 * 5; 9;",
                expected: Object::Number(10.),
            },
            ReturnTest {
                input: "9; return 2 * 5; 9;",
                expected: Object::Number(10.),
            },
            ReturnTest {
                input: "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
                expected: Object::Number(10.),
            },
        ];
        for test in if_else_tests {
            let evaluated = test_eval(test.input);
            match test.expected {
                Object::Number(expected) => test_numeric_object(evaluated, expected),
                _ => panic!("Invalid expected object"),
            }
        }
    }
}
