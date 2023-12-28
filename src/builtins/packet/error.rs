use std::fmt;

#[derive(Debug)]
pub enum PacketError {
    InvalidLength(usize),
    InvalidMacAddress,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::InvalidLength(len) => write!(f, "invalid-length: {len}"),
            Self::InvalidMacAddress => write!(f, "invalid-mac-address"),
        }
    }
}
