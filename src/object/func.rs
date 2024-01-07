use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::Object;
use crate::code::definitions::Instructions;

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
    pub line: usize,
}

impl CompiledFunction {
    pub fn new(
        instructions: Instructions,
        num_locals: usize,
        num_params: usize,
        line: usize,
    ) -> Self {
        Self {
            instructions: Rc::new(instructions),
            num_locals,
            num_params,
            line,
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
