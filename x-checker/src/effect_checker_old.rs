//! Effect type checking for x Language
//! 
//! This module implements type checking for algebraic effects,
//! ensuring that all performed effects have corresponding handlers.

use std::collections::{HashSet, HashMap};
use crate::types::{Type, TypeVar};
use crate::error_reporting::TypeError;
use x_parser::Symbol;

/// Effect type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Effect {
    pub name: String,
    pub params: Vec<Type>,
}

/// Effect row (set of effects with possible row variable)
#[derive(Debug, Clone, PartialEq)]
pub struct EffectRow {
    /// Concrete effects
    pub effects: HashSet<Effect>,
    /// Row variable for extensibility
    pub row_var: Option<TypeVar>,
}

impl EffectRow {
    /// Empty effect row
    pub fn empty() -> Self {
        EffectRow {
            effects: HashSet::new(),
            row_var: None,
        }
    }
    
    /// Pure computation (no effects)
    pub fn pure() -> Self {
        Self::empty()
    }
    
    /// Single effect
    pub fn single(effect: Effect) -> Self {
        let mut effects = HashSet::new();
        effects.insert(effect);
        EffectRow {
            effects,
            row_var: None,
        }
    }
    
    /// Polymorphic row with variable
    pub fn var(tv: TypeVar) -> Self {
        EffectRow {
            effects: HashSet::new(),
            row_var: Some(tv),
        }
    }
    
    /// Union of two effect rows
    pub fn union(&self, other: &EffectRow) -> Result<EffectRow, TypeError> {
        let mut effects = self.effects.clone();
        effects.extend(other.effects.clone());
        
        // Handle row variables
        let row_var = match (&self.row_var, &other.row_var) {
            (None, None) => None,
            (Some(v), None) | (None, Some(v)) => Some(v.clone()),
            (Some(v1), Some(v2)) if v1 == v2 => Some(v1.clone()),
            (Some(_), Some(_)) => {
                return Err(TypeError::EffectRowMismatch {
                    message: "Cannot unify different row variables".to_string(),
                    span: x_parser::Span::dummy(),
                });
            }
        };
        
        Ok(EffectRow { effects, row_var })
    }
    
    /// Check if this row is a subset of another
    pub fn is_subset_of(&self, other: &EffectRow) -> bool {
        // All our effects must be in the other row
        for effect in &self.effects {
            if !other.effects.contains(effect) && other.row_var.is_none() {
                return false;
            }
        }
        
        // If we have a row variable, the other must too
        if self.row_var.is_some() && other.row_var.is_none() {
            return false;
        }
        
        true
    }
    
    /// Remove handled effects
    pub fn remove_effects(&self, handled: &HashSet<String>) -> EffectRow {
        let effects = self.effects
            .iter()
            .filter(|e| !handled.contains(&e.name))
            .cloned()
            .collect();
            
        EffectRow {
            effects,
            row_var: self.row_var.clone(),
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
    handled_effects: HashSet<String>,
    /// Handler type parameters
    params: HashMap<String, Type>,
}

impl EffectContext {
    pub fn new() -> Self {
        EffectContext {
            handlers: Vec::new(),
            current_effects: EffectRow::empty(),
        }
    }
    
    /// Enter a with handler
    pub fn enter_handler(&mut self, effects: HashSet<String>) {
        self.handlers.push(HandlerScope {
            handled_effects: effects,
            params: HashMap::new(),
        });
    }
    
    /// Exit a with handler
    pub fn exit_handler(&mut self) -> Result<(), TypeError> {
        if self.handlers.pop().is_none() {
            return Err(TypeError::InternalError {
                message: "No handler to exit".to_string(),
                span: x_parser::Span::dummy(),
            });
        }
        Ok(())
    }
    
    /// Check if an effect is handled
    pub fn is_handled(&self, effect_name: &str) -> bool {
        self.handlers.iter().any(|h| h.handled_effects.contains(effect_name))
    }
    
    /// Get unhandled effects
    pub fn get_unhandled_effects(&self) -> EffectRow {
        let mut handled = HashSet::new();
        for handler in &self.handlers {
            handled.extend(handler.handled_effects.clone());
        }
        
        self.current_effects.remove_effects(&handled)
    }
    
    /// Perform an effect
    pub fn perform_effect(&mut self, effect: Effect) -> Result<(), TypeError> {
        if !self.is_handled(&effect.name) {
            self.current_effects.effects.insert(effect.clone());
        }
        Ok(())
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
            effects: EffectRow::pure(),
        }
    }
    
    /// Type with single effect
    pub fn with_effect(typ: Type, effect: Effect) -> Self {
        EffectType {
            typ,
            effects: EffectRow::single(effect),
        }
    }
    
    /// Type with effect row
    pub fn with_effects(typ: Type, effects: EffectRow) -> Self {
        EffectType { typ, effects }
    }
}

/// Effect checker
pub struct EffectChecker {
    context: EffectContext,
    /// Effect definitions
    effect_defs: HashMap<String, EffectDef>,
}

/// Effect definition
#[derive(Debug, Clone)]
pub struct EffectDef {
    pub name: String,
    pub params: Vec<String>,
    pub operations: HashMap<String, OperationType>,
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
        self.effect_defs.insert(def.name.clone(), def);
    }
    
    /// Check a with expression
    pub fn check_with(
        &mut self,
        handlers: &[String],
        body_check: impl FnOnce(&mut Self) -> Result<EffectType, TypeError>,
    ) -> Result<EffectType, TypeError> {
        // Enter handler scope
        let handled: HashSet<String> = handlers.iter().cloned().collect();
        self.context.enter_handler(handled.clone());
        
        // Check body
        let body_type = body_check(self)?;
        
        // Exit handler scope
        self.context.exit_handler()?;
        
        // Remove handled effects from result
        let remaining_effects = body_type.effects.remove_effects(&handled);
        
        Ok(EffectType {
            typ: body_type.typ,
            effects: remaining_effects,
        })
    }
    
    /// Check an effect operation
    pub fn check_perform(
        &mut self,
        effect_name: &str,
        op_name: &str,
        args: &[Type],
    ) -> Result<EffectType, TypeError> {
        // Look up effect definition
        let effect_def = self.effect_defs.get(effect_name)
            .ok_or_else(|| TypeError::UnknownEffect {
                effect_name: effect_name.to_string(),
                span: x_parser::Span::dummy(),
            })?;
        
        // Look up operation
        let op_type = effect_def.operations.get(op_name)
            .ok_or_else(|| TypeError::UnknownOperation {
                effect_name: effect_name.to_string(),
                operation_name: op_name.to_string(),
                span: x_parser::Span::dummy(),
            })?;
        
        // Check argument types
        if args.len() != op_type.params.len() {
            return Err(TypeError::ArityMismatch {
                expected: op_type.params.len(),
                found: args.len(),
                span: x_parser::Span::dummy(),
            });
        }
        
        // Type check arguments (simplified - should unify)
        for (arg, param) in args.iter().zip(&op_type.params) {
            if arg != param {
                return Err(TypeError::TypeMismatch {
                    expected: param.clone(),
                    found: arg.clone(),
                    span: x_parser::Span::dummy(),
                });
            }
        }
        
        // Create effect
        let effect = Effect {
            name: effect_name.to_string(),
            params: vec![], // TODO: handle effect parameters
        };
        
        // Add to context if not handled
        self.context.perform_effect(effect.clone())?;
        
        // Return type with effect
        Ok(EffectType::with_effect(op_type.result.clone(), effect))
    }
    
    /// Check function application
    pub fn check_app(
        &mut self,
        func_type: &EffectType,
        arg_types: &[EffectType],
    ) -> Result<EffectType, TypeError> {
        // Extract function type
        let (param_types, result_type, func_effects) = match &func_type.typ {
            Type::Fun { params, return_type, effects } => (params, return_type, effects),
            _ => return Err(TypeError::NotAFunction {
                typ: func_type.typ.clone(),
                span: x_parser::Span::dummy(),
            }),
        };
        
        // Check arity
        if arg_types.len() != param_types.len() {
            return Err(TypeError::ArityMismatch {
                expected: param_types.len(),
                found: arg_types.len(),
                span: x_parser::Span::dummy(),
            });
        }
        
        // Collect all effects
        let mut all_effects = func_effects.clone();
        for arg in arg_types {
            all_effects = all_effects.union(&arg.effects)?;
        }
        
        // Check that all required effects are handled
        let unhandled = self.context.get_unhandled_effects();
        if !func_effects.is_subset_of(&unhandled) {
            return Err(TypeError::UnhandledEffects {
                required: func_effects.clone(),
                available: unhandled,
                span: x_parser::Span::dummy(),
            });
        }
        
        Ok(EffectType {
            typ: *result_type.clone(),
            effects: all_effects,
        })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_checking() {
        let mut checker = EffectChecker::new();
        
        // Register State effect
        checker.register_effect(EffectDef {
            name: "State".to_string(),
            params: vec!["s".to_string()],
            operations: HashMap::from([
                ("get".to_string(), OperationType {
                    params: vec![],
                    result: Type::Var(TypeVar::new("s")),
                }),
                ("put".to_string(), OperationType {
                    params: vec![Type::Var(TypeVar::new("s"))],
                    result: Type::Con("Unit".to_string()),
                }),
            ]),
        });
        
        // Test unhandled effect
        let result = checker.check_perform("State", "get", &[]);
        assert!(result.is_ok());
        let effect_type = result.unwrap();
        assert_eq!(effect_type.effects.effects.len(), 1);
        
        // Test handled effect
        let result = checker.check_with(
            &["State".to_string()],
            |checker| checker.check_perform("State", "get", &[]),
        );
        assert!(result.is_ok());
        let effect_type = result.unwrap();
        assert_eq!(effect_type.effects.effects.len(), 0); // Effect was handled
    }
}