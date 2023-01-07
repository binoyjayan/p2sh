use lazy_static::lazy_static;

use super::*;

type PrefixParserFn = fn(&mut Parser) -> Expression;
type InfixParserFn = fn(&mut Parser, Expression) -> Expression;

#[derive(Copy, Clone, Default)]
pub struct ParseRule {
    pub prefix: Option<PrefixParserFn>,
    pub infix: Option<InfixParserFn>,
}

impl ParseRule {
    pub fn new(prefix: Option<PrefixParserFn>, infix: Option<InfixParserFn>) -> Self {
        Self { infix, prefix }
    }
}

lazy_static! {
    pub static ref PARSE_RULES: Vec<ParseRule> = {
        let mut rules = vec![ParseRule::default(); TokenType::NumberOfTokens as usize];
        rules[TokenType::Identifier as usize] =
            ParseRule::new(Some(Parser::parse_identifier), None);
        rules[TokenType::Number as usize] = ParseRule::new(Some(Parser::parse_number), None);
        rules[TokenType::Bang as usize] =
            ParseRule::new(Some(Parser::parse_prefix_expression), None);
        rules[TokenType::Minus as usize] =
            ParseRule::new(Some(Parser::parse_prefix_expression), None);
        rules
    };
}

impl Parser {
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
}
