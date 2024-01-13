use std::convert::From;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Ipv4Address(pub u8, pub u8, pub u8, pub u8);

impl Ipv4Address {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != 4 {
            panic!("Invalid byte length for IPv4 address");
        }
        Self(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 4 {
            return Err("Invalid IPv4 address format");
        }

        let mut bytes = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            match part.parse::<u8>() {
                Ok(value) => bytes[i] = value,
                _ => return Err("Invalid IPv4 address format"),
            }
        }
        Ok(Self(bytes[0], bytes[1], bytes[2], bytes[3]))
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
    }
}

impl From<&Ipv4Address> for Vec<u8> {
    fn from(ip: &Ipv4Address) -> Self {
        vec![ip.0, ip.1, ip.2, ip.3]
    }
}
