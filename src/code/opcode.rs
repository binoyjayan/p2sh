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
    GreaterEq,
    Minus,
    Bang,
    Jump,
    JumpIfFalse,
    JumpIfFalseNoPop,
    Null,
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
    Not,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    Dup,
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
            12 => Opcode::GreaterEq,
            13 => Opcode::Minus,
            14 => Opcode::Bang,
            15 => Opcode::Jump,
            16 => Opcode::JumpIfFalse,
            17 => Opcode::JumpIfFalseNoPop,
            18 => Opcode::Null,
            19 => Opcode::DefineGlobal,
            20 => Opcode::GetGlobal,
            21 => Opcode::SetGlobal,
            22 => Opcode::Array,
            23 => Opcode::Map,
            24 => Opcode::GetIndex,
            25 => Opcode::SetIndex,
            26 => Opcode::Call,
            27 => Opcode::ReturnValue,
            28 => Opcode::Return,
            29 => Opcode::DefineLocal,
            30 => Opcode::GetLocal,
            31 => Opcode::SetLocal,
            32 => Opcode::GetBuiltinFn,
            33 => Opcode::GetBuiltinVar,
            34 => Opcode::Closure,
            35 => Opcode::GetFree,
            36 => Opcode::SetFree,
            37 => Opcode::CurrClosure,
            38 => Opcode::Not,
            39 => Opcode::And,
            40 => Opcode::Or,
            41 => Opcode::Xor,
            42 => Opcode::ShiftLeft,
            43 => Opcode::ShiftRight,
            44 => Opcode::Dup,
            _ => Opcode::Invalid,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(code: Opcode) -> Self {
        code as u8
    }
}
