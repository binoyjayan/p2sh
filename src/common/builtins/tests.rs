#![allow(unused_imports)]
use std::fmt;
use std::rc::Rc;

use super::builtin_format;
use super::format_buf;
use super::Object;
use std::collections::HashMap;

#[test]
fn test_builtin_function_format() {
    struct FormatTest {
        args: Vec<Rc<Object>>,
        expected: &'static str,
    }
    let format_tests = vec![
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{}".to_string())),
                Rc::new(Object::Number(69420.)),
            ],
            expected: "69420",
        },
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:b}".to_string())),
                Rc::new(Object::Number(69420.)),
            ],
            expected: "10000111100101100",
        },
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:o}".to_string())),
                Rc::new(Object::Number(69420.)),
            ],
            expected: "207454",
        },
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:x}".to_string())),
                Rc::new(Object::Number(69420.)),
            ],
            expected: "10f2c",
        },
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:X}".to_string())),
                Rc::new(Object::Number(69420.)),
            ],
            expected: "10F2C",
        },
        // Default justify (numbers)
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:5}".to_string())),
                Rc::new(Object::Number(1.)),
            ],
            expected: "    1",
        },
        // Right justify
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:>5}".to_string())),
                Rc::new(Object::Number(1.)),
            ],
            expected: "    1",
        },
        // Left justify
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:<5}".to_string())),
                Rc::new(Object::Number(1.)),
            ],
            expected: "1    ",
        },
        // Right justify with padding
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:0>5}".to_string())),
                Rc::new(Object::Number(1.)),
            ],
            expected: "00001",
        },
        // Left justify with padding
        FormatTest {
            args: vec![
                Rc::new(Object::Str("{:0<5}".to_string())),
                Rc::new(Object::Number(1.)),
            ],
            expected: "10000",
        },
        // Indexed arguments
        FormatTest {
            args: vec![
                Rc::new(Object::Str("My name is {1}, {0} {1}".to_string())),
                Rc::new(Object::Str("James".to_string())),
                Rc::new(Object::Str("Bond".to_string())),
            ],
            expected: "My name is Bond, James Bond",
        },
        FormatTest {
            args: vec![
                Rc::new(Object::Str(
                    "{0:<10},{1:0<05},{2},{3:b},{3:o},{4:x},{4:X}".to_string(),
                )),
                Rc::new(Object::Str("Hello".to_string())),
                Rc::new(Object::Number(1.)),
                Rc::new(Object::Bool(true)),
                Rc::new(Object::Number(10.)),
                Rc::new(Object::Number(65535.)),
            ],
            expected: "Hello     ,10000,true,1010,12,ffff,FFFF",
        },
    ];

    let mut count: usize = 0;
    for (i, t) in format_tests.iter().enumerate() {
        let result = builtin_format(t.args.clone());
        if let Ok(obj) = result {
            if let Object::Str(s) = &*obj {
                if s != t.expected {
                    eprintln!("[{}] Incorrect result. want: {}, got: {}", i, t.expected, s);
                    count += 1;
                }
            } else {
                eprintln!("[{}] Test failed: Bad result", i);
                count += 1;
            }
        } else {
            eprintln!("[{}] Test failed: Bad result", i);
            count += 1;
        }
    }
    if count != 0 {
        panic!("{} format tests failed", count);
    }
}
