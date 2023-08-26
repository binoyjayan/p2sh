use std::rc::Rc;

use crate::code::definitions::Instructions;
use crate::common::object::CompiledFunction;

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub func: Rc<CompiledFunction>,
    pub ip: usize, // instruction pointer
    pub bp: usize, // base pointer
}

impl Frame {
    pub fn new(func: Rc<CompiledFunction>, bp: usize) -> Frame {
        Frame { func, ip: 0, bp }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
