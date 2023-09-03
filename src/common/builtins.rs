use std::rc::Rc;

use crate::common::object::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTINS: Vec<BuiltinFunction> = {
        vec![
            BuiltinFunction::new("len".into(), builtin_len),
            BuiltinFunction::new("puts".into(), builtin_puts),
            BuiltinFunction::new("first".into(), builtin_first),
            BuiltinFunction::new("last".into(), builtin_last),
            BuiltinFunction::new("rest".into(), builtin_rest),
            BuiltinFunction::new("push".into(), builtin_push),
        ]
    };
}

fn builtin_len(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }
    match args[0].as_ref() {
        Object::Str(s) => Ok(Rc::new(Object::Number(s.len() as f64))),
        Object::Arr(a) => Ok(Rc::new(Object::Number(a.elements.len() as f64))),
        _ => Err(String::from("argument to 'len' not supported")),
    }
}

fn builtin_puts(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        println!();
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

fn builtin_push(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() < 2 {
        return Err(String::from("'push' needs atleast two arguments"));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            let mut new_array = a.clone();
            new_array.elements.push(args[1].clone());
            Ok(Rc::new(Object::Arr(new_array)))
        }
        _ => Err(String::from("argument to 'push' not supported")),
    }
}
