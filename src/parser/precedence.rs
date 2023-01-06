// Precedence levels in order from lowest to highest

use std::convert::From;

#[derive(PartialEq, Eq, PartialOrd, Copy, Clone)]
pub enum Precedence {
    Lowest = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! - (Prefix)
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub fn next(self) -> Self {
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
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => panic!("Cannot convert {} into Precedence", v),
        }
    }
}
