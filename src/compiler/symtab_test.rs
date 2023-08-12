#![allow(unused_imports)]
use std::collections::HashMap;

use super::symtab::Symbol;
use super::symtab::SymbolScope;
use super::symtab::SymbolTable;

#[test]
fn test_define() {
    let mut expected = HashMap::new();
    expected.insert(
        String::from("a"),
        Symbol::new("a", SymbolScope::GlobalScope, 0),
    );
    expected.insert(
        String::from("b"),
        Symbol::new("b", SymbolScope::GlobalScope, 1),
    );

    let mut global = SymbolTable::default();
    let a = global.define("a");
    assert_eq!(*a, expected["a"]);

    let b = global.define("b");
    assert_eq!(*b, expected["b"]);
}

#[test]
fn test_resolve_global() {
    let mut global = SymbolTable::default();
    let _ = global.define("a");
    let _ = global.define("b");

    let expected = vec![
        Symbol::new("a", SymbolScope::GlobalScope, 0),
        Symbol::new("b", SymbolScope::GlobalScope, 1),
    ];

    for sym in expected.iter() {
        match global.resolve(&sym.name) {
            None => panic!("name {} not resolvable", sym.name),
            Some(s) => assert_eq!(sym, &*s),
        }
    }
}
