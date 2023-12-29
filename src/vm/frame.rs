use std::rc::Rc;

use crate::code::definitions::Instructions;
use crate::object::func::Closure;

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub closure: Rc<Closure>,
    pub ip: usize, // instruction pointer
    pub bp: usize, // base pointer
}

impl Frame {
    pub fn new(closure: Rc<Closure>, bp: usize) -> Frame {
        Frame { closure, ip: 0, bp }
    }

    pub fn instructions(&self) -> &Rc<Instructions> {
        &self.closure.func.instructions
    }
}
