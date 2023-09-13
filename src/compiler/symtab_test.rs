#![allow(unused_imports)]
use std::collections::HashMap;

use super::symtab::Symbol;
use super::symtab::SymbolScope;
use super::symtab::SymbolTable;

#[test]
fn test_define() {
    let mut expected = HashMap::new();
    expected.insert("a", Symbol::new("a", SymbolScope::Global, 0));
    expected.insert("b", Symbol::new("b", SymbolScope::Global, 1));
    expected.insert("c", Symbol::new("c", SymbolScope::Local, 0));
    expected.insert("d", Symbol::new("d", SymbolScope::Local, 1));
    expected.insert("e", Symbol::new("e", SymbolScope::Local, 0));
    expected.insert("f", Symbol::new("f", SymbolScope::Local, 1));

    let mut global = SymbolTable::default();
    let a = global.define("a");
    assert_eq!(*a, expected["a"]);

    let b = global.define("b");
    assert_eq!(*b, expected["b"]);

    let mut first_local = SymbolTable::new_enclosed(global.clone());
    let c = first_local.define("c");
    assert_eq!(*c, expected["c"]);

    let d = first_local.define("d");
    assert_eq!(*d, expected["d"]);

    let mut second_local = SymbolTable::new_enclosed(first_local.clone());
    let e = second_local.define("e");
    assert_eq!(*e, expected["e"]);

    let f = second_local.define("f");
    assert_eq!(*f, expected["f"]);
}

#[test]
fn test_resolve_global() {
    let mut global = SymbolTable::default();
    let _ = global.define("a");
    let _ = global.define("b");

    let expected = vec![
        Symbol::new("a", SymbolScope::Global, 0),
        Symbol::new("b", SymbolScope::Global, 1),
    ];

    for sym in expected.iter() {
        match global.resolve(&sym.name) {
            None => panic!("name {} not resolvable", sym.name),
            Some(s) => assert_eq!(sym, &*s),
        }
    }
}

#[test]
fn test_resolve_local() {
    let mut global = SymbolTable::default();
    global.define("a");
    global.define("b");

    let mut local = SymbolTable::new_enclosed(global.clone());
    local.define("c");
    local.define("d");

    let expected = vec![
        Symbol::new("a", SymbolScope::Global, 0),
        Symbol::new("b", SymbolScope::Global, 1),
        Symbol::new("c", SymbolScope::Local, 0),
        Symbol::new("d", SymbolScope::Local, 1),
    ];

    for sym_exp in &expected {
        let result = local.resolve(&sym_exp.name);
        assert!(result.is_some(), "name {} not resolvable", sym_exp.name);
        let symbol_eval = result.unwrap();
        assert_eq!(
            *symbol_eval, *sym_exp,
            "expected {} to resolve to {:?}, got={:?}",
            sym_exp.name, sym_exp, symbol_eval
        );
    }
}

#[test]
fn test_resolve_nested_local() {
    struct ResolveTest {
        table: SymbolTable,
        expected_symbols: Vec<Symbol>,
    }
    let mut global = SymbolTable::default();
    global.define("a");
    global.define("b");

    let mut first_local = SymbolTable::new_enclosed(global.clone());
    first_local.define("c");
    first_local.define("d");

    let mut second_local = SymbolTable::new_enclosed(first_local.clone());
    second_local.define("e");
    second_local.define("f");

    let mut tests = vec![
        ResolveTest {
            table: first_local,
            expected_symbols: vec![
                Symbol::new("a", SymbolScope::Global, 0),
                Symbol::new("b", SymbolScope::Global, 1),
                Symbol::new("c", SymbolScope::Local, 0),
                Symbol::new("d", SymbolScope::Local, 1),
            ],
        },
        ResolveTest {
            table: second_local,
            expected_symbols: vec![
                Symbol::new("a", SymbolScope::Global, 0),
                Symbol::new("b", SymbolScope::Global, 1),
                Symbol::new("e", SymbolScope::Local, 0),
                Symbol::new("f", SymbolScope::Local, 1),
            ],
        },
    ];

    for tt in &mut tests {
        for sym in &tt.expected_symbols {
            let result = tt.table.resolve(&sym.name);
            assert!(result.is_some(), "name {} not resolvable", sym.name);
            let symbol_eval = result.unwrap();
            assert_eq!(
                *symbol_eval, *sym,
                "expected {} to resolve to {:?}, got={:?}",
                sym.name, sym, symbol_eval
            );
        }
    }
}

#[test]
fn test_define_resolve_builtins() {
    let expected = vec![
        Symbol::new("a", SymbolScope::Builtin, 0),
        Symbol::new("c", SymbolScope::Builtin, 1),
        Symbol::new("e", SymbolScope::Builtin, 2),
        Symbol::new("f", SymbolScope::Builtin, 3),
    ];

    let mut global = SymbolTable::default();
    for (i, sym) in expected.iter().enumerate() {
        global.define_builtin(i, &sym.name);
    }
    let mut first_local = SymbolTable::new_enclosed(global.clone());
    let mut second_local = SymbolTable::new_enclosed(first_local.clone());

    // Expect every symbol that has been defined in the global scope with
    // define_builtin to resolve to the new BuiltinScope
    for tab in [&mut global, &mut first_local, &mut second_local] {
        for sym_exp in &expected {
            let result = tab.resolve(&sym_exp.name);
            assert!(result.is_some(), "name {} not resolvable", sym_exp.name);
            let symbol_eval = result.unwrap();
            assert_eq!(
                *symbol_eval, *sym_exp,
                "expected {} to resolve to {:?}, got={:?}",
                sym_exp.name, sym_exp, symbol_eval
            );
        }
    }
}

#[test]
fn test_resolve_free() {
    struct ResolveTest {
        table: SymbolTable,
        expected_symbols: Vec<Symbol>,
        expected_free_symbols: Vec<Symbol>,
    }

    let mut global = SymbolTable::default();
    global.define("a");
    global.define("b");

    let mut first_local = SymbolTable::new_enclosed(global.clone());
    first_local.define("c");
    first_local.define("d");

    // inner-most scope
    let mut second_local = SymbolTable::new_enclosed(first_local.clone());
    second_local.define("e");
    second_local.define("f");

    let mut tests = vec![
        ResolveTest {
            table: first_local,
            expected_symbols: vec![
                Symbol::new("a", SymbolScope::Global, 0),
                Symbol::new("b", SymbolScope::Global, 1),
                Symbol::new("c", SymbolScope::Local, 0),
                Symbol::new("d", SymbolScope::Local, 1),
            ],
            expected_free_symbols: Vec::new(),
        },
        ResolveTest {
            table: second_local,
            expected_symbols: vec![
                Symbol::new("a", SymbolScope::Global, 0),
                Symbol::new("b", SymbolScope::Global, 1),
                Symbol::new("c", SymbolScope::Free, 0),
                Symbol::new("d", SymbolScope::Free, 1),
                Symbol::new("e", SymbolScope::Local, 0),
                Symbol::new("f", SymbolScope::Local, 1),
            ],
            expected_free_symbols: vec![
                Symbol::new("c", SymbolScope::Local, 0),
                Symbol::new("d", SymbolScope::Local, 1),
            ],
        },
    ];

    for t in &mut tests {
        // The test expects that all the identifiers used in the
        // arithmetic expressions can be resolved correctly.
        for sym in &t.expected_symbols {
            let result = t.table.resolve(&sym.name);
            assert!(result.is_some(), "name {} not resolvable", sym.name);
            let symbol_eval = result.unwrap();

            assert_eq!(
                *symbol_eval, *sym,
                "expected {:?} to resolve to {:?}, got={:?}",
                sym.name, sym, symbol_eval
            );
        }

        assert_eq!(
            t.table.free_symbols.len(),
            t.expected_free_symbols.len(),
            "wrong number of free symbols. got={}, want={}",
            t.table.free_symbols.len(),
            t.expected_free_symbols.len()
        );
        // Iterate through 'expected_free_symbols' and make sure they match
        // the symbol table’s 'free_symbols'. FreeSymbols should contain the
        // original symbols of the the enclosing scope. For example, when the
        // the symbol table is asked to resolve 'c' and 'd' while being in the
        // 'second_local' scope, return symbols with FreeScope. But at the same
        // time, the original symbols, which were created when the names were
        // defined, should be added to FreeSymbols. A "free variable" is a
        // relative term. A free variable in the current scope could be a local
        // binding in the enclosing scope.
        for (i, sym) in t.expected_free_symbols.iter().enumerate() {
            let result = &t.table.free_symbols[i].clone();
            assert_eq!(
                **result, *sym,
                "wrong free symbol. got={:?}, want={:?}",
                result, sym
            );
        }
    }
}

// make sure that the symbol table does not automatically mark
// every symbol as a free variable if it can't resolve it.
// #[test]
#[test]
fn test_resolve_unresolvable_free() {
    let mut global = SymbolTable::default();
    global.define("a");
    let mut first_local = SymbolTable::new_enclosed(global.clone());
    first_local.define("c");
    let mut second_local = SymbolTable::new_enclosed(first_local.clone());
    second_local.define("e");
    second_local.define("f");

    let expected = vec![
        Symbol::new("a", SymbolScope::Global, 0),
        Symbol::new("c", SymbolScope::Free, 0),
        Symbol::new("e", SymbolScope::Local, 0),
        Symbol::new("f", SymbolScope::Local, 1),
    ];

    for sym in &expected {
        let result = second_local.resolve(&sym.name);
        assert!(result.is_some(), "name {} not resolvable", sym.name);
        let symbol_eval = result.unwrap();

        assert_eq!(
            *symbol_eval, *sym,
            "expected {} to resolve to {:?}, got={:?}",
            sym.name, sym, symbol_eval
        );
    }

    let expected_unresolvable = vec!["b", "d"];

    for name in &expected_unresolvable {
        let result = second_local.resolve(name);
        assert!(
            result.is_none(),
            "name {} resolved, but was unexpected not to",
            name
        );
    }
}

#[test]
fn test_define_and_resolve_function_name() {
    let mut global = SymbolTable::default();
    let _ = global.define_function_name("a");

    let expected = Symbol::new("a", SymbolScope::Function, 0);

    match global.resolve(&expected.name) {
        None => panic!("name {} not resolvable", expected.name),
        Some(s) => assert_eq!(expected, *s, "mismatch in function names"),
    }
}

#[test]
fn test_shadowing_function_name() {
    let mut global = SymbolTable::default();
    let _ = global.define_function_name("a");
    let _ = global.define("a");

    let expected = Symbol::new("a", SymbolScope::Global, 0);

    match global.resolve(&expected.name) {
        None => panic!("name {} not resolvable", expected.name),
        Some(s) => assert_eq!(expected, *s, "mismatch in function names"),
    }
}
