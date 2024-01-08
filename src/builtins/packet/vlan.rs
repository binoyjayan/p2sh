use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use super::ethernet::EtherType;
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct VlanHeader {
    priority: ClassOfService, // Priority Code Point [PCP]
    dei: bool,                // Drop Eligible Indicator
    vlan_id: u16,             // VLAN Identifier
    ethertype: EtherType,
}

impl From<&VlanHeader> for Vec<u8> {
    fn from(hdr: &VlanHeader) -> Self {
        let mut bytes = Vec::new();
        let prio_dei: u8 = (hdr.priority.0 << 5) | ((hdr.dei as u8) << 4);
        let msb_vlan_id = ((hdr.vlan_id >> 8) & 0x0F) as u8; // 4 msb of vlan-id
        let lsb_vlan_id = (hdr.vlan_id & 0xFF) as u8; // 8 lsb of vlan-id
        bytes.push(prio_dei | msb_vlan_id);
        bytes.push(lsb_vlan_id);
        bytes.extend_from_slice(&hdr.ethertype.0.to_be_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Vlan {
    header: RefCell<VlanHeader>,
    pub rawdata: RefCell<Rc<Vec<u8>>>, // Raw data of the entire packet
    pub offset: usize,                 // Offset of the VLAN header
    pub inner: RefCell<Option<Rc<Object>>>, // Inner packet
}

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

impl fmt::Display for ClassOfService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
            "<id:{},type:{}>",
            self.header.borrow().vlan_id,
            self.header.borrow().ethertype,
        )?;
        if let Some(inner) = self.inner.borrow().clone() {
            write!(f, " {}", inner)
        } else {
            write!(f, " [len: {}]", self.rawdata.borrow().len())
        }
    }
}

impl From<&Vlan> for Vec<u8> {
    fn from(vlan: &Vlan) -> Self {
        let header = vlan.header.borrow().clone();
        let mut bytes: Vec<u8> = (&header).into();
        if let Some(inner) = vlan.inner.borrow().clone() {
            let b: Vec<u8> = inner.as_ref().into();
            bytes.extend_from_slice(&b);
        } else {
            let data = vlan.rawdata.borrow().clone();
            bytes.extend_from_slice(&data[vlan.offset..]);
        }
        bytes
    }
}

impl Vlan {
    // off is the offset of the VLAN header when it is encapsulated in
    // another protocol. For example, if the VLAN header is encapsulated
    // in an 802.1ad QinQ header, then off is the offset of the QinQ header.
    pub fn from_bytes(rawdata: Rc<Vec<u8>>, off: usize) -> Result<Self, PacketError> {
        if rawdata.len() < off + VLAN_HEADER_SIZE {
            return Err(PacketError::InvalidLength(rawdata.len()));
        }
        let priority = ClassOfService(rawdata[off] >> 5);
        let dei = (rawdata[off] >> 4) & 1 == 1;
        let vlan_id = (((rawdata[off] as u16) & 0x0F) << 8) | (rawdata[off + 1] as u16);
        let ethertype = EtherType(((rawdata[off + 2] as u16) << 8) | (rawdata[off + 3] as u16));
        let offset = off + VLAN_HEADER_SIZE;
        let header = RefCell::new(VlanHeader {
            priority,
            dei,
            vlan_id,
            ethertype,
        });
        Ok(Self {
            header,
            rawdata: RefCell::new(rawdata),
            offset,
            inner: RefCell::new(None),
        })
    }
    pub fn get_ethertype_raw(&self) -> u16 {
        self.header.borrow().ethertype.0
    }
    pub fn get_priority(&self) -> Rc<Object> {
        let priority: u8 = self.header.borrow().priority.clone().into();
        Rc::new(Object::Integer(priority as i64))
    }
    pub fn get_dei(&self) -> Rc<Object> {
        Rc::new(Object::Bool(self.header.borrow().dei))
    }
    pub fn get_vlan_id(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().vlan_id as i64))
    }
    pub fn get_ethertype(&self) -> Rc<Object> {
        Rc::new(Object::Integer(self.header.borrow().ethertype.0 as i64))
    }
    pub fn set_priority(&self, priority: Rc<Object>) -> Result<(), String> {
        match priority.as_ref() {
            Object::Integer(priority) => {
                if *priority < 0 || *priority > 7 {
                    return Err("Invalid value for VLAN property priority".to_string());
                }
                self.header.borrow_mut().priority = ClassOfService(*priority as u8);
                Ok(())
            }
            _ => Err("Invalid value for VLAN property priority".to_string()),
        }
    }
    pub fn set_dei(&self, dei: Rc<Object>) -> Result<(), String> {
        match dei.as_ref() {
            Object::Bool(dei) => {
                self.header.borrow_mut().dei = *dei;
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
