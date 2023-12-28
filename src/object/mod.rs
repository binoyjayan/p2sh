use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::ops;
use std::rc::Rc;

use crate::builtins::packet::error::PacketError;
use crate::builtins::packet::ethernet::Ethernet;
use crate::builtins::packet::vlan::Vlan;
use crate::builtins::pcap::Pcap;
use crate::builtins::pcap::PcapPacket;
use crate::code::definitions::Instructions;

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
            _ => "".hash(state),
        }
    }
}

pub type BuiltinFunctionProto = fn(Vec<Rc<Object>>) -> Result<Rc<Object>, String>;

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: &'static str,
    pub func: BuiltinFunctionProto,
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl BuiltinFunction {
    pub const fn new(name: &'static str, func: BuiltinFunctionProto) -> BuiltinFunction {
        BuiltinFunction { name, func }
    }
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
pub struct Array {
    pub elements: RefCell<Vec<Rc<Object>>>,
}

impl Array {
    pub fn new(elements: Vec<Rc<Object>>) -> Self {
        Self {
            elements: RefCell::new(elements),
        }
    }
    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.elements.borrow().is_empty()
    }
    pub fn get(&self, idx: usize) -> Rc<Object> {
        match self.elements.borrow().get(idx) {
            Some(value) => value.clone(),
            None => Rc::new(Object::Null),
        }
    }
    pub fn last(&self) -> Rc<Object> {
        match self.elements.borrow().last() {
            Some(value) => value.clone(),
            None => Rc::new(Object::Null),
        }
    }
    pub fn push(&self, obj: Rc<Object>) {
        self.elements.borrow_mut().push(obj);
    }
    pub fn set(&self, idx: usize, obj: Rc<Object>) {
        self.elements.borrow_mut()[idx] = obj;
    }
}

#[derive(Debug, Default)]
pub struct HMap {
    pub pairs: RefCell<HashMap<Rc<Object>, Rc<Object>>>,
}

impl HMap {
    pub fn new(pairs: HashMap<Rc<Object>, Rc<Object>>) -> Self {
        Self {
            pairs: RefCell::new(pairs),
        }
    }
    pub fn len(&self) -> usize {
        self.pairs.borrow().len()
    }
    pub fn get(&self, key: &Rc<Object>) -> Rc<Object> {
        match self.pairs.borrow().get(key) {
            Some(value) => value.clone(),
            None => Rc::new(Object::Null),
        }
    }
    pub fn contains(&self, key: &Rc<Object>) -> bool {
        self.pairs.borrow().contains_key(key)
    }
    pub fn insert(&self, key: Rc<Object>, val: Rc<Object>) -> Rc<Object> {
        match self.pairs.borrow_mut().insert(key, val) {
            Some(v) => v,
            None => Rc::new(Object::Null),
        }
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elements_str = self
            .elements
            .borrow()
            .iter()
            .fold(String::new(), |mut acc, p| {
                let _ = write!(&mut acc, "{}, ", p);
                acc
            });
        let elements_str = elements_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "[{}]", elements_str)
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        let self_elements = self.elements.borrow();
        let other_elements = other.elements.borrow();

        if self_elements.len() != other_elements.len() {
            return false;
        }

        for (a, b) in self_elements.iter().zip(other_elements.iter()) {
            if *a != *b {
                return false;
            }
        }
        true
    }
}

impl Eq for Array {}

impl fmt::Display for HMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pairs_str = self
            .pairs
            .borrow()
            .iter()
            .fold(String::new(), |mut acc, (k, v)| {
                let _ = write!(&mut acc, "{}: {}, ", k, v);
                acc
            });
        let pairs_str = pairs_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "map {{{}}}", pairs_str)
    }
}

// compare HMap objects without considering the order of key-value pairs
impl PartialEq for HMap {
    fn eq(&self, other: &Self) -> bool {
        let self_pairs = self.pairs.borrow();
        let other_pairs = other.pairs.borrow();

        if self_pairs.len() != other_pairs.len() {
            return false;
        }

        for (key, value) in self_pairs.iter() {
            if let Some(other_value) = other_pairs.get(key) {
                if value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Eq for HMap {}

// Hold the instructions of a compiled function and to pass them
// from the compiler to the VM as part of the bytecode, as a constant
// OpCall tells the VM to start executing an object of type CompiledFunction
// sitting on top of the stack.
// OpReturnValue tells the VM to return the value on top of the stack
// to the calling context.
// OpReturn is similar to OpReturnValue except that it returns Null.
#[derive(Debug, Default)]
pub struct CompiledFunction {
    pub instructions: Rc<Instructions>,
    pub num_locals: usize,
    pub num_params: usize,
}

impl CompiledFunction {
    pub fn new(instructions: Instructions, num_locals: usize, num_params: usize) -> Self {
        Self {
            instructions: Rc::new(instructions),
            num_locals,
            num_params,
        }
    }
}

impl fmt::Display for CompiledFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<compiled function>")
    }
}

impl PartialEq for CompiledFunction {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
    }
}

impl Eq for CompiledFunction {}

// A closure object has a pointer to the function it wraps, a function, and
// a place to keep the free variables it carries around, 'free'. This object
// is used to represent functions that 'close over' their environment at the
// time of their definition. The environment here is captured in a vector of
// Object's. Note that closures are only created at runtime and aren't
// available to the compiler. Instead an opcode 'OpClosure' is used by the
// compiler to inform the VM to create a closure and wrap the function and
// its environment.
#[derive(Debug, Default)]
pub struct Closure {
    pub func: Rc<CompiledFunction>,
    pub free: RefCell<Vec<Rc<Object>>>,
}

impl Closure {
    pub fn new(func: Rc<CompiledFunction>, free: Vec<Rc<Object>>) -> Self {
        Self {
            func,
            free: RefCell::new(free),
        }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<closure>")
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.func == other.func
    }
}

#[derive(Debug)]
pub enum FileHandle {
    Reader(RefCell<io::BufReader<fs::File>>),
    Writer(RefCell<io::BufWriter<fs::File>>),
    Stdin,
    Stdout,
    Stderr,
}

impl FileHandle {
    pub fn new_reader(reader: io::BufReader<fs::File>) -> Self {
        Self::Reader(RefCell::new(reader))
    }
    pub fn new_writer(writer: io::BufWriter<fs::File>) -> Self {
        Self::Writer(RefCell::new(writer))
    }
}

impl fmt::Display for FileHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Reader(_) => write!(f, "<file: reader>"),
            Self::Writer(_) => write!(f, "<file: writer>"),
            Self::Stdin => write!(f, "<stdin>"),
            Self::Stdout => write!(f, "<stdout>"),
            Self::Stderr => write!(f, "<stderr>"),
        }
    }
}

/// The error object.
/// It is different from runtime error in the sense that a runtime error
/// is not interpreted by the programming language but will terminate the
/// program. However, the error objects are used in the builtin functions
/// for error handling.
#[derive(Debug)]
pub enum ErrorObj {
    IO(io::Error),
    Utf8(std::string::FromUtf8Error),
    Packet(PacketError),
}

impl fmt::Display for ErrorObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::IO(e) => write!(f, "io-error: {e}"),
            Self::Utf8(e) => write!(f, "utf8-error: {e}"),
            Self::Packet(e) => write!(f, "packet-error: {}", e),
        }
    }
}
