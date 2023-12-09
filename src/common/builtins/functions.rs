use std::fs;
use std::io::Read;
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
    BuiltinFunction::new("tolower", builtin_tolower),
    BuiltinFunction::new("toupper", builtin_toupper),
    BuiltinFunction::new("open", builtin_open),
    BuiltinFunction::new("read", builtin_read),
    BuiltinFunction::new("write", builtin_write),
    BuiltinFunction::new("read_to_string", builtin_read_to_string),
    BuiltinFunction::new("decode_utf8", decode_utf8),
    BuiltinFunction::new("encode_utf8", encode_utf8),
    BuiltinFunction::new("read_line", builtin_read_line),
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
    match obj {
        Object::Str(_) => Ok(Rc::clone(&args[0])),
        Object::Null
        | Object::Integer(_)
        | Object::Char(_)
        | Object::Byte(_)
        | Object::Bool(_)
        | Object::Arr(_)
        | Object::Map(_) => Ok(Rc::new(Object::Str(obj.to_string()))),
        _ => Err(String::from("unsupported argument")),
    }
    // Ok(Rc::new(Object::Str(obj.to_string())))
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

fn builtin_tolower(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Char(c) => {
            let c = c.to_ascii_lowercase();
            Ok(Rc::new(Object::Char(c)))
        }
        Object::Byte(b) => {
            let b = b.to_ascii_lowercase();
            Ok(Rc::new(Object::Byte(b)))
        }
        Object::Str(s) => {
            let s = s.to_ascii_lowercase();
            Ok(Rc::new(Object::Str(s)))
        }
        _ => Err(String::from("argument should be an integer")),
    }
}

fn builtin_toupper(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    match args[0].as_ref() {
        Object::Char(c) => {
            let c = c.to_ascii_uppercase();
            Ok(Rc::new(Object::Char(c)))
        }
        Object::Byte(b) => {
            let b = b.to_ascii_uppercase();
            Ok(Rc::new(Object::Byte(b)))
        }
        Object::Str(s) => {
            let s = s.to_ascii_uppercase();
            Ok(Rc::new(Object::Str(s)))
        }
        _ => Err(String::from("argument should be an integer")),
    }
}

/// Opens a file handle
/// # Arguments
/// * `args` - A vector of Rc<Object> containing the path to the file (Object::Str) and an optional
///           second argument specifying the mode (Object::Str).
/// # Returns
/// Returns a Result containing a file handle wrapped in an Object::File,
/// or an error message if the operation fails.
fn builtin_open(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("takes one or two arguments. got={}", args.len()));
    }

    let path = if let Object::Str(s) = args[0].as_ref() {
        s
    } else {
        return Err(String::from("argument should be a string"));
    };

    let mode = if args.len() == 2 {
        if let Object::Str(s) = args[1].as_ref() {
            s
        } else {
            return Err(String::from("second argument should be a string"));
        }
    } else {
        "r"
    };

    match mode {
        "r" => {
            // opens a file for reading, error if the file does not exist
            let file = fs::File::open(path);
            match file {
                Ok(file) => {
                    let handle = FileHandle::new(file);
                    Ok(Rc::new(Object::File(Rc::new(handle))))
                }
                Err(_) => Err("failed to open file for reading".to_string()),
            }
        }
        "a" => {
            // open a file for appending, create the file if it does not exist
            let file = fs::OpenOptions::new().append(true).open(path);
            match file {
                Ok(file) => {
                    let handle = FileHandle::new(file);
                    Ok(Rc::new(Object::File(Rc::new(handle))))
                }
                Err(_) => Err("failed to open file for appending".to_string()),
            }
        }
        "w" => {
            // open a file for writing, create the file if it does not exist
            let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(path);
            match file {
                Ok(file) => {
                    let handle = FileHandle::new(file);
                    Ok(Rc::new(Object::File(Rc::new(handle))))
                }
                Err(_) => Err("failed to open file for writing".to_string()),
            }
        }
        "x" => {
            // create the specified file, returns an error if the file exist
            let file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path);
            match file {
                Ok(file) => {
                    let handle = FileHandle::new(file);
                    Ok(Rc::new(Object::File(Rc::new(handle))))
                }
                Err(_) => Err("failed to create file".to_string()),
            }
        }
        "T" => {
            // open a file for writing, with truncation
            let file = fs::OpenOptions::new().write(true).truncate(true).open(path);
            match file {
                Ok(file) => {
                    let handle = FileHandle::new(file);
                    Ok(Rc::new(Object::File(Rc::new(handle))))
                }
                Err(_) => Err("failed to open file for truncation".to_string()),
            }
        }
        _ => Err(String::from("invalid file open mode")),
    }
}

/// Reads bytes from a file handle into an array of Object::Byte variants.
/// # Arguments
/// * `args` - A vector of Rc<Object> containing the file handle (Object::File) and an optional
///            second argument specifying the number of bytes to read (Object::Integer).
/// # Returns
/// Returns a Result containing an array of Object::Byte variants wrapped in an Object::Arr,
/// or an error message if the operation fails.
///
fn builtin_read(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("takes one or two arguments. got={}", args.len()));
    }

    if let Object::File(f) = args[0].as_ref() {
        let mut file = f.file.borrow_mut();
        let mut total_bytes_read = 0;
        // Read in chunks of 4096 bytes
        let mut buffer = [0; 4096];

        let num_bytes_to_read = if args.len() == 2 {
            if let Object::Integer(num) = args[1].as_ref() {
                *num as usize
            } else {
                return Err(String::from("second argument should be an integer"));
            }
        } else {
            // Read until the end of the file
            usize::MAX
        };

        let mut result_bytes = Vec::new();

        while total_bytes_read < num_bytes_to_read {
            let bytes_remaining = num_bytes_to_read - total_bytes_read;
            let read_len = buffer.len().min(bytes_remaining);
            let buf_slice = &mut buffer[..read_len];
            match file.read(buf_slice) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break; // EOF
                    }
                    // Copy bytes off
                    for byte in buf_slice.iter().take(bytes_read) {
                        result_bytes.push(Rc::new(Object::Byte(*byte)));
                    }
                    // Got less bytes than requested, so we're done
                    if bytes_read < read_len {
                        break;
                    }
                    total_bytes_read += bytes_read;
                }
                Err(e) => {
                    return Err(format!("failed to read file: {}", e));
                }
            }
        }

        Ok(Rc::new(Object::Arr(Rc::new(Array::new(result_bytes)))))
    } else {
        Err(String::from("first argument should be a file handle"))
    }
}

/// Reads bytes from a file handle and return it as a string
/// # Arguments
/// * `args` - A vector of Rc<Object> containing the file handle (Object::File).
/// # Returns
/// Returns a Result containing a string wrapped in an Object::Str,
/// or an error message if the operation fails.
fn builtin_read_to_string(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    if let Object::File(f) = args[0].as_ref() {
        let mut file = f.file.borrow_mut();
        // Read all data
        let mut result_bytes = Vec::new();
        match file.read_to_end(&mut result_bytes) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("failed to read file: {}", e));
            }
        }

        match String::from_utf8(result_bytes) {
            Ok(s) => Ok(Rc::new(Object::Str(s))),
            Err(e) => Err(format!("failed to decode bytes: {}", e)),
        }
    } else {
        Err(String::from("first argument should be a file handle"))
    }
}

/// Decodes a UTF-8 encoded byte array into a string
/// # Arguments
/// * `args` - A vector of Rc<Object> containing an array of Object::Byte variants.
/// # Returns
/// Returns a Result containing a string wrapped in an Object::Str,
/// or an error message if the operation fails.
fn decode_utf8(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    if let Object::Arr(arr) = args[0].as_ref() {
        let mut bytes = Vec::new();
        for obj in arr.elements.borrow().iter() {
            if let Object::Byte(b) = obj.as_ref() {
                bytes.push(*b);
            } else {
                return Err(String::from("array should contain only bytes"));
            }
        }
        match String::from_utf8(bytes) {
            Ok(s) => Ok(Rc::new(Object::Str(s))),
            Err(e) => Err(format!("failed to decode bytes: {}", e)),
        }
    } else {
        Err(String::from("argument should be an array of bytes"))
    }
}

/// Encodes a string into a UTF-8 encoded byte array
/// # Arguments
/// * `args` - A vector of Rc<Object> containing a string wrapped in an Object::Str.
/// # Returns
/// Returns a Result containing an array of Object::Byte variants wrapped in an Object::Arr,
/// or an error message if the operation fails.
fn encode_utf8(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 1 {
        return Err(format!("takes one argument. got={}", args.len()));
    }

    if let Object::Str(s) = args[0].as_ref() {
        let mut bytes = Vec::new();
        for b in s.as_bytes() {
            bytes.push(Rc::new(Object::Byte(*b)));
        }
        Ok(Rc::new(Object::Arr(Rc::new(Array::new(bytes)))))
    } else {
        Err(String::from("argument should be a string"))
    }
}

/// Writes a byte or an array of bytes to a file handle
/// # Arguments
/// * `args` - A vector of Rc<Object> containing the file handle (Object::File) and a byte or an
///           array of bytes (Object::Byte or Object::Arr).
/// # Returns
/// Returns a Result containing the number of bytes written wrapped in an Object::Integer,
/// or an error message if the operation fails.
fn builtin_write(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if args.len() != 2 {
        return Err(format!("takes two arguments. got={}", args.len()));
    }

    if let Object::File(f) = args[0].as_ref() {
        let mut file = f.file.borrow_mut();
        match args[1].as_ref() {
            Object::Byte(b) => {
                let buf = [*b];
                match file.write(&buf) {
                    // Return number of bytes written
                    Ok(n) => Ok(Rc::new(Object::Integer(n as i64))),
                    Err(e) => Err(format!("failed to write to file: {}", e)),
                }
            }
            // Handle array of bytes
            Object::Arr(arr) => {
                let mut buf = Vec::new();
                for obj in arr.elements.borrow().iter() {
                    if let Object::Byte(b) = obj.as_ref() {
                        buf.push(*b);
                    } else {
                        return Err(String::from("array should contain only bytes"));
                    }
                }
                match file.write(&buf) {
                    Ok(n) => Ok(Rc::new(Object::Integer(n as i64))),
                    Err(e) => Err(format!("failed to write to file: {}", e)),
                }
            }
            Object::Str(s) => {
                let bytes = s.as_bytes();
                match file.write(bytes) {
                    Ok(n) => Ok(Rc::new(Object::Integer(n as i64))),
                    Err(e) => Err(format!("failed to write to file: {}", e)),
                }
            }
            _ => Err(String::from("second argument should be a byte or string")),
        }
    } else {
        Err(String::from("first argument should be a file handle"))
    }
}

/// Reads a line from stdin
/// # Returns
/// Returns a Result containing a string wrapped in an Object::Str,
/// or an error message if the operation fails.
fn builtin_read_line(args: Vec<Rc<Object>>) -> Result<Rc<Object>, String> {
    if !args.is_empty() {
        return Err(format!("takes no argument(s). got={}", args.len()));
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("failed to read from stdin: {}", e));
        }
    }

    Ok(Rc::new(Object::Str(input)))
}
