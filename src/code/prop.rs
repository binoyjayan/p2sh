use std::fmt;

/// Packet object property type
/// To be kept in sync with 'PACKET_PROP_MAP'
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PacketPropType {
    Magic,
    Major,
    Minor,
    ThisZone,
    SigFlags,
    Snaplen,
    LinkType,
    Sec,
    USec,
    Caplen,
    Wirelen,
    Payload,
    Eth,
    Src,
    Dst,
    EtherType,
    Id,
    Priority,
    Dei,
    Ipv4,
    #[default]
    Invalid,
}

impl From<u8> for PacketPropType {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Magic,
            1 => Self::Major,
            2 => Self::Minor,
            3 => Self::ThisZone,
            4 => Self::SigFlags,
            5 => Self::Snaplen,
            6 => Self::LinkType,
            7 => Self::Sec,
            8 => Self::USec,
            9 => Self::Caplen,
            10 => Self::Wirelen,
            11 => Self::Payload,
            12 => Self::Eth,
            13 => Self::Src,
            14 => Self::Dst,
            15 => Self::EtherType,
            16 => Self::Id,
            17 => Self::Priority,
            18 => Self::Dei,
            19 => Self::Ipv4,
            _ => Self::Invalid,
        }
    }
}

impl From<PacketPropType> for u8 {
    fn from(code: PacketPropType) -> Self {
        code as u8
    }
}

impl fmt::Display for PacketPropType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_representation = match self {
            PacketPropType::Magic => "magic",
            PacketPropType::Major => "major",
            PacketPropType::Minor => "minor",
            PacketPropType::ThisZone => "thiszone",
            PacketPropType::SigFlags => "sigflags",
            PacketPropType::Snaplen => "snaplen",
            PacketPropType::LinkType => "linktype",
            PacketPropType::Sec => "sec",
            PacketPropType::USec => "usec",
            PacketPropType::Caplen => "caplen",
            PacketPropType::Wirelen => "wirelen",
            PacketPropType::Payload => "payload",
            PacketPropType::Eth => "eth",
            PacketPropType::Src => "src",
            PacketPropType::Dst => "dst",
            PacketPropType::EtherType => "ethertype",
            PacketPropType::Id => "id",
            PacketPropType::Priority => "priority",
            PacketPropType::Dei => "dei",
            PacketPropType::Ipv4 => "ipv4",
            PacketPropType::Invalid => "invalid",
        };
        write!(f, "{}", string_representation)
    }
}
