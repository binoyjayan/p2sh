use crate::evaluator::object::BuiltinFunction;
use crate::evaluator::object::Object;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref BUILTINS: HashMap<String, BuiltinFunction> = {
        let mut m: HashMap<String, BuiltinFunction> = HashMap::new();
        m.insert(
            "len".into(),
            BuiltinFunction::new("len".into(), 1, builtin_len),
        );
        m
    };
}

fn builtin_len(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Str(s)) => Ok(Object::Number(s.len() as f64)),
        _ => Err(String::from("argument to 'len' not supported")),
    }
}
