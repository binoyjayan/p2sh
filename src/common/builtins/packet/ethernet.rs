use super::error::PacketError;
use super::macaddress::MacAddress;
use std::fmt;

#[derive(Debug)]
pub struct Ethernet {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

impl fmt::Display for Ethernet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}: {} -> {}>",
            self.ethertype, self.source, self.destination
        )
    }
}

impl Ethernet {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 14 {
            return Err(PacketError::InvalidLength(bytes.len()));
        }
        let destination = MacAddress::from_bytes(&bytes[0..6]);
        let source = MacAddress::from_bytes(&bytes[6..12]);
        let ethertype = EtherType(((bytes[12] as u16) << 8) | (bytes[13] as u16));
        let payload = bytes[14..].to_vec();
        Ok(Self {
            destination,
            source,
            ethertype,
            payload,
        })
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
