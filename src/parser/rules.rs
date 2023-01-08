use lazy_static::lazy_static;

use super::*;

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
        rules[TokenType::Identifier as usize] =
            ParseRule::new(Some(Parser::parse_identifier), None, Precedence::Lowest);
        rules[TokenType::Number as usize] =
            ParseRule::new(Some(Parser::parse_number), None, Precedence::Lowest);
        rules[TokenType::Bang as usize] = ParseRule::new(
            Some(Parser::parse_prefix_expression),
            None,
            Precedence::Lowest,
        );
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
            self.errors.push(msg);
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
}
