use std::fmt;

#[derive(Debug)]
pub struct RTError {
    pub msg: String,
}

impl fmt::Display for RTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime error: {}", self.msg)
    }
}

impl RTError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}
