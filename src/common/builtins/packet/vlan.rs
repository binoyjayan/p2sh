use std::fmt;
use std::rc::Rc;

use super::error::PacketError;
use super::ethernet::EtherType;

#[derive(Debug)]
pub struct Vlan {
    pub pcp: ClassOfService, // Priority Code Point
    pub dei: u8,             // Drop Eligible Indicator
    pub vlan_id: u16,        // VLAN Identifier
    pub ethertype: EtherType,
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
        write!(f, "<id: {}, eth: {}>", self.vlan_id, self.ethertype)
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
        Ok(Self {
            pcp,
            dei,
            vlan_id,
            ethertype,
            rawdata,
            offset,
        })
    }
}
