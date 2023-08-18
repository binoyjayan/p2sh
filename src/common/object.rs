use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use std::rc::Rc;

use crate::common::environment::Environment;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::*;

#[derive(Debug)]
pub enum Object {
    Nil,
    Str(String),
    Number(f64),
    Bool(bool),
    Return(Rc<Object>),
    Func(Function),
    Builtin(BuiltinFunction),
    Arr(Array),
    Map(HMap),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (Object::Str(a), Object::Str(b)) => a.eq(b),
            (Object::Number(a), Object::Number(b)) => a.eq(b),
            (Object::Bool(a), Object::Bool(b)) => a.eq(b),
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
            Self::Arr(val) => write!(f, "{}", val),
            Self::Map(val) => write!(f, "{}", val),
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
    // Variadic functions do not have arity
    pub arity: Option<usize>,
    pub func: BuiltinFunctionProto,
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "builtin function")
    }
}

impl BuiltinFunction {
    pub fn new(name: String, arity: Option<usize>, func: BuiltinFunctionProto) -> BuiltinFunction {
        BuiltinFunction { name, arity, func }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Rc<Object>>,
}

#[derive(Debug, Clone)]
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

impl fmt::Display for HMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pairs_str = self
            .pairs
            .iter()
            .map(|(k, v)| format!("{}: {}, ", k, v))
            .collect::<String>();
        let pairs_str = pairs_str.trim_end_matches(|c| c == ' ' || c == ',');
        write!(f, "{{ {} }}", pairs_str)
    }
}
