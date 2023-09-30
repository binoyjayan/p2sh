use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use std::rc::Rc;

use crate::code::definitions::Instructions;
use crate::common::environment::Environment;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::*;

// TODO: Wrap BuiltinFunction in an Rc
#[derive(Debug)]
pub enum Object {
    Nil,
    Str(String),
    Number(f64),
    Bool(bool),
    Return(Rc<Object>),
    Func(Rc<Function>),
    Builtin(Box<BuiltinFunction>),
    CompiledFunc(Rc<CompiledFunction>),
    Arr(Rc<Array>),
    Map(Rc<HMap>),
    Clos(Rc<Closure>),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (Object::Str(a), Object::Str(b)) => a.eq(b),
            (Object::Number(a), Object::Number(b)) => a.eq(b),
            (Object::Bool(a), Object::Bool(b)) => a.eq(b),
            (Object::Arr(a), Object::Arr(b)) => a.eq(b),
            (Object::Map(a), Object::Map(b)) => a.eq(b),
            (Object::Builtin(a), Object::Builtin(b)) => a.eq(b),
            (Object::CompiledFunc(a), Object::CompiledFunc(b)) => a.eq(b),
            (Object::Clos(a), Object::Clos(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl Eq for Object {}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Nil, Object::Nil) => None,
            (Object::Str(a), Object::Str(b)) => a.partial_cmp(b),
            (Object::Number(a), Object::Number(b)) => a.partial_cmp(b),
            (Object::Bool(a), Object::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::Nil => Object::Nil,
            Object::Str(s) => Object::Str(s.clone()),
            Object::Number(n) => Object::Number(*n),
            Object::Bool(b) => Object::Bool(*b),
            Object::Return(r) => Object::Return(r.clone()),
            Object::Func(f) => Object::Func(f.clone()),
            Object::Builtin(f) => Object::Builtin(f.clone()),
            Object::Arr(a) => Object::Arr(a.clone()),
            Object::Map(m) => Object::Map(m.clone()),
            Object::CompiledFunc(f) => Object::CompiledFunc(f.clone()),
            Object::Clos(f) => Object::Clos(f.clone()),
        }
    }
}

impl Object {
    pub fn is_nil(&self) -> bool {
        matches!(self, Object::Nil)
    }
    pub fn is_string(&self) -> bool {
        matches!(self, Object::Str(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Object::Number(_))
    }
    pub fn is_falsey(&self) -> bool {
        matches!(self, Object::Bool(false) | Object::Nil)
    }
    pub fn is_a_valid_key(&self) -> bool {
        matches!(self, Object::Str(_) | Object::Number(_) | Object::Bool(_))
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Str(s) => write!(f, "{}", s),
            Self::Number(val) => write!(f, "{}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::Return(val) => write!(f, "{}", val),
            Self::Func(val) => write!(f, "{}", val),
            Self::Builtin(val) => write!(f, "{}", val),
            Self::CompiledFunc(val) => write!(f, "{}", val),
            Self::Arr(val) => write!(f, "{}", val),
            Self::Map(val) => write!(f, "{}", val),
            Self::Clos(val) => write!(f, "{}", val),
        }
    }
}

impl ops::Add for &Object {
    type Output = Object;

    fn add(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Number(a), &Object::Number(b)) => Object::Number(a + b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl ops::Sub for &Object {
    type Output = Object;
    fn sub(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Number(a), &Object::Number(b)) => Object::Number(a - b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl ops::Mul for &Object {
    type Output = Object;
    fn mul(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Number(a), &Object::Number(b)) => Object::Number(a * b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl ops::Div for &Object {
    type Output = Object;
    fn div(self, other: &Object) -> Object {
        match (self, other) {
            (&Object::Number(a), &Object::Number(b)) => Object::Number(a / b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl ops::Neg for &Object {
    type Output = Object;
    fn neg(self) -> Object {
        match self {
            &Object::Number(a) => Object::Number(-a),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Number(ref n) => {
                // Use the built-in hash function for f64
                state.write_u64(n.to_bits());
            }
            Object::Bool(ref b) => b.hash(state),
            Object::Str(ref s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params_str = self
            .params
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let params_str = params_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "fn({}) {{\n{}\n}}\n", params_str, self.body)
    }
}

pub type BuiltinFunctionProto = fn(Vec<Rc<Object>>) -> Result<Rc<Object>, String>;

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub func: BuiltinFunctionProto,
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl BuiltinFunction {
    pub fn new(name: String, func: BuiltinFunctionProto) -> BuiltinFunction {
        BuiltinFunction { name, func }
    }
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Rc<Object>>,
}

#[derive(Debug, Clone, Default)]
pub struct HMap {
    pub pairs: HashMap<Rc<Object>, Rc<Object>>,
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elements_str = self
            .elements
            .iter()
            .map(|p| format!("{}, ", p))
            .collect::<String>();
        let elements_str = elements_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "[{}]", elements_str)
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        if self.elements.len() != other.elements.len() {
            return false;
        }
        for (a, b) in self.elements.iter().zip(&other.elements) {
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
            .iter()
            .map(|(k, v)| format!(r#""{}": {}, "#, k, v))
            .collect::<String>();
        let pairs_str = pairs_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{{{}}}", pairs_str)
    }
}

// compare HMap objects without considering the order of key-value pairs
impl PartialEq for HMap {
    fn eq(&self, other: &Self) -> bool {
        if self.pairs.len() != other.pairs.len() {
            return false;
        }
        for (key, value) in &self.pairs {
            if let Some(other_value) = other.pairs.get(key) {
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
// OpReturn is similar to OpReturnValue except that it returns Nil.
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
pub struct Closure {
    pub func: Rc<CompiledFunction>,
    pub free: Vec<Rc<Object>>,
}

impl Closure {
    pub fn new(func: Rc<CompiledFunction>, free: Vec<Rc<Object>>) -> Self {
        Self { func, free }
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
