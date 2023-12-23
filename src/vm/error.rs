use std::fmt;

/// Runtime error. It is different from error objects used in the builtin
/// functions in the sense that a runtime error is not interpreted by the
/// programming language but will terminate the program. However, the error
/// objects used in the builtin functions are returned to the program for
/// further processing.
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
