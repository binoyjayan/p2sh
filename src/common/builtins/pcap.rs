use std::fmt;
use std::io::{self, Read};
use std::rc::Rc;

use crate::common::object::FileHandle;

const PCAP_MAGIC_MS: u32 = 0xA1B2C3D4;
const PCAP_MAGIC_NS: u32 = 0xA1B23C4D;

#[derive(Debug)]
enum PcapTsFormat {
    MicroSeconds,
    NanoSeconds,
}

#[allow(unused)]
#[derive(Debug)]
pub struct PcapGlobalHeader {
    magic_number: u32,
    version_major: u16,
    version_minor: u16,
    thiszone: i32,
    sigfigs: u32,
    snaplen: u32,
    linktype: u32,
}

impl Default for PcapGlobalHeader {
    fn default() -> Self {
        Self {
            magic_number: PCAP_MAGIC_MS,
            version_major: 2,
            version_minor: 4,
            thiszone: 0,
            sigfigs: 0,
            snaplen: 65535,
            linktype: 1,
        }
    }
}

impl PcapGlobalHeader {
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < 24 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid pcap global header size: {:X}", data.len()),
            ));
        }

        let magic_number = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let version_major = u16::from_le_bytes([data[4], data[5]]);
        let version_minor = u16::from_le_bytes([data[6], data[7]]);
        let thiszone = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let sigfigs = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let snaplen = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let linktype = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);

        if magic_number != PCAP_MAGIC_MS && magic_number != PCAP_MAGIC_NS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid pcap magic number -- {:X}", magic_number,),
            ));
        }

        Ok(Self {
            magic_number,
            version_major,
            version_minor,
            thiszone,
            sigfigs,
            snaplen,
            linktype,
        })
    }
    pub fn snaplen(&self) -> u32 {
        self.snaplen
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct PcapPacketHeader {
    ts_sec: u32,  // Timestamp seconds
    ts_usec: u32, // Timestamp in nanoseconds or microseconds
    caplen: u32,  // Length of portion present
    wirelen: u32, // Length of the packet (off wire)
}

impl fmt::Display for PcapPacketHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<pcap-packet caplen: {}, wirelen: {}>",
            self.caplen, self.wirelen
        )
    }
}

impl PcapPacketHeader {
    // Create a new instance of PcapPacketHeader from a byte slice
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid pcap packet header size",
            ));
        }
        Ok(Self {
            ts_sec: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            ts_usec: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            caplen: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            wirelen: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        })
    }
    pub fn caplen(&self) -> u32 {
        self.caplen
    }
}

#[derive(Debug)]
pub struct PcapPacket {
    pub header: PcapPacketHeader,
    pub data: Vec<u8>,
}

impl fmt::Display for PcapPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} [len: {}]", self.header, self.data.len())
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Pcap {
    pub file: Rc<FileHandle>,
    header: PcapGlobalHeader,
    ts_format: PcapTsFormat,
}

impl fmt::Display for Pcap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.file)
    }
}

impl Pcap {
    pub fn from_file(file: Rc<FileHandle>) -> io::Result<Self> {
        let mut global_header_data = [0u8; 24]; // Size of pcap global header
        match file.as_ref() {
            FileHandle::Reader(reader) => {
                reader.borrow_mut().read_exact(&mut global_header_data)?;
            }
            FileHandle::Stdin => {
                io::stdin().read_exact(&mut global_header_data)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid file handle",
                ))
            }
        }
        let global_header = PcapGlobalHeader::from_bytes(&global_header_data)?;
        let ts_format = if global_header.magic_number == PCAP_MAGIC_MS {
            PcapTsFormat::MicroSeconds
        } else {
            PcapTsFormat::NanoSeconds
        };

        Ok(Self {
            file,
            header: global_header,
            ts_format,
        })
    }

    pub fn next_packet(&self) -> io::Result<PcapPacket> {
        let mut packet_header_data = [0u8; 16]; // Size of pcap packet header

        match self.file.as_ref() {
            FileHandle::Reader(reader) => {
                reader.borrow_mut().read_exact(&mut packet_header_data)?;
                let packet_header = PcapPacketHeader::from_bytes(&packet_header_data)?;

                // Check if caplen is greater than the snaplen to avoid potential issues
                if packet_header.caplen() > self.header.snaplen() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid caplen value exceeds snaplen",
                    ));
                }

                // Read the payload data based on the caplen from the packet header
                let mut packet_data = vec![0u8; packet_header.caplen() as usize];
                reader.borrow_mut().read_exact(&mut packet_data)?;

                // return whole packet
                Ok(PcapPacket {
                    header: packet_header,
                    data: packet_data,
                })
            }
            FileHandle::Stdin => {
                // Read bytes from stdin
                io::stdin().read_exact(&mut packet_header_data)?;
                let packet_header = PcapPacketHeader::from_bytes(&packet_header_data)?;
                // Check if caplen is greater than the snaplen to avoid potential issues
                if packet_header.caplen() > self.header.snaplen() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid caplen value exceeds snaplen",
                    ));
                }
                let mut packet_data = vec![0u8; packet_header.caplen() as usize];
                io::stdin().read_exact(&mut packet_data)?;
                Ok(PcapPacket {
                    header: packet_header,
                    data: packet_data,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid file handle",
            )),
        }
    }
}
