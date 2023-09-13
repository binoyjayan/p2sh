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
    Free,
    Function,
}

impl fmt::Display for SymbolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolScope::Global => write!(f, "GLOBAL"),
            SymbolScope::Local => write!(f, "LOCAL"),
            SymbolScope::Builtin => write!(f, "BUILTIN"),
            SymbolScope::Free => write!(f, "FREE"),
            SymbolScope::Function => write!(f, "FUNCTION"),
        }
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct SymbolTable {
    store: HashMap<String, Rc<Symbol>>,
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

    pub fn define_function_name(&mut self, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(name, SymbolScope::Function, 0));
        self.store.insert(name.to_string(), Rc::clone(&symbol));
        symbol
    }

    pub fn resolve(&mut self, name: &str) -> Option<Rc<Symbol>> {
        let symbol = self.store.get(name).cloned();
        if let Some(symbol_ref) = symbol {
            return Some(Rc::clone(&symbol_ref));
        } else if let Some(outer) = &mut self.outer {
            if let Some(obj) = outer.resolve(name) {
                if matches!(obj.scope, SymbolScope::Global | SymbolScope::Builtin) {
                    return Some(obj);
                } else {
                    return Some(self.define_free(obj));
                }
            }
        }
        None
    }

    pub fn define_builtin(&mut self, index: usize, name: &str) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol::new(name, SymbolScope::Builtin, index));
        self.store.insert(name.to_string(), Rc::clone(&symbol));
        symbol
    }

    fn define_free(&mut self, original: Rc<Symbol>) -> Rc<Symbol> {
        self.free_symbols.push(original.clone());
        let len = self.free_symbols.len();

        let symbol = Rc::new(Symbol::new(&original.name, SymbolScope::Free, len - 1));

        self.store.insert(symbol.name.clone(), symbol.clone());

        symbol
    }
}
