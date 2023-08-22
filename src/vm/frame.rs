use std::rc::Rc;

use crate::code::definitions::Instructions;
use crate::common::object::CompiledFunction;

#[derive(Debug, Clone)]
pub struct Frame {
    pub func: Rc<CompiledFunction>,
    pub ip: usize,
}

impl Frame {
    pub fn new(func: Rc<CompiledFunction>) -> Frame {
        Frame { func, ip: 0 }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
