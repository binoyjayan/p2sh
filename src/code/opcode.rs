#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]

pub enum Opcode {
    Constant,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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
            6 => Opcode::Mod,
            7 => Opcode::True,
            8 => Opcode::False,
            9 => Opcode::Equal,
            10 => Opcode::NotEqual,
            11 => Opcode::Greater,
            12 => Opcode::Minus,
            13 => Opcode::Bang,
            14 => Opcode::Jump,
            15 => Opcode::JumpIfFalse,
            16 => Opcode::Nil,
            17 => Opcode::DefineGlobal,
            18 => Opcode::GetGlobal,
            19 => Opcode::SetGlobal,
            20 => Opcode::Array,
            21 => Opcode::Map,
            22 => Opcode::GetIndex,
            23 => Opcode::SetIndex,
            24 => Opcode::Call,
            25 => Opcode::ReturnValue,
            26 => Opcode::Return,
            27 => Opcode::DefineLocal,
            28 => Opcode::GetLocal,
            29 => Opcode::SetLocal,
            30 => Opcode::GetBuiltinFn,
            31 => Opcode::GetBuiltinVar,
            32 => Opcode::Closure,
            33 => Opcode::GetFree,
            34 => Opcode::SetFree,
            35 => Opcode::CurrClosure,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
