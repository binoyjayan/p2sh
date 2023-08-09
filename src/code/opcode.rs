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
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
