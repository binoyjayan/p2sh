#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]

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
    Jump,
    JumpIfFalse,
    Nil,
    GetGlobal,
    SetGlobal,
    Array,
    Map,
    Index,
    Call,
    ReturnValue,
    Return,
    GetLocal,
    SetLocal,
    GetBuiltin,
    Closure,
    GetFree,
    #[default]
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
            13 => Opcode::Jump,
            14 => Opcode::JumpIfFalse,
            15 => Opcode::Nil,
            16 => Opcode::GetGlobal,
            17 => Opcode::SetGlobal,
            18 => Opcode::Array,
            19 => Opcode::Map,
            20 => Opcode::Index,
            21 => Opcode::Call,
            22 => Opcode::ReturnValue,
            23 => Opcode::Return,
            24 => Opcode::GetLocal,
            25 => Opcode::SetLocal,
            26 => Opcode::GetBuiltin,
            27 => Opcode::Closure,
            28 => Opcode::GetFree,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
