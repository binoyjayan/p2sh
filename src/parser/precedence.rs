// Precedence levels in order from lowest to highest

use std::convert::From;

#[derive(PartialEq, Eq, PartialOrd, Copy, Clone, Default, Debug)]
pub enum Precedence {
    #[default]
    Lowest = 0,
    Assignment, // =
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
            2 => Precedence::LogicalOr,
            3 => Precedence::LogicalAnd,
            4 => Precedence::Relational,
            5 => Precedence::BitwiseOr,
            6 => Precedence::BitwiseXor,
            7 => Precedence::BitwiseAnd,
            8 => Precedence::Shift,
            9 => Precedence::Term,
            10 => Precedence::Factor,
            11 => Precedence::Unary,
            12 => Precedence::Call,
            13 => Precedence::Primary,
            _ => panic!("Cannot convert {} into Precedence", v),
        }
    }
}
