//! Context management for AI code generation
//! 
//! This module manages the context in which code is generated, including
//! available symbols, types, and the current scope.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use x_parser::{Symbol, Span, FileId, span::ByteOffset};
use x_parser::ast::*;
use x_checker::types::{Type as CheckerType, TypeScheme};
use crate::intent::{CodeIntent, IntentTarget};

/// Code generation context
#[derive(Debug, Clone)]
pub struct CodeGenContext {
    /// Current module being generated
    pub current_module: Option<Symbol>,
    
    /// Available symbols in the current scope
    pub symbols: SymbolScope,
    
    /// Available types
    pub types: TypeScope,
    
    /// Imported modules
    pub imports: HashSet<ModulePath>,
    
    /// Current function context (if inside a function)
    pub function_context: Option<FunctionContext>,
    
    /// Generated items in the current session
    pub generated_items: Vec<GeneratedItem>,
    
    /// User preferences and constraints
    pub preferences: UserPreferences,
}

/// Symbol scope tracking
#[derive(Debug, Clone)]
pub struct SymbolScope {
    /// Stack of scopes (innermost first)
    scopes: Vec<HashMap<Symbol, SymbolInfo>>,
    
    /// Global symbols
    globals: HashMap<Symbol, SymbolInfo>,
}

/// Information about a symbol
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub typ: Option<TypeScheme>,
    pub defined_in: Option<ModulePath>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Value,
    Function,
    Constructor,
    Module,
    Type,
    Effect,
}

/// Type scope tracking
#[derive(Debug, Clone)]
pub struct TypeScope {
    /// Available types
    types: HashMap<Symbol, TypeInfo>,
    
    /// Type aliases
    aliases: HashMap<Symbol, CheckerType>,
}

/// Information about a type
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub kind: TypeKind,
    pub params: Vec<Symbol>,
    pub constructors: Vec<Symbol>,
    pub defined_in: Option<ModulePath>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Data,
    Alias,
    Abstract,
    Builtin,
}

/// Context for function generation
#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub name: Symbol,
    pub parameters: Vec<(Symbol, Option<CheckerType>)>,
    pub return_type: Option<CheckerType>,
    pub local_bindings: HashMap<Symbol, CheckerType>,
    pub used_effects: HashSet<Symbol>,
}

/// Previously generated item
#[derive(Debug, Clone)]
pub struct GeneratedItem {
    pub name: Symbol,
    pub kind: GeneratedItemKind,
    pub ast: Item,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum GeneratedItemKind {
    Function,
    Type,
    Value,
    Module,
    Effect,
}

/// User preferences for code generation
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub style: CodeStyle,
    pub naming_convention: NamingConvention,
    pub prefer_point_free: bool,
    pub prefer_explicit_types: bool,
    pub max_line_length: usize,
}

#[derive(Debug, Clone)]
pub enum CodeStyle {
    Functional,
    Imperative,
    Mixed,
}

#[derive(Debug, Clone)]
pub enum NamingConvention {
    CamelCase,
    SnakeCase,
    PascalCase,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            current_module: None,
            symbols: SymbolScope::new(),
            types: TypeScope::new(),
            imports: HashSet::new(),
            function_context: None,
            generated_items: Vec::new(),
            preferences: UserPreferences::default(),
        }
    }
    
    /// Add a generated item to the context
    pub fn add_generated_item(&mut self, item: GeneratedItem) {
        // Update symbol table
        match &item.kind {
            GeneratedItemKind::Function => {
                if let Item::ValueDef(def) = &item.ast {
                    let module_path = self.current_module.as_ref()
                        .map(|name| ModulePath::single(*name, Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1))));
                    self.symbols.add_function(
                        def.name,
                        None, // Type will be inferred
                        module_path,
                        def.visibility.clone(),
                    );
                }
            }
            GeneratedItemKind::Type => {
                if let Item::TypeDef(def) = &item.ast {
                    self.types.add_type(
                        def.name,
                        match &def.kind {
                            TypeDefKind::Data(_) => TypeKind::Data,
                            TypeDefKind::Alias(_) => TypeKind::Alias,
                            TypeDefKind::Abstract => TypeKind::Abstract,
                        },
                        def.type_params.iter().map(|p| p.name).collect(),
                        self.current_module.as_ref()
                            .map(|name| ModulePath::single(*name, Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1)))),
                    );
                }
            }
            _ => {}
        }
        
        self.generated_items.push(item);
    }
    
    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.symbols.push_scope();
    }
    
    /// Exit the current scope
    pub fn pop_scope(&mut self) {
        self.symbols.pop_scope();
    }
    
    /// Enter a function context
    pub fn enter_function(&mut self, name: Symbol, params: Vec<(Symbol, Option<CheckerType>)>) {
        self.function_context = Some(FunctionContext {
            name,
            parameters: params,
            return_type: None,
            local_bindings: HashMap::new(),
            used_effects: HashSet::new(),
        });
    }
    
    /// Exit the function context
    pub fn exit_function(&mut self) {
        self.function_context = None;
    }
    
    /// Find similar names (for suggestions)
    pub fn find_similar_names(&self, name: &str) -> Vec<(Symbol, f64)> {
        let mut candidates = Vec::new();
        
        // Check all symbols
        self.symbols.all_symbols().into_iter().for_each(|sym| {
            let sym_str = sym.as_str();
            let distance = levenshtein_distance(name, sym_str);
            let similarity = 1.0 - (distance as f64 / name.len().max(sym_str.len()) as f64);
            
            if similarity > 0.6 {
                candidates.push((sym, similarity));
            }
        });
        
        // Sort by similarity
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        candidates
    }
    
    /// Get available constructors for a type
    pub fn get_constructors(&self, type_name: Symbol) -> Option<Vec<Symbol>> {
        self.types.get_type_info(type_name)
            .map(|info| info.constructors.clone())
    }
    
    /// Check if a symbol is in scope
    pub fn is_in_scope(&self, symbol: Symbol) -> bool {
        self.symbols.lookup(symbol).is_some()
    }
    
    /// Get all available effects
    pub fn available_effects(&self) -> Vec<Symbol> {
        self.symbols.all_symbols()
            .into_iter()
            .filter(|sym| {
                matches!(
                    self.symbols.lookup(*sym).map(|info| &info.kind),
                    Some(SymbolKind::Effect)
                )
            })
            .collect()
    }
}

impl SymbolScope {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            globals: HashMap::new(),
        }
    }
    
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    
    pub fn add_local(&mut self, name: Symbol, info: SymbolInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }
    
    pub fn add_global(&mut self, name: Symbol, info: SymbolInfo) {
        self.globals.insert(name, info);
    }
    
    pub fn add_function(
        &mut self,
        name: Symbol,
        typ: Option<TypeScheme>,
        module: Option<ModulePath>,
        visibility: Visibility,
    ) {
        let info = SymbolInfo {
            kind: SymbolKind::Function,
            typ,
            defined_in: module,
            visibility,
        };
        self.add_global(name, info);
    }
    
    pub fn lookup(&self, name: Symbol) -> Option<&SymbolInfo> {
        // Search local scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(&name) {
                return Some(info);
            }
        }
        
        // Then check globals
        self.globals.get(&name)
    }
    
    pub fn all_symbols(&self) -> Vec<Symbol> {
        let mut symbols = HashSet::new();
        
        // Collect from all scopes
        for scope in &self.scopes {
            symbols.extend(scope.keys().cloned());
        }
        
        // Add globals
        symbols.extend(self.globals.keys().cloned());
        
        symbols.into_iter().collect()
    }
}

impl TypeScope {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
    
    pub fn add_type(
        &mut self,
        name: Symbol,
        kind: TypeKind,
        params: Vec<Symbol>,
        module: Option<ModulePath>,
    ) {
        let info = TypeInfo {
            kind,
            params,
            constructors: Vec::new(),
            defined_in: module,
        };
        self.types.insert(name, info);
    }
    
    pub fn add_constructor(&mut self, type_name: Symbol, constructor: Symbol) {
        if let Some(info) = self.types.get_mut(&type_name) {
            info.constructors.push(constructor);
        }
    }
    
    pub fn add_alias(&mut self, name: Symbol, target: CheckerType) {
        self.aliases.insert(name, target);
    }
    
    pub fn get_type_info(&self, name: Symbol) -> Option<&TypeInfo> {
        self.types.get(&name)
    }
    
    pub fn resolve_alias(&self, name: Symbol) -> Option<&CheckerType> {
        self.aliases.get(&name)
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            style: CodeStyle::Functional,
            naming_convention: NamingConvention::SnakeCase,
            prefer_point_free: false,
            prefer_explicit_types: false,
            max_line_length: 80,
        }
    }
}

/// Compute Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }
    
    matrix[len1][len2]
}

/// Context builder for specific intents
pub struct ContextBuilder {
    context: CodeGenContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: CodeGenContext::new(),
        }
    }
    
    /// Build context for an intent
    pub fn build_for_intent(&mut self, intent: &CodeIntent) -> Result<CodeGenContext> {
        // Add standard library symbols
        self.add_stdlib_symbols();
        
        // Configure based on intent target
        match &intent.target {
            IntentTarget::Function { .. } => {
                // Function generation might need common utility functions
                self.add_common_functions();
            }
            IntentTarget::DataType { .. } => {
                // Data type generation might need common types
                self.add_common_types();
            }
            IntentTarget::Algorithm { name, .. } => {
                // Algorithm implementation might need specific utilities
                self.add_algorithm_utilities(name);
            }
            _ => {}
        }
        
        // Apply constraints to preferences
        for constraint in &intent.constraints {
            self.apply_constraint(constraint);
        }
        
        Ok(self.context.clone())
    }
    
    fn add_stdlib_symbols(&mut self) {
        // Add built-in functions
        let builtins = vec![
            ("print", SymbolKind::Function),
            ("print_endline", SymbolKind::Function),
            ("string_of_int", SymbolKind::Function),
            ("int_of_string", SymbolKind::Function),
            ("float_of_int", SymbolKind::Function),
            ("int_of_float", SymbolKind::Function),
            ("+", SymbolKind::Function),
            ("-", SymbolKind::Function),
            ("*", SymbolKind::Function),
            ("/", SymbolKind::Function),
            ("==", SymbolKind::Function),
            ("!=", SymbolKind::Function),
            (">", SymbolKind::Function),
            ("<", SymbolKind::Function),
            (">=", SymbolKind::Function),
            ("<=", SymbolKind::Function),
            ("&&", SymbolKind::Function),
            ("||", SymbolKind::Function),
            ("not", SymbolKind::Function),
        ];
        
        for (name, kind) in builtins {
            self.context.symbols.add_global(
                Symbol::intern(name),
                SymbolInfo {
                    kind,
                    typ: None,
                    defined_in: None,
                    visibility: Visibility::Public,
                },
            );
        }
        
        // Add built-in types
        let builtin_types = vec![
            ("Int", TypeKind::Builtin),
            ("Float", TypeKind::Builtin),
            ("String", TypeKind::Builtin),
            ("Bool", TypeKind::Builtin),
            ("Unit", TypeKind::Builtin),
            ("List", TypeKind::Builtin),
            ("Option", TypeKind::Builtin),
            ("Result", TypeKind::Builtin),
        ];
        
        for (name, kind) in builtin_types {
            self.context.types.add_type(
                Symbol::intern(name),
                kind,
                Vec::new(),
                None,
            );
        }
    }
    
    fn add_common_functions(&mut self) {
        // Add commonly used functions
        let common_fns = vec![
            ("map", SymbolKind::Function),
            ("filter", SymbolKind::Function),
            ("fold_left", SymbolKind::Function),
            ("fold_right", SymbolKind::Function),
            ("length", SymbolKind::Function),
            ("append", SymbolKind::Function),
            ("concat", SymbolKind::Function),
            ("reverse", SymbolKind::Function),
        ];
        
        for (name, kind) in common_fns {
            self.context.symbols.add_global(
                Symbol::intern(name),
                SymbolInfo {
                    kind,
                    typ: None,
                    defined_in: None,
                    visibility: Visibility::Public,
                },
            );
        }
    }
    
    fn add_common_types(&mut self) {
        // Option constructors
        self.context.types.add_constructor(
            Symbol::intern("Option"),
            Symbol::intern("None"),
        );
        self.context.types.add_constructor(
            Symbol::intern("Option"),
            Symbol::intern("Some"),
        );
        
        // Result constructors
        self.context.types.add_constructor(
            Symbol::intern("Result"),
            Symbol::intern("Ok"),
        );
        self.context.types.add_constructor(
            Symbol::intern("Result"),
            Symbol::intern("Error"),
        );
        
        // List constructors
        self.context.types.add_constructor(
            Symbol::intern("List"),
            Symbol::intern("[]"),
        );
        self.context.types.add_constructor(
            Symbol::intern("List"),
            Symbol::intern("::"),
        );
    }
    
    fn add_algorithm_utilities(&mut self, algorithm_name: &str) {
        match algorithm_name.to_lowercase().as_str() {
            "sort" | "quicksort" | "mergesort" => {
                // Add comparison utilities
                self.context.symbols.add_global(
                    Symbol::intern("compare"),
                    SymbolInfo {
                        kind: SymbolKind::Function,
                        typ: None,
                        defined_in: None,
                        visibility: Visibility::Public,
                    },
                );
            }
            "search" | "binary_search" => {
                // Add array/list utilities
                self.context.symbols.add_global(
                    Symbol::intern("array_get"),
                    SymbolInfo {
                        kind: SymbolKind::Function,
                        typ: None,
                        defined_in: None,
                        visibility: Visibility::Public,
                    },
                );
            }
            _ => {}
        }
    }
    
    fn apply_constraint(&mut self, constraint: &crate::intent::Constraint) {
        use crate::intent::{Constraint, StyleConstraint};
        
        match constraint {
            Constraint::Style(style) => match style {
                StyleConstraint::Functional => {
                    self.context.preferences.style = CodeStyle::Functional;
                }
                StyleConstraint::Imperative => {
                    self.context.preferences.style = CodeStyle::Imperative;
                }
                StyleConstraint::PointFree => {
                    self.context.preferences.prefer_point_free = true;
                }
                StyleConstraint::Verbose => {
                    self.context.preferences.prefer_explicit_types = true;
                }
                StyleConstraint::Concise => {
                    self.context.preferences.prefer_explicit_types = false;
                    self.context.preferences.prefer_point_free = true;
                }
            },
            _ => {}
        }
    }
}