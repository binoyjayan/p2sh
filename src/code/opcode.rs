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
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    Array,
    Map,
    GetIndex,
    SetIndex,
    Call,
    ReturnValue,
    Return,
    DefineLocal,
    GetLocal,
    SetLocal,
    GetBuiltinFn,
    GetBuiltinVar,
    Closure,
    GetFree,
    SetFree,
    CurrClosure,
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
            16 => Opcode::DefineGlobal,
            17 => Opcode::GetGlobal,
            18 => Opcode::SetGlobal,
            19 => Opcode::Array,
            20 => Opcode::Map,
            21 => Opcode::GetIndex,
            22 => Opcode::SetIndex,
            23 => Opcode::Call,
            24 => Opcode::ReturnValue,
            25 => Opcode::Return,
            26 => Opcode::DefineLocal,
            27 => Opcode::GetLocal,
            28 => Opcode::SetLocal,
            29 => Opcode::GetBuiltinFn,
            30 => Opcode::GetBuiltinVar,
            31 => Opcode::Closure,
            32 => Opcode::GetFree,
            33 => Opcode::SetFree,
            34 => Opcode::CurrClosure,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
