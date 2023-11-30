use std::io::{self, Write};
use std::process;
use std::rc::Rc;
use std::thread;
use std::time;
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
    BuiltinFunction::new("get", builtin_get),
    BuiltinFunction::new("contains", builtin_contains),
    BuiltinFunction::new("insert", builtin_insert),
    BuiltinFunction::new("str", builtin_str),
    BuiltinFunction::new("int", builtin_int),
    BuiltinFunction::new("float", builtin_float),
    BuiltinFunction::new("char", builtin_char),
    BuiltinFunction::new("byte", builtin_byte),
    BuiltinFunction::new("time", builtin_time),
    BuiltinFunction::new("exit", builtin_exit),
    BuiltinFunction::new("flush_stdout", flush_stdout),
    BuiltinFunction::new("flush_stderr", flush_stderr),
    BuiltinFunction::new("format", builtin_format),
    BuiltinFunction::new("print", builtin_print),
    BuiltinFunction::new("println", builtin_println),
    BuiltinFunction::new("eprint", builtin_eprint),
    BuiltinFunction::new("eprintln", builtin_eprintln),
    BuiltinFunction::new("round", builtin_round),
    BuiltinFunction::new("sleep", builtin_sleep),
];

fn builtin_len(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Str(s) => Ok(Rc::new(Object::Integer(s.len() as i64))),
        Object::Arr(a) => Ok(Rc::new(Object::Integer(a.len() as i64))),
        Object::Map(m) => Ok(Rc::new(Object::Integer(m.len() as i64))),
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_puts(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() {
        println!();
        return Ok(Rc::new(Object::Null));
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
    // puts returns Null
    Ok(Rc::new(Object::Null))
}

fn builtin_first(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => Ok(a.get(0)),
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_last(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => Ok(a.last()),
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_rest(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            if a.is_empty() {
                Ok(Rc::new(Object::Null))
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
            Ok(Rc::new(Object::Null))
        }
        _ => Err(String::from("unsupported argument")),
    }
}

// Get an array item by index or a map value by key
// Return Null if index is out of bounds or if the key doesn't exist
fn builtin_get(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Arr(arr) => {
            if let Object::Integer(index) = args[1].as_ref() {
                let index = *index as usize;
                Ok(arr.get(index))
            } else {
                Err(String::from("unsupported argument"))
            }
        }
        Object::Map(map) => Ok(map.get(&args[1])),
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_contains(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Map(map) => {
            let key = args[1].clone();
            let contains = map.contains(&key);
            Ok(Rc::new(Object::Bool(contains)))
        }
        _ => Err(String::from("unsupported argument")),
    }
}

// Insert a key-value pair into a map. If the key already exists,
// the old value is returned, otherwise Null is returned.
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
        Object::Null
            | Object::Str(_)
            | Object::Integer(_)
            | Object::Char(_)
            | Object::Byte(_)
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
            if let Ok(num) = s.parse::<i64>() {
                Ok(Rc::new(Object::Integer(num)))
            } else {
                Err(String::from("failed to parse string into an int"))
            }
        }
        Object::Integer(_) => Ok(Rc::clone(&args[0])),
        Object::Float(n) => Ok(Rc::new(Object::Integer(*n as i64))),
        Object::Char(b) => Ok(Rc::new(Object::Integer(*b as i64))),
        Object::Byte(b) => Ok(Rc::new(Object::Integer(*b as i64))),
        Object::Bool(b) => {
            if *b {
                Ok(Rc::new(Object::Integer(1)))
            } else {
                Ok(Rc::new(Object::Integer(0)))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_float(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    let obj = args[0].as_ref();
    match obj {
        Object::Str(s) => {
            if let Ok(num) = s.parse::<f64>() {
                Ok(Rc::new(Object::Float(num)))
            } else {
                Err(String::from("failed to parse string into a float"))
            }
        }
        Object::Float(_) => Ok(Rc::clone(&args[0])),
        Object::Integer(n) => Ok(Rc::new(Object::Float(*n as f64))),
        Object::Char(b) => Ok(Rc::new(Object::Float(*b as i64 as f64))),
        Object::Byte(b) => Ok(Rc::new(Object::Float(*b as f64))),
        Object::Bool(b) => {
            if *b {
                Ok(Rc::new(Object::Float(1.)))
            } else {
                Ok(Rc::new(Object::Float(0.)))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_char(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    let obj = args[0].as_ref();
    match obj {
        Object::Char(_) => Ok(Rc::clone(&args[0])),
        Object::Byte(b) => {
            if let Some(c) = std::char::from_u32(*b as u32) {
                Ok(Rc::new(Object::Char(c)))
            } else {
                Err(String::from("failed to parse byte"))
            }
        }
        Object::Integer(s) => {
            if let Some(c) = std::char::from_u32(*s as u32) {
                Ok(Rc::new(Object::Char(c)))
            } else {
                Err(String::from("failed to parse integer"))
            }
        }
        Object::Float(n) => {
            if let Some(c) = std::char::from_u32(*n as u32) {
                Ok(Rc::new(Object::Char(c)))
            } else {
                Err(String::from("failed to parse float"))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_byte(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    let obj = args[0].as_ref();
    match obj {
        Object::Byte(_) => Ok(Rc::clone(&args[0])),
        Object::Char(c) => Ok(Rc::new(Object::Byte(*c as u8))),
        Object::Bool(b) => {
            if *b {
                Ok(Rc::new(Object::Byte(1)))
            } else {
                Ok(Rc::new(Object::Byte(0)))
            }
        }
        Object::Integer(s) => {
            if let Some(b) = std::char::from_u32(*s as u32) {
                Ok(Rc::new(Object::Byte(b as u8)))
            } else {
                Err(String::from("failed to parse integer"))
            }
        }
        Object::Float(n) => {
            if let Some(b) = std::char::from_u32(*n as u32) {
                Ok(Rc::new(Object::Byte(b as u8)))
            } else {
                Err(String::from("failed to parse float"))
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
    let seconds = duration.as_secs() as i64;
    Ok(Rc::new(Object::Integer(seconds)))
}

#[allow(unreachable_code)]
fn builtin_exit(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Integer(code) => {
            process::exit(*code as i32);
        }
        _ => return Err(String::from("unsupported argument")),
    }
    process::exit(0);
    Ok(Rc::new(Object::Null))
}

fn flush_stdout(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }
    io::stdout().flush().expect("Failed to flush stdout");
    Ok(Rc::new(Object::Null))
}

fn flush_stderr(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }
    io::stderr().flush().expect("Failed to flush stderr");
    Ok(Rc::new(Object::Null))
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
        len += s.len() as i64;
    }
    Ok(Rc::new(Object::Integer(len)))
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
        len += s.len() as i64;
    }
    // Newline at the end
    println!();
    len += 1;
    Ok(Rc::new(Object::Integer(len)))
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
        len += s.len() as i64;
    }
    Ok(Rc::new(Object::Integer(len)))
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
        len += s.len() as i64;
    }
    // Newline at the end
    eprintln!();
    len += 1;
    Ok(Rc::new(Object::Integer(len)))
}

fn builtin_round(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Float(f) => {
            if let Object::Integer(n) = args[1].as_ref() {
                let multiplier = 10i64.pow(*n as u32);
                let rounded = (f * multiplier as f64).round() / multiplier as f64;
                Ok(Rc::new(Object::Float(rounded)))
            } else {
                Err(String::from("second argument should be an integer"))
            }
        }
        _ => Err(String::from("first argument should be a float")),
    }
}

fn builtin_sleep(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Integer(n) => {
            thread::sleep(time::Duration::from_secs(*n as u64));
            Ok(Rc::new(Object::Null))
        }
        _ => Err(String::from("argument should be an integer")),
    }
}
