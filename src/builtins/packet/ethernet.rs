use super::error::PacketError;
use super::macaddress::MacAddress;
use crate::object::Object;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct EthernetHeader {
    pub dest: MacAddress,   // Destination MAC address
    pub source: MacAddress, // Source MAC address
    pub ethertype: EtherType,
}

#[derive(Debug)]
pub struct Ethernet {
    pub header: RefCell<EthernetHeader>, // Header of the ethernet packet
    pub rawdata: Rc<Vec<u8>>,            // Raw data of the entire packet
    pub offset: usize,                   // Offset of the ethernet header
}

pub const ETHERNET_HEADER_SIZE: usize = 14;

impl fmt::Display for Ethernet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}: {} -> {}>",
            self.header.borrow().ethertype,
            self.header.borrow().source,
            self.header.borrow().dest
        )
    }
}

impl Ethernet {
    // off is the offset of the ethernet header when it is encapsulated in
    // another protocol. For example, if the ethernet header is encapsulated
    // in an 802.1Q VLAN header, then off is the offset of the VLAN header.
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + ETHERNET_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let destination = MacAddress::from_bytes(&rawdata[off..off + 6]);
        let source = MacAddress::from_bytes(&rawdata[off + 6..off + 12]);
        let ethertype = EtherType(((rawdata[off + 12] as u16) << 8) | (rawdata[off + 13] as u16));
        let offset = off + ETHERNET_HEADER_SIZE;
        let header = RefCell::new(EthernetHeader {
            dest: destination,
            source,
            ethertype,
        });
        Ok(Self {
            header,
            rawdata,
            offset,
        })
    }
    pub fn get_src(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().source.to_string()))
    }
    pub fn get_dst(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().dest.to_string()))
    }
    pub fn get_ethertype(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().ethertype.to_string()))
    }
    pub fn set_src(&self, src: Rc<Object>) -> Result<(), String> {
        match src.as_ref() {
            Object::Str(src) => match MacAddress::from_str(src) {
                Ok(mac) => {
                    self.header.borrow_mut().source = mac;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for ethernet property src".to_string()),
        }
    }
    pub fn set_dst(&self, src: Rc<Object>) -> Result<(), String> {
        match src.as_ref() {
            Object::Str(dst) => match MacAddress::from_str(dst) {
                Ok(mac) => {
                    self.header.borrow_mut().dest = mac;
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            },
            _ => Err("Invalid value for ethernet property dest".to_string()),
        }
    }
    pub fn set_ethertype(&self, src: Rc<Object>) -> Result<(), String> {
        match src.as_ref() {
            Object::Integer(ethertype) => {
                self.header.borrow_mut().ethertype = EtherType(*ethertype as u16);
                Ok(())
            }
            _ => Err("Invalid value for ethernet property ethertype".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct EtherType(pub u16);

#[allow(unused, non_upper_case_globals, non_snake_case)]
pub mod EtherTypes {
    use super::EtherType;
    pub const Ipv4: EtherType = EtherType(0x0800);
    pub const Ipv6: EtherType = EtherType(0x86DD);
    pub const Arp: EtherType = EtherType(0x0806);
    pub const WakeOnLAN: EtherType = EtherType(0x0842);
    pub const Rarp: EtherType = EtherType(0x8035);
    pub const AppleTalk: EtherType = EtherType(0x809B);
    pub const AppleTalkARP: EtherType = EtherType(0x80F3);
    pub const Vlan: EtherType = EtherType(0x8100);
    pub const QinQ: EtherType = EtherType(0x9100);
    pub const NovellIPX: EtherType = EtherType(0x8137);
    pub const NovellNetware: EtherType = EtherType(0x8138);
    pub const IPv6OverEthernet: EtherType = EtherType(0x86DD);
    pub const CobraNet: EtherType = EtherType(0x8819);
    pub const MPLSUnicast: EtherType = EtherType(0x8847);
    pub const MPLSMulticast: EtherType = EtherType(0x8848);
    pub const PPoEDiscoveryStage: EtherType = EtherType(0x8863);
    pub const PPoESessionStage: EtherType = EtherType(0x8864);
    pub const EAPOL: EtherType = EtherType(0x888E);
    pub const HyperSCSI: EtherType = EtherType(0x889A);
    pub const HomePlug1_0MME: EtherType = EtherType(0x88E1);
    pub const IEEE8021X: EtherType = EtherType(0x88E5);
    pub const Profinet: EtherType = EtherType(0x8892);
    pub const LLDP: EtherType = EtherType(0x88CC);
    pub const EthernetPowerlink: EtherType = EtherType(0x88AB);
    pub const ECTP: EtherType = EtherType(0x9000);
}

impl fmt::Display for EtherType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EtherType(0x0800) => write!(f, "IPv4"),
            EtherType(0x86DD) => write!(f, "IPv6"),
            EtherType(0x0806) => write!(f, "ARP"),
            EtherType(0x0842) => write!(f, "WakeOnLAN"),
            EtherType(0x8035) => write!(f, "RARP"),
            EtherType(0x809B) => write!(f, "AppleTalk"),
            EtherType(0x80F3) => write!(f, "AppleTalkARP"),
            EtherType(0x8100) => write!(f, "VLAN"),
            EtherType(0x9100) => write!(f, "QinQ"),
            EtherType(0x8137) => write!(f, "NovellIPX"),
            EtherType(0x8138) => write!(f, "NovellNetware"),
            EtherType(0x8819) => write!(f, "CobraNet"),
            EtherType(0x8847) => write!(f, "MPLSUnicast"),
            EtherType(0x8848) => write!(f, "MPLSMulticast"),
            EtherType(0x8863) => write!(f, "PPoEDiscoveryStage"),
            EtherType(0x8864) => write!(f, "PPoESessionStage"),
            EtherType(0x888E) => write!(f, "EAPOL"),
            EtherType(0x889A) => write!(f, "HyperSCSI"),
            EtherType(0x88E1) => write!(f, "HomePlug1_0MME"),
            EtherType(0x88E5) => write!(f, "IEEE8021X"),
            EtherType(0x8892) => write!(f, "Profinet"),
            EtherType(0x88CC) => write!(f, "LLDP"),
            EtherType(0x88AB) => write!(f, "EthernetPowerlink"),
            EtherType(0x9000) => write!(f, "ECTP"),
            _ => write!(f, "EthType {}", self.0),
        }
    }
}
