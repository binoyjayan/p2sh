use lazy_static::lazy_static;
use std::io::{self, Write};
use std::process;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod print;
mod tests;

use crate::common::object::*;
use print::format_buf;

lazy_static! {
    pub static ref BUILTINS: Vec<BuiltinFunction> = {
        vec![
            BuiltinFunction::new("len".into(), builtin_len),
            BuiltinFunction::new("puts".into(), builtin_puts),
            BuiltinFunction::new("first".into(), builtin_first),
            BuiltinFunction::new("last".into(), builtin_last),
            BuiltinFunction::new("rest".into(), builtin_rest),
            BuiltinFunction::new("push".into(), builtin_push),
            BuiltinFunction::new("str".into(), builtin_str),
            BuiltinFunction::new("time".into(), builtin_time),
            BuiltinFunction::new("exit".into(), builtin_exit),
            BuiltinFunction::new("flush_stdout".into(), flush_stdout),
            BuiltinFunction::new("flush_stderr".into(), flush_stderr),
            BuiltinFunction::new("format".into(), builtin_format),
            BuiltinFunction::new("print".into(), builtin_print),
            BuiltinFunction::new("println".into(), builtin_println),
            BuiltinFunction::new("eprint".into(), builtin_eprint),
            BuiltinFunction::new("eprintln".into(), builtin_eprintln),
        ]
    };
}

fn builtin_len(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Str(s) => Ok(Rc::new(Object::Number(s.len() as f64))),
        Object::Arr(a) => Ok(Rc::new(Object::Number(a.elements.len() as f64))),
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
            if let Some(first_element) = a.elements.get(0) {
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
            if let Some(last_element) = a.elements.last() {
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
            if a.elements.is_empty() {
                Ok(Rc::new(Object::Nil))
            } else {
                Ok(Rc::new(Object::Arr(Rc::new(Array {
                    elements: a.elements[1..].to_vec(),
                }))))
            }
        }
        _ => Err(String::from("unsupported argument")),
    }
}

fn builtin_push(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }
    match args[0].as_ref() {
        Object::Arr(a) => {
            let mut new_array = a.clone();
            // Use interior mutability of the Array
            let new_array_mut = Rc::make_mut(&mut new_array);
            new_array_mut.elements.push(args[1].clone());
            Ok(Rc::new(Object::Arr(new_array)))
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

fn builtin_format(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
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
