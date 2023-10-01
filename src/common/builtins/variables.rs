pub enum BuiltinVarType {
    Argv,
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
            _ => Self::Max,
        }
    }
}

impl From<BuiltinVarType> for &'static str {
    fn from(var: BuiltinVarType) -> Self {
        match var {
            BuiltinVarType::Argv => "argv",
            BuiltinVarType::Max => "",
        }
    }
}

impl From<BuiltinVarType> for usize {
    fn from(var: BuiltinVarType) -> Self {
        var as usize
    }
}
