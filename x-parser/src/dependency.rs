//! Dependency management system inspired by Unison
//! 
//! This module provides fine-grained dependency tracking at the function level,
//! allowing explicit imports and dependency tree extraction.

use crate::{ast::*, symbol::Symbol, span::Span};
use std::collections::{HashMap, HashSet, VecDeque};

/// Check if a symbol is a builtin function
fn is_builtin(name: &Symbol) -> bool {
    matches!(name.as_str(), 
        "+" | "-" | "*" | "/" | "%" | 
        "==" | "!=" | "<" | ">" | "<=" | ">=" |
        "&&" | "||" | "!" |
        "print" | "println" | "error" | "panic" |
        "true" | "false" | "unit" |
        "if" | "then" | "else" | "let" | "in" | "do"
    )
}

/// Represents a dependency relationship between definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    /// The name of the dependency
    pub name: Symbol,
    /// Whether this is a direct dependency or transitive
    pub is_direct: bool,
    /// The kind of dependency (function, type, effect, etc.)
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Function,
    Type,
    Effect,
    Constructor,
    Pattern,
}

/// Tracks dependencies for a single definition
#[derive(Debug, Clone)]
pub struct DefinitionDependencies {
    /// The definition itself
    pub name: Symbol,
    /// Direct dependencies this definition requires
    pub direct_deps: HashSet<Symbol>,
    /// All transitive dependencies
    pub transitive_deps: HashSet<Symbol>,
    /// Explicit imports declared for this definition
    pub explicit_imports: Vec<Import>,
}

/// Represents an explicit import declaration
#[derive(Debug, Clone)]
pub struct Import {
    /// The module or namespace to import from
    pub from: Option<ModulePath>,
    /// The specific items to import
    pub items: Vec<ImportItem>,
    /// The span of the import declaration
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImportItem {
    /// Import a specific name
    Named(Symbol),
    /// Import with an alias
    Aliased { name: Symbol, alias: Symbol },
    /// Import all items from a module
    Wildcard,
}

/// Manages dependencies for an entire module or codebase
#[derive(Debug, Clone)]
pub struct DependencyManager {
    /// Map from definition name to its dependencies
    pub definitions: HashMap<Symbol, DefinitionDependencies>,
    /// Reverse dependency map (who depends on this definition)
    reverse_deps: HashMap<Symbol, HashSet<Symbol>>,
    /// Type definitions and their constructors
    type_constructors: HashMap<Symbol, Vec<Symbol>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            reverse_deps: HashMap::new(),
            type_constructors: HashMap::new(),
        }
    }

    /// Extract dependencies from an expression
    pub fn extract_dependencies(expr: &Expr) -> HashSet<Symbol> {
        let mut deps = HashSet::new();
        Self::collect_expr_deps(expr, &mut deps);
        deps
    }

    fn collect_expr_deps(expr: &Expr, deps: &mut HashSet<Symbol>) {
        match expr {
            Expr::Var(name, _) => {
                // Skip builtin functions
                if !is_builtin(name) {
                    deps.insert(*name);
                }
            }
            Expr::App(f, args, _) => {
                Self::collect_expr_deps(f, deps);
                for arg in args {
                    Self::collect_expr_deps(arg, deps);
                }
            }
            Expr::Lambda { body, .. } => {
                Self::collect_expr_deps(body, deps);
            }
            Expr::Let { value, body, .. } => {
                Self::collect_expr_deps(value, deps);
                Self::collect_expr_deps(body, deps);
            }
            Expr::Match { scrutinee, arms, .. } => {
                Self::collect_expr_deps(scrutinee, deps);
                for arm in arms {
                    Self::collect_expr_deps(&arm.body, deps);
                }
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                Self::collect_expr_deps(condition, deps);
                Self::collect_expr_deps(then_branch, deps);
                Self::collect_expr_deps(else_branch, deps);
            }
            Expr::Handle { expr, handlers, .. } => {
                Self::collect_expr_deps(expr, deps);
                for handler in handlers {
                    Self::collect_expr_deps(&handler.body, deps);
                }
            }
            Expr::Do { statements, .. } => {
                for stmt in statements {
                    match stmt {
                        crate::ast::DoStatement::Let { expr, .. } => {
                            Self::collect_expr_deps(expr, deps);
                        }
                        crate::ast::DoStatement::Bind { expr, .. } => {
                            Self::collect_expr_deps(expr, deps);
                        }
                        crate::ast::DoStatement::Expr(expr) => {
                            Self::collect_expr_deps(expr, deps);
                        }
                    }
                }
            }
            Expr::Resume { value, .. } => {
                Self::collect_expr_deps(value, deps);
            }
            Expr::Perform { effect, args, .. } => {
                // Effect is a symbol
                deps.insert(*effect);
                for arg in args {
                    Self::collect_expr_deps(arg, deps);
                }
            }
            Expr::Ann { expr, .. } => {
                Self::collect_expr_deps(expr, deps);
            }
            Expr::Literal(_, _) => {
                // No dependencies
            }
        }
    }

    /// Add a definition with its dependencies
    pub fn add_definition(&mut self, name: Symbol, deps: HashSet<Symbol>) {
        let def_deps = DefinitionDependencies {
            name,
            direct_deps: deps.clone(),
            transitive_deps: HashSet::new(),
            explicit_imports: Vec::new(),
        };
        
        self.definitions.insert(name, def_deps);
        
        // Update reverse dependencies
        for dep in deps {
            self.reverse_deps
                .entry(dep)
                .or_insert_with(HashSet::new)
                .insert(name);
        }
    }

    /// Compute transitive dependencies for all definitions
    pub fn compute_transitive_deps(&mut self) {
        let names: Vec<_> = self.definitions.keys().cloned().collect();
        
        for name in names {
            let transitive = self.compute_transitive_for(&name);
            if let Some(def) = self.definitions.get_mut(&name) {
                def.transitive_deps = transitive;
            }
        }
    }

    fn compute_transitive_for(&self, name: &Symbol) -> HashSet<Symbol> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = HashSet::new();
        
        if let Some(def) = self.definitions.get(name) {
            for dep in &def.direct_deps {
                queue.push_back(*dep);
            }
        }
        
        while let Some(dep) = queue.pop_front() {
            if visited.insert(dep) {
                result.insert(dep);
                
                if let Some(def) = self.definitions.get(&dep) {
                    for transitive_dep in &def.direct_deps {
                        if !visited.contains(transitive_dep) {
                            queue.push_back(*transitive_dep);
                        }
                    }
                }
            }
        }
        
        result
    }

    /// Get all dependencies (direct and transitive) for a definition
    pub fn get_all_dependencies(&self, name: &Symbol) -> Option<&HashSet<Symbol>> {
        self.definitions.get(name).map(|def| &def.transitive_deps)
    }

    /// Get definitions that depend on the given name
    pub fn get_dependents(&self, name: &Symbol) -> Option<&HashSet<Symbol>> {
        self.reverse_deps.get(name)
    }

    /// Extract a minimal set of definitions needed for the given names
    pub fn extract_closure(&self, roots: &[Symbol]) -> HashSet<Symbol> {
        let mut closure = HashSet::new();
        let mut queue = VecDeque::new();
        
        for root in roots {
            queue.push_back(*root);
        }
        
        while let Some(name) = queue.pop_front() {
            if closure.insert(name) {
                if let Some(def) = self.definitions.get(&name) {
                    for dep in &def.direct_deps {
                        if !closure.contains(dep) {
                            queue.push_back(*dep);
                        }
                    }
                }
            }
        }
        
        closure
    }

    /// Generate a topologically sorted list of definitions
    pub fn topological_sort(&self, names: &HashSet<Symbol>) -> Vec<Symbol> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();
        
        fn visit(
            name: Symbol,
            definitions: &HashMap<Symbol, DefinitionDependencies>,
            visited: &mut HashSet<Symbol>,
            temp_mark: &mut HashSet<Symbol>,
            sorted: &mut Vec<Symbol>,
            names: &HashSet<Symbol>,
        ) {
            if visited.contains(&name) {
                return;
            }
            
            if temp_mark.contains(&name) {
                // Circular dependency detected
                return;
            }
            
            temp_mark.insert(name);
            
            if let Some(def) = definitions.get(&name) {
                for dep in &def.direct_deps {
                    if names.contains(dep) {
                        visit(*dep, definitions, visited, temp_mark, sorted, names);
                    }
                }
            }
            
            temp_mark.remove(&name);
            visited.insert(name);
            sorted.push(name);
        }
        
        for name in names {
            visit(*name, &self.definitions, &mut visited, &mut temp_mark, &mut sorted, names);
        }
        
        sorted
    }

    /// Check for circular dependencies
    pub fn find_circular_dependencies(&self) -> Vec<Vec<Symbol>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut in_stack = HashSet::new();
        
        fn find_cycles(
            name: Symbol,
            definitions: &HashMap<Symbol, DefinitionDependencies>,
            visited: &mut HashSet<Symbol>,
            stack: &mut Vec<Symbol>,
            in_stack: &mut HashSet<Symbol>,
            cycles: &mut Vec<Vec<Symbol>>,
        ) {
            visited.insert(name);
            stack.push(name);
            in_stack.insert(name);
            
            if let Some(def) = definitions.get(&name) {
                for dep in &def.direct_deps {
                    if !visited.contains(dep) {
                        find_cycles(*dep, definitions, visited, stack, in_stack, cycles);
                    } else if in_stack.contains(dep) {
                        // Found a cycle
                        if let Some(start) = stack.iter().position(|&x| x == *dep) {
                            cycles.push(stack[start..].to_vec());
                        }
                    }
                }
            }
            
            stack.pop();
            in_stack.remove(&name);
        }
        
        for name in self.definitions.keys() {
            if !visited.contains(name) {
                find_cycles(*name, &self.definitions, &mut visited, &mut stack, &mut in_stack, &mut cycles);
            }
        }
        
        cycles
    }
}

/// Builder for generating code with explicit dependencies
pub struct DependencyCodeGenerator {
    manager: DependencyManager,
}

impl DependencyCodeGenerator {
    pub fn new(manager: DependencyManager) -> Self {
        Self { manager }
    }

    /// Generate code for a set of definitions with all dependencies
    pub fn generate_with_dependencies(&self, roots: &[Symbol]) -> Vec<Symbol> {
        let closure = self.manager.extract_closure(roots);
        self.manager.topological_sort(&closure)
    }

    /// Generate import statements for a definition
    pub fn generate_imports(&self, name: &Symbol) -> Vec<Import> {
        if let Some(def) = self.manager.definitions.get(name) {
            def.explicit_imports.clone()
        } else {
            Vec::new()
        }
    }
}