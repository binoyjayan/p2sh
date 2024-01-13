use std::fmt;
use std::io;

use crate::builtins::protocols::error::PacketError;

/// The error object.
/// It is different from runtime error in the sense that a runtime error
/// is not interpreted by the programming language but will terminate the
/// program. However, the error objects are used in the builtin functions
/// for error handling.
#[derive(Debug)]
pub enum ErrorObj {
    IO(io::Error),
    Utf8(std::string::FromUtf8Error),
    Packet(PacketError),
}

impl fmt::Display for ErrorObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::IO(e) => write!(f, "io-error: {e}"),
            Self::Utf8(e) => write!(f, "utf8-error: {e}"),
            Self::Packet(e) => write!(f, "packet-error: {}", e),
        }
    }
}
