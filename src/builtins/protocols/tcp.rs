use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct TcpHeader {
    srcport: u16,     // Source port number
    dstport: u16,     // Destination port number
    sequence: u32,    // Sequence number
    ack: u32,         // Acknowledgment number
    data_off: u8,     // Data offset
    flags: u16,       // Flags for TCP
    window_size: u16, // Window size
    checksum: u16,    // Checksum for integrity
    urgent: u16,      // Urgent pointer
}

impl From<&TcpHeader> for Vec<u8> {
    fn from(hdr: &TcpHeader) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&hdr.srcport.to_be_bytes());
        bytes.extend_from_slice(&hdr.dstport.to_be_bytes());
        bytes.extend_from_slice(&hdr.sequence.to_be_bytes());
        bytes.extend_from_slice(&hdr.ack.to_be_bytes());
        bytes.extend_from_slice(&hdr.flags.to_be_bytes());
        bytes.extend_from_slice(&hdr.window_size.to_be_bytes());
        bytes.extend_from_slice(&hdr.checksum.to_be_bytes());
        bytes
    }
}

#[derive(Debug)]
pub struct Tcp {
    header: RefCell<TcpHeader>,             // Header of the TCP packet
    pub rawdata: RefCell<Rc<Vec<u8>>>,      // Raw data of the entire packet
    pub offset: usize,                      // Offset of the TCP header
    pub inner: RefCell<Option<Rc<Object>>>, // Inner packet
}

pub const TCP_HEADER_SIZE: usize = 20; // Basic TCP header size

impl fmt::Display for Tcp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<port:{}:{} seq:{} ack:{}>",
            self.header.borrow().srcport,
            self.header.borrow().dstport,
            self.header.borrow().sequence,
            self.header.borrow().ack
        )
    }
}

impl From<&Tcp> for Vec<u8> {
    fn from(tcp: &Tcp) -> Self {
        let header = tcp.header.borrow().clone();
        let mut bytes: Vec<u8> = (&header).into();
        let data = tcp.rawdata.borrow().clone();
        bytes.extend_from_slice(&data[tcp.offset..]);
        bytes
    }
}

impl Tcp {
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + TCP_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let srcport = u16::from_be_bytes([rawdata[off], rawdata[off + 1]]);
        let dstport = u16::from_be_bytes([rawdata[off + 2], rawdata[off + 3]]);
        let sequence = u32::from_be_bytes([
            rawdata[off + 4],
            rawdata[off + 5],
            rawdata[off + 6],
            rawdata[off + 7],
        ]);
        let ack = u32::from_be_bytes([
            rawdata[off + 8],
            rawdata[off + 9],
            rawdata[off + 10],
            rawdata[off + 11],
        ]);
        let data_off = rawdata[off + 12] >> 4;
        let flags = u16::from_be_bytes([rawdata[off + 12], rawdata[off + 13]]);
        let window_size = u16::from_be_bytes([rawdata[off + 14], rawdata[off + 15]]);
        let checksum = u16::from_be_bytes([rawdata[off + 16], rawdata[off + 17]]);
        let urgent = u16::from_be_bytes([rawdata[off + 18], rawdata[off + 19]]);

        let header = RefCell::new(TcpHeader {
            srcport,
            dstport,
            sequence,
            ack,
            data_off,
            flags,
            window_size,
            checksum,
            urgent,
        });

        Ok(Self {
            header,
            rawdata: RefCell::new(rawdata),
            offset: off + TCP_HEADER_SIZE,
            inner: RefCell::new(None),
        })
    }

    pub fn get_source_port(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().srcport as i64))
    }

    pub fn get_destination_port(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().dstport as i64))
    }

    pub fn get_sequence(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().sequence as i64))
    }

    pub fn get_ack(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().ack as i64))
    }

    pub fn get_data_off(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().data_off as i64))
    }

    pub fn get_flags(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().flags as i64))
    }

    pub fn get_window_size(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().window_size as i64))
    }

    pub fn get_checksum(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().checksum as i64))
    }

    pub fn get_urgent(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().urgent as i64))
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

    pub fn set_sequence(&self, sequence: Rc<Object>) -> Result<(), String> {
        match sequence.as_ref() {
            Object::Integer(seq) => {
                self.header.borrow_mut().sequence = *seq as u32;
                Ok(())
            }
            _ => Err("Invalid value for sequence number".to_string()),
        }
    }

    pub fn set_ack(&self, ack: Rc<Object>) -> Result<(), String> {
        match ack.as_ref() {
            Object::Integer(ack) => {
                self.header.borrow_mut().ack = *ack as u32;
                Ok(())
            }
            _ => Err("Invalid value for acknowledgment number".to_string()),
        }
    }

    pub fn set_data_off(&self, data_off: Rc<Object>) -> Result<(), String> {
        match data_off.as_ref() {
            Object::Integer(data_off_value) => {
                self.header.borrow_mut().data_off = *data_off_value as u8;
                Ok(())
            }
            _ => Err("Invalid value for data offset".to_string()),
        }
    }

    pub fn set_flags(&self, flags: Rc<Object>) -> Result<(), String> {
        match flags.as_ref() {
            Object::Integer(flags_value) => {
                self.header.borrow_mut().flags = *flags_value as u16;
                Ok(())
            }
            _ => Err("Invalid value for flags".to_string()),
        }
    }

    pub fn set_window_size(&self, window_size: Rc<Object>) -> Result<(), String> {
        match window_size.as_ref() {
            Object::Integer(size) => {
                self.header.borrow_mut().window_size = *size as u16;
                Ok(())
            }
            _ => Err("Invalid value for window size".to_string()),
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

    pub fn set_urgent(&self, urgent: Rc<Object>) -> Result<(), String> {
        match urgent.as_ref() {
            Object::Integer(urgent_value) => {
                self.header.borrow_mut().urgent = *urgent_value as u16;
                Ok(())
            }
            _ => Err("Invalid value for urgent pointer".to_string()),
        }
    }
}
