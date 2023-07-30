use crate::common::object::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref BUILTINS: HashMap<String, BuiltinFunction> = {
        let mut m: HashMap<String, BuiltinFunction> = HashMap::new();
        m.insert(
            "len".into(),
            BuiltinFunction::new("len".into(), Some(1), builtin_len),
        );
        m.insert(
            "first".into(),
            BuiltinFunction::new("first".into(), Some(1), builtin_first),
        );
        m.insert(
            "last".into(),
            BuiltinFunction::new("last".into(), Some(1), builtin_last),
        );
        m.insert(
            "rest".into(),
            BuiltinFunction::new("rest".into(), Some(1), builtin_rest),
        );
        m.insert(
            "puts".into(),
            BuiltinFunction::new("puts".into(), None, builtin_puts),
        );
        m
    };
}

fn builtin_len(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Str(s)) => Ok(Object::Number(s.len() as f64)),
        Some(Object::Arr(a)) => Ok(Object::Number(a.elements.len() as f64)),
        _ => Err(String::from("argument to 'len' not supported")),
    }
}

fn builtin_first(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Arr(a)) => {
            if a.elements.is_empty() {
                Ok(Object::Nil)
            } else {
                Ok(a.elements[0].clone())
            }
        }
        _ => Err(String::from("argument to 'first' not supported")),
    }
}

fn builtin_last(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Arr(a)) => {
            let length = a.elements.len();
            if a.elements.is_empty() {
                Ok(Object::Nil)
            } else {
                Ok(a.elements[length - 1].clone())
            }
        }
        _ => Err(String::from("argument to 'last' not supported")),
    }
}

fn builtin_rest(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Arr(a)) => {
            if a.elements.is_empty() {
                Ok(Object::Nil)
            } else {
                Ok(Object::Arr(Array {
                    elements: a.elements[1..].to_vec(),
                }))
            }
        }
        _ => Err(String::from("argument to 'last' not supported")),
    }
}

fn builtin_puts(args: Vec<Object>) -> Result<Object, String> {
    for obj in args {
        match obj {
            Object::Str(t) => {
                println!("{}", t);
            }
            o => {
                println!("{}", o);
            }
        }
    }
    // puts returns Nil
    Ok(Object::Nil)
}
