use std::rc::Rc;

use super::error::RTError;
use super::interpreter::VM;
use crate::builtins::packet::ethernet::EtherTypes;
use crate::builtins::packet::ethernet::Ethernet;
use crate::builtins::packet::ipv4::Ipv4Packet;
use crate::builtins::packet::ipv4::Protocols;
use crate::builtins::packet::udp::Udp;
use crate::builtins::packet::vlan::Vlan;
use crate::builtins::pcap::Pcap;
use crate::builtins::pcap::PcapPacket;
use crate::code::prop::PacketPropType;
use crate::object::error::ErrorObj;
use crate::object::Object;

// define constant for max protool depth
pub const MAX_PROTO_DEPTH: usize = 10;

impl VM {
    pub fn get_inner(
        &self,
        obj: &Rc<Object>,
        depth: usize,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        if depth == 0 {
            return Ok(obj.clone());
        }
        let obj = match obj.as_ref() {
            Object::Packet(pkt) => {
                // Clone the wrapped Rc object so we do not get BorrowMutError
                let wrapped = pkt.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    // Parse inner packet from bytes
                    let obj =
                        self.exec_prop_packet(pkt.clone(), PacketPropType::Eth, None, line)?;
                    self.get_inner(&obj, depth - 1, line)?
                }
            }
            Object::Eth(eth) => {
                // Clone the wrapped Rc object so we do not get BorrowMutError
                let wrapped = eth.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    // Parse inner packet from bytes
                    match eth.get_ethertype_raw() {
                        EtherTypes::Vlan => {
                            let obj =
                                self.exec_prop_eth(eth.clone(), PacketPropType::Vlan, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        EtherTypes::Ipv4 => {
                            let obj =
                                self.exec_prop_eth(eth.clone(), PacketPropType::Ipv4, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
                }
            }
            Object::Vlan(vlan) => {
                let wrapped = vlan.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    // Parse inner packet from bytes
                    match vlan.get_ethertype_raw() {
                        EtherTypes::Vlan => {
                            let obj = self.exec_prop_vlan(
                                vlan.clone(),
                                PacketPropType::Vlan,
                                None,
                                line,
                            )?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        EtherTypes::Ipv4 => {
                            let obj = self.exec_prop_vlan(
                                vlan.clone(),
                                PacketPropType::Ipv4,
                                None,
                                line,
                            )?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
                }
            }
            Object::Ipv4(ipv4) => {
                // Clone the wrapped Rc object so we do not get BorrowMutError
                let wrapped = ipv4.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    // Parse inner packet from bytes
                    match ipv4.get_protocol_raw() {
                        Protocols::Udp => {
                            let obj =
                                self.exec_prop_ipv4(ipv4.clone(), PacketPropType::Udp, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
                }
            }
            Object::Udp(udp) => {
                // Clone the wrapped Rc object so we do not get BorrowMutError
                let wrapped = udp.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    Rc::new(Object::Null)
                }
            }
            _ => obj.clone(),
        };
        Ok(obj)
    }

    /// Execute a property expression
    /// left: The object on which the property is being accessed
    /// prop: The property being accessed
    /// setval: The value to be set if this is a SetProp operation
    /// line: The line number of the property expression
    /// Returns: Ok(()) if the property expression is executed successfully
    pub fn exec_prop_expr(
        &self,
        left: Rc<Object>,
        prop: u8,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let prop: PacketPropType = PacketPropType::from(prop);
        let obj = match left.as_ref() {
            Object::Pcap(pcap) => self.exec_prop_pcap(pcap.clone(), prop, setval, line)?,
            Object::Packet(pkt) => self.exec_prop_packet(pkt.clone(), prop, setval, line)?,
            Object::Eth(eth) => self.exec_prop_eth(eth.clone(), prop, setval, line)?,
            Object::Vlan(v) => self.exec_prop_vlan(v.clone(), prop, setval, line)?,
            Object::Ipv4(ipv4) => self.exec_prop_ipv4(ipv4.clone(), prop, setval, line)?,
            Object::Udp(udp) => self.exec_prop_udp(udp.clone(), prop, setval, line)?,
            _ => {
                let msg = format!("{}: Object does not have any property", left);
                return Err(RTError::new(&msg, line));
            }
        };
        Ok(obj)
    }

    /// Execute a pcap property expression
    /// pcap: The pcap on which the property is being accessed
    /// prop: The property being accessed
    /// setval: The value to be set if this is a SetProp operation
    /// line: The line number of the property expression
    /// Returns: Ok(obj) if the property expression is executed successfully
    fn exec_prop_pcap(
        &self,
        pcap: Rc<Pcap>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Magic => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_magic_number(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_magic_number()
                }
            }
            PacketPropType::Major => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_version_major(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_version_major()
                }
            }
            PacketPropType::Minor => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_version_minor(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_version_minor()
                }
            }
            PacketPropType::ThisZone => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_thiszone(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_thiszone()
                }
            }
            PacketPropType::SigFigs => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_sigfigs(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_sigfigs()
                }
            }
            PacketPropType::Snaplen => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_snaplen(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_snaplen()
                }
            }
            PacketPropType::LinkType => {
                if let Some(val) = setval {
                    if let Err(e) = pcap.set_linktype(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pcap.get_linktype()
                }
            }

            _ => {
                return Err(RTError::new("Invalid pcap property", line));
            }
        };
        Ok(obj)
    }

    /// Execute a packet property expression
    /// pkt: The packet on which the property is being accessed
    /// prop: The property being accessed
    /// setval: The value to be set if this is a SetProp operation
    /// line: The line number of the property expression
    /// Returns: Ok(obj) if the property expression is executed successfully
    pub fn exec_prop_packet(
        &self,
        pkt: Rc<PcapPacket>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Sec => {
                if let Some(val) = setval {
                    if let Err(e) = pkt.set_ts_sec(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pkt.get_ts_sec()
                }
            }
            PacketPropType::USec => {
                if let Some(val) = setval {
                    if let Err(e) = pkt.set_ts_usec(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pkt.get_ts_usec()
                }
            }
            PacketPropType::Caplen => {
                if let Some(val) = setval {
                    if let Err(e) = pkt.set_caplen(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pkt.get_caplen()
                }
            }
            PacketPropType::Wirelen => {
                if let Some(val) = setval {
                    if let Err(e) = pkt.set_wirelen(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    pkt.get_wirelen()
                }
            }
            PacketPropType::Eth | PacketPropType::Payload => {
                if let Some(val) = setval {
                    pkt.inner.replace(Some(val.clone()));
                    val
                } else {
                    // Borrow the inner object and return the cloned object
                    // immediately so the borrowing is kept to the scope of the
                    // if let statement. This allows us to borrow the inner object
                    // again in the following statement to create a new object
                    // and replace the inner object with it.
                    if let Some(inner) = pkt.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Ethernet::from_bytes(Rc::clone(&pkt.rawdata.borrow()), 0) {
                        Ok(ethernet) => Rc::new(Object::Eth(Rc::new(ethernet))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    pkt.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid packet property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }

    /// Execute an ethernet property expression
    /// eth: The pcap on which the property is being accessed
    /// prop: The property being accessed
    /// setval: The value to be set if this is a SetProp operation
    /// line: The line number of the property expression
    /// Returns: Ok(obj) if the property expression is executed successfully
    fn exec_prop_eth(
        &self,
        eth: Rc<Ethernet>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Dst => {
                if let Some(val) = setval {
                    if let Err(e) = eth.set_dst(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    eth.get_dst()
                }
            }
            PacketPropType::Src => {
                if let Some(val) = setval {
                    if let Err(e) = eth.set_src(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    eth.get_src()
                }
            }
            PacketPropType::EtherType => {
                if let Some(val) = setval {
                    if let Err(e) = eth.set_ethertype(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    eth.get_ethertype()
                }
            }
            PacketPropType::Vlan => {
                if let Some(val) = setval {
                    eth.inner.replace(Some(val.clone()));
                    val
                } else {
                    // Borrow the inner object and return the cloned object
                    // immediately so the borrowing is kept to the scope of the
                    // if let statement. This allows us to borrow the inner object
                    // again in the following statement to create a new object
                    // and replace the inner object with it.
                    if let Some(inner) = eth.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Vlan::from_bytes(Rc::clone(&eth.rawdata.borrow()), eth.offset) {
                        Ok(vlan) => Rc::new(Object::Vlan(Rc::new(vlan))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    eth.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Ipv4 => {
                if let Some(val) = setval {
                    eth.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = eth.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Ipv4Packet::from_bytes(
                        Rc::clone(&eth.rawdata.borrow()),
                        eth.offset,
                    ) {
                        Ok(ipv4) => Rc::new(Object::Ipv4(Rc::new(ipv4))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    eth.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid ethernet property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }

    fn exec_prop_vlan(
        &self,
        vlan: Rc<Vlan>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Priority => {
                if let Some(val) = setval {
                    if let Err(e) = vlan.set_priority(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    vlan.get_priority()
                }
            }
            PacketPropType::Dei => {
                if let Some(val) = setval {
                    if let Err(e) = vlan.set_dei(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    vlan.get_dei()
                }
            }
            PacketPropType::Id => {
                if let Some(val) = setval {
                    if let Err(e) = vlan.set_vlan_id(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    vlan.get_vlan_id()
                }
            }
            PacketPropType::EtherType => {
                if let Some(val) = setval {
                    if let Err(e) = vlan.set_ethertype(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    vlan.get_ethertype()
                }
            }
            PacketPropType::Vlan => {
                if let Some(val) = setval {
                    vlan.inner.replace(Some(val.clone()));
                    val
                } else {
                    // Borrow the inner object and return the cloned object
                    // immediately so the borrowing is kept to the scope of the
                    // if let statement. This allows us to borrow the inner object
                    // again in the following statement to create a new object
                    // and replace the inner object with it.
                    if let Some(inner) = vlan.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Vlan::from_bytes(Rc::clone(&vlan.rawdata.borrow()), vlan.offset)
                    {
                        Ok(vlan) => Rc::new(Object::Vlan(Rc::new(vlan))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    vlan.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Ipv4 => {
                if let Some(val) = setval {
                    vlan.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = vlan.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Ipv4Packet::from_bytes(
                        Rc::clone(&vlan.rawdata.borrow()),
                        vlan.offset,
                    ) {
                        Ok(ipv4) => Rc::new(Object::Ipv4(Rc::new(ipv4))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    vlan.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid vlan property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }

    fn exec_prop_ipv4(
        &self,
        ipv4: Rc<Ipv4Packet>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Version => {
                if setval.is_some() {
                    return Err(RTError::new("Cannot set ipv4 property version", line));
                } else {
                    ipv4.get_version()
                }
            }
            PacketPropType::Ihl => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_ihl(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_ihl()
                }
            }
            PacketPropType::TotalLength => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_total_length(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_total_length()
                }
            }
            PacketPropType::Id => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_identification(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_identification()
                }
            }
            PacketPropType::Dscp => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_dscp(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_dscp()
                }
            }
            PacketPropType::Ecn => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_ecn(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_ecn()
                }
            }
            PacketPropType::Flags => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_flags(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_flags()
                }
            }
            PacketPropType::FragmentOffset => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_fragment_offset(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_fragment_offset()
                }
            }
            PacketPropType::Ttl => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_ttl(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_ttl()
                }
            }
            PacketPropType::Protocol => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_protocol(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_protocol()
                }
            }
            PacketPropType::Checksum => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_checksum(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_checksum()
                }
            }
            PacketPropType::Src => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_src(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_src()
                }
            }
            PacketPropType::Dst => {
                if let Some(val) = setval {
                    if let Err(e) = ipv4.set_dst(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv4.get_dst()
                }
            }
            PacketPropType::Udp => {
                if let Some(val) = setval {
                    ipv4.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = ipv4.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Udp::from_bytes(Rc::clone(&ipv4.rawdata.borrow()), ipv4.offset)
                    {
                        Ok(udp) => Rc::new(Object::Udp(Rc::new(udp))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    ipv4.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid ipv4 property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }

    fn exec_prop_udp(
        &self,
        udp: Rc<Udp>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::SrcPort => {
                if let Some(val) = setval {
                    if let Err(e) = udp.set_source_port(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    udp.get_source_port()
                }
            }
            PacketPropType::DstPort => {
                if let Some(val) = setval {
                    if let Err(e) = udp.set_destination_port(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    udp.get_destination_port()
                }
            }
            PacketPropType::Length => {
                if let Some(val) = setval {
                    if let Err(e) = udp.set_length(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    udp.get_length()
                }
            }
            PacketPropType::Checksum => {
                if let Some(val) = setval {
                    if let Err(e) = udp.set_checksum(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    udp.get_checksum()
                }
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid udp property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }
}
