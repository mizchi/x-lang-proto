//! Symbol interning for efficient string handling
//! 
//! This module provides a global symbol table for interning strings,
//! which is essential for performance when dealing with many identifiers.

#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::fmt;

/// Interned string symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Symbol(u32);

impl Symbol {
    /// Intern a string and return its symbol
    pub fn intern(s: &str) -> Self {
        GlobalInterner::get().with_mut(|interner| interner.intern(s))
    }
    
    /// Get the string representation of this symbol
    pub fn as_str(self) -> &'static str {
        GlobalInterner::get().with(|interner| interner.resolve(self))
    }
    
    /// Get the raw symbol ID
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    /// Create a symbol from a raw ID (unsafe - only for deserialization)
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the provided `id` corresponds to a valid symbol
    /// that has been previously interned. Using an invalid id will result in undefined behavior.
    pub unsafe fn from_u32(id: u32) -> Self {
        Symbol(id)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol::intern(s)
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol::intern(&s)
    }
}

/// Thread-safe symbol interner
struct SymbolInterner {
    symbols: Vec<String>,
    indices: HashMap<String, u32>,
}

impl SymbolInterner {
    fn new() -> Self {
        SymbolInterner {
            symbols: Vec::new(),
            indices: HashMap::new(),
        }
    }
    
    fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&index) = self.indices.get(s) {
            Symbol(index)
        } else {
            let index = self.symbols.len() as u32;
            self.symbols.push(s.to_string());
            self.indices.insert(s.to_string(), index);
            Symbol(index)
        }
    }
    
    fn resolve(&self, symbol: Symbol) -> &'static str {
        // SAFETY: This is a memory leak, but it's intentional for symbols
        // In a real implementation, you'd use Arena allocation or Arc<str>
        unsafe {
            let s = &self.symbols[symbol.0 as usize];
            std::mem::transmute::<&str, &'static str>(s.as_str())
        }
    }
    
    fn len(&self) -> usize {
        self.symbols.len()
    }
}

/// Global symbol interner with thread-safe access
struct GlobalInterner {
    inner: Mutex<SymbolInterner>,
}

impl GlobalInterner {
    fn new() -> Self {
        GlobalInterner {
            inner: Mutex::new(SymbolInterner::new()),
        }
    }
    
    fn with_mut<R>(&self, f: impl FnOnce(&mut SymbolInterner) -> R) -> R {
        f(&mut self.inner.lock().unwrap())
    }
    
    fn with<R>(&self, f: impl FnOnce(&SymbolInterner) -> R) -> R {
        f(&self.inner.lock().unwrap())
    }
}

static INTERNER: OnceLock<GlobalInterner> = OnceLock::new();

impl GlobalInterner {
    fn get() -> &'static GlobalInterner {
        INTERNER.get_or_init(GlobalInterner::new)
    }
}

/// Commonly used symbols (pre-interned for efficiency)
pub mod symbols {
    use super::Symbol;
    use std::sync::OnceLock;
    
    macro_rules! define_symbols {
        ($($name:ident = $value:literal),* $(,)?) => {
            $(
                pub fn $name() -> Symbol {
                    static SYMBOL: OnceLock<Symbol> = OnceLock::new();
                    *SYMBOL.get_or_init(|| Symbol::intern($value))
                }
            )*
        };
    }
    
    define_symbols! {
        // Special symbols
        UNDERSCORE = "_",
        UNIT = "()",
        
        // Built-in types
        INT = "Int",
        FLOAT = "Float", 
        STRING = "String",
        BOOL = "Bool",
        UNIT_TYPE = "Unit",
        
        // Built-in effects
        IO = "IO",
        STATE = "State",
        EXCEPT = "Except",
        ASYNC = "Async",
        
        // Keywords
        LET = "let",
        FUN = "fun",
        IN = "in",
        IF = "if",
        THEN = "then",
        ELSE = "else",
        MATCH = "match",
        WITH = "with",
        DO = "do",
        HANDLE = "handle",
        RESUME = "resume",
        RETURN = "return",
        EFFECT = "effect",
        HANDLER = "handler",
        TYPE = "type",
        DATA = "data",
        MODULE = "module",
        IMPORT = "import",
        EXPORT = "export",
        PURE = "pure",
        FORALL = "forall",
        
        // Operators
        PLUS = "+",
        MINUS = "-",
        MULTIPLY = "*",
        DIVIDE = "/",
        EQUAL = "=",
        EQUAL_EQUAL = "==",
        NOT_EQUAL = "!=",
        LESS = "<",
        LESS_EQUAL = "<=",
        GREATER = ">",
        GREATER_EQUAL = ">=",
        AND = "&&",
        OR = "||",
        NOT = "!",
        ARROW = "->",
        FAT_ARROW = "=>",
        PIPE = "|",
        CONS = "::",
        
        // Common function names
        MAP = "map",
        FILTER = "filter",
        FOLD = "fold",
        PRINT = "print",
        READ = "read",
        WRITE = "write",
        GET = "get",
        PUT = "put",
        THROW = "throw",
        CATCH = "catch",
        
        // Standard library modules
        PRELUDE = "Prelude",
        LIST = "List",
        OPTION = "Option",
        RESULT = "Result",
        CORE = "Core",
        STD = "Std",
        
        // LSP-related
        MAIN = "main",
        INIT = "init",
        UPDATE = "update",
        VIEW = "view",
    }
    
    /// Get all predefined symbols for initialization
    pub fn all_predefined() -> Vec<Symbol> {
        vec![
            UNDERSCORE(), UNIT(), INT(), FLOAT(), STRING(), BOOL(), UNIT_TYPE(),
            IO(), STATE(), EXCEPT(), ASYNC(), LET(), FUN(), IN(), IF(), THEN(),
            ELSE(), MATCH(), WITH(), DO(), HANDLE(), RESUME(), RETURN(), EFFECT(),
            HANDLER(), TYPE(), DATA(), MODULE(), IMPORT(), EXPORT(), PURE(), FORALL(),
            PLUS(), MINUS(), MULTIPLY(), DIVIDE(), EQUAL(), EQUAL_EQUAL(), NOT_EQUAL(),
            LESS(), LESS_EQUAL(), GREATER(), GREATER_EQUAL(), AND(), OR(), NOT(),
            ARROW(), FAT_ARROW(), PIPE(), CONS(), MAP(), FILTER(), FOLD(), PRINT(),
            READ(), WRITE(), GET(), PUT(), THROW(), CATCH(), PRELUDE(), LIST(),
            OPTION(), RESULT(), CORE(), STD(), MAIN(), INIT(), UPDATE(), VIEW(),
        ]
    }
}

/// Symbol table for managing scoped symbols
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<Symbol, SymbolInfo>>,
}

/// Information about a symbol in scope
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub kind: SymbolKind,
    pub visibility: SymbolVisibility,
    pub span: crate::span::Span,
    pub module: Option<Symbol>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Local variable
    Variable,
    /// Function
    Function,
    /// Type constructor
    Type,
    /// Data constructor
    Constructor,
    /// Effect
    Effect,
    /// Effect operation
    Operation,
    /// Module
    Module,
    /// Type parameter
    TypeParameter,
    /// Effect parameter
    EffectParameter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolVisibility {
    Local,
    Private,
    Public,
    Exported,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Option<HashMap<Symbol, SymbolInfo>> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None
        }
    }
    
    /// Insert a symbol in the current scope
    pub fn insert(&mut self, info: SymbolInfo) -> Option<SymbolInfo> {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(info.symbol, info)
        } else {
            None
        }
    }
    
    /// Lookup a symbol in all scopes (innermost first)
    pub fn lookup(&self, symbol: Symbol) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(&symbol) {
                return Some(info);
            }
        }
        None
    }
    
    /// Check if a symbol exists in the current scope
    pub fn exists_in_current_scope(&self, symbol: Symbol) -> bool {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.contains_key(&symbol)
        } else {
            false
        }
    }
    
    /// Get all symbols in the current scope
    pub fn current_scope_symbols(&self) -> Vec<&SymbolInfo> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.values().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all visible symbols (for completion)
    pub fn all_visible_symbols(&self) -> Vec<&SymbolInfo> {
        let mut symbols = Vec::new();
        for scope in &self.scopes {
            symbols.extend(scope.values());
        }
        symbols
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the symbol interner with predefined symbols
pub fn init_symbols() {
    // Pre-intern common symbols for better performance
    let _symbols = symbols::all_predefined();
}

/// Get statistics about the symbol interner
pub fn interner_stats() -> InternerStats {
    GlobalInterner::get().with(|interner| {
        InternerStats {
            symbol_count: interner.len(),
        }
    })
}

#[derive(Debug, Clone)]
pub struct InternerStats {
    pub symbol_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, ByteOffset, Span};

    #[test]
    fn test_symbol_interning() {
        let s1 = Symbol::intern("hello");
        let s2 = Symbol::intern("hello");
        let s3 = Symbol::intern("world");
        
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s3.as_str(), "world");
    }

    #[test]
    fn test_predefined_symbols() {
        let int_sym = symbols::INT();
        let string_sym = symbols::STRING();
        
        assert_eq!(int_sym.as_str(), "Int");
        assert_eq!(string_sym.as_str(), "String");
        assert_ne!(int_sym, string_sym);
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
        
        let var_info = SymbolInfo {
            symbol: Symbol::intern("x"),
            kind: SymbolKind::Variable,
            visibility: SymbolVisibility::Local,
            span,
            module: None,
        };
        
        table.insert(var_info.clone());
        
        let found = table.lookup(Symbol::intern("x"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().symbol, Symbol::intern("x"));
        
        // Test scoping
        table.enter_scope();
        assert!(!table.exists_in_current_scope(Symbol::intern("x")));
        assert!(table.lookup(Symbol::intern("x")).is_some()); // Still visible from outer scope
        
        table.exit_scope();
        assert!(table.lookup(Symbol::intern("x")).is_some());
    }

    #[test]
    fn test_symbol_conversion() {
        let sym1: Symbol = "test".into();
        let sym2: Symbol = "test".to_string().into();
        
        assert_eq!(sym1, sym2);
        assert_eq!(sym1.as_str(), "test");
    }
}