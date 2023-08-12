use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: usize,
}

impl Symbol {
    pub fn new(name: &str, scope: SymbolScope, index: usize) -> Self {
        Self {
            name: name.to_string(),
            scope,
            index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolScope {
    GlobalScope,
}

impl fmt::Display for SymbolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolScope::GlobalScope => write!(f, "GLOBAL"),
        }
    }
}

#[derive(Default)]
pub struct SymbolTable {
    store: HashMap<String, Rc<Symbol>>,
    num_definitions: usize,
}

impl SymbolTable {
    pub fn define(&mut self, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(
            name,
            SymbolScope::GlobalScope,
            self.num_definitions,
        ));
        self.store.insert(name.to_string(), Rc::clone(&symbol));
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Rc<Symbol>> {
        self.store.get(name).cloned()
    }
}
