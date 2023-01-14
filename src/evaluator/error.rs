use std::fmt;

// Runtime error
#[derive(Debug)]
pub struct RTError {
    pub msg: String,
    pub line: usize,
}

impl fmt::Display for RTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Runtime error: {}", self.line, self.msg)
    }
}

impl RTError {
    pub fn new(msg: &str, line: usize) -> Self {
        Self {
            msg: msg.to_string(),
            line,
        }
    }
}
