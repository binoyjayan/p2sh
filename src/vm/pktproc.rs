use std::rc::Rc;

use super::error::RTError;
use super::interpreter::VM;
use crate::builtins::packet::ethernet::Ethernet;
use crate::builtins::packet::vlan::Vlan;
use crate::builtins::pcap::Pcap;
use crate::builtins::pcap::PcapPacket;
use crate::code::prop::PacketPropType;
use crate::object::error::ErrorObj;
use crate::object::Object;

// define constant for max protool depth
pub const MAX_PROTO_DEPTH: usize = 4;

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
                        0x8100 => {
                            let obj = self.exec_prop_eth(eth.clone(), PacketPropType::Vlan, None, line)?;
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
                        0x8100 => {
                            let obj = self.exec_prop_vlan(vlan.clone(), PacketPropType::Vlan, None, line)?;
                            self.get_inner(&obj, depth - 1, line)?
                        }
                        _ => Rc::new(Object::Null),
                    }
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
            _ => {
                return Err(RTError::new("Object does not have any property", line));
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
            _ => {
                return Err(RTError::new(
                    &format!("Invalid vlan property '{}'", prop),
                    line,
                ));
            }
        };
        Ok(obj)
    }
}
