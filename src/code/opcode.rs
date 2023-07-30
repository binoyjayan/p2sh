#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub enum Opcode {
    Constant,
    Pop,
    Add,
    Invalid,
}

impl From<u8> for Opcode {
    fn from(code: u8) -> Self {
        match code {
            0 => Opcode::Constant,
            1 => Opcode::Pop,
            2 => Opcode::Add,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
