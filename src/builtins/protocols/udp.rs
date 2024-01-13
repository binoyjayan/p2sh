use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct UdpHeader {
    srcport: u16,  // Source port number
    dstport: u16,  // Destination port number
    length: u16,   // Length of the UDP packet
    checksum: u16, // Checksum for integrity
}

impl From<&UdpHeader> for Vec<u8> {
    fn from(hdr: &UdpHeader) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&hdr.srcport.to_be_bytes());
        bytes.extend_from_slice(&hdr.dstport.to_be_bytes());
        bytes.extend_from_slice(&hdr.length.to_be_bytes());
        bytes.extend_from_slice(&hdr.checksum.to_be_bytes());
        bytes
    }
}

#[derive(Debug)]
pub struct Udp {
    header: RefCell<UdpHeader>,             // Header of the UDP packet
    pub rawdata: RefCell<Rc<Vec<u8>>>,      // Raw data of the entire packet
    pub offset: usize,                      // Offset of the UDP header
    pub inner: RefCell<Option<Rc<Object>>>, // Inner packet
}

pub const UDP_HEADER_SIZE: usize = 8;

impl fmt::Display for Udp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<port:{}:{} len:{}>",
            self.header.borrow().srcport,
            self.header.borrow().dstport,
            self.header.borrow().length
        )
    }
}

impl From<&Udp> for Vec<u8> {
    fn from(udp: &Udp) -> Self {
        let header = udp.header.borrow().clone();
        let mut bytes: Vec<u8> = (&header).into();
        let data = udp.rawdata.borrow().clone();
        bytes.extend_from_slice(&data[udp.offset..]);
        bytes
    }
}

impl Udp {
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + UDP_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let srcport = u16::from_be_bytes([rawdata[off], rawdata[off + 1]]);
        let dstport = u16::from_be_bytes([rawdata[off + 2], rawdata[off + 3]]);
        let length = u16::from_be_bytes([rawdata[off + 4], rawdata[off + 5]]);
        let checksum = u16::from_be_bytes([rawdata[off + 6], rawdata[off + 7]]);
        let header = RefCell::new(UdpHeader {
            srcport,
            dstport,
            length,
            checksum,
        });
        Ok(Self {
            header,
            rawdata: RefCell::new(rawdata),
            offset: off + UDP_HEADER_SIZE,
            inner: RefCell::new(None),
        })
    }

    pub fn get_source_port(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().srcport as i64))
    }

    pub fn get_destination_port(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().dstport as i64))
    }

    pub fn get_length(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().length as i64))
    }

    pub fn get_checksum(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().checksum as i64))
    }

    pub fn set_source_port(&self, port: Rc<Object>) -> Result<(), String> {
        match port.as_ref() {
            Object::Integer(port_value) => {
                self.header.borrow_mut().srcport = *port_value as u16;
                Ok(())
            }
            _ => Err("Invalid value for source port".to_string()),
        }
    }

    pub fn set_destination_port(&self, port: Rc<Object>) -> Result<(), String> {
        match port.as_ref() {
            Object::Integer(port_value) => {
                self.header.borrow_mut().dstport = *port_value as u16;
                Ok(())
            }
            _ => Err("Invalid value for destination port".to_string()),
        }
    }

    pub fn set_length(&self, length: Rc<Object>) -> Result<(), String> {
        match length.as_ref() {
            Object::Integer(len) => {
                self.header.borrow_mut().length = *len as u16;
                Ok(())
            }
            _ => Err("Invalid value for length".to_string()),
        }
    }

    pub fn set_checksum(&self, checksum: Rc<Object>) -> Result<(), String> {
        match checksum.as_ref() {
            Object::Integer(checksum_value) => {
                self.header.borrow_mut().checksum = *checksum_value as u16;
                Ok(())
            }
            _ => Err("Invalid value for checksum".to_string()),
        }
    }
}
