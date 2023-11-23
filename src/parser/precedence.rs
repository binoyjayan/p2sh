// Precedence levels in order from lowest to highest

use std::convert::From;

#[derive(PartialEq, Eq, PartialOrd, Copy, Clone, Default, Debug)]
pub enum Precedence {
    #[default]
    Lowest = 0,
    Assignment, // =
    MatchOr,    // | (in match pattern)
    Range,      // .. ..=
    LogicalOr,  // ||
    LogicalAnd, // &&
    Relational, // == != < > <= >=
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseAnd, // &
    Shift,      // << >>
    Term,       // + -
    Factor,     // * / %
    Unary,      // ! - ~ (Prefix)
    Call,       // [] . ()
    Primary,
}

impl Precedence {
    pub fn _next(self) -> Self {
        if self == Self::Primary {
            panic!("Precedence::Primary does not have a next()")
        }
        let curr = self as usize;
        (curr + 1).into()
    }
    pub fn _prev(self) -> Self {
        if self == Self::Lowest {
            panic!("Precedence::None does not have a prev()")
        }
        let curr = self as usize;
        (curr - 1).into()
    }
}

impl From<usize> for Precedence {
    fn from(v: usize) -> Self {
        match v {
            0 => Precedence::Lowest,
            1 => Precedence::Assignment,
            2 => Precedence::MatchOr,
            3 => Precedence::Range,
            4 => Precedence::LogicalOr,
            5 => Precedence::LogicalAnd,
            6 => Precedence::Relational,
            7 => Precedence::BitwiseOr,
            8 => Precedence::BitwiseXor,
            9 => Precedence::BitwiseAnd,
            10 => Precedence::Shift,
            11 => Precedence::Term,
            12 => Precedence::Factor,
            13 => Precedence::Unary,
            14 => Precedence::Call,
            15 => Precedence::Primary,
            _ => panic!("Cannot convert {} into Precedence", v),
        }
    }
}
