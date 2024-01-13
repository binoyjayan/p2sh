use std::fmt;

#[derive(Debug, Clone)]
pub struct Ipv6Address(
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
);

impl Ipv6Address {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != 16 {
            panic!("Invalid byte length for IPv6 address");
        }
        Self(
            ((bytes[0] as u16) << 8) | bytes[1] as u16,
            ((bytes[2] as u16) << 8) | bytes[3] as u16,
            ((bytes[4] as u16) << 8) | bytes[5] as u16,
            ((bytes[6] as u16) << 8) | bytes[7] as u16,
            ((bytes[8] as u16) << 8) | bytes[9] as u16,
            ((bytes[10] as u16) << 8) | bytes[11] as u16,
            ((bytes[12] as u16) << 8) | bytes[13] as u16,
            ((bytes[14] as u16) << 8) | bytes[15] as u16,
        )
    }

    /// Create an IPv6 address from a string that uses zero compression
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        // Split the string by colons to get each segment
        let segments: Vec<&str> = s.split(':').collect();

        // Ensure we have at most 8 segments for a valid IPv6 address
        if segments.len() > 8 {
            return Err("Invalid IPv6 address format");
        }

        let mut parts = [0u16; 8];
        let mut part_index = 0; // Index to fill in the parts array

        // Flags to handle zero compression
        let mut compressed = false;
        let mut compression_index = 0; // Index where compression starts

        for (i, &segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                if compressed {
                    return Err("Invalid IPv6 address format");
                }
                compressed = true;
                compression_index = i;
                continue;
            }

            if part_index >= 8 {
                return Err("Invalid IPv6 address format");
            }

            // Convert segment to u16 value
            match u16::from_str_radix(segment, 16) {
                Ok(value) => parts[part_index] = value,
                Err(_) => return Err("Invalid segment in IPv6 address"),
            }

            part_index += 1;
        }

        // Handle zero compression
        if compressed {
            // Calculate the number of segments we need to shift
            let shift = 8 - part_index;

            // Shift parts to make room for the compressed segments
            for i in (compression_index + shift..8).rev() {
                parts[i] = parts[i - shift];
            }

            // Fill in the compressed segments with zeros
            for part in parts.iter_mut().skip(compression_index).take(shift) {
                *part = 0;
            }
        } else if part_index != 8 {
            // If no compression, ensure we have exactly 8 parts
            return Err("Invalid IPv6 address format");
        }

        Ok(Self(
            parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7],
        ))
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7
        )
    }
}

impl From<&Ipv6Address> for Vec<u8> {
    fn from(ip: &Ipv6Address) -> Self {
        vec![
            (ip.0 >> 8) as u8,
            ip.0 as u8,
            (ip.1 >> 8) as u8,
            ip.1 as u8,
            (ip.2 >> 8) as u8,
            ip.2 as u8,
            (ip.3 >> 8) as u8,
            ip.3 as u8,
            (ip.4 >> 8) as u8,
            ip.4 as u8,
            (ip.5 >> 8) as u8,
            ip.5 as u8,
            (ip.6 >> 8) as u8,
            ip.6 as u8,
            (ip.7 >> 8) as u8,
            ip.7 as u8,
        ]
    }
}
