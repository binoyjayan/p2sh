use super::*;
use lazy_static::lazy_static;

type PrefixParserFn = fn(&mut Parser) -> Expression;
type InfixParserFn = fn(&mut Parser, Expression) -> Expression;

#[derive(Copy, Clone, Default)]
pub struct ParseRule {
    pub prefix: Option<PrefixParserFn>,
    pub infix: Option<InfixParserFn>,
    pub precedence: Precedence,
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
        }
    }
}

lazy_static! {
    pub static ref PARSE_RULES: Vec<ParseRule> = {
        let mut rules = vec![ParseRule::default(); TokenType::NumberOfTokens as usize];
        // Terminal expressions
        rules[TokenType::Identifier as usize] =
            ParseRule::new(Some(Parser::parse_identifier), None, Precedence::Lowest);
        rules[TokenType::Number as usize] =
            ParseRule::new(Some(Parser::parse_number), None, Precedence::Lowest);
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
        // Function
        rules[TokenType::LeftBracket as usize] =
            ParseRule::new(Some(Parser::parse_array_literal), None, Precedence::Lowest);
        rules
    };
}

impl Parser {
    pub fn peek_precedence(&self) -> Precedence {
        PARSE_RULES[self.peek_next.ttype as usize].precedence
    }
    pub fn curr_precedence(&self) -> Precedence {
        PARSE_RULES[self.current.ttype as usize].precedence
    }

    fn parse_identifier(&mut self) -> Expression {
        Expression::Ident(Identifier {
            token: self.current.clone(),
            value: self.current.literal.clone(),
        })
    }

    fn parse_number(&mut self) -> Expression {
        if let Ok(value) = self.current.literal.parse() {
            Expression::Number(NumberLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a number", self.current.literal);
            self.push_error(&msg);
            Expression::Nil
        }
    }

    fn parse_string(&mut self) -> Expression {
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
        Expression::Bool(BooleanExpr {
            token: self.current.clone(),
            value: self.curr_token_is(&TokenType::True),
        })
    }

    // Override operator precedence using grouped expression
    fn parse_grouped(&mut self) -> Expression {
        self.next_token();
        let expr = self.parse_expression(Precedence::Lowest);
        if self.expect_peek(&TokenType::RightParen) {
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
        let condition = self.parse_expression(Precedence::Lowest);
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
        Expression::Function(FunctionLiteral {
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
        });

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let token_ident = self.current.clone();
            let ident_value = token_ident.literal.clone();
            identifiers.push(Identifier {
                token: token_ident,
                value: ident_value,
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
        args.push(self.parse_expression(Precedence::Lowest));
        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest));
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
}
