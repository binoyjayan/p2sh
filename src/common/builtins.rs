use std::rc::Rc;

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

fn builtin_len(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    match args.get(0) {
        Some(obj) => match obj.as_ref() {
            Object::Str(s) => Ok(Rc::new(Object::Number(s.len() as f64))),
            Object::Arr(a) => Ok(Rc::new(Object::Number(a.elements.len() as f64))),
            _ => Err(String::from("argument to 'len' not supported")),
        },
        None => Err(String::from("argument to 'len' not provided")),
    }
}

fn builtin_first(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if let Some(obj) = args.get(0) {
        match obj.as_ref() {
            Object::Arr(a) => {
                if let Some(first_element) = a.elements.get(0) {
                    return Ok(Rc::clone(first_element));
                } else {
                    return Ok(Rc::new(Object::Nil));
                }
            }
            _ => return Err(String::from("argument to 'first' not supported")),
        }
    }
    Err(String::from("no arguments provided"))
}

fn builtin_last(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if let Some(obj) = args.get(0) {
        match obj.as_ref() {
            Object::Arr(a) => {
                if let Some(last_element) = a.elements.last() {
                    return Ok(Rc::clone(last_element));
                } else {
                    return Ok(Rc::new(Object::Nil));
                }
            }
            _ => return Err(String::from("argument to 'last' not supported")),
        }
    }

    Err(String::from("no arguments provided"))
}

fn builtin_rest(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if let Some(obj) = args.get(0) {
        match obj.as_ref() {
            Object::Arr(a) => {
                if a.elements.is_empty() {
                    Ok(Rc::new(Object::Nil))
                } else {
                    Ok(Rc::new(Object::Arr(Array {
                        elements: a.elements[1..].to_vec(),
                    })))
                }
            }
            _ => Err(String::from("argument to 'last' not supported")),
        }
    } else {
        Err(String::from("argument to 'last' not provided"))
    }
}

fn builtin_puts(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("no arguments provided"));
    }

    for obj in args {
        match obj.as_ref() {
            Object::Str(t) => {
                println!("{}", t);
            }
            o => {
                println!("{}", o);
            }
        }
    }
    // puts returns Nil
    Ok(Rc::new(Object::Nil))
}
