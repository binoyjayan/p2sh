use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use super::ethernet::EtherType;
use crate::object::Object;

#[derive(Debug)]
pub struct VlanHeader {
    pub pcp: ClassOfService, // Priority Code Point
    pub dei: u8,             // Drop Eligible Indicator
    pub vlan_id: u16,        // VLAN Identifier
    pub ethertype: EtherType,
}

#[derive(Debug)]
pub struct Vlan {
    pub header: RefCell<VlanHeader>,
    pub rawdata: Rc<Vec<u8>>, // Raw data of the entire packet
    pub offset: usize,        // Offset of the VLAN header
}

#[allow(unused)]
pub const VLAN_HEADER_SIZE: usize = 4;

#[derive(Debug, Clone)]
pub struct ClassOfService(pub u8);

/// IEEE 802.1p classes
#[allow(unused, non_upper_case_globals, non_snake_case)]
pub mod ClassesOfService {
    use super::ClassOfService;
    pub const BestEffort: ClassOfService = ClassOfService(0);
    pub const Background: ClassOfService = ClassOfService(1);
    pub const ExcellentEffort: ClassOfService = ClassOfService(2);
    pub const CriticalApplications: ClassOfService = ClassOfService(3);
    pub const VideoVoiceApplications: ClassOfService = ClassOfService(4);
    pub const InternetworkControl: ClassOfService = ClassOfService(5);
    pub const NetworkControl: ClassOfService = ClassOfService(6);
    pub const Reserved: ClassOfService = ClassOfService(7);
}

impl From<ClassOfService> for u8 {
    fn from(item: ClassOfService) -> Self {
        item.0
    }
}

impl fmt::Display for Vlan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<id: {}, eth: {}>",
            self.header.borrow().vlan_id,
            self.header.borrow().ethertype
        )
    }
}

impl Vlan {
    #[allow(unused)]
    // off is the offset of the VLAN header when it is encapsulated in
    // another protocol. For example, if the VLAN header is encapsulated
    // in an 802.1ad QinQ header, then off is the offset of the QinQ header.
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + VLAN_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let pcp = ClassOfService(rawdata[off] >> 5);
        let dei = (rawdata[off] >> 4) & 1;
        let vlan_id = ((rawdata[0] as u16) << 8) | (rawdata[off + 1] as u16);
        let ethertype = EtherType(((rawdata[off + 2] as u16) << 8) | (rawdata[off + 3] as u16));
        let offset = off + VLAN_HEADER_SIZE;
        let header = RefCell::new(VlanHeader {
            pcp,
            dei,
            vlan_id,
            ethertype,
        });
        Ok(Self {
            header,
            rawdata,
            offset,
        })
    }
    pub fn get_pcp(&self) -> Rc<Object> {
        let pcp: u8 = self.header.borrow().pcp.clone().into();
        Rc::new(Object::Integer(pcp as i64))
    }
    pub fn get_dei(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().dei as i64))
    }
    pub fn get_vlan_id(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().vlan_id as i64))
    }
    pub fn get_ethertype(&self) -> Rc<Object> {
        Rc::new(Object::Str(self.header.borrow().ethertype.to_string()))
    }
    pub fn set_pcp(&self, pcp: Rc<Object>) -> Result<(), String> {
        match pcp.as_ref() {
            Object::Integer(pcp) => {
                if *pcp < 0 || *pcp > 7 {
                    return Err("Invalid value for VLAN property pcp".to_string());
                }
                self.header.borrow_mut().pcp = ClassOfService(*pcp as u8);
                Ok(())
            }
            _ => Err("Invalid value for VLAN property pcp".to_string()),
        }
    }
    pub fn set_dei(&self, dei: Rc<Object>) -> Result<(), String> {
        match dei.as_ref() {
            Object::Integer(dei) => {
                if *dei < 0 || *dei > 1 {
                    return Err("Invalid value for VLAN property dei".to_string());
                }
                self.header.borrow_mut().dei = *dei as u8;
                Ok(())
            }
            _ => Err("Invalid value for VLAN property dei".to_string()),
        }
    }
    pub fn set_vlan_id(&self, vlan_id: Rc<Object>) -> Result<(), String> {
        match vlan_id.as_ref() {
            Object::Integer(vlan_id) => {
                if *vlan_id < 0 || *vlan_id > 4095 {
                    return Err("Invalid value for VLAN property vlan_id".to_string());
                }
                self.header.borrow_mut().vlan_id = *vlan_id as u16;
                Ok(())
            }
            _ => Err("Invalid value for VLAN property vlan_id".to_string()),
        }
    }
    pub fn set_ethertype(&self, ethertype: Rc<Object>) -> Result<(), String> {
        match ethertype.as_ref() {
            Object::Integer(ethertype) => {
                if *ethertype < 0 || *ethertype > 65535 {
                    return Err("Invalid value for VLAN property ethertype".to_string());
                }
                self.header.borrow_mut().ethertype = EtherType(*ethertype as u16);
                Ok(())
            }
            _ => Err("Invalid value for VLAN property ethertype".to_string()),
        }
    }
}
