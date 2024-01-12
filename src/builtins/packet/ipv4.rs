use std::cell::RefCell;
use std::convert::From;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use super::ipv4addr::Ipv4Address;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct Ipv4Header {
    version: u8,
    ihl: u8,
    dscp: u8,
    ecn: u8,
    total_length: u16,
    identification: u16,
    flags: u8,
    fragment_offset: u16,
    ttl: u8,
    protocol: Protocol,
    checksum: u16,
    source: Ipv4Address,
    destination: Ipv4Address,
}

#[derive(Debug)]
pub struct Ipv4Packet {
    header: RefCell<Ipv4Header>,
    pub rawdata: RefCell<Rc<Vec<u8>>>,
    pub offset: usize,
    pub inner: RefCell<Option<Rc<Object>>>,
}

pub const IPV4_HEADER_SIZE: usize = 20;

impl fmt::Display for Ipv4Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}:{}->{}>",
            self.header.borrow().protocol,
            self.header.borrow().source,
            self.header.borrow().destination
        )?;
        if let Some(inner) = self.inner.borrow().clone() {
            write!(f, " {}", inner)
        } else {
            write!(f, " [len: {}]", self.rawdata.borrow().len() - self.offset)
        }
    }
}

impl Ipv4Packet {
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + IPV4_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let version_ihl = rawdata[off] & 0xF;
        let version = (version_ihl >> 4) & 0xF;
        let ihl = version_ihl & 0xF;
        let dscp_ecn = rawdata[off + 1];
        let dscp = dscp_ecn >> 2;
        let ecn = dscp_ecn & 0x03;
        let total_length = ((rawdata[off + 2] as u16) << 8) | (rawdata[off + 3] as u16);
        let identification = ((rawdata[off + 4] as u16) << 8) | (rawdata[off + 5] as u16);
        let flags_fragment_offset = ((rawdata[off + 6] as u16) << 8) | (rawdata[off + 7] as u16);
        let flags = (flags_fragment_offset >> 13) as u8;
        let fragment_offset = flags_fragment_offset & 0x1FFF;
        let ttl = rawdata[off + 8];
        let protocol = Protocol(rawdata[off + 9]);
        let checksum = ((rawdata[off + 10] as u16) << 8) | (rawdata[off + 11] as u16);
        let source = Ipv4Address(
            rawdata[off + 12],
            rawdata[off + 13],
            rawdata[off + 14],
            rawdata[off + 15],
        );
        let destination = Ipv4Address(
            rawdata[off + 16],
            rawdata[off + 17],
            rawdata[off + 18],
            rawdata[off + 19],
        );

        // Handle ipv4 options
        let mut options = Vec::new();
        if ihl > 5 {
            let mut i: usize = 20;
            while i < ihl as usize * 4 {
                options.push(rawdata[off + i]);
                i += 1;
            }
        }
        //  offset of payload
        let off = off + ihl as usize * 4;

        let header = Ipv4Header {
            version,
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            checksum,
            source,
            destination,
        };
        Ok(Self {
            header: RefCell::new(header),
            rawdata: RefCell::new(rawdata),
            offset: off,
            inner: RefCell::new(None),
        })
    }

    pub fn get_version(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().version as i64))
    }
    pub fn get_ihl(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().ihl as i64))
    }

    pub fn get_src(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().source.to_string()))
    }
    pub fn set_src(&self, source_address: Rc<Object>) -> Result<(), String> {
        match source_address.as_ref() {
            Object::Str(source_address) => match Ipv4Address::from_str(source_address) {
                Ok(ipv4_addr) => {
                    self.header.borrow_mut().source = ipv4_addr;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for Ipv4 property source_address".to_string()),
        }
    }

    pub fn get_dst(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().destination.to_string()))
    }
    pub fn set_dst(&self, destination_address: Rc<Object>) -> Result<(), String> {
        match destination_address.as_ref() {
            Object::Str(destination_address) => match Ipv4Address::from_str(destination_address) {
                Ok(ipv4_addr) => {
                    self.header.borrow_mut().destination = ipv4_addr;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for Ipv4 property destination_address".to_string()),
        }
    }

    pub fn set_ihl(&self, ihl: Rc<Object>) -> Result<(), String> {
        match ihl.as_ref() {
            Object::Integer(ihl) => {
                if *ihl < 0 || *ihl > 15 {
                    return Err("Invalid value for Ipv4 property ihl".to_string());
                }
                self.header.borrow_mut().ihl = *ihl as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property ihl".to_string()),
        }
    }

    pub fn get_dscp(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().dscp as i64))
    }
    pub fn set_dscp(&self, dscp: Rc<Object>) -> Result<(), String> {
        match dscp.as_ref() {
            Object::Integer(dscp) => {
                if *dscp < 0 || *dscp > 63 {
                    return Err("Invalid value for Ipv4 property dscp".to_string());
                }
                self.header.borrow_mut().dscp = *dscp as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property dscp".to_string()),
        }
    }

    pub fn get_ecn(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().ecn as i64))
    }
    pub fn set_ecn(&self, ecn: Rc<Object>) -> Result<(), String> {
        match ecn.as_ref() {
            Object::Integer(ecn) => {
                if *ecn < 0 || *ecn > 3 {
                    return Err("Invalid value for Ipv4 property ecn".to_string());
                }
                self.header.borrow_mut().ecn = *ecn as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property ecn".to_string()),
        }
    }

    pub fn get_total_length(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().total_length as i64))
    }
    pub fn set_total_length(&self, total_length: Rc<Object>) -> Result<(), String> {
        match total_length.as_ref() {
            Object::Integer(total_length) => {
                if *total_length < 0 || *total_length > 65535 {
                    return Err("Invalid value for Ipv4 property total_length".to_string());
                }
                self.header.borrow_mut().total_length = *total_length as u16;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property total_length".to_string()),
        }
    }

    pub fn get_identification(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().identification as i64))
    }
    pub fn set_identification(&self, identification: Rc<Object>) -> Result<(), String> {
        match identification.as_ref() {
            Object::Integer(identification) => {
                if *identification < 0 || *identification > 65535 {
                    return Err("Invalid value for Ipv4 property identification".to_string());
                }
                self.header.borrow_mut().identification = *identification as u16;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property identification".to_string()),
        }
    }

    pub fn get_flags(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().flags as i64))
    }
    pub fn set_flags(&self, flags: Rc<Object>) -> Result<(), String> {
        match flags.as_ref() {
            Object::Integer(flags) => {
                if *flags < 0 || *flags > 7 {
                    return Err("Invalid value for Ipv4 property flags".to_string());
                }
                self.header.borrow_mut().flags = *flags as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property flags".to_string()),
        }
    }

    pub fn get_fragment_offset(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().fragment_offset as i64))
    }
    pub fn set_fragment_offset(&self, fragment_offset: Rc<Object>) -> Result<(), String> {
        match fragment_offset.as_ref() {
            Object::Integer(fragment_offset) => {
                if *fragment_offset < 0 || *fragment_offset > 8191 {
                    return Err("Invalid value for Ipv4 property fragment_offset".to_string());
                }
                self.header.borrow_mut().fragment_offset = *fragment_offset as u16;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property fragment_offset".to_string()),
        }
    }

    pub fn get_ttl(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().ttl as i64))
    }
    pub fn set_ttl(&self, ttl: Rc<Object>) -> Result<(), String> {
        match ttl.as_ref() {
            Object::Integer(ttl) => {
                if *ttl < 0 || *ttl > 255 {
                    return Err("Invalid value for Ipv4 property ttl".to_string());
                }
                self.header.borrow_mut().ttl = *ttl as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property ttl".to_string()),
        }
    }

    pub fn get_protocol(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().protocol.0 as i64))
    }
    pub fn set_protocol(&self, protocol: Rc<Object>) -> Result<(), String> {
        match protocol.as_ref() {
            Object::Integer(protocol) => {
                if *protocol < 0 || *protocol > 255 {
                    return Err("Invalid value for Ipv4 property protocol".to_string());
                }
                self.header.borrow_mut().protocol = Protocol(*protocol as u8);
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property protocol".to_string()),
        }
    }

    pub fn get_checksum(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().checksum as i64))
    }
    pub fn set_checksum(&self, checksum: Rc<Object>) -> Result<(), String> {
        match checksum.as_ref() {
            Object::Integer(checksum) => {
                if *checksum < 0 || *checksum > 65535 {
                    return Err("Invalid value for Ipv4 property checksum".to_string());
                }
                self.header.borrow_mut().checksum = *checksum as u16;
                Ok(())
            }
            _ => Err("Invalid value for Ipv4 property checksum".to_string()),
        }
    }
}

impl From<&Ipv4Packet> for Vec<u8> {
    fn from(packet: &Ipv4Packet) -> Self {
        let header = packet.header.borrow();
        let mut bytes = Vec::new();
        // Version (4 bits) and IHL (4 bits)
        let version_ihl: u8 = (header.version << 4) | header.ihl;
        bytes.push(version_ihl);
        // DSCP (6 bits) and ECN (2 bits)
        let dscp_ecn: u8 = (header.dscp << 2) | header.ecn;
        bytes.push(dscp_ecn);
        // Total Length (16 bits)
        bytes.extend_from_slice(&header.total_length.to_be_bytes());
        // Identification (16 bits)
        bytes.extend_from_slice(&header.identification.to_be_bytes());
        // Flags (3 bits) and Fragment Offset (13 bits)
        let flags_fragment_offset: u16 = ((header.flags as u16) << 13) | header.fragment_offset;
        bytes.extend_from_slice(&flags_fragment_offset.to_be_bytes());
        // Time to Live (8 bits)
        bytes.push(header.ttl);
        // Protocol (8 bits)
        bytes.push(header.protocol.0);
        // Checksum (16 bits)
        bytes.extend_from_slice(&header.checksum.to_be_bytes());
        // Source Address (32 bits)
        let b: Vec<u8> = (&header.source).into();
        bytes.extend_from_slice(&b);
        // Destination Address (32 bits)
        let b: Vec<u8> = (&header.destination).into();
        bytes.extend_from_slice(&b);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Protocol(pub u8);

#[allow(unused, non_upper_case_globals, non_snake_case)]
pub mod Protocols {
    use super::Protocol;
    pub const Icmp: Protocol = Protocol(1);
    pub const Igmp: Protocol = Protocol(2);
    pub const Tcp: Protocol = Protocol(6);
    pub const Udp: Protocol = Protocol(17);
    pub const Rdp: Protocol = Protocol(27);
    pub const Rsvp: Protocol = Protocol(46);
    pub const Gre: Protocol = Protocol(47);
    pub const Esp: Protocol = Protocol(50);
    pub const Ah: Protocol = Protocol(51);
    pub const Eigrp: Protocol = Protocol(88);
    pub const Ospf: Protocol = Protocol(89);
    pub const Pim: Protocol = Protocol(103);
    pub const Hsrp: Protocol = Protocol(112);
    pub const L2tp: Protocol = Protocol(115);
    pub const Sctp: Protocol = Protocol(132);
    pub const Snmp: Protocol = Protocol(161);
    pub const SnmpTrap: Protocol = Protocol(162);
    pub const Bgp: Protocol = Protocol(179);
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            1 => write!(f, "Icmp"),
            2 => write!(f, "Igmp"),
            6 => write!(f, "Tcp"),
            17 => write!(f, "Udp"),
            27 => write!(f, "Rdp"),
            46 => write!(f, "Rsvp"),
            47 => write!(f, "Gre"),
            50 => write!(f, "Esp"),
            51 => write!(f, "Ah"),
            88 => write!(f, "Eigrp"),
            89 => write!(f, "Ospf"),
            103 => write!(f, "Pim"),
            112 => write!(f, "Hsrp"),
            115 => write!(f, "L2tp"),
            132 => write!(f, "Sctp"),
            161 => write!(f, "Snmp"),
            162 => write!(f, "SnmpTrap"),
            179 => write!(f, "Bgp"),
            _ => write!(f, "Proto:{}", self.0),
        }
    }
}
