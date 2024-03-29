use std::rc::Rc;

use super::error::RTError;
use super::interpreter::VM;
use crate::builtins::pcap::Pcap;
use crate::builtins::pcap::PcapPacket;
use crate::builtins::protocols::ethernet::EtherTypes;
use crate::builtins::protocols::ethernet::Ethernet;
use crate::builtins::protocols::ipv4::Ipv4Packet;
use crate::builtins::protocols::ipv4::Protocols;
use crate::builtins::protocols::ipv6::Ipv6Packet;
use crate::builtins::protocols::ipv6::NextHeaders;
use crate::builtins::protocols::tcp::Tcp;
use crate::builtins::protocols::udp::Udp;
use crate::builtins::protocols::vlan::Vlan;
use crate::code::prop::PacketPropType;
use crate::object::array::Array;
use crate::object::error::ErrorObj;
use crate::object::Object;

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
                        EtherTypes::Ipv6 => {
                            let obj =
                                self.exec_prop_eth(eth.clone(), PacketPropType::Ipv6, None, line)?;
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
                        EtherTypes::Ipv6 => {
                            let obj = self.exec_prop_vlan(
                                vlan.clone(),
                                PacketPropType::Ipv6,
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
                        Protocols::Tcp => {
                            let obj =
                                self.exec_prop_ipv4(ipv4.clone(), PacketPropType::Tcp, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        Protocols::Ipv6 => {
                            let obj = self.exec_prop_ipv4(
                                ipv4.clone(),
                                PacketPropType::Ipv6,
                                None,
                                line,
                            )?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
                }
            }
            Object::Ipv6(ipv6) => {
                // Clone the wrapped Rc object so we do not get BorrowMutError
                let wrapped = ipv6.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    // Parse inner packet from bytes
                    match ipv6.get_next_header_raw() {
                        NextHeaders::Udp => {
                            let obj =
                                self.exec_prop_ipv6(ipv6.clone(), PacketPropType::Udp, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        NextHeaders::Tcp => {
                            let obj =
                                self.exec_prop_ipv6(ipv6.clone(), PacketPropType::Tcp, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
                }
            }
            Object::Udp(udp) => {
                let wrapped = udp.inner.borrow().clone();
                if let Some(inner) = wrapped.as_ref() {
                    self.get_inner(inner, depth - 1, line)?
                } else {
                    Rc::new(Object::Null)
                }
            }
            Object::Tcp(tcp) => {
                let wrapped = tcp.inner.borrow().clone();
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
            Object::Ipv6(ipv6) => self.exec_prop_ipv6(ipv6.clone(), prop, setval, line)?,
            Object::Udp(udp) => self.exec_prop_udp(udp.clone(), prop, setval, line)?,
            Object::Tcp(tcp) => self.exec_prop_tcp(tcp.clone(), prop, setval, line)?,
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
            PacketPropType::Eth => {
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
            PacketPropType::Payload => {
                // return the payload as bytes
                let payload = pkt.rawdata.borrow().clone();
                let mut elements = Vec::new();
                for byte in payload.iter() {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
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
            PacketPropType::Ipv6 => {
                if let Some(val) = setval {
                    eth.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = eth.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Ipv6Packet::from_bytes(
                        Rc::clone(&eth.rawdata.borrow()),
                        eth.offset,
                    ) {
                        Ok(ipv6) => Rc::new(Object::Ipv6(Rc::new(ipv6))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    eth.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Payload => {
                let payload = eth.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the ethernet header
                for byte in payload.iter().skip(eth.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
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
            PacketPropType::Payload => {
                let payload = vlan.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the vlan header
                for byte in payload.iter().skip(vlan.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
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
            PacketPropType::Tcp => {
                if let Some(val) = setval {
                    ipv4.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = ipv4.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Tcp::from_bytes(Rc::clone(&ipv4.rawdata.borrow()), ipv4.offset)
                    {
                        Ok(tcp) => Rc::new(Object::Tcp(Rc::new(tcp))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    ipv4.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Ipv6 => {
                if let Some(val) = setval {
                    ipv4.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = ipv4.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Ipv6Packet::from_bytes(
                        Rc::clone(&ipv4.rawdata.borrow()),
                        ipv4.offset,
                    ) {
                        Ok(ipv6) => Rc::new(Object::Ipv6(Rc::new(ipv6))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    ipv4.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Payload => {
                let payload = ipv4.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the ipv4 header
                for byte in payload.iter().skip(ipv4.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
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

    fn exec_prop_ipv6(
        &self,
        ipv6: Rc<Ipv6Packet>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::Version => {
                if setval.is_some() {
                    return Err(RTError::new("Cannot set ipv4 property version", line));
                } else {
                    ipv6.get_version()
                }
            }
            PacketPropType::TrafficClass => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_traffic_class(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_traffic_class()
                }
            }
            PacketPropType::FlowLabel => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_flow_label(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_flow_label()
                }
            }
            PacketPropType::Length => {
                // Payload length
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_payload_length(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_payload_length()
                }
            }
            PacketPropType::NextHeader => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_next_header(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_next_header()
                }
            }
            PacketPropType::HopLimit => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_hop_limit(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_hop_limit()
                }
            }
            PacketPropType::Src => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_src(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_src()
                }
            }
            PacketPropType::Dst => {
                if let Some(val) = setval {
                    if let Err(e) = ipv6.set_dst(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    ipv6.get_dst()
                }
            }
            PacketPropType::Udp => {
                if let Some(val) = setval {
                    ipv6.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = ipv6.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Udp::from_bytes(Rc::clone(&ipv6.rawdata.borrow()), ipv6.offset)
                    {
                        Ok(udp) => Rc::new(Object::Udp(Rc::new(udp))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    ipv6.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Tcp => {
                if let Some(val) = setval {
                    ipv6.inner.replace(Some(val.clone()));
                    val
                } else {
                    if let Some(inner) = ipv6.inner.borrow().as_ref() {
                        return Ok(inner.clone());
                    }
                    let obj = match Tcp::from_bytes(Rc::clone(&ipv6.rawdata.borrow()), ipv6.offset)
                    {
                        Ok(tcp) => Rc::new(Object::Tcp(Rc::new(tcp))),
                        Err(e) => Rc::new(Object::Err(ErrorObj::Packet(e))),
                    };
                    // Borrow the inner object again and replace its content
                    ipv6.inner.replace(Some(obj.clone()));
                    obj
                }
            }
            PacketPropType::Payload => {
                let payload = ipv6.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the ipv4 header
                for byte in payload.iter().skip(ipv6.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid ipv6 property '{}'", prop),
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
            PacketPropType::Payload => {
                let payload = udp.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the udp header
                for byte in payload.iter().skip(udp.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
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

    fn exec_prop_tcp(
        &self,
        tcp: Rc<Tcp>,
        prop: PacketPropType,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        let obj = match prop {
            PacketPropType::SrcPort => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_source_port(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_source_port()
                }
            }
            PacketPropType::DstPort => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_destination_port(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_destination_port()
                }
            }
            PacketPropType::Sequence => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_sequence(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_sequence()
                }
            }
            PacketPropType::Ack => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_ack(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_ack()
                }
            }
            PacketPropType::DataOffset | PacketPropType::Length => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_data_off(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_data_off()
                }
            }
            PacketPropType::Flags => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_flags(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_flags()
                }
            }
            PacketPropType::WindowSize => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_window_size(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_window_size()
                }
            }
            PacketPropType::Checksum => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_checksum(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_checksum()
                }
            }
            PacketPropType::Urgent => {
                if let Some(val) = setval {
                    if let Err(e) = tcp.set_urgent(val.clone()) {
                        return Err(RTError::new(&e, line));
                    }
                    val
                } else {
                    tcp.get_urgent()
                }
            }
            PacketPropType::Payload => {
                let payload = tcp.rawdata.borrow().clone();
                let mut elements = Vec::new();
                // start at offset 'offset' to skip the tcp header
                for byte in payload.iter().skip(tcp.offset) {
                    elements.push(Rc::new(Object::Byte(*byte)));
                }
                let arr = Array::new(elements);
                Rc::new(Object::Arr(Rc::new(arr)))
            }
            _ => {
                return Err(RTError::new(
                    &format!("Invalid tcp property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }
}
