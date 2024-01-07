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
    in_match_pattern: bool,
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

    // Report error at a specified line. This is used for reporting errors
    // in expressions that had an error in a prior sub expression which
    // causes the parser to synchronize and skip tokens until the next
    // statement. This helps report error at a line prior to synchronization.
    pub fn push_error_at(&mut self, err: &str, line: usize) {
        self.errors.push(format!("[line {}] {}", line, err));
        self.synchronize();
    }

    pub fn push_error(&mut self, err: &str) {
        self.push_error_at(err, self.scanner.get_line())
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
                TokenType::Function
                    | TokenType::Let
                    | TokenType::If
                    | TokenType::Return
                    | TokenType::Loop
                    | TokenType::While
                    | TokenType::Break
                    | TokenType::Continue
                    | TokenType::Match
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
            TokenType::Loop => self.parse_loop_statement(None),
            TokenType::While => self.parse_while_statement(None),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::Function => self.parse_function_statement(),
            TokenType::LeftBrace => self.parse_block_begin(),
            TokenType::Filter => self.parse_filter_statement(),
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
        let value = self.parse_expression(Precedence::Lowest, false);

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
            context: ParseContext {
                access: AccessType::Set,
            },
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
        let value = if self.peek_token_is(&TokenType::Semicolon)
            || self.peek_token_is(&TokenType::RightBrace)
        {
            // No return value
            None
        } else {
            self.next_token();
            Some(self.parse_expression(Precedence::Lowest, false))
        };

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        let ret_stmt = ReturnStmt {
            token: token_ret,
            value,
        };
        Ok(Statement::Return(ret_stmt))
    }

    fn parse_block_begin(&mut self) -> Result<Statement, ParseError> {
        Ok(Statement::Block(self.parse_block_statement()))
    }

    fn parse_loop_statement(&mut self, label: Option<Token>) -> Result<Statement, ParseError> {
        let token = self.current.clone();
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Ok(Statement::Invalid);
        }
        let body = self.parse_block_statement();
        Ok(Statement::Loop(LoopStmt { token, label, body }))
    }

    fn parse_while_statement(&mut self, label: Option<Token>) -> Result<Statement, ParseError> {
        let token = self.current.clone();
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest, false);
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Ok(Statement::Invalid);
        }
        let body = self.parse_block_statement();
        Ok(Statement::While(WhileStmt {
            token,
            label,
            condition,
            body,
        }))
    }

    fn parse_break_statement(&mut self) -> Result<Statement, ParseError> {
        // The break token
        let token = self.current.clone();
        // If there is a label, parse it
        let label = if self.peek_token_is(&TokenType::Identifier) {
            self.next_token();
            Some(self.current.clone())
        } else {
            None
        };
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        let break_stmt = BreakStmt { token, label };
        Ok(Statement::Break(break_stmt))
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, ParseError> {
        // The continue token
        let token = self.current.clone();
        // If there is a label, parse it
        let label = if self.peek_token_is(&TokenType::Identifier) {
            self.next_token();
            Some(self.current.clone())
        } else {
            None
        };
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        let con_stmt = ContinueStmt { token, label };
        Ok(Statement::Continue(con_stmt))
    }

    // Function statements are of the form 'fn <name>(<params>) { <body> }'.
    // They differ from function expressions and are parsed differently.
    // However, the underlying implementations are the same.
    fn parse_function_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.current.clone(); // fn keyword
        if !self.peek_token_is(&TokenType::Identifier) {
            return self.parse_expr_statement();
        }
        // Advance to the function name
        self.next_token();
        let name = self.current.clone(); // fn name
        if !self.expect_peek(&TokenType::LeftParen) {
            return Ok(Statement::Invalid);
        }
        let params = self.parse_function_params();
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Ok(Statement::Invalid);
        }
        let body = self.parse_block_statement();
        // The name of the string is known here
        Ok(Statement::Function(FunctionLiteral {
            name: name.literal,
            token,
            params,
            body,
        }))
    }

    fn parse_filter_statement(&mut self) -> Result<Statement, ParseError> {
        let token: Token = self.current.clone();
        // advance to the condition expression
        self.next_token();
        let filter = self.parse_expression(Precedence::Assignment, false);

        // The action block is optional
        let action = if self.peek_token_is(&TokenType::LeftBrace) {
            self.next_token();
            Some(self.parse_block_statement())
        } else {
            None
        };

        Ok(Statement::Filter(FilterStmt {
            token,
            filter: Box::new(filter),
            action,
        }))
    }

    // Parse a statement as expression statement if it is none of the
    // other statement types. However, if the statement begins with
    // an idenifier and a colon, it is a label and should be followed
    // by a loop statement.
    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let token_expr = self.current.clone();
        if self.curr_token_is(&TokenType::Identifier) && self.peek_token_is(&TokenType::Colon) {
            // Advance the token to the colon (':')
            self.next_token();
            // match peek token to be loop and while
            return match self.peek_next.ttype {
                TokenType::Loop => {
                    // Advance the token to the loop/while keyword
                    self.next_token();
                    // pass the label token to the loop statement
                    return self.parse_loop_statement(Some(token_expr));
                }
                TokenType::While => {
                    self.next_token();
                    // pass the label token to the while statement
                    return self.parse_while_statement(Some(token_expr));
                }
                _ => {
                    // only loops support labels for now
                    Ok(Statement::Invalid)
                }
            };
        }
        let expr = self.parse_expression(Precedence::Assignment, false);
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
    fn parse_expression(&mut self, precedence: Precedence, property: bool) -> Expression {
        let can_assign = precedence <= Precedence::Assignment;
        self.peek_invalid_assignment(can_assign);

        // If there is a prefix parser for the current token
        if let Some(prefix) = self.curr_prefix() {
            let mut left_expr = prefix(self, property);

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
