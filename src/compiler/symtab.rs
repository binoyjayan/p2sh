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
    Global,
    Local,
    Builtin,
}

impl fmt::Display for SymbolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolScope::Global => write!(f, "GLOBAL"),
            SymbolScope::Local => write!(f, "LOCAL"),
            SymbolScope::Builtin => write!(f, "BUILTIN"),
        }
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct SymbolTable {
    store: HashMap<String, Rc<Symbol>>,
    pub num_definitions: usize,
    pub outer: Option<Rc<SymbolTable>>,
}

impl SymbolTable {
    pub fn new_enclosed(outer: SymbolTable) -> SymbolTable {
        SymbolTable {
            store: HashMap::new(),
            num_definitions: 0,
            outer: Some(Rc::new(outer)),
        }
    }

    // If the SymbolTable being called is not enclosed in another SymbolTable,
    // i.e. its outer field is not set, then its scope is global.
    // If it is enclosed, the scope is local.
    pub fn define(&mut self, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(
            name,
            if self.outer.is_none() {
                SymbolScope::Global
            } else {
                SymbolScope::Local
            },
            self.num_definitions,
        ));

        self.store.insert(name.to_string(), Rc::clone(&symbol));
        self.num_definitions += 1;

        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Rc<Symbol>> {
        if let Some(symbol_ref) = self.store.get(name) {
            Some(symbol_ref.clone())
        } else if let Some(outer) = &self.outer {
            outer.resolve(name)
        } else {
            None
        }
    }

    pub fn define_builtin(&mut self, index: usize, name: &str) -> Symbol {
        let symbol = Symbol::new(name, SymbolScope::Builtin, index);
        self.store.insert(name.to_string(), Rc::new(symbol.clone()));
        symbol
    }
}
