pub mod ast;
pub mod precedence;
pub mod rules;
pub mod tests;

use crate::scanner::token::*;
use crate::scanner::*;
use ast::expr::*;
use ast::stmt::*;
use ast::*;

use self::precedence::Precedence;

type ParseError = String;
type ParseErrors = Vec<ParseError>;

#[derive(Default)]
pub struct Parser {
    scanner: Scanner,
    previous: Token,
    current: Token,
    peek_next: Token,
    errors: ParseErrors,
}

impl Parser {
    pub fn new(scanner: Scanner) -> Self {
        let mut parser = Self {
            scanner,
            ..Default::default()
        };
        parser.next_token();
        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        self.previous = self.current.clone();
        self.current = self.peek_next.clone();
        self.peek_next = self.scanner.next_token();
    }

    fn prev_token_is(&self, ttype: &TokenType) -> bool {
        self.previous.ttype == *ttype
    }

    fn curr_token_is(&self, ttype: &TokenType) -> bool {
        self.current.ttype == *ttype
    }

    fn peek_token_is(&self, ttype: &TokenType) -> bool {
        self.peek_next.ttype == *ttype
    }

    fn expect_peek(&mut self, ttype: &TokenType) -> bool {
        if self.peek_token_is(ttype) {
            self.next_token();
            true
        } else {
            self.peek_error(ttype);
            false
        }
    }

    pub fn push_error(&mut self, err: &str) {
        self.errors
            .push(format!("[line {}] {}", self.scanner.get_line(), err));
        self.synchronize();
    }

    pub fn parse_errors(&self) -> &Vec<String> {
        &self.errors
    }

    pub fn peek_error(&mut self, ttype: &TokenType) {
        let msg = format!(
            "expected token {}, got {} instead",
            ttype, self.peek_next.ttype
        );
        self.push_error(&msg);
    }

    // Synchronize parser upon encountering error
    fn synchronize(&mut self) {
        while !self.peek_token_is(&TokenType::Eof) {
            if self.prev_token_is(&TokenType::Semicolon) {
                return;
            }
            if matches!(
                self.peek_next.ttype,
                TokenType::Function | TokenType::Let | TokenType::If | TokenType::Return
            ) {
                return;
            }
            self.next_token();
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program::default();

        while self.current.ttype != TokenType::Eof {
            // TODO: Revisit error handling
            if let Ok(stmt) = self.parse_statement() {
                program.statements.push(stmt)
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current.ttype {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Loop => self.parse_loop_statement(),
            TokenType::Break => self.parse_break_statement(),
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let token_let = self.current.clone();
        if !self.expect_peek(&TokenType::Identifier) {
            return Ok(Statement::Invalid);
        }
        let token_ident = self.current.clone();
        if !self.expect_peek(&TokenType::Assign) {
            return Ok(Statement::Invalid);
        }
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);

        // If the value expression is a function, the update the function node
        // with the identifier contained from compiling the let statement since
        // the name of the function is not available within the function
        // expression. This is needed so that self references to recursive
        // functions can be parsed effectively.
        let value = if let Expression::Function(mut func) = value.clone() {
            func.name = token_ident.literal.clone();
            // use the updated function
            Expression::Function(func)
        } else {
            // if any other expression, use the original value
            value
        };

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        let identifier = Identifier {
            token: token_ident.clone(),
            value: token_ident.literal,
            access: AccessType::Set,
        };
        let let_stmt = LetStmt {
            token: token_let,
            name: identifier,
            value,
        };
        Ok(Statement::Let(let_stmt))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let token_ret = self.current.clone();
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        let ret_stmt = ReturnStmt {
            token: token_ret,
            value,
        };
        Ok(Statement::Return(ret_stmt))
    }

    fn parse_loop_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.current.clone();
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Ok(Statement::Invalid);
        }
        let body = self.parse_block_statement();
        Ok(Statement::Loop(LoopStmt { token, body }))
    }

    fn parse_break_statement(&mut self) -> Result<Statement, ParseError> {
        let token_ret = self.current.clone();
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        let break_stmt = BreakStmt { token: token_ret };
        Ok(Statement::Break(break_stmt))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let token_expr = self.current.clone();
        let expr = self.parse_expression(Precedence::Assignment);
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        // Mark if the statement is an assignment expression statement
        let is_assign = matches!(expr, Expression::Assign(_));
        Ok(Statement::Expr(ExpressionStmt {
            token: token_expr,
            value: expr,
            is_assign,
        }))
    }

    /// Parsing an expression statement starts with 'parse_expression()'
    /// It first tries to find a prefix parser for the current token called.
    /// with 'Precedence::Lowest' as the parameter. The first token will always
    /// belong to some kind of prefix expression. It may turn out to be nested
    /// as an operand inside one or more infix expressions but as the code is
    /// read from left to right, the first token always belong to a prefix
    /// expression. It can be as simple as an identifier or a numeric
    /// expression, or as complicated as prefix expression such as '-' that
    /// accepts an arbitrary expression as its operand. If there is no prefix
    /// parse function, it is a syntax error. Otherwise, call the prefix
    /// parse function to parse the the current token and assign the resulting
    /// AST node into 'left_expr'. After parsing that, the prefix expression
    /// is done. Now look for an infix parser for the next token. If one is found,
    /// it means the prefix expression that was already compiled might be an
    /// operand to the infix operator, but only if 'precedence' is low enough
    /// to permit the infix operator. If the next token is too low precedence,
    /// or isn't an infix operator at all, the parsing is done. Otherwise,
    /// consume the operator and hand off control to the infix parser that was
    /// found. It consumes whatever other tokens it needs (the operator and
    /// the right operand) and returns back to parse_expression(). The infix
    /// parse function then creates a binary operator ast node with the left
    /// and right operand and the operator. Note that the infix parse function
    /// is passed the left operand as argument since it was already consumed.
    /// Also note that the right operand itself can be an prefix expression
    /// in itself (e.g. a numeric expression) or another infix expression
    /// such as a binary '+'. Then the loop continues and see if the next
    /// token is also a valid infix operator that can take the entire preceding
    /// expression as its operand. Continue the loop crunching through infix
    /// operators and their operands until a token is hit that that isn't an
    /// infix operator or is too low precedence.
    ///
    /// The associativity of infix expressions depends on the precedence
    /// condition used in the loop.
    /// 'a + b + c' -->> ((a + b) + c) when 'precedence < self.peek_precedence()'
    /// 'a + b + c' -->> (a + (b + c)) when 'precedence <= self.peek_precedence()'
    ///
    fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let can_assign = precedence <= Precedence::Assignment;
        self.peek_invalid_assignment(can_assign);

        // If there is a prefix parser for the current token
        if let Some(prefix) = self.curr_prefix() {
            let mut left_expr = prefix(self);

            // Continue parsing if the next token is not a semi-colon and has
            // a valid precedence for the current infix expression
            while self.peek_valid_expression(precedence) {
                if let Some(infix) = &self.peek_infix() {
                    self.next_token();
                    left_expr = infix(self, left_expr);
                }
            }
            left_expr
        } else {
            self.no_prefix_parse_error();
            Expression::Invalid
        }
    }

    fn no_prefix_parse_error(&mut self) {
        let msg = format!("failed to parse token '{}'", self.current);
        self.push_error(&msg);
    }

    pub fn print_errors(&self) -> bool {
        if self.errors.is_empty() {
            return false;
        }
        for msg in &self.errors {
            eprintln!("{}", msg);
        }
        true
    }

    pub fn peek_invalid_assignment(&mut self, can_assign: bool) {
        // Check if the precedence is low enough to allow assignment

        if !can_assign && self.peek_token_is(&TokenType::Assign) {
            self.push_error("Invalid assignment target");
        }
    }
}
