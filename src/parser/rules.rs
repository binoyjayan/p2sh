use super::*;
use crate::code::prop::PacketPropType;
use lazy_static::lazy_static;
use std::collections::HashMap;

type PrefixParserFn = fn(&mut Parser, bool) -> Expression;
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
        rules[TokenType::Null as usize] =
            ParseRule::new(Some(Parser::parse_null), None, Precedence::Lowest);
        rules[TokenType::Underscore as usize] =
            ParseRule::new(Some(Parser::parse_underscore), None, Precedence::Lowest);
        rules[TokenType::Identifier as usize] =
            ParseRule::new(Some(Parser::parse_identifier), None, Precedence::Lowest);
        rules[TokenType::Decimal as usize] =
            ParseRule::new(Some(Parser::parse_decimal), None, Precedence::Lowest);
        rules[TokenType::Octal as usize] =
            ParseRule::new(Some(Parser::parse_octal), None, Precedence::Lowest);
        rules[TokenType::Hexadecimal as usize] =
            ParseRule::new(Some(Parser::parse_hexadecimal), None, Precedence::Lowest);
        rules[TokenType::Binary as usize] =
            ParseRule::new(Some(Parser::parse_binary), None, Precedence::Lowest);
        rules[TokenType::Float as usize] =
            ParseRule::new(Some(Parser::parse_float), None, Precedence::Lowest);
        rules[TokenType::Str as usize] =
            ParseRule::new(Some(Parser::parse_string), None, Precedence::Lowest);
        rules[TokenType::Char as usize] =
            ParseRule::new(Some(Parser::parse_char), None, Precedence::Lowest);
        rules[TokenType::Byte as usize] =
            ParseRule::new(Some(Parser::parse_byte), None, Precedence::Lowest);
        rules[TokenType::Stdin as usize] =
            ParseRule::new(Some(Parser::parse_builtin_id), None, Precedence::Lowest);
        rules[TokenType::Stdout as usize] =
            ParseRule::new(Some(Parser::parse_builtin_id), None, Precedence::Lowest);
        rules[TokenType::Stderr as usize] =
            ParseRule::new(Some(Parser::parse_builtin_id), None, Precedence::Lowest);
        // Unary - Logical '!'
        rules[TokenType::Bang as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            None,
            Precedence::Lowest,
        );
        // Unary - Bitwise '~'
        rules[TokenType::BitwiseNot as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            None,
            Precedence::Lowest,
        );
        // Unary - Dollar '$'
        rules[TokenType::Dollar as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            None,
            Precedence::Primary,
        );
        // Binary - Relational
        rules[TokenType::Equal as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        rules[TokenType::BangEqual as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        rules[TokenType::Less as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        rules[TokenType::LessEqual as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        rules[TokenType::Greater as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        rules[TokenType::GreaterEqual as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Relational,
        );
        // Binary - Arithmetic
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
        rules[TokenType::Modulo as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Factor,
        );
        // Binary - Bitwise
        rules[TokenType::BitwiseAnd as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::BitwiseAnd,
        );
        rules[TokenType::BitwiseXor as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::BitwiseXor,
        );
        rules[TokenType::BitwiseOr as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::BitwiseOr,
        );
        // Binary - Shift
        rules[TokenType::LeftShift as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Shift,
        );
        rules[TokenType::RightShift as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::Shift,
        );
        // Binary - Logical
        rules[TokenType::LogicalAnd as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::LogicalAnd,
        );
        rules[TokenType::LogicalOr as usize] = ParseRule::new(
            None,
            Some(Parser::parse_infix_expression),
            Precedence::LogicalOr,
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
        rules[TokenType::Match as usize] =
            ParseRule::new(Some(Parser::parse_match_expr), None, Precedence::Lowest);
        // Function
        rules[TokenType::Function as usize] =
            ParseRule::new(Some(Parser::parse_function_expression), None, Precedence::Lowest);
        // Array literal (prefix) and index operator (infix) parser
        rules[TokenType::LeftBracket as usize] =
            ParseRule::new(Some(Parser::parse_array_literal), Some(Parser::parse_index_expression), Precedence::Call);
        rules[TokenType::Map as usize] =
            ParseRule::new(Some(Parser::parse_hash_literal), None, Precedence::Lowest);
        // Assignment
        rules[TokenType::Assign as usize] = ParseRule::new_with_assoc(
            None,
            Some(Parser::parse_assignment_expression),
            Precedence::Assignment,
            Associativity::Right
        );
        // Range operators
        rules[TokenType::RangeEx as usize] = ParseRule::new_with_assoc(
            None,
            Some(Parser::parse_range_expression),
            Precedence::Range,
            Associativity::Right
        );
        rules[TokenType::RangeInc as usize] = ParseRule::new_with_assoc(
            None,
            Some(Parser::parse_range_expression),
            Precedence::Range,
            Associativity::Right
        );
        // Dot expressions
        rules[TokenType::Dot as usize] =
            ParseRule::new(None, Some(Parser::parse_dot_expression), Precedence::Call);
        rules
    };
}

// Packet property definitions
lazy_static! {
    static ref PACKET_PROP_MAP: HashMap<String, PacketPropType> = {
        let mut map = HashMap::new();
        map.insert("magic".to_string(), PacketPropType::Magic);
        map.insert("major".to_string(), PacketPropType::Major);
        map.insert("minor".to_string(), PacketPropType::Minor);
        map.insert("thiszone".to_string(), PacketPropType::ThisZone);
        map.insert("sigflags".to_string(), PacketPropType::SigFigs);
        map.insert("snaplen".to_string(), PacketPropType::Snaplen);
        map.insert("linktype".to_string(), PacketPropType::LinkType);
        map.insert("sec".to_string(), PacketPropType::Sec);
        map.insert("usec".to_string(), PacketPropType::USec);
        map.insert("nsec".to_string(), PacketPropType::USec);
        map.insert("caplen".to_string(), PacketPropType::Caplen);
        map.insert("wirelen".to_string(), PacketPropType::Wirelen);
        map.insert("payload".to_string(), PacketPropType::Payload);
        map.insert("eth".to_string(), PacketPropType::Eth);
        map.insert("src".to_string(), PacketPropType::Src);
        map.insert("dst".to_string(), PacketPropType::Dst);
        map.insert("type".to_string(), PacketPropType::EtherType);
        map.insert("vlan".to_string(), PacketPropType::Vlan);
        map.insert("id".to_string(), PacketPropType::Id);
        map.insert("priority".to_string(), PacketPropType::Priority);
        map.insert("dei".to_string(), PacketPropType::Dei);
        map
    };
}

impl Parser {
    // Return the precedence of the current token. If the current token is a
    // bitwise OR operator and the parser is currently parsing a match pattern,
    // return the precedence of the match pattern OR operator. Otherwise, return
    // the precedence of the current token.
    pub fn curr_precedence(&self) -> Precedence {
        if self.in_match_pattern && self.current.ttype == TokenType::BitwiseOr {
            Precedence::MatchOr
        } else {
            PARSE_RULES[self.current.ttype as usize].precedence
        }
    }
    pub fn curr_prefix(&self) -> Option<PrefixParserFn> {
        PARSE_RULES[self.current.ttype as usize].prefix
    }
    pub fn peek_infix(&self) -> Option<InfixParserFn> {
        PARSE_RULES[self.peek_next.ttype as usize].infix
    }
    pub fn peek_precedence(&self) -> Precedence {
        if self.in_match_pattern && self.peek_next.ttype == TokenType::BitwiseOr {
            Precedence::MatchOr
        } else {
            PARSE_RULES[self.peek_next.ttype as usize].precedence
        }
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

    fn parse_null(&mut self, _: bool) -> Expression {
        Expression::Null(NullLiteral {
            token: self.current.clone(),
        })
    }

    fn parse_underscore(&mut self, _: bool) -> Expression {
        Expression::Score(Underscore {
            token: self.current.clone(),
            value: self.current.literal.clone(),
        })
    }

    /// Parse an identifier or a property of a packet object,
    /// If it is packet property, then validate the property name.
    fn parse_identifier(&mut self, property: bool) -> Expression {
        let access = self.peek_access_type();
        let value = self.current.literal.clone();
        if property {
            if let Some(ptype) = PACKET_PROP_MAP.get(&value) {
                Expression::Prop(PktPropExpr {
                    token: self.current.clone(),
                    value: *ptype,
                    context: ParseContext { access },
                })
            } else {
                let msg = format!("invalid property '{}'", self.current.literal);
                self.push_error(&msg);
                Expression::Invalid
            }
        } else {
            Expression::Ident(Identifier {
                token: self.current.clone(),
                value,
                context: ParseContext { access },
            })
        }
    }

    fn parse_builtin_id(&mut self, _: bool) -> Expression {
        let access = self.peek_access_type();
        Expression::Builtin(BuiltinID {
            token: self.current.clone(),
            value: self.current.literal.clone(),
            context: ParseContext { access },
        })
    }

    fn parse_decimal(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Integer(IntegerLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse '{}' as an integer", self.current.literal);
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_octal(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        let str_value = &self.current.literal[2..];
        if let Ok(value) = i64::from_str_radix(str_value, 8) {
            Expression::Integer(IntegerLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!(
                "could not parse '{}' as an octal integer",
                self.current.literal
            );
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_hexadecimal(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        let str_value = &self.current.literal[2..];
        if let Ok(value) = i64::from_str_radix(str_value, 16) {
            Expression::Integer(IntegerLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!(
                "could not parse '{}' as a hexadecimal integer",
                self.current.literal
            );
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_binary(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        let str_value = &self.current.literal[2..];
        if let Ok(value) = i64::from_str_radix(str_value, 2) {
            Expression::Integer(IntegerLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!(
                "could not parse '{}' as a binary integer",
                self.current.literal
            );
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_float(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Float(FloatLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a float", self.current.literal);
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_string(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Str(StringLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a string", self.current.literal);
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_char(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        if let Ok(value) = self.current.literal.parse() {
            Expression::Char(CharLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a char", self.current.literal);
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    fn parse_byte(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        if let Some(value) = self.current.literal.bytes().next() {
            Expression::Byte(ByteLiteral {
                token: self.current.clone(),
                value,
            })
        } else {
            let msg = format!("could not parse {} as a byte", self.current.literal);
            self.push_error(&msg);
            Expression::Invalid
        }
    }

    // Parse unary expressions such as '-' and '!'
    fn parse_prefix_expression(&mut self, _: bool) -> Expression {
        let operator = self.current.literal.clone();
        let token = self.current.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Unary, false);

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

        let right = self.parse_expression(precedence, false);
        Expression::Binary(BinaryExpr {
            token,
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_boolean(&mut self, _: bool) -> Expression {
        self.peek_invalid_assignment(false);
        Expression::Bool(BooleanExpr {
            token: self.current.clone(),
            value: self.curr_token_is(&TokenType::True),
        })
    }

    // Override operator precedence using grouped expression
    fn parse_grouped(&mut self, _: bool) -> Expression {
        self.next_token();
        let expr = self.parse_expression(Precedence::Assignment, false);
        if self.expect_peek(&TokenType::RightParen) {
            // check for cases such as '(a) = b'
            self.peek_invalid_assignment(false);
            expr
        } else {
            Expression::Invalid
        }
    }

    fn parse_if_expr(&mut self, _: bool) -> Expression {
        let token = self.current.clone();
        // advance to the condition expression
        self.next_token();
        let condition = self.parse_expression(Precedence::Assignment, false);
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Expression::Invalid;
        }
        let then_stmt = self.parse_block_statement();
        // Check if an else branch exists
        let else_stmt = if self.peek_token_is(&TokenType::Else) {
            self.next_token();
            if self.peek_token_is(&TokenType::If) {
                self.next_token();
                ElseIfExpr::ElseIf(Box::new(self.parse_if_expr(false)))
            } else if self.peek_token_is(&TokenType::LeftBrace) {
                self.next_token();
                ElseIfExpr::Else(self.parse_block_statement())
            } else {
                return Expression::Invalid;
            }
        } else {
            ElseIfExpr::Empty
        };

        match else_stmt {
            ElseIfExpr::ElseIf(else_if) => Expression::If(IfExpr {
                token,
                condition: Box::new(condition),
                then_stmt,
                else_if: ElseIfExpr::ElseIf(Box::new(*else_if)),
            }),
            ElseIfExpr::Else(else_stmt) => Expression::If(IfExpr {
                token,
                condition: Box::new(condition),
                then_stmt,
                else_if: ElseIfExpr::Else(else_stmt),
            }),
            ElseIfExpr::Empty => Expression::If(IfExpr {
                token,
                condition: Box::new(condition),
                then_stmt,
                else_if: ElseIfExpr::Empty,
            }),
        }
    }

    fn parse_match_expr(&mut self, _: bool) -> Expression {
        // The match token
        let token = self.current.clone();
        // advance to the condition expression
        self.next_token();
        let condition = self.parse_expression(Precedence::Assignment, false);
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Expression::Invalid;
        }
        let mut arms = Vec::new();
        let mut def_arm = false;
        while !self.peek_token_is(&TokenType::RightBrace) && !self.peek_token_is(&TokenType::Eof) {
            self.next_token();

            // Save the line number of the match arm in case there are errors
            // on the pattern itself and parser would synchronize out of the
            // match expression.
            let line = self.scanner.get_line();
            match self.parse_match_pattern() {
                Ok(patterns) => {
                    if !self.expect_peek(&TokenType::MatchArm) {
                        return Expression::Invalid;
                    }
                    let arm_token = self.current.clone();
                    // body of the match arm can be either a block statement or an expression
                    // advance to the '{' token or to the expression
                    self.next_token();
                    let body = if self.curr_token_is(&TokenType::LeftBrace) {
                        self.parse_block_statement()
                    } else {
                        let token = self.current.clone();
                        let expr = self.parse_expression(Precedence::Assignment, false);
                        // Create a block statement from expression statement
                        // Make a dummy token for the block statement
                        BlockStatement {
                            token: Token::new(TokenType::LeftBrace, "{", line),
                            statements: vec![Statement::Expr(ExpressionStmt {
                                token,
                                value: expr,
                                is_assign: false,
                            })],
                        }
                    };

                    // If there is a comma (,) consume it and continue to the next arm
                    if self.peek_token_is(&TokenType::Comma) {
                        self.next_token();
                    }

                    let arm = MatchArm {
                        token: arm_token,
                        patterns,
                        body,
                    };
                    // Look for duplicate default (_) patterns
                    if arm.is_default() {
                        if def_arm {
                            self.push_error_at("multiple default arms in match expression", line);
                            return Expression::Invalid;
                        }
                        def_arm = true;
                    }
                    arms.push(arm);
                }
                Err(msg) => {
                    self.push_error_at(&msg, line);
                    return Expression::Invalid;
                }
            }
        }
        if !self.expect_peek(&TokenType::RightBrace) {
            return Expression::Invalid;
        }

        if def_arm {
            // If there is a default (_) pattern, it must be the last arm
            if !arms[arms.len() - 1].is_default() {
                self.push_error_at("unreachable pattern", arms[arms.len() - 1].token.line);
                return Expression::Invalid;
            }
        } else {
            // If there is no default (_) pattern, add one at the end. The body
            // of the default (_) pattern is a null expression statement.
            let default_arm = MatchArm {
                token: Token::new(TokenType::Underscore, "_", token.line),
                patterns: vec![MatchPattern::Default(Underscore {
                    token: Token::new(TokenType::Underscore, "_", token.line),
                    value: "_".to_string(),
                })],
                body: BlockStatement {
                    token: Token::new(TokenType::LeftBrace, "{", token.line),
                    statements: [Statement::Expr(ExpressionStmt {
                        token: Token::new(TokenType::Null, "null", token.line),
                        value: Expression::Null(NullLiteral {
                            token: Token::new(TokenType::Null, "null", token.line),
                        }),
                        is_assign: false,
                    })]
                    .to_vec(),
                },
            };
            arms.push(default_arm);
        }
        let match_expr = MatchExpr {
            token,
            expr: Box::new(condition),
            arms,
        };
        Expression::Match(match_expr)
    }

    // Parse match patterns that includes strings, integers and ranges
    // separated by the '|' operator. It is different from the bitwise OR
    // operator. The patterns are parsed as a vector of MatchPatternVariant's.
    // The operands of bitwise expressions, are parsed recursively and are
    // converted to vector of MatchPatternVariant's. Also set the flag
    // 'in_match_pattern' to true to indicate that the parser is currently
    // parsing a match pattern. This helps use different precedence for
    // involving bitwise OR and match pattern OR tokens.
    fn parse_match_pattern(&mut self) -> Result<Vec<MatchPattern>, String> {
        self.in_match_pattern = true;
        let pattern = self.parse_expression(Precedence::Assignment, false);
        self.in_match_pattern = false;
        let patterns = Self::convert_to_pattern_list(pattern);

        if let Ok(ref patterns) = patterns {
            // Find duplicate default (_) patterns
            let default_count = patterns
                .iter()
                .filter(|&pattern| pattern.is_default())
                .count();

            if default_count > 1 {
                return Err("multiple default patterns in match arm".to_string());
            } else if patterns.len() > 1 && default_count == 1 {
                return Err("default pattern cannot be used with other patterns".to_string());
            }
        }
        patterns
    }

    // Convert the expression to a vector of MatchPatternVariant's by recursively
    // parsing the bitwise OR expressions. These operands are are converted to
    // MatchPatternVariant's and pushed to the vector. The vector is returned.
    fn convert_to_pattern_list(expr: Expression) -> Result<Vec<MatchPattern>, String> {
        let mut patterns = Vec::new();
        let pattern_exp = match expr {
            Expression::Score(expr) => MatchPattern::Default(expr),
            Expression::Bool(expr) => MatchPattern::Boolean(expr),
            Expression::Integer(expr) => MatchPattern::Integer(expr),
            Expression::Char(expr) => MatchPattern::Char(expr),
            Expression::Byte(expr) => MatchPattern::Byte(expr),
            Expression::Str(expr) => MatchPattern::Str(expr),
            Expression::Range(expr) => MatchPattern::Range(expr),
            Expression::Binary(expr) => {
                // convert the bitwise or expression to patterns recursively
                match expr.operator.as_ref() {
                    // bitwise OR operator
                    "|" => {
                        let mut left_patterns = Self::convert_to_pattern_list(*expr.left)?;
                        patterns.append(&mut left_patterns);
                        let mut right_patterns = Self::convert_to_pattern_list(*expr.right)?;
                        patterns.append(&mut right_patterns);
                    }
                    _ => {
                        return Err(format!(
                            "invalid operation in match pattern '{}'",
                            expr.operator
                        ));
                    }
                }
                return Ok(patterns);
            }
            _ => {
                return Err(format!("invalid pattern in match arm '{}'", expr));
            }
        };
        patterns.push(pattern_exp);
        Ok(patterns)
    }

    pub fn parse_block_statement(&mut self) -> BlockStatement {
        let token = self.current.clone();
        let mut statements = Vec::new();
        self.next_token();

        while !self.curr_token_is(&TokenType::RightBrace) && !self.curr_token_is(&TokenType::Eof) {
            if let Ok(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        BlockStatement { token, statements }
    }

    // Function expressions are of the form 'fn(<params>) { <body> }' and differ
    // from function statements. They are anonymous functions that can be assigned
    // to a variable or passed as an argument to another function. Additionally,
    // they can be used to define closures, which sets them apart from function
    // statements. Despite these differences, the underlying implementations are
    // the same.
    fn parse_function_expression(&mut self, _: bool) -> Expression {
        let token = self.current.clone();
        if !self.expect_peek(&TokenType::LeftParen) {
            return Expression::Invalid;
        }
        let params = self.parse_function_params();
        if !self.expect_peek(&TokenType::LeftBrace) {
            return Expression::Invalid;
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

    pub fn parse_function_params(&mut self) -> Vec<Identifier> {
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
            context: ParseContext {
                access: AccessType::Get,
            },
        });

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let token_ident = self.current.clone();
            let ident_value = token_ident.literal.clone();
            identifiers.push(Identifier {
                token: token_ident,
                value: ident_value,
                context: ParseContext {
                    access: AccessType::Get,
                },
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
        args.push(self.parse_expression(Precedence::Assignment, false));
        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Assignment, false));
        }

        if !self.expect_peek(&ttype_end) {
            return Vec::new();
        }
        args
    }

    fn parse_array_literal(&mut self, _: bool) -> Expression {
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
        let index = self.parse_expression(Precedence::Assignment, false);
        if !self.expect_peek(&TokenType::RightBracket) {
            return Expression::Invalid;
        }

        let context = ParseContext {
            access: self.peek_access_type(),
        };

        Expression::Index(IndexExpr {
            token,
            left: Box::new(left),
            index: Box::new(index),
            context,
        })
    }

    // Use the "map" keyword to create a hash map. Using a keyword helps
    // disambiguate between a hash map and a block statement.
    fn parse_hash_literal(&mut self, _: bool) -> Expression {
        let mut pairs = Vec::new();
        // Consume the 'map' keyword
        self.next_token();
        let token = self.current.clone();

        while !self.peek_token_is(&TokenType::RightBrace) {
            // consume the first '{' or a ',' in each iteration
            self.next_token();
            let key: Expression = self.parse_expression(Precedence::Assignment, false);

            if !self.expect_peek(&TokenType::Colon) {
                return Expression::Invalid;
            }
            // consume the colon (':') character
            self.next_token();
            let value = self.parse_expression(Precedence::Assignment, false);
            pairs.push((key, value));

            if !self.peek_token_is(&TokenType::RightBrace) && !self.expect_peek(&TokenType::Comma) {
                return Expression::Invalid;
            }
        }
        // Consume the end brace '}'
        if !self.expect_peek(&TokenType::RightBrace) {
            return Expression::Invalid;
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
        let right = self.parse_expression(precedence, false);

        Expression::Assign(AssignExpr {
            token,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_range_expression(&mut self, left: Expression) -> Expression {
        match self.parse_ranges(left) {
            Ok(expr) => Expression::Range(expr),
            Err(msg) => {
                self.push_error(&msg);
                Expression::Invalid
            }
        }
    }

    fn parse_ranges(&mut self, left: Expression) -> Result<RangeExpr, String> {
        let operator = self.current.literal.clone();
        let token = self.current.clone();
        // precedence of the operator
        let precedence = self.curr_precedence();

        // advance to the next token
        self.next_token();

        let right = self.parse_expression(precedence, false);
        // Check if both left and right operands are integers or identifiers
        let is_valid_range = matches!(
            (&left, &right),
            (&Expression::Integer(_), &Expression::Integer(_))
                | (&Expression::Str(_), &Expression::Str(_))
                | (&Expression::Char(_), &Expression::Char(_))
                | (&Expression::Byte(_), &Expression::Byte(_))
                | (&Expression::Ident(_), &Expression::Ident(_))
        );

        if is_valid_range {
            Ok(RangeExpr {
                token,
                operator,
                begin: Box::new(left),
                end: Box::new(right),
            })
        } else {
            Err(format!("invalid use of range operator '{}'", operator))
        }
    }

    fn parse_dot_expression(&mut self, left: Expression) -> Expression {
        // The dot ('.') token
        let token = self.current.clone();
        // precedence of the operator
        let op_prec = self.curr_precedence();
        // advance to the property token
        self.next_token();
        let access = self.peek_access_type();

        // To avoid expressions such as '0x8100 < eth.type = 1'
        let precedence = match access {
            AccessType::Get => op_prec,
            AccessType::Set => Precedence::Assignment,
        };
        let property = self.parse_expression(precedence, true);

        let context = ParseContext { access };
        Expression::Dot(DotExpr {
            token,
            left: Box::new(left),
            property: Box::new(property),
            context,
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
