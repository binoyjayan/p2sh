pub mod ast;
pub mod precedence;
pub mod rules;
pub mod tests;

use crate::scanner::*;
use crate::token::*;
use ast::*;
use rules::*;

use self::precedence::Precedence;

type ParseError = String;
type ParseErrors = Vec<ParseError>;

#[derive(Default)]
pub struct Parser {
    scanner: Scanner,
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
        self.current = self.peek_next.clone();
        self.peek_next = self.scanner.next_token();
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

    // check if we have run out of tokens
    fn is_at_end(&self) -> bool {
        self.peek_next.ttype == TokenType::Eof
    }

    pub fn parse_errors(&self) -> &Vec<String> {
        &self.errors
    }

    pub fn peek_error(&mut self, ttype: &TokenType) {
        let msg = format!(
            "expected next token to be {}, got {} instead",
            ttype, self.peek_next.ttype
        );
        self.errors.push(msg);
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program::default();

        while self.current.ttype != TokenType::Eof {
            match self.parse_statement() {
                Ok(stmt) => program.statements.push(stmt),
                Err(_) => {}
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current.ttype {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let token_let = self.current.clone();
        // TODO: return false?
        self.expect_peek(&TokenType::Identifier);
        let token_ident = self.current.clone();
        self.expect_peek(&TokenType::Assign);

        while !self.curr_token_is(&TokenType::Semicolon) && !self.is_at_end() {
            self.next_token();
        }

        let identifier = Identifier {
            token: token_ident.clone(),
            value: token_ident.literal,
        };
        // TODO: Use the right 'value'
        let let_stmt = LetStmt {
            token: token_let,
            name: identifier,
            value: Expression::Nil,
        };
        Ok(Statement::Let(let_stmt))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let token_ret = self.current.clone();
        self.next_token();
        while !self.curr_token_is(&TokenType::Semicolon) && !self.is_at_end() {
            self.next_token();
        }
        let ret_stmt = ReturnStmt {
            token: token_ret,
            value: Expression::Nil,
        };
        Ok(Statement::Return(ret_stmt))
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let token_expr = self.current.clone();
        let expr = self.parse_expression(Precedence::Lowest);
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Statement::Expr(ExpressionStmt {
            token: token_expr,
            value: expr,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let ttype = self.current.ttype as usize;
        if let Some(prefix) = &PARSE_RULES[ttype].prefix {
            prefix(self)
        } else {
            self.no_prefix_parse_error(self.current.ttype);
            Expression::Nil
        }
    }

    fn no_prefix_parse_error(&mut self, ttype: TokenType) {
        let msg = format!("no prefix parser is available for token type '{}'", ttype);
        self.errors.push(msg);
    }
}
