pub mod ast;
pub mod tests;

use crate::scanner::*;
use crate::token::*;
use ast::*;

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
            _ => Err("Invalid Statement".to_string()),
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
}
