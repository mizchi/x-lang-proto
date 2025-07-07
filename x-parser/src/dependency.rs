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
}

impl DependencyManager {
    fn collect_pattern_vars(pattern: &crate::ast::Pattern, vars: &mut HashSet<Symbol>) {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Variable(name, _) => {
                vars.insert(*name);
            }
            Pattern::Wildcard(_) => {}
            Pattern::Literal(_, _) => {}
            Pattern::Constructor { args, .. } => {
                for p in args {
                    Self::collect_pattern_vars(p, vars);
                }
            }
            Pattern::Tuple { patterns, .. } => {
                for p in patterns {
                    Self::collect_pattern_vars(p, vars);
                }
            }
            Pattern::Record { fields, rest, .. } => {
                for p in fields.values() {
                    Self::collect_pattern_vars(p, vars);
                }
                if let Some(rest_pattern) = rest {
                    Self::collect_pattern_vars(rest_pattern, vars);
                }
            }
            Pattern::Or { left, right, .. } => {
                // Or patterns bind the same variables in each branch
                Self::collect_pattern_vars(left, vars);
                Self::collect_pattern_vars(right, vars);
            }
            Pattern::As { pattern, name, .. } => {
                vars.insert(*name);
                Self::collect_pattern_vars(pattern, vars);
            }
            Pattern::Ann { pattern, .. } => {
                Self::collect_pattern_vars(pattern, vars);
            }
        }
    }
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Extract dependencies from an expression
    pub fn extract_dependencies(expr: &Expr) -> HashSet<Symbol> {
        let mut deps = HashSet::new();
        let mut bound_vars = HashSet::new();
        Self::collect_expr_deps(expr, &mut deps, &mut bound_vars);
        deps
    }
    
    /// Extract dependencies from a value definition  
    pub fn extract_dependencies_from_def(def: &crate::ast::ValueDef) -> HashSet<Symbol> {
        let mut deps = HashSet::new();
        let mut bound_vars = HashSet::new();
        
        // Add parameters to bound variables
        for param in &def.parameters {
            Self::collect_pattern_vars(param, &mut bound_vars);
        }
        
        // If the body is a lambda, don't extract dependencies from the lambda itself,
        // since its parameters are already bound
        match &def.body {
            Expr::Lambda { .. } => {
                // For lambdas, we handle parameters internally in collect_expr_deps
                Self::collect_expr_deps(&def.body, &mut deps, &mut bound_vars);
            }
            _ => {
                Self::collect_expr_deps(&def.body, &mut deps, &mut bound_vars);
            }
        }
        
        deps
    }

    fn collect_expr_deps(expr: &Expr, deps: &mut HashSet<Symbol>, bound_vars: &mut HashSet<Symbol>) {
        match expr {
            Expr::Var(name, _) => {
                // Skip builtin functions and bound variables
                if !is_builtin(name) && !bound_vars.contains(name) {
                    deps.insert(*name);
                }
            }
            Expr::App(f, args, _) => {
                Self::collect_expr_deps(f, deps, bound_vars);
                for arg in args {
                    Self::collect_expr_deps(arg, deps, bound_vars);
                }
            }
            Expr::Lambda { parameters, body, .. } => {
                // Add parameters to bound variables
                let mut new_bound = bound_vars.clone();
                for param in parameters {
                    Self::collect_pattern_vars(param, &mut new_bound);
                }
                Self::collect_expr_deps(body, deps, &mut new_bound);
            }
            Expr::Let { pattern, value, body, .. } => {
                Self::collect_expr_deps(value, deps, bound_vars);
                // Add pattern variables to bound variables for body
                let mut new_bound = bound_vars.clone();
                Self::collect_pattern_vars(pattern, &mut new_bound);
                Self::collect_expr_deps(body, deps, &mut new_bound);
            }
            Expr::Match { scrutinee, arms, .. } => {
                Self::collect_expr_deps(scrutinee, deps, bound_vars);
                for arm in arms {
                    // Add pattern variables to bound variables for arm body
                    let mut new_bound = bound_vars.clone();
                    Self::collect_pattern_vars(&arm.pattern, &mut new_bound);
                    Self::collect_expr_deps(&arm.body, deps, &mut new_bound);
                }
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                Self::collect_expr_deps(condition, deps, bound_vars);
                Self::collect_expr_deps(then_branch, deps, bound_vars);
                Self::collect_expr_deps(else_branch, deps, bound_vars);
            }
            Expr::Handle { expr, handlers, .. } => {
                Self::collect_expr_deps(expr, deps, bound_vars);
                for handler in handlers {
                    // Add handler parameters to bound variables
                    let mut new_bound = bound_vars.clone();
                    for param in &handler.parameters {
                        Self::collect_pattern_vars(param, &mut new_bound);
                    }
                    Self::collect_expr_deps(&handler.body, deps, &mut new_bound);
                }
            }
            Expr::Do { statements, .. } => {
                let mut do_bound = bound_vars.clone();
                for stmt in statements {
                    match stmt {
                        crate::ast::DoStatement::Let { pattern, expr, .. } => {
                            Self::collect_expr_deps(expr, deps, &mut do_bound);
                            Self::collect_pattern_vars(pattern, &mut do_bound);
                        }
                        crate::ast::DoStatement::Bind { pattern, expr, .. } => {
                            Self::collect_expr_deps(expr, deps, &mut do_bound);
                            Self::collect_pattern_vars(pattern, &mut do_bound);
                        }
                        crate::ast::DoStatement::Expr(expr) => {
                            Self::collect_expr_deps(expr, deps, &mut do_bound);
                        }
                    }
                }
            }
            Expr::Resume { value, .. } => {
                Self::collect_expr_deps(value, deps, bound_vars);
            }
            Expr::Perform { effect, args, .. } => {
                // Effect is a symbol
                deps.insert(*effect);
                for arg in args {
                    Self::collect_expr_deps(arg, deps, bound_vars);
                }
            }
            Expr::Ann { expr, .. } => {
                Self::collect_expr_deps(expr, deps, bound_vars);
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
                .or_default()
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