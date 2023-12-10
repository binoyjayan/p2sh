use super::stmt::*;
use crate::scanner::token::*;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Expression {
    Null(NullLiteral),
    Score(Underscore),
    Ident(Identifier),
    Builtin(BuiltinID),
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Str(StringLiteral),
    Char(CharLiteral),
    Byte(ByteLiteral),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Bool(BooleanExpr),
    If(IfExpr),
    Match(MatchExpr),
    Function(FunctionLiteral),
    Call(CallExpr),
    Array(ArrayLiteral),
    Hash(HashLiteral),
    Index(IndexExpr),
    Assign(AssignExpr),
    Range(RangeExpr),
    Invalid,
}

// Type of access to a symbol or expresion
#[derive(Clone, Debug)]
pub enum AccessType {
    Get,
    Set,
}

#[derive(Clone, Debug)]
pub struct Underscore {
    pub token: Token,
    pub value: String,
}

impl fmt::Display for Underscore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
    pub access: AccessType,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct BuiltinID {
    pub token: Token,
    pub value: String,
    pub access: AccessType,
}

impl fmt::Display for BuiltinID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "builtin<{}>", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct CharLiteral {
    pub token: Token,
    pub value: char,
}

impl fmt::Display for CharLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct ByteLiteral {
    pub token: Token,
    pub value: u8,
}

impl fmt::Display for ByteLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl fmt::Display for IntegerLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct FloatLiteral {
    pub token: Token,
    pub value: f64,
}

impl fmt::Display for FloatLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct NullLiteral {
    pub token: Token,
}

impl fmt::Display for NullLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub token: Token, //operator token
    pub operator: String,
    pub right: Box<Expression>,
}

impl fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}{})", self.token, self.right)
    }
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub token: Token, //operator token
    pub operator: String,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.left, self.token, self.right)
    }
}

#[derive(Clone, Debug)]
pub struct BooleanExpr {
    pub token: Token,
    pub value: bool,
}

impl fmt::Display for BooleanExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token.literal)
    }
}

// 'ElseIfExpr' expression can be 'Empty' when there is no else component
// in an if expression. It can be 'Else' when there is an else component
// in an if expression. It can be 'ElseIf' when there is an 'else if'
// component in an if expression.
#[derive(Clone, Debug)]
pub enum ElseIfExpr {
    Empty,
    Else(BlockStatement),
    ElseIf(Box<Expression>),
}

impl fmt::Display for ElseIfExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ElseIfExpr::Empty => write!(f, ""),
            ElseIfExpr::Else(stmt) => write!(f, " else {{ {} }}", stmt),
            ElseIfExpr::ElseIf(else_if) => write!(f, " else {}", else_if),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub token: Token, // if token
    pub condition: Box<Expression>,
    pub then_stmt: BlockStatement,
    pub else_if: ElseIfExpr,
}

impl fmt::Display for IfExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {{ {} }}", self.condition, self.then_stmt)?;
        write!(f, " else {{ {} }}", self.else_if)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum MatchPattern {
    Integer(IntegerLiteral),
    Char(CharLiteral),
    Byte(ByteLiteral),
    Str(StringLiteral),
    Range(RangeExpr),
    Default(Underscore),
}

impl fmt::Display for MatchPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            MatchPattern::Integer(num) => write!(f, "{}", num),
            MatchPattern::Str(s) => write!(f, "{}", s),
            MatchPattern::Char(c) => write!(f, "{}", c),
            MatchPattern::Byte(b) => write!(f, "{}", b),
            MatchPattern::Range(r) => write!(f, "{}", r),
            MatchPattern::Default(u) => write!(f, "{}", u),
        }
    }
}

impl MatchPattern {
    pub fn is_default(&self) -> bool {
        matches!(self, MatchPattern::Default(_))
    }

    // Compare two patterns of match expression and return true if both
    // patterns have the same type. Otherswise, return false.
    pub fn matches_type(&self, other: &MatchPattern) -> bool {
        if self.is_default() || other.is_default() {
            return true;
        }

        match (self, other) {
            (MatchPattern::Integer(_), MatchPattern::Integer(_))
            | (MatchPattern::Str(_), MatchPattern::Str(_))
            | (MatchPattern::Char(_), MatchPattern::Char(_))
            | (MatchPattern::Byte(_), MatchPattern::Byte(_)) => true,
            (MatchPattern::Range(r1), MatchPattern::Range(r2)) => {
                matches!(
                    (&*r1.begin, &*r1.end, &*r2.begin, &*r2.end),
                    (
                        Expression::Integer(_),
                        Expression::Integer(_),
                        Expression::Integer(_),
                        Expression::Integer(_),
                    ) | (
                        Expression::Str(_),
                        Expression::Str(_),
                        Expression::Str(_),
                        Expression::Str(_),
                    ) | (
                        Expression::Char(_),
                        Expression::Char(_),
                        Expression::Char(_),
                        Expression::Char(_),
                    ) | (
                        Expression::Byte(_),
                        Expression::Byte(_),
                        Expression::Byte(_),
                        Expression::Byte(_),
                    )
                )
            }
            (MatchPattern::Default(_), _) | (_, MatchPattern::Default(_)) => true,
            (MatchPattern::Range(r), o) | (o, MatchPattern::Range(r)) => {
                matches!(
                    (&*r.begin, &*r.end, o),
                    (
                        Expression::Integer(_),
                        Expression::Integer(_),
                        MatchPattern::Integer(_)
                    ) | (Expression::Str(_), Expression::Str(_), MatchPattern::Str(_))
                        | (
                            Expression::Char(_),
                            Expression::Char(_),
                            MatchPattern::Char(_)
                        )
                        | (
                            Expression::Byte(_),
                            Expression::Byte(_),
                            MatchPattern::Byte(_)
                        )
                )
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub token: Token, // token '=>'
    pub patterns: Vec<MatchPattern>,
    pub body: BlockStatement,
}

impl fmt::Display for MatchArm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Display patterns separated by ' | '
        let pat_str = self
            .patterns
            .iter()
            .map(|p| format!("{} | ", p))
            .collect::<String>();
        let pat_str = pat_str.trim_end_matches(|c| c == ' ' || c == ',');
        let body = format!("{}", self.body);
        write!(f, " {} => {{ {} }}", pat_str, body.trim())?;
        Ok(())
    }
}

impl MatchArm {
    pub fn is_default(&self) -> bool {
        self.patterns.len() == 1 && self.patterns[0].is_default()
    }
}

#[derive(Clone, Debug)]
pub struct MatchExpr {
    pub token: Token, // match token
    pub expr: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

impl fmt::Display for MatchExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "match {} {{", self.expr)?;
        for arm in self.arms.iter() {
            write!(f, "{}", arm)?;
        }
        write!(f, "}}")
    }
}

#[derive(Clone, Debug)]
pub struct FunctionLiteral {
    pub name: String, // name of the function
    pub token: Token,
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
}

impl fmt::Display for FunctionLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Display parameters separated by ', '
        let params_str = self
            .params
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let params_str = params_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{} ({}) {}", self.token, params_str, self.body)
    }
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub token: Token,          // The '(' Token
    pub func: Box<Expression>, // Identifier or FunctionLiteral
    pub args: Vec<Expression>,
}

impl fmt::Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args_str = self
            .args
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let args_str = args_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{}({})", self.func, args_str)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    pub token: Token, // [
    pub elements: Vec<Expression>,
}

impl fmt::Display for ArrayLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elements_str = self
            .elements
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let elements_str = elements_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "[{}]", elements_str)
    }
}

#[derive(Clone, Debug)]
pub struct HashLiteral {
    pub token: Token, // map token
    // A HashMap is not required here since this is a literal
    // that exists only as an initializer. During evaluation, the
    // literal will be converted into a Object of type HashMap.
    pub pairs: Vec<(Expression, Expression)>,
}

impl fmt::Display for HashLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pairs_str = self
            .pairs
            .iter()
            .map(|p| format!("{}: {}, ", p.0, p.1))
            .collect::<String>();
        let pairs_str = pairs_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{{{}}}", pairs_str)
    }
}

// Index expression looks like '<expr>[<expr>]'
#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub token: Token, // [
    pub left: Box<Expression>,
    pub index: Box<Expression>,
    pub access: AccessType,
}

impl fmt::Display for IndexExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}[{}])", self.left, self.index)
    }
}

#[derive(Clone, Debug)]
pub struct AssignExpr {
    pub token: Token, //operator token
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl fmt::Display for AssignExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.left, self.token, self.right)
    }
}

#[derive(Clone, Debug)]
pub struct RangeExpr {
    pub token: Token, //operator token
    pub operator: String,
    pub begin: Box<Expression>,
    pub end: Box<Expression>,
}

impl fmt::Display for RangeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}{}{})", self.begin, self.token, self.end)
    }
}

impl Expression {
    #[allow(dead_code)]
    fn token_literal(&self) -> String {
        match &self {
            Expression::Score(u) => u.token.literal.clone(),
            Expression::Ident(ident) => ident.token.literal.clone(),
            Expression::Builtin(bid) => bid.token.literal.clone(),
            Expression::Integer(num) => num.token.literal.clone(),
            Expression::Float(num) => num.token.literal.clone(),
            Expression::Str(s) => s.token.literal.clone(),
            Expression::Char(c) => c.token.literal.clone(),
            Expression::Byte(b) => b.token.literal.clone(),
            Expression::Unary(unary) => unary.token.literal.clone(),
            Expression::Binary(binary) => binary.token.literal.clone(),
            Expression::Bool(b) => b.token.literal.clone(),
            Expression::If(i) => i.token.literal.clone(),
            Expression::Match(m) => m.token.literal.clone(),
            Expression::Function(f) => f.token.literal.clone(),
            Expression::Call(c) => c.token.literal.clone(),
            Expression::Array(s) => s.token.literal.clone(),
            Expression::Hash(h) => h.token.literal.clone(),
            Expression::Index(idx) => idx.token.literal.clone(),
            Expression::Assign(asn) => asn.token.literal.clone(),
            Expression::Range(r) => r.token.literal.clone(),
            Expression::Null(null) => null.token.literal.clone(),
            Expression::Invalid => "invalid".to_string(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expression::Score(u) => write!(f, "{}", u),
            Expression::Ident(ident) => write!(f, "{}", ident),
            Expression::Builtin(bid) => write!(f, "{}", bid),
            Expression::Integer(num) => write!(f, "{}", num),
            Expression::Float(num) => write!(f, "{}", num),
            Expression::Str(s) => write!(f, "{}", s),
            Expression::Char(c) => write!(f, "{}", c),
            Expression::Byte(b) => write!(f, "{}", b),
            Expression::Unary(prefix) => write!(f, "{}", prefix),
            Expression::Binary(binary) => write!(f, "{}", binary),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::If(i) => write!(f, "{}", i),
            Expression::Match(m) => write!(f, "{}", m),
            Expression::Function(fun) => write!(f, "{}", fun),
            Expression::Call(c) => write!(f, "{}", c),
            Expression::Array(s) => write!(f, "{}", s),
            Expression::Hash(h) => write!(f, "{}", h),
            Expression::Index(idx) => write!(f, "{}", idx),
            Expression::Assign(asn) => write!(f, "{}", asn),
            Expression::Range(r) => write!(f, "{}", r),
            Expression::Null(null) => write!(f, "{}", null),
            Expression::Invalid => write!(f, "INVALID EXPRESSION"),
        }
    }
}
