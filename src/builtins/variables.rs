pub enum BuiltinVarType {
    Argv,
    NP, // Number of packets processed so far
    PL, // Length of the current packet
    WL, // Length of the current packet on wire
    Max,
}

impl BuiltinVarType {
    pub fn count() -> usize {
        Self::Max as usize
    }
    pub fn range() -> std::ops::Range<usize> {
        0..Self::count()
    }
}

impl From<usize> for BuiltinVarType {
    fn from(var: usize) -> Self {
        match var {
            0 => Self::Argv,
            1 => Self::NP,
            2 => Self::PL,
            3 => Self::WL,
            _ => Self::Max,
        }
    }
}

impl From<BuiltinVarType> for &'static str {
    fn from(var: BuiltinVarType) -> Self {
        match var {
            BuiltinVarType::Argv => "argv",
            BuiltinVarType::NP => "NP",
            BuiltinVarType::PL => "PL",
            BuiltinVarType::WL => "WL",
            BuiltinVarType::Max => "",
        }
    }
}

impl From<BuiltinVarType> for usize {
    fn from(var: BuiltinVarType) -> Self {
        var as usize
    }
}
