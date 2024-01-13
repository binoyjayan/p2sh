use std::convert::From;
use std::fmt;

use super::error::PacketError;

#[derive(Debug, Clone)]
pub struct MacAddress(pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl MacAddress {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
    }

    pub fn from_str(s: &str) -> Result<Self, PacketError> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 6 {
            return Err(PacketError::InvalidMacAddress);
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            match u8::from_str_radix(part, 16) {
                Ok(value) => bytes[i] = value,
                Err(_) => return Err(PacketError::InvalidMacAddress),
            }
        }
        Ok(Self(
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
        ))
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0, self.1, self.2, self.3, self.4, self.5
        )
    }
}

impl From<&MacAddress> for Vec<u8> {
    fn from(eth: &MacAddress) -> Self {
        vec![eth.0, eth.1, eth.2, eth.3, eth.4, eth.5]
    }
}
