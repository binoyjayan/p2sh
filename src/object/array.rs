use std::cell::RefCell;
use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

use super::Object;

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

impl From<&Array> for Vec<u8> {
    fn from(obj: &Array) -> Self {
        let mut bytes = Vec::new();
        for element in obj.elements.borrow().iter() {
            let b: Vec<u8> = element.as_ref().into();
            bytes.extend_from_slice(&b);
        }
        bytes
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
