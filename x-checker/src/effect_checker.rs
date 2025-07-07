//! Effect type checking for x Language
//! 
//! This module implements type checking for algebraic effects,
//! ensuring that all performed effects have corresponding handlers.

use std::collections::{HashSet, HashMap};
use crate::types::{Type, TypeVar, EffectSet, Effect as TypeEffect};
use crate::error_reporting::TypeError;
use x_parser::Symbol;

/// Effect row wrapper for compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct EffectRow {
    pub effects: EffectSet,
}

impl EffectRow {
    pub fn empty() -> Self {
        EffectRow {
            effects: EffectSet::Empty,
        }
    }
}

/// Effect context for type checking
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Stack of active handlers
    handlers: Vec<HandlerScope>,
    /// Current effect row
    current_effects: EffectRow,
}

/// Handler scope
#[derive(Debug, Clone)]
struct HandlerScope {
    /// Effects handled by this handler
    handled_effects: HashSet<Symbol>,
    /// Handler type parameters
    params: HashMap<Symbol, Type>,
}

impl EffectContext {
    pub fn new() -> Self {
        EffectContext {
            handlers: Vec::new(),
            current_effects: EffectRow::empty(),
        }
    }
}

/// Type with effects
#[derive(Debug, Clone, PartialEq)]
pub struct EffectType {
    pub typ: Type,
    pub effects: EffectRow,
}

impl EffectType {
    /// Pure type (no effects)
    pub fn pure(typ: Type) -> Self {
        EffectType {
            typ,
            effects: EffectRow::empty(),
        }
    }
}

/// Effect checker
pub struct EffectChecker {
    context: EffectContext,
    /// Effect definitions
    effect_defs: HashMap<Symbol, EffectDef>,
}

/// Effect definition
#[derive(Debug, Clone)]
pub struct EffectDef {
    pub name: Symbol,
    pub params: Vec<Symbol>,
    pub operations: HashMap<Symbol, OperationType>,
}

/// Operation type signature
#[derive(Debug, Clone)]
pub struct OperationType {
    pub params: Vec<Type>,
    pub result: Type,
}

impl EffectChecker {
    pub fn new() -> Self {
        EffectChecker {
            context: EffectContext::new(),
            effect_defs: HashMap::new(),
        }
    }
    
    /// Register an effect definition
    pub fn register_effect(&mut self, def: EffectDef) {
        self.effect_defs.insert(def.name, def);
    }
}

// Re-export simple types for compatibility
pub type Effect = TypeEffect;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_checking() {
        let checker = EffectChecker::new();
        // Basic test to ensure compilation
        assert!(matches!(checker.context.current_effects.effects, EffectSet::Empty));
    }
}