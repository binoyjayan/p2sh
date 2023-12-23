use std::fmt;

#[derive(Debug)]
pub struct MacAddress(pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl MacAddress {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
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
