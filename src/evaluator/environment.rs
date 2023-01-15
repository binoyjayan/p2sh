use super::error::*;
use super::object::*;
use crate::token::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    pub values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn get(&self, token: &Token) -> Result<Object, RTError> {
        let name = token.literal.clone();
        if let Some(obj) = self.values.get(&name) {
            Ok(obj.clone())
        } else {
            Err(RTError::new(
                &format!("Undefined identifier: '{}'", name),
                1,
            ))
        }
    }

    pub fn set(&mut self, token: &Token, value: Object) -> Result<Object, RTError> {
        self.values.insert(token.literal.clone(), value.clone());
        Ok(value)
    }

    pub fn assign(&mut self, token: &Token, value: Object) -> Result<Object, RTError> {
        let name = token.literal.clone();
        // Sets the value of the entry, and returns the entry’s old value
        if let Some(obj) = self.values.insert(name.clone(), value) {
            return Ok(obj);
        }
        return Err(RTError::new(
            &format!("Undefined identifier: '{}'", name),
            token.line,
        ));
    }
}
