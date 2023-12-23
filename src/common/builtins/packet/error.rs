use std::fmt;

#[derive(Debug)]
pub enum PacketError {
    InvalidLength(usize),
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::InvalidLength(len) => write!(f, "invalid-length: {len}"),
        }
    }
}
