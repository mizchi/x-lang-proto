//! x Language Type Checker
//! 
//! This crate provides type checking and semantic analysis for x Language.
//! It includes a sophisticated effect system, type inference, and program verification.

pub mod types;
pub mod inference;
pub mod effects;
pub mod effect_checker;
pub mod unification;
pub mod resolver;
pub mod error_reporting;
pub mod binary_type_checker;
pub mod constraints;
pub mod checker;
pub mod builtins;

// Re-export core types
pub use types::{Type, TypeScheme, TypeVar, TypeEnv};
pub use inference::{InferenceContext, InferenceResult};
pub use types::{Effect, EffectSet};
pub use error_reporting::{TypeError, TypeErrorReporter};
pub use checker::{TypeChecker, CheckResult, EffectConstraint};

use x_parser::{CompilationUnit, Symbol, Span};

/// Type check a compilation unit
pub fn type_check(cu: &CompilationUnit) -> CheckResult {
    let mut checker = TypeChecker::new();
    checker.check_compilation_unit(cu)
}

/// Type check with custom environment
pub fn type_check_with_env(cu: &CompilationUnit, env: TypeEnv) -> CheckResult {
    let mut checker = TypeChecker::with_env(env);
    checker.check_compilation_unit(cu)
}

/// Incremental type checking interface using Salsa
#[salsa::query_group(TypeCheckDatabase)]
pub trait TypeCheckDb: salsa::Database {
    /// Get the type scheme for a symbol
    #[salsa::input]
    fn symbol_type(&self, symbol: Symbol) -> Option<TypeScheme>;

    /// Get inferred type for an expression
    fn infer_expression_type(&self, expr_id: ExprId) -> types::Type;

    /// Get effect set for an expression
    fn infer_expression_effects(&self, expr_id: ExprId) -> EffectSet;

    /// Check if two types are compatible
    fn types_compatible(&self, t1: types::Type, t2: types::Type) -> bool;

    /// Resolve symbol in scope
    fn resolve_symbol(&self, symbol: Symbol, scope_id: ScopeId) -> Option<SymbolInfo>;
}

/// Expression identifier for incremental type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

/// Scope identifier for symbol resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Symbol information for resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub type_scheme: TypeScheme,
    pub span: Span,
    pub visibility: x_parser::Visibility,
}

/// Database implementation for type checking
#[salsa::database(TypeCheckDatabase)]
#[derive(Default)]
pub struct TypeCheckDatabaseImpl {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for TypeCheckDatabaseImpl {}

/// Convenience functions for type checking queries
impl TypeCheckDatabaseImpl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set type for a symbol
    pub fn set_type_for_symbol(&mut self, symbol: Symbol, type_scheme: TypeScheme) {
        self.set_symbol_type(symbol, Some(type_scheme));
    }

    /// Check if compilation unit is well-typed
    pub fn is_well_typed(&self, cu: &CompilationUnit) -> bool {
        let result = type_check(cu);
        result.errors.is_empty()
    }
}

/// Query implementations
fn infer_expression_type(_db: &dyn TypeCheckDb, _expr_id: ExprId) -> types::Type {
    // TODO: Implement incremental expression type inference
    types::Type::Unknown
}

fn infer_expression_effects(_db: &dyn TypeCheckDb, _expr_id: ExprId) -> EffectSet {
    // TODO: Implement incremental effect inference
    EffectSet::empty()
}

fn types_compatible(_db: &dyn TypeCheckDb, t1: types::Type, t2: types::Type) -> bool {
    // TODO: Implement type compatibility checking
    let mut unifier = crate::unification::Unifier::new();
    unifier.unify(&t1, &t2).is_ok()
}

fn resolve_symbol(_db: &dyn TypeCheckDb, _symbol: Symbol, _scope_id: ScopeId) -> Option<SymbolInfo> {
    // TODO: Implement incremental symbol resolution
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{parse_source, SyntaxStyle, FileId};

    #[test]
    fn test_basic_type_checking() {
        let source = "let x = 42";
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::OCaml).unwrap();
        
        let result = type_check(&cu);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_type_database() {
        let mut db = TypeCheckDatabaseImpl::new();
        
        let symbol = Symbol::intern("test");
        let type_scheme = TypeScheme::monotype(types::Type::Con(Symbol::intern("Int")));
        
        db.set_type_for_symbol(symbol, type_scheme.clone());
        assert_eq!(db.symbol_type(symbol), Some(type_scheme));
    }

    #[test]
    fn test_incremental_queries() {
        let db = TypeCheckDatabaseImpl::new();
        
        let expr_id = ExprId(0);
        let _typ = db.infer_expression_type(expr_id);
        let _effects = db.infer_expression_effects(expr_id);
        
        // These should not panic even with unimplemented queries
    }
}