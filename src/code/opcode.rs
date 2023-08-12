#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub enum Opcode {
    Constant,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    True,
    False,
    Equal,
    NotEqual,
    Greater,
    Minus,
    Bang,
    Invalid,
}

impl From<u8> for Opcode {
    fn from(code: u8) -> Self {
        match code {
            0 => Opcode::Constant,
            1 => Opcode::Pop,
            2 => Opcode::Add,
            3 => Opcode::Sub,
            4 => Opcode::Mul,
            5 => Opcode::Div,
            6 => Opcode::True,
            7 => Opcode::False,
            8 => Opcode::Equal,
            9 => Opcode::NotEqual,
            10 => Opcode::Greater,
            11 => Opcode::Minus,
            12 => Opcode::Bang,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
