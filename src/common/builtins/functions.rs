use std::io::{self, Write};
use std::process;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::print::format_buf;
use crate::common::object::*;

pub const BUILTINFNS: &[BuiltinFunction] = &[
    BuiltinFunction::new("len", builtin_len),
    BuiltinFunction::new("puts", builtin_puts),
    BuiltinFunction::new("first", builtin_first),
    BuiltinFunction::new("last", builtin_last),
    BuiltinFunction::new("rest", builtin_rest),
    BuiltinFunction::new("push", builtin_push),
    BuiltinFunction::new("insert", builtin_insert),
    BuiltinFunction::new("str", builtin_str),
    BuiltinFunction::new("int", builtin_int),
    BuiltinFunction::new("time", builtin_time),
    BuiltinFunction::new("exit", builtin_exit),
    BuiltinFunction::new("flush_stdout", flush_stdout),
    BuiltinFunction::new("flush_stderr", flush_stderr),
    BuiltinFunction::new("format", builtin_format),
    BuiltinFunction::new("print", builtin_print),
    BuiltinFunction::new("println", builtin_println),
    BuiltinFunction::new("eprint", builtin_eprint),
    BuiltinFunction::new("eprintln", builtin_eprintln),
];

fn builtin_len(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Str(s) => Ok(Rc::new(Object::Number(s.len() as f64))),
        Object::Arr(a) => Ok(Rc::new(Object::Number(a.elements.borrow().len() as f64))),
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_puts(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        println!();
        return Ok(Rc::new(Object::Nil));
    }

    for obj in args {
        match obj.as_ref() {
            Object::Str(t) => {
                // Avoid quotes around string
                print!("{}", t);
            }
            o => {
                print!("{}", o);
            }
        }
    }
    println!();
    // puts returns Nil
    Ok(Rc::new(Object::Nil))
}

fn builtin_first(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            if let Some(first_element) = a.elements.borrow().get(0) {
                Ok(Rc::clone(first_element))
            } else {
                Ok(Rc::new(Object::Nil))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_last(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            if let Some(last_element) = a.elements.borrow().last() {
                Ok(Rc::clone(last_element))
            } else {
                Ok(Rc::new(Object::Nil))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_rest(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            if a.elements.borrow().is_empty() {
                Ok(Rc::new(Object::Nil))
            } else {
                let slice = a.elements.borrow()[1..].to_vec();
                Ok(Rc::new(Object::Arr(Rc::new(Array::new(slice)))))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

// Insert a value to the end of an array
fn builtin_push(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Arr(arr) => {
            arr.push(args[1].clone());
            Ok(Rc::new(Object::Nil))
        }
        _ => Err(String::from("unsupported argument")),
    }
}

// Insert a key-value pair into a map. If the key already exists,
// the old value is returned, otherwise Nil is returned.
fn builtin_insert(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 3 {
        return Err(format!("takes three arguments. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Map(map) => {
            let key = args[1].clone();
            let val = args[2].clone();
            let old = map.insert(key, val);
            Ok(old)
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_str(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    let obj = args[0].as_ref();
    if !matches!(
        obj,
        Object::Nil
            | Object::Str(_)
            | Object::Number(_)
            | Object::Bool(_)
            | Object::Arr(_)
            | Object::Map(_)
    ) {
        return Err(String::from("unsupported argument"));
    }
    Ok(Rc::new(Object::Str(obj.to_string())))
}

fn builtin_int(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    let obj = args[0].as_ref();
    match obj {
        Object::Str(s) => {
            if let Ok(num) = s.parse::<f64>() {
                Ok(Rc::new(Object::Number(num)))
            } else {
                Err(String::from("failed to parse string into an int"))
            }
        }
        Object::Number(_) => Ok(Rc::clone(&args[0])),
        Object::Bool(b) => {
            if *b {
                Ok(Rc::new(Object::Number(1.0)))
            } else {
                Ok(Rc::new(Object::Number(0.0)))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_time(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }
    let current_time = SystemTime::now();
    let duration = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds = duration.as_secs();
    Ok(Rc::new(Object::Number(seconds as f64)))
}

#[allow(unreachable_code)]
fn builtin_exit(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Number(code) => {
            process::exit(*code as i32);
        }
        _ => return Err(String::from("unsupported argument")),
    }
    process::exit(0);
    Ok(Rc::new(Object::Nil))
}

fn flush_stdout(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }
    io::stdout().flush().expect("Failed to flush stdout");
    Ok(Rc::new(Object::Nil))
}

fn flush_stderr(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }
    io::stderr().flush().expect("Failed to flush stderr");
    Ok(Rc::new(Object::Nil))
}

pub fn builtin_format(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("takes atleast one argument. got none"));
    }
    let collector = format_buf(args)?;
    // Join the collected formatted output
    let buf: String = collector.0.into_iter().collect();
    // Return buf
    Ok(Rc::new(Object::Str(buf)))
}

fn builtin_print(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("takes atleast one argument. got none"));
    }
    let mut len = 0;
    let collector = format_buf(args)?;
    // Print the collected formatted output
    for s in &collector.0 {
        print!("{}", s);
        len += s.len();
    }
    Ok(Rc::new(Object::Number(len as f64)))
}

fn builtin_println(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("takes atleast one argument. got none"));
    }
    let mut len = 0;
    let collector = format_buf(args)?;
    // Print the collected formatted output
    for s in &collector.0 {
        print!("{}", s);
        len += s.len();
    }
    // Newline at the end
    println!();
    len += 1;
    Ok(Rc::new(Object::Number(len as f64)))
}

fn builtin_eprint(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("takes atleast one argument. got none"));
    }
    let mut len = 0;
    let collector = format_buf(args)?;
    // Print the collected formatted output
    for s in &collector.0 {
        eprint!("{}", s);
        len += s.len();
    }
    Ok(Rc::new(Object::Number(len as f64)))
}

fn builtin_eprintln(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        return Err(String::from("takes atleast one argument. got none"));
    }
    let mut len = 0;
    let collector = format_buf(args)?;
    // Print the collected formatted output
    for s in &collector.0 {
        eprint!("{}", s);
        len += s.len();
    }
    // Newline at the end
    eprintln!();
    len += 1;
    Ok(Rc::new(Object::Number(len as f64)))
}
