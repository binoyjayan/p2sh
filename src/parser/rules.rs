use super::*;
use lazy_static::lazy_static;

type PrefixParserFn = fn(&mut Parser) -> Expression;
type InfixParserFn = fn(&mut Parser, Expression) -> Expression;

#[derive(Clone, Default)]
pub struct ParseRule {
    pub prefix: Option<PrefixParserFn>,
    pub infix: Option<InfixParserFn>,
    pub precedence: Precedence,
    pub associativity: Associativity,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum Associativity {
    #[default]
    Left,
    Right,
}

impl ParseRule {
    pub fn new(
        prefix: Option<PrefixParserFn>,
        infix: Option<InfixParserFn>,
        precedence: Precedence,
    ) -> Self {
        Self {
            infix,
            prefix,
            precedence,
            associativity: Associativity::Left,
        }
    }

    pub fn new_with_assoc(
        prefix: Option<PrefixParserFn>,
        infix: Option<InfixParserFn>,
        precedence: Precedence,
        associativity: Associativity,
    ) -> Self {
        Self {
            infix,
            prefix,
            precedence,
            associativity,
        }
    }
}

lazy_static! {
    pub static ref PARSE_RULES: Vec<ParseRule> = {
        let mut rules = vec![ParseRule::default(); TokenType::NumberOfTokens as usize];
        // Terminal expressions
        rules[TokenType::Identifier as usize] =
            ParseRule::new(Some(Parser::parse_identifier), None, Precedence::Lowest);
        rules[TokenType::Integer as usize] =
            ParseRule::new(Some(Parser::parse_integer), None, Precedence::Lowest);
            rules[TokenType::Float as usize] =
            ParseRule::new(Some(Parser::parse_float), None, Precedence::Lowest);
        rules[TokenType::Str as usize] =
            ParseRule::new(Some(Parser::parse_string), None, Precedence::Lowest);
        // Logical
        rules[TokenType::Bang as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            None,
            Precedence::Lowest,
        );
        // Binary
        rules[TokenType::Equal as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Equality,
        );
        rules[TokenType::BangEqual as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Equality,
        );
        rules[TokenType::Less as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Comparison,
        );
        rules[TokenType::Greater as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Comparison,
        );
        rules[TokenType::Plus as usize] =
            ParseRule::new(None, Some(Parser::parse_infix_expression), Precedence::Term);
        rules[TokenType::Minus as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            Some(Parser::parse_infix_expression),
            Precedence::Term,
        );
        rules[TokenType::Asterisk as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Factor,
        );
        rules[TokenType::Slash as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Factor,
        );
        // Boolean
        rules[TokenType::True as usize] =
            ParseRule::new(Some(Parser::parse_boolean), None, Precedence::Lowest);
        rules[TokenType::False as usize] =
            ParseRule::new(Some(Parser::parse_boolean), None, Precedence::Lowest);
        // Grouped expressions (prefix parser) and call expressions (infix parser)
        rules[TokenType::LeftParen as usize] =
            ParseRule::new(Some(Parser::parse_grouped), Some(Parser::parse_call_expression), Precedence::Call);
        // Control flow
        rules[TokenType::If as usize] =
            ParseRule::new(Some(Parser::parse_if_expr), None, Precedence::Lowest);
        // Function
        rules[TokenType::Function as usize] =
            ParseRule::new(Some(Parser::parse_function_literal), None, Precedence::Lowest);
        // Array literal (prefix) and index operator (infix) parser
        rules[TokenType::LeftBracket as usize] =
            ParseRule::new(Some(Parser::parse_array_literal), Some(Parser::parse_index_expression), Precedence::Call);
        rules[TokenType::LeftBrace as usize] =
            ParseRule::new(Some(Parser::parse_hash_literal), None, Precedence::Lowest);
        // Assignment
        rules[TokenType::Assign as usize] = ParseRule::new_with_assoc(
            None,
            Some(Parser::parse_assignment_expression),
            Precedence::Assignment,
            Associativity::Right
        );
        rules
    };
}

impl Parser {
    pub fn curr_precedence(&self) -> Precedence {
        PARSE_RULES[self.current.ttype as usize].precedence
    }
    pub fn curr_prefix(&self) -> Option<PrefixParserFn> {
        PARSE_RULES[self.current.ttype as usize].prefix
    }
    pub fn peek_infix(&self) -> Option<InfixParserFn> {
        PARSE_RULES[self.peek_next.ttype as usize].infix
    }
    pub fn peek_precedence(&self) -> Precedence {
        PARSE_RULES[self.peek_next.ttype as usize].precedence
    }
    pub fn peek_associativity(&self) -> Associativity {
        PARSE_RULES[self.peek_next.ttype as usize].associativity
    }

    // Return true if the next token is not semi-colon and also has a valid
    // precedence for the current expression. Parse greedy if the operator is
    // right associative. e.g. the assigmment. The calls to 'peek_token_is()'
    /// with TokenType's Semicolon and Eof are actually redundant.
    /// peek_precedence() returns 'Lowest' as the default precedence for the
    /// token types Semicolon and Eof. It only makes the code look more logical.
    pub fn peek_valid_expression(&self, precedence: Precedence) -> bool {
        let precedence_cond = match self.peek_associativity() {
            Associativity::Left => precedence < self.peek_precedence(),
            Associativity::Right => precedence <= self.peek_precedence(),
        };
        precedence_cond
            && !self.peek_token_is(&TokenType::Semicolon)
            && !self.peek_token_is(&TokenType::Eof)
    }

    fn parse_identifier(&mut self) -> Expression {
        let access = self.peek_access_type();
        Expression::Ident(Identifier {
            token: self.current.clone(),
            value: self.current.literal.clone(),
            access,
        })
    }

    fn parse_integer(&mut self) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Integer(IntegerLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as an integer", self.current.literal);
            self.push_error(&msg);
            Expression::Nil
        }
    }

    fn parse_float(&mut self) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Float(FloatLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a float", self.current.literal);
            self.push_error(&msg);
            Expression::Nil
        }
    }

    fn parse_string(&mut self) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Str(StringLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a string", self.current.literal);
            self.push_error(&msg);
            Expression::Nil
        }
    }

    // Parse unary expressions such as '-' and '!'
    fn parse_prefix_expression(&mut self) -> Expression {
        let operator = self.current.literal.clone();
        let token = self.current.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Unary);

        Expression::Unary(UnaryExpr {
            token,
            operator,
            right: Box::new(right),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Expression {
        let operator = self.current.literal.clone();
        let token = self.current.clone();
        // precedence of the operator
        let precedence = self.curr_precedence();

        // advance to the next token
        self.next_token();

        let right = self.parse_expression(precedence);
        Expression::Binary(BinaryExpr {
            token,
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_boolean(&mut self) -> Expression {
        self.peek_invalid_assignment(false);
        Expression::Bool(BooleanExpr {
            token: self.current.clone(),
            value: self.curr_token_is(&TokenType::True),
        })
    }

    // Override operator precedence using grouped expression
    fn parse_grouped(&mut self) -> Expression {
        self.next_token();
        let expr = self.parse_expression(Precedence::Assignment);
        if self.expect_peek(&TokenType::RightParen) {
            // check for cases such as '(a) = b'
            self.peek_invalid_assignment(false);
            expr
        } else {
            Expression::Nil
        }
    }

    fn parse_if_expr(&mut self) -> Expression {
        let token = self.current.clone();
        if !self.expect_peek(&TokenType::LeftParen) {
            return Expression::Nil;
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Assignment);
        if !self.expect_peek(&TokenType::RightParen) {
            return Expression::Nil;
        }
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Expression::Nil;
        }

        let then_stmt = self.parse_block_statement();

        // Check if an else branch exists
        let else_stmt = if self.peek_token_is(&TokenType::Else) {
            self.next_token();
            if !self.expect_peek(&TokenType::LeftBrace) {
                return Expression::Nil;
            }
            Some(self.parse_block_statement())
        } else {
            None
        };

        Expression::If(IfExpr {
            token,
            condition: Box::new(condition),
            then_stmt,
            else_stmt,
        })
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let mut statements = Vec::new();
        self.next_token();

        while !self.curr_token_is(&TokenType::RightBrace) && !self.curr_token_is(&TokenType::Eof) {
            if let Ok(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        BlockStatement { statements }
    }

    fn parse_function_literal(&mut self) -> Expression {
        let token = self.current.clone();
        if !self.expect_peek(&TokenType::LeftParen) {
            return Expression::Nil;
        }
        let params = self.parse_function_params();
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Expression::Nil;
        }
        let body = self.parse_block_statement();
        // The name of the string is unknown here so just use it as a
        // placeholder. Fill this in after parsing the 'let' statement.
        // The name of the function is available in that statement.
        Expression::Function(FunctionLiteral {
            name: String::new(),
            token,
            params,
            body,
        })
    }

    fn parse_function_params(&mut self) -> Vec<Identifier> {
        let mut identifiers = Vec::new();
        if self.peek_token_is(&TokenType::RightParen) {
            self.next_token();
            return identifiers;
        }
        self.next_token();
        let token_ident = self.current.clone();
        let ident_value = token_ident.literal.clone();
        identifiers.push(Identifier {
            token: token_ident,
            value: ident_value,
            access: AccessType::Get,
        });

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let token_ident = self.current.clone();
            let ident_value = token_ident.literal.clone();
            identifiers.push(Identifier {
                token: token_ident,
                value: ident_value,
                access: AccessType::Get,
            });
        }

        if !self.expect_peek(&TokenType::RightParen) {
            return Vec::new();
        }

        identifiers
    }

    // Call expressions do not have new token types. A call expression is an
    // identifier followed by a '(', a set of arguments separated by ','
    // followed by a ')' token. That makes it an infix parse expression since
    // the token '(' is in the middle of the identifier and the arguments list.
    fn parse_call_expression(&mut self, func: Expression) -> Expression {
        let token = self.previous.clone();

        Expression::Call(CallExpr {
            token,
            func: Box::new(func),
            args: self.parse_expression_list(TokenType::RightParen),
        })
    }

    // Generic function that parses call arguments as well as array literal
    // expression as both of those are essentially a comma separated list
    // of expressions. The only difference is the end token that is used to
    // indicate the end of the list. This token type is passed as an argument.
    fn parse_expression_list(&mut self, ttype_end: TokenType) -> Vec<Expression> {
        let mut args = Vec::new();

        if self.peek_token_is(&ttype_end) {
            self.next_token();
            return args;
        }
        self.next_token();
        args.push(self.parse_expression(Precedence::Assignment));
        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Assignment));
        }

        if !self.expect_peek(&ttype_end) {
            return Vec::new();
        }
        args
    }

    fn parse_array_literal(&mut self) -> Expression {
        let token = self.current.clone();

        Expression::Array(ArrayLiteral {
            token,
            elements: self.parse_expression_list(TokenType::RightBracket),
        })
    }

    // The index operator do not have a single operator between the operands
    // on each side. But in order to parse them, it is easier to pretend that
    // they do. The index expression 'a[0]' is treated as an infix expression
    // with an expression 'a' on the left and an index '0' on the right.
    fn parse_index_expression(&mut self, left: Expression) -> Expression {
        // The left bracket ('[') token
        let token = self.current.clone();
        // advance to the index token
        self.next_token();
        let index = self.parse_expression(Precedence::Assignment);
        if !self.expect_peek(&TokenType::RightBracket) {
            return Expression::Nil;
        }

        let access = self.peek_access_type();

        Expression::Index(IndexExpr {
            token,
            left: Box::new(left),
            index: Box::new(index),
            access,
        })
    }

    fn parse_hash_literal(&mut self) -> Expression {
        let token = self.current.clone();
        let mut pairs = Vec::new();

        while !self.peek_token_is(&TokenType::RightBrace) {
            // consume the first '{' or a ',' in each iteration
            self.next_token();
            let key: Expression = self.parse_expression(Precedence::Assignment);

            if !self.expect_peek(&TokenType::Colon) {
                return Expression::Nil;
            }
            // consume the colon (':') character
            self.next_token();
            let value = self.parse_expression(Precedence::Assignment);
            pairs.push((key, value));

            if !self.peek_token_is(&TokenType::RightBrace) && !self.expect_peek(&TokenType::Comma) {
                return Expression::Nil;
            }
        }
        // Consume the end brace '}'
        if !self.expect_peek(&TokenType::RightBrace) {
            return Expression::Nil;
        }
        Expression::Hash(HashLiteral { token, pairs })
    }

    fn parse_assignment_expression(&mut self, left: Expression) -> Expression {
        // The assignment operator ('=') token
        let token = self.current.clone();

        // precedence of the operator
        let precedence = self.curr_precedence();
        // advance to the next token
        self.next_token();
        let right = self.parse_expression(precedence);

        Expression::Assign(AssignExpr {
            token,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    fn peek_access_type(&self) -> AccessType {
        if self.peek_token_is(&TokenType::Assign) {
            AccessType::Set
        } else {
            AccessType::Get
        }
    }
}
