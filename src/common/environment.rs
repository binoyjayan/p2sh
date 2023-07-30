use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::common::object::*;
use crate::scanner::token::*;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    env: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            env: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }
}

impl Environment {
    /// While looking up the identifier in the environment, start with the
    /// inner most which appears at the front of the linked list until the end.
    /// Return  the value of the object when a match is found. Return None
    /// if all the enclosing environments also does not define the identifier.
    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(obj) = self.env.get(name) {
            Some(obj.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, token: &Token, value: Object) {
        self.env.insert(token.literal.clone(), value);
    }
}
