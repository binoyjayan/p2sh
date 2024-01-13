use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use super::ipv6addr::Ipv6Address;
use crate::object::Object;

pub const IPV6_HEADER_SIZE: usize = 40;

#[derive(Debug, Clone)]
pub struct Ipv6Header {
    version: u8,
    traffic_class: u8,
    flow_label: u32,
    payload_length: u16,
    next_header: NextHeader,
    hop_limit: u8,
    source: Ipv6Address,
    destination: Ipv6Address,
}

#[derive(Debug)]
pub struct Ipv6Packet {
    header: RefCell<Ipv6Header>,
    pub rawdata: RefCell<Rc<Vec<u8>>>,
    pub offset: usize,
    pub inner: RefCell<Option<Rc<Object>>>,
}

impl fmt::Display for Ipv6Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}:{}->{}>",
            self.header.borrow().next_header,
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

impl From<&Ipv6Header> for Vec<u8> {
    fn from(hdr: &Ipv6Header) -> Self {
        let mut bytes = Vec::new();
        bytes.push((hdr.version << 4) | ((hdr.traffic_class >> 4) & 0xF));
        bytes.push(((hdr.traffic_class << 4) & 0xF0) | ((hdr.flow_label >> 16) as u8));
        bytes.extend_from_slice(&(((hdr.flow_label & 0xFFFF) as u16).to_be_bytes()));
        bytes.extend_from_slice(&hdr.payload_length.to_be_bytes());
        bytes.push(hdr.next_header.0);
        bytes.push(hdr.hop_limit);
        let b: Vec<u8> = (&hdr.source).into();
        bytes.extend_from_slice(&b);
        let b: Vec<u8> = (&hdr.destination).into();
        bytes.extend_from_slice(&b);
        bytes
    }
}

impl From<&Ipv6Packet> for Vec<u8> {
    fn from(ipv6: &Ipv6Packet) -> Self {
        let header = ipv6.header.borrow().clone();
        let mut bytes: Vec<u8> = (&header).into();
        if let Some(inner) = ipv6.inner.borrow().clone() {
            let data: Vec<u8> = inner.as_ref().into();
            bytes.extend_from_slice(&data);
        } else {
            let data = ipv6.rawdata.borrow().clone();
            bytes.extend_from_slice(&data[ipv6.offset..]);
        }
        bytes
    }
}

impl Ipv6Packet {
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + IPV6_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }

        let version_traffic_class = rawdata[off] >> 4;
        let traffic_class = (rawdata[off] & 0x0F) << 4 | (rawdata[off + 1] >> 4);
        let flow_label = ((rawdata[off + 1] as u32 & 0x0F) << 16)
            | ((rawdata[off + 2] as u32) << 8)
            | (rawdata[off + 3] as u32);
        let payload_length = ((rawdata[off + 4] as u16) << 8) | (rawdata[off + 5] as u16);
        let next_header = NextHeader(rawdata[off + 6]);
        let hop_limit = rawdata[off + 7];
        let source = Ipv6Address::from_bytes(&rawdata[off + 8..off + 24]);
        let destination = Ipv6Address::from_bytes(&rawdata[off + 24..off + 40]);
        let offset = off + IPV6_HEADER_SIZE;

        let header = Ipv6Header {
            version: version_traffic_class,
            traffic_class,
            flow_label,
            payload_length,
            next_header,
            hop_limit,
            source,
            destination,
        };

        Ok(Self {
            header: RefCell::new(header),
            rawdata: RefCell::new(rawdata),
            offset,
            inner: RefCell::new(None),
        })
    }

    pub fn get_version(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().version as i64))
    }

    pub fn get_traffic_class(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().traffic_class as i64))
    }

    pub fn get_flow_label(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().flow_label as i64))
    }

    pub fn get_payload_length(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().payload_length as i64))
    }

    pub fn get_next_header_raw(&self) -> NextHeader {
        self.header.borrow().next_header.clone()
    }

    pub fn get_next_header(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().next_header.0 as i64))
    }

    pub fn get_hop_limit(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().hop_limit as i64))
    }

    pub fn get_src(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().source.to_string()))
    }

    pub fn get_dst(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().destination.to_string()))
    }

    pub fn set_traffic_class(&self, tc: Rc<Object>) -> Result<(), String> {
        match tc.as_ref() {
            Object::Integer(tc) => {
                self.header.borrow_mut().traffic_class = *tc as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv6 property traffic class".to_string()),
        }
    }

    pub fn set_flow_label(&self, flow_label: Rc<Object>) -> Result<(), String> {
        match flow_label.as_ref() {
            Object::Integer(flow_label) => {
                self.header.borrow_mut().flow_label = *flow_label as u32;
                Ok(())
            }
            _ => Err("Invalid value for Ipv6 property flow label".to_string()),
        }
    }

    pub fn set_payload_length(&self, payload_length: Rc<Object>) -> Result<(), String> {
        match payload_length.as_ref() {
            Object::Integer(payload_length) => {
                self.header.borrow_mut().payload_length = *payload_length as u16;
                Ok(())
            }
            _ => Err("Invalid value for Ipv6 property payload length".to_string()),
        }
    }

    pub fn set_next_header(&self, next_header: Rc<Object>) -> Result<(), String> {
        match next_header.as_ref() {
            Object::Integer(next_header) => {
                self.header.borrow_mut().next_header.0 = *next_header as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv6 property next header".to_string()),
        }
    }

    pub fn set_hop_limit(&self, hop_limit: Rc<Object>) -> Result<(), String> {
        match hop_limit.as_ref() {
            Object::Integer(hop_limit) => {
                self.header.borrow_mut().hop_limit = *hop_limit as u8;
                Ok(())
            }
            _ => Err("Invalid value for Ipv6 property hop limit".to_string()),
        }
    }

    pub fn set_src(&self, source_address: Rc<Object>) -> Result<(), String> {
        match source_address.as_ref() {
            Object::Str(src) => match Ipv6Address::from_str(src) {
                Ok(ipv4_addr) => {
                    self.header.borrow_mut().source = ipv4_addr;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for Ipv6 property source address".to_string()),
        }
    }
    pub fn set_dst(&self, destination_address: Rc<Object>) -> Result<(), String> {
        match destination_address.as_ref() {
            Object::Str(dst) => match Ipv6Address::from_str(dst) {
                Ok(ipv4_addr) => {
                    self.header.borrow_mut().destination = ipv4_addr;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for Ipv6 property destination address".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextHeader(pub u8);

#[allow(unused, non_upper_case_globals, non_snake_case)]
pub mod NextHeaders {
    use super::NextHeader;
    pub const HopByHop: NextHeader = NextHeader(0);
    pub const Tcp: NextHeader = NextHeader(6);
    pub const Udp: NextHeader = NextHeader(17);
    pub const Ipv6Encap: NextHeader = NextHeader(41);
    pub const Ipv6Route: NextHeader = NextHeader(43);
    pub const Ipv6Frag: NextHeader = NextHeader(44);
    pub const Rsvp: NextHeader = NextHeader(46);
    pub const EncapSecPayload: NextHeader = NextHeader(50);
    pub const AuthHdr: NextHeader = NextHeader(51);
    pub const Icmpv6: NextHeader = NextHeader(58);
    pub const NoNxt: NextHeader = NextHeader(59);
    pub const Ipv6Opts: NextHeader = NextHeader(60);
    pub const Mobility: NextHeader = NextHeader(135);
    pub const Hip: NextHeader = NextHeader(139);
    pub const Shim6: NextHeader = NextHeader(140);
    pub const Rsvd1: NextHeader = NextHeader(253);
    pub const Rsvd2: NextHeader = NextHeader(254);
}

impl fmt::Display for NextHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            0 => write!(f, "HopByHop"),
            6 => write!(f, "Tcp"),
            17 => write!(f, "Udp"),
            41 => write!(f, "Ipv6Encap"),
            43 => write!(f, "Ipv6Route"),
            44 => write!(f, "Ipv6Frag"),
            46 => write!(f, "Rsvp"),
            50 => write!(f, "EncapSecPayload"),
            51 => write!(f, "AuthHdr"),
            58 => write!(f, "Icmpv6"),
            59 => write!(f, "NoNxt"),
            60 => write!(f, "Ipv6Opts"),
            135 => write!(f, "Mobility"),
            139 => write!(f, "Hip"),
            140 => write!(f, "Shim6"),
            253 => write!(f, "Rsvd1"),
            254 => write!(f, "Rsvd1"),
            _ => write!(f, "NxtHdr:{}", self.0),
        }
    }
}
