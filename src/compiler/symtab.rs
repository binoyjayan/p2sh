use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// Symbol is a struct that represents a symbol.
// It has a name, scope, index, and depth.
// The scope is the scope in which the symbol is defined.
// The index is the index of the symbol used by the VM at runtime.
// The depth is the depth of the scope in which the symbol is defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
    pub depth: usize,
}

impl Symbol {
    pub fn new(name: &str, scope: SymbolScope, index: usize, depth: usize) -> Self {
        Self {
            name: name.to_string(),
            scope,
            index,
            depth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolScope {
    Global,
    Local,
    BuiltinFn,
    BuiltinVar,
    Free,
    Function,
}

impl fmt::Display for SymbolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolScope::Global => write!(f, "GLOBAL"),
            SymbolScope::Local => write!(f, "LOCAL"),
            SymbolScope::BuiltinFn => write!(f, "BUILTINFN"),
            SymbolScope::BuiltinVar => write!(f, "BUILTINVAR"),
            SymbolScope::Free => write!(f, "FREE"),
            SymbolScope::Function => write!(f, "FUNCTION"),
        }
    }
}

/// Implementation of symbol table to store multiple symbols
/// for the same name, with the ability to resolve to the most
/// recently defined symbol. It also has ability to store
/// free symbols, which are symbols that are not defined in the
/// current scope, but in an enclosing scope.
/// This is used for closures, where the free symbols are
/// variables that are defined in the enclosing scope.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct SymbolTable {
    store: HashMap<String, Vec<Rc<Symbol>>>,
    num_definitions: usize,
    pub outer: Option<Box<SymbolTable>>,
    // original symbols of the enclosing scope
    pub free_symbols: Vec<Rc<Symbol>>,
}

impl SymbolTable {
    pub fn new_enclosed(outer: SymbolTable) -> SymbolTable {
        SymbolTable {
            store: HashMap::new(),
            num_definitions: 0,
            outer: Some(Box::new(outer)),
            free_symbols: Vec::new(),
        }
    }

    pub fn get_num_definitions(&self) -> usize {
        self.num_definitions
    }

    // If the SymbolTable being called is not enclosed in another SymbolTable,
    // i.e. its outer field is not set, then its scope is global.
    // If it is enclosed, the scope is local.
    pub fn define(&mut self, name: &str, depth: usize) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(
            name,
            if self.outer.is_none() {
                SymbolScope::Global
            } else {
                SymbolScope::Local
            },
            self.num_definitions,
            depth, // Initialize the depth
        ));

        self.store
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(Rc::clone(&symbol));

        self.num_definitions += 1;

        symbol
    }

    pub fn define_function_name(&mut self, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(name, SymbolScope::Function, 0, 0));
        self.store
            .insert(name.to_string(), vec![Rc::clone(&symbol)]);
        symbol
    }

    // Resolve a symbol by name and depth. If the symbol is not found in the
    // current scope, it will look in the outer scope. If the symbol is not
    // found in the outer scope, it will return None. If the symbol is found
    // in the outer scope, it will return a new symbol with the scope set to
    // Free if the symbol is not Global or in-built's.
    // If there are more than one symbol with the same name, it will return
    // the most recently defined symbol with a depth less than or equal to
    // the given depth.
    pub fn resolve(&mut self, name: &str, depth: usize) -> Option<Rc<Symbol>> {
        if let Some(symbols) = self.store.get(name) {
            for symbol in symbols.iter().rev() {
                if symbol.depth <= depth {
                    return Some(Rc::clone(symbol));
                }
            }
        } else if let Some(outer) = &mut self.outer {
            if let Some(obj) = outer.resolve(name, depth) {
                if matches!(
                    obj.scope,
                    SymbolScope::Global | SymbolScope::BuiltinFn | SymbolScope::BuiltinVar
                ) {
                    return Some(obj);
                } else {
                    return Some(self.define_free(obj));
                }
            }
        }
        None
    }

    pub fn define_builtin_fn(&mut self, index: usize, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(name, SymbolScope::BuiltinFn, index, 0));
        self.store
            .insert(name.to_string(), vec![Rc::clone(&symbol)]);
        symbol
    }

    pub fn define_builtin_var(&mut self, index: usize, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(name, SymbolScope::BuiltinVar, index, 0));
        self.store
            .insert(name.to_string(), vec![Rc::clone(&symbol)]);
        symbol
    }

    fn define_free(&mut self, original: Rc<Symbol>) -> Rc<Symbol> {
        self.free_symbols.push(original.clone());
        let len = self.free_symbols.len();

        let symbol = Rc::new(Symbol::new(
            &original.name,
            SymbolScope::Free,
            len - 1,
            original.depth,
        ));

        self.store.insert(symbol.name.clone(), vec![symbol.clone()]);

        symbol
    }
}
