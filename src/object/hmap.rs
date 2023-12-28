use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

use super::Object;

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
