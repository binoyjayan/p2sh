use std::cmp::Ordering;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use std::rc::Rc;

use crate::builtins::packet::ethernet::Ethernet;
use crate::builtins::packet::vlan::Vlan;
use crate::builtins::pcap::Pcap;
use crate::builtins::pcap::PcapPacket;
use crate::object::array::Array;
use crate::object::error::ErrorObj;
use crate::object::file::FileHandle;
use crate::object::func::BuiltinFunction;
use crate::object::func::Closure;
use crate::object::func::CompiledFunction;
use crate::object::hmap::HMap;

pub mod array;
pub mod error;
pub mod file;
pub mod func;
pub mod hmap;

#[derive(Debug)]
pub enum Object {
    Null,
    Str(String),
    Char(char),
    Byte(u8),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Return(Rc<Object>),
    Builtin(Rc<BuiltinFunction>),
    Func(Rc<CompiledFunction>),
    Arr(Rc<Array>),
    Map(Rc<HMap>),
    Clos(Rc<Closure>),
    File(Rc<FileHandle>),
    Err(ErrorObj),
    Pcap(Rc<Pcap>),
    Packet(Rc<PcapPacket>),
    Eth(Rc<Ethernet>),
    Vlan(Rc<Vlan>),
}

impl From<&Object> for Vec<u8> {
    fn from(obj: &Object) -> Self {
        match obj {
            Object::Null => Vec::new(),
            Object::Str(v) => v.as_bytes().to_vec(),
            Object::Char(v) => v.to_string().as_bytes().to_vec(),
            Object::Byte(v) => vec![*v],
            Object::Integer(v) => v.to_be_bytes().to_vec(),
            Object::Float(v) => v.to_be_bytes().to_vec(),
            Object::Bool(v) => vec![*v as u8],
            Object::Arr(v) => v.as_ref().into(),
            Object::Map(v) => v.as_ref().into(),
            Object::Packet(v) => v.as_ref().into(),
            Object::Eth(v) => v.as_ref().into(),
            Object::Vlan(v) => v.as_ref().into(),
            _ => Vec::new(),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Null, Object::Null) => true,
            (Object::Str(a), Object::Str(b)) => a.eq(b),
            (Object::Char(a), Object::Char(b)) => a.eq(b),
            (Object::Byte(a), Object::Byte(b)) => a.eq(b),
            (Object::Integer(a), Object::Integer(b)) => a.eq(b),
            (Object::Integer(a), Object::Float(b)) => (*a as f64).eq(b),
            (Object::Float(a), Object::Integer(b)) => a.eq(&(*b as f64)),
            (Object::Float(a), Object::Float(b)) => a.eq(b),
            (Object::Bool(a), Object::Bool(b)) => a.eq(b),
            (Object::Arr(a), Object::Arr(b)) => a.eq(b),
            (Object::Map(a), Object::Map(b)) => a.eq(b),
            (Object::Builtin(a), Object::Builtin(b)) => a.eq(b),
            (Object::Func(a), Object::Func(b)) => a.eq(b),
            (Object::Clos(a), Object::Clos(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl Eq for Object {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Null, Object::Null) => None,
            (Object::Str(a), Object::Str(b)) => a.partial_cmp(b),
            (Object::Char(a), Object::Char(b)) => a.partial_cmp(b),
            (Object::Byte(a), Object::Byte(b)) => a.partial_cmp(b),
            (Object::Integer(a), Object::Integer(b)) => a.partial_cmp(b),
            (Object::Float(a), Object::Float(b)) => a.partial_cmp(b),
            (Object::Bool(a), Object::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Handle the case where the objects are not comparable.
        // For simplicity, we'll just return Ordering::Equal.
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Object {
    pub fn is_null(&self) -> bool {
        matches!(self, Object::Null)
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Object::Str(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Object::Integer(_) | Object::Float(_))
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Object::Integer(n) => *n == 0,
            Object::Float(n) => *n == 0.,
            Object::Byte(n) => *n == 0,
            _ => false,
        }
    }
    pub fn is_falsey(&self) -> bool {
        match self {
            Object::Bool(false) | Object::Integer(0) | Object::Null => true,
            // floating point types cannot be used in patterns
            Object::Float(v) => *v == 0.,
            Object::Char(c) => *c == '\0',
            Object::Byte(b) => *b == 0,
            Object::Str(s) => s.is_empty(),
            Object::Arr(a) => a.elements.borrow().is_empty(),
            Object::Map(m) => m.pairs.borrow().is_empty(),
            _ => false,
        }
    }

    pub fn is_a_valid_key(&self) -> bool {
        matches!(
            self,
            Object::Str(_)
                | Object::Char(_)
                | Object::Byte(_)
                | Object::Integer(_)
                | Object::Float(_)
                | Object::Bool(_)
                | Object::Null
                | Object::Builtin(_)
                | Object::Arr(_)
        )
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Object::Err(_))
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Null => write!(f, "null"),
            // Wrap the string in quotes while printing string expressions
            // But do not use this in builtins "puts" and "print"
            Self::Str(s) => write!(f, r#""{}""#, s),
            Self::Char(c) => write!(f, "'{}'", c),
            Self::Byte(b) => write!(f, "{}", b),
            Self::Integer(val) => write!(f, "{}", val),
            Self::Float(val) => write!(f, "{}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::Return(val) => write!(f, "{}", val),
            Self::Builtin(val) => write!(f, "{}", val),
            Self::Func(val) => write!(f, "{}", val),
            Self::Arr(val) => write!(f, "{}", val),
            Self::Map(val) => write!(f, "{}", val),
            Self::Clos(val) => write!(f, "{}", val),
            Self::File(val) => write!(f, "{}", val),
            Self::Err(val) => write!(f, "{}", val),
            Self::Pcap(val) => write!(f, "{}", val),
            Self::Packet(val) => write!(f, "{}", val),
            Self::Eth(val) => write!(f, "{}", val),
            Self::Vlan(val) => write!(f, "{}", val),
        }
    }
}

impl ops::Add for &Object {
    type Output = Object;

    fn add(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a + b),
            (&Object::Float(a), &Object::Float(b)) => Object::Float(a + b),
            (&Object::Integer(a), &Object::Float(b)) => Object::Float(a as f64 + b),
            (&Object::Float(a), &Object::Integer(b)) => Object::Float(a + b as f64),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a + b),
            (&Object::Integer(a), &Object::Byte(b)) => Object::Integer(a + b as i64),
            (&Object::Byte(a), &Object::Integer(b)) => Object::Integer(a as i64 + b),
            (&Object::Float(a), &Object::Byte(b)) => Object::Float(a + b as f64),
            (&Object::Byte(a), &Object::Float(b)) => Object::Float(a as f64 + b),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::Sub for &Object {
    type Output = Object;
    fn sub(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a - b),
            (&Object::Float(a), &Object::Float(b)) => Object::Float(a - b),
            (&Object::Integer(a), &Object::Float(b)) => Object::Float(a as f64 - b),
            (&Object::Float(a), &Object::Integer(b)) => Object::Float(a - b as f64),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a - b),
            (&Object::Integer(a), &Object::Byte(b)) => Object::Integer(a - b as i64),
            (&Object::Byte(a), &Object::Integer(b)) => Object::Integer(a as i64 - b),
            (&Object::Float(a), &Object::Byte(b)) => Object::Float(a - b as f64),
            (&Object::Byte(a), &Object::Float(b)) => Object::Float(a as f64 - b),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::Mul for &Object {
    type Output = Object;
    fn mul(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a * b),
            (&Object::Float(a), &Object::Float(b)) => Object::Float(a * b),
            (&Object::Integer(a), &Object::Float(b)) => Object::Float(a as f64 * b),
            (&Object::Float(a), &Object::Integer(b)) => Object::Float(a * b as f64),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a * b),
            (&Object::Integer(a), &Object::Byte(b)) => Object::Integer(a * b as i64),
            (&Object::Byte(a), &Object::Integer(b)) => Object::Integer(a as i64 * b),
            (&Object::Float(a), &Object::Byte(b)) => Object::Float(a * b as f64),
            (&Object::Byte(a), &Object::Float(b)) => Object::Float(a as f64 * b),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::Div for &Object {
    type Output = Object;
    fn div(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a / b),
            (&Object::Float(a), &Object::Float(b)) => Object::Float(a / b),
            (&Object::Integer(a), &Object::Float(b)) => Object::Float(a as f64 / b),
            (&Object::Float(a), &Object::Integer(b)) => Object::Float(a / b as f64),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a / b),
            (&Object::Integer(a), &Object::Byte(b)) => Object::Integer(a / b as i64),
            (&Object::Byte(a), &Object::Integer(b)) => Object::Integer(a as i64 / b),
            (&Object::Float(a), &Object::Byte(b)) => Object::Float(a / b as f64),
            (&Object::Byte(a), &Object::Float(b)) => Object::Float(a as f64 / b),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::Rem for &Object {
    type Output = Object;
    fn rem(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a % b),
            (&Object::Float(a), &Object::Float(b)) => Object::Float(a % b),
            (&Object::Integer(a), &Object::Float(b)) => Object::Float(a as f64 % b),
            (&Object::Float(a), &Object::Integer(b)) => Object::Float(a % b as f64),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a % b),
            (&Object::Integer(a), &Object::Byte(b)) => Object::Integer(a % b as i64),
            (&Object::Byte(a), &Object::Integer(b)) => Object::Integer(a as i64 % b),
            (&Object::Float(a), &Object::Byte(b)) => Object::Float(a % b as f64),
            (&Object::Byte(a), &Object::Float(b)) => Object::Float(a as f64 % b),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::Neg for &Object {
    type Output = Object;
    fn neg(self) -> Object {
        match *self {
            Object::Integer(a) => Object::Integer(-a),
            Object::Float(f) => Object::Float(-f),
            _ => panic!("Invalid binary operation"),
        }
    }
}

impl ops::BitAnd for &Object {
    type Output = Object;

    fn bitand(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a & b),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a & b),
            _ => panic!("Invalid bitwise operation"),
        }
    }
}

impl ops::BitOr for &Object {
    type Output = Object;

    fn bitor(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a | b),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a | b),
            _ => panic!("Invalid bitwise operation"),
        }
    }
}

impl ops::BitXor for &Object {
    type Output = Object;

    fn bitxor(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Integer(a), &Object::Integer(b)) => Object::Integer(a ^ b),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a ^ b),
            _ => panic!("Invalid bitwise operation"),
        }
    }
}

impl ops::Shl<&Object> for &Object {
    type Output = Object;

    fn shl(self, rhs: &Object) -> Object {
        match (self, rhs) {
            (&Object::Integer(a), Object::Integer(b)) => Object::Integer(a << b),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a << b),
            _ => panic!("Invalid bitwise operation"),
        }
    }
}

impl ops::Shr<&Object> for &Object {
    type Output = Object;

    fn shr(self, rhs: &Object) -> Object {
        match (self, rhs) {
            (&Object::Integer(a), Object::Integer(b)) => Object::Integer(a >> b),
            (&Object::Byte(a), &Object::Byte(b)) => Object::Byte(a >> b),
            _ => panic!("Invalid bitwise operation"),
        }
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(ref n) => n.hash(state),
            Object::Char(ref ch) => ch.hash(state),
            Object::Byte(ref b) => b.hash(state),
            Object::Float(ref f) => {
                // Use the built-in hash function for f64
                state.write_u64(f.to_bits());
            }
            Object::Bool(ref b) => b.hash(state),
            Object::Str(ref s) => s.hash(state),
            Object::Builtin(f) => f.name.hash(state),
            Object::Arr(ref a) => a.hash(state),
            _ => "".hash(state),
        }
    }
}
