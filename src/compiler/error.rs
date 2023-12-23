use std::fmt;

// Compile error
#[derive(Debug)]
pub struct CompileError {
    pub msg: String,
    pub line: usize,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] compile error: {}", self.line, self.msg)
    }
}

impl CompileError {
    pub fn new(msg: &str, line: usize) -> Self {
        Self {
            msg: msg.to_string(),
            line,
        }
    }
}
