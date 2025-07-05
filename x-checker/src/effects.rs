//! Effect system implementation for x Language
//! 
//! This module implements algebraic effects and handlers with:
//! - Row polymorphism for extensible effect sets
//! - Effect inference and checking
//! - Handler type checking
//! - Effect elimination through handlers

use x_parser::{
    Handler, HandlerClause, EffectDef, EffectOperation,
    Symbol,
    Span, FileId,
};
use x_parser::span::ByteOffset;
use crate::types::*;
use std::result::Result as StdResult;

use std::collections::{HashMap, HashSet};

/// Effect system context
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Available effects
    effects: HashMap<Symbol, EffectDefinition>,
    
    /// Effect handlers in scope
    handlers: Vec<HandlerInfo>,
    
    /// Current effect requirements
    required_effects: EffectSet,
}

/// Complete effect definition
#[derive(Debug, Clone, PartialEq)]
pub struct EffectDefinition {
    pub name: Symbol,
    pub operations: HashMap<Symbol, OperationSignature>,
    pub span: Span,
}

/// Operation signature with full type information
#[derive(Debug, Clone, PartialEq)]
pub struct OperationSignature {
    pub name: Symbol,
    pub type_params: Vec<TypeVar>,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub resumption_type: Type, // Type of the continuation
    pub span: Span,
}

/// Handler information for effect handling
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub effect: Symbol,
    pub operations: HashMap<Symbol, HandlerClause>,
    pub return_clause: Option<ReturnClause>,
    pub handled_type: Type,
    pub result_type: Type,
}

/// Return clause for handlers
#[derive(Debug, Clone)]
pub struct ReturnClause {
    pub parameter: Symbol,
    pub body: Type, // Simplified for now
}

impl EffectContext {
    pub fn new() -> Self {
        EffectContext {
            effects: HashMap::new(),
            handlers: Vec::new(),
            required_effects: EffectSet::Empty,
        }
    }
    
    /// Register an effect definition
    pub fn register_effect(&mut self, effect_def: &EffectDef) -> StdResult<(), String> {
        let mut operations = HashMap::new();
        
        for op_def in &effect_def.operations {
                let signature = self.effect_operation_to_signature(op_def)?;
            operations.insert(op_def.name, signature);
        }
        
        let definition = EffectDefinition {
            name: effect_def.name,
            operations,
            span: effect_def.span,
        };
        
        self.effects.insert(effect_def.name, definition);
        Ok(())
    }
    
    /// Look up an effect definition
    pub fn lookup_effect(&self, name: Symbol) -> Option<&EffectDefinition> {
        self.effects.get(&name)
    }
    
    /// Look up an operation in an effect
    pub fn lookup_operation(&self, effect: Symbol, operation: Symbol) -> Option<&OperationSignature> {
        self.effects.get(&effect)?
            .operations.get(&operation)
    }
    
    /// Add a handler to the context
    pub fn push_handler(&mut self, handler: HandlerInfo) {
        self.handlers.push(handler);
    }
    
    /// Remove the most recent handler
    pub fn pop_handler(&mut self) -> Option<HandlerInfo> {
        self.handlers.pop()
    }
    
    /// Check if an effect is handled in the current context
    pub fn is_effect_handled(&self, effect: Symbol) -> bool {
        self.handlers.iter().any(|h| h.effect == effect)
    }
    
    /// Get unhandled effects from a set
    pub fn get_unhandled_effects(&self, effects: &EffectSet) -> EffectSet {
        match effects {
            EffectSet::Empty => EffectSet::Empty,
            EffectSet::Var(_) => effects.clone(), // Conservative: assume unhandled
            EffectSet::Row { effects: effect_list, tail } => {
                let unhandled: Vec<Effect> = effect_list.iter()
                    .filter(|e| !self.is_effect_handled(e.name))
                    .cloned()
                    .collect();
                
                let unhandled_tail = tail.as_ref()
                    .map(|t| Box::new(self.get_unhandled_effects(t)));
                
                if unhandled.is_empty() && unhandled_tail.is_none() {
                    EffectSet::Empty
                } else {
                    EffectSet::Row {
                        effects: unhandled,
                        tail: unhandled_tail,
                    }
                }
            }
        }
    }
    
    /// Infer the effect of a perform operation
    pub fn infer_perform_effect(
        &self,
        effect: Symbol,
        operation: Symbol,
        args: &[Type],
    ) -> StdResult<(Type, EffectSet), String> {
        let effect_def = self.lookup_effect(effect)
            .ok_or_else(|| format!("Unknown effect: {}", effect))?;
        
        let op_sig = effect_def.operations.get(&operation)
            .ok_or_else(|| format!("Unknown operation {} in effect {}", operation, effect))?;
        
        // Check argument count
        if args.len() != op_sig.params.len() {
            return Err(format!(
                "Operation {} expects {} arguments, got {}",
                operation,
                op_sig.params.len(),
                args.len()
            ));
        }
        
        // Create effect set containing this effect
        let effect_instance = Effect {
            name: effect,
            operations: vec![Operation {
                name: operation,
                params: args.to_vec(),
                return_type: op_sig.return_type.clone(),
            }],
        };
        
        let effect_set = EffectSet::Row {
            effects: vec![effect_instance],
            tail: None,
        };
        
        Ok((op_sig.return_type.clone(), effect_set))
    }
    
    /// Type check a handler
    pub fn check_handler(
        &mut self,
        handler: &Handler,
        body_type: &Type,
        body_effects: &EffectSet,
    ) -> StdResult<(Type, EffectSet), String> {
        let effect_def = self.lookup_effect(handler.effect)
            .ok_or_else(|| format!("Unknown effect: {}", handler.effect))?;
        
        // Check that all operations are handled
        for (op_name, _op_sig) in &effect_def.operations {
            if !handler.clauses.iter().any(|clause| clause.operation == *op_name) {
                return Err(format!(
                    "Missing handler for operation {} in effect {}",
                    op_name,
                    handler.effect
                ));
            }
        }
        
        // Check each handler clause
        let mut result_type = body_type.clone();
        for clause in &handler.clauses {
            let clause_result = self.check_handler_clause(
                clause,
                &effect_def,
                &result_type,
            )?;
            
            // All clauses should produce the same result type
            // TODO: Implement proper unification here
            result_type = clause_result;
        }
        
        // Check return clause if present
        if let Some(return_clause) = &handler.return_clause {
            // TODO: Check return clause type
        }
        
        // Remove the handled effect from the effect set
        let remaining_effects = self.remove_effect_from_set(body_effects, handler.effect);
        
        Ok((result_type, remaining_effects))
    }
    
    /// Check a single handler clause
    fn check_handler_clause(
        &self,
        clause: &HandlerClause,
        effect_def: &EffectDefinition,
        expected_result_type: &Type,
    ) -> StdResult<Type, String> {
        let op_sig = effect_def.operations.get(&clause.operation)
            .ok_or_else(|| format!(
                "Operation {} not found in effect {}",
                clause.operation,
                effect_def.name
            ))?;
        
        // Check parameter count
        if clause.params.len() != op_sig.params.len() + 1 { // +1 for continuation
            return Err(format!(
                "Handler clause for {} expects {} parameters (including continuation), got {}",
                clause.operation,
                op_sig.params.len() + 1,
                clause.params.len()
            ));
        }
        
        // TODO: Type check the handler body with proper environment
        // For now, return the expected result type
        Ok(expected_result_type.clone())
    }
    
    /// Remove an effect from an effect set
    fn remove_effect_from_set(&self, effects: &EffectSet, effect_to_remove: Symbol) -> EffectSet {
        match effects {
            EffectSet::Empty => EffectSet::Empty,
            EffectSet::Var(_) => effects.clone(), // Conservative
            EffectSet::Row { effects: effect_list, tail } => {
                let remaining: Vec<Effect> = effect_list.iter()
                    .filter(|e| e.name != effect_to_remove)
                    .cloned()
                    .collect();
                
                let remaining_tail = tail.as_ref()
                    .map(|t| Box::new(self.remove_effect_from_set(t, effect_to_remove)));
                
                if remaining.is_empty() && remaining_tail.is_none() {
                    EffectSet::Empty
                } else {
                    EffectSet::Row {
                        effects: remaining,
                        tail: remaining_tail,
                    }
                }
            }
        }
    }
    
    /// Convert effect operation to signature
    fn effect_operation_to_signature(&self, op_def: &EffectOperation) -> StdResult<OperationSignature, String> {
        // TODO: Convert AST types to internal types
        let params = op_def.parameters.iter()
            .map(|_param| Type::Hole) // Placeholder
            .collect();
        
        Ok(OperationSignature {
            name: op_def.name,
            type_params: Vec::new(), // TODO: Extract from operation definition
            params,
            return_type: Type::Hole, // TODO: Convert from AST
            resumption_type: Type::Hole, // TODO: Infer resumption type
            span: op_def.span,
        })
    }
}

impl Default for EffectContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect subtyping and compatibility
impl EffectSet {
    /// Check if this effect set is a subset of another (subtyping)
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        match (self, other) {
            (EffectSet::Empty, _) => true,
            (_, EffectSet::Empty) => false,
            (EffectSet::Var(_), EffectSet::Var(_)) => true, // Conservative
            (EffectSet::Row { effects: e1, tail: t1 }, EffectSet::Row { effects: e2, tail: t2 }) => {
                // All effects in e1 must be in e2 or its tail
                for effect in e1 {
                    if !e2.iter().any(|e| e.name == effect.name) {
                        if let Some(tail) = t2 {
                            if !self.effect_in_set(&effect.name, tail) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
                
                // Check tail compatibility
                match (t1, t2) {
                    (Some(tail1), Some(tail2)) => tail1.is_subset_of(tail2),
                    (None, _) => true,
                    (Some(_), None) => false,
                }
            }
            _ => false,
        }
    }
    
    // contains_effect is now implemented in types.rs
    
    fn effect_in_set(&self, effect: &Symbol, set: &EffectSet) -> bool {
        set.contains_effect(*effect)
    }
    
    /// Merge two effect sets
    pub fn merge(&self, other: &EffectSet) -> EffectSet {
        match (self, other) {
            (EffectSet::Empty, other) => other.clone(),
            (self_set, EffectSet::Empty) => self_set.clone(),
            (EffectSet::Row { effects: e1, tail: t1 }, 
             EffectSet::Row { effects: e2, tail: t2 }) => {
                let mut merged_effects = e1.clone();
                
                // Add effects from e2 that aren't already present
                for effect in e2 {
                    if !merged_effects.iter().any(|e| e.name == effect.name) {
                        merged_effects.push(effect.clone());
                    }
                }
                
                let merged_tail = match (t1, t2) {
                    (Some(tail1), Some(tail2)) => Some(Box::new(tail1.merge(tail2))),
                    (Some(tail), None) | (None, Some(tail)) => Some(tail.clone()),
                    (None, None) => None,
                };
                
                EffectSet::Row {
                    effects: merged_effects,
                    tail: merged_tail,
                }
            }
            _ => {
                // Conservative merge for variables
                EffectSet::Row {
                    effects: Vec::new(),
                    tail: Some(Box::new(self.clone())),
                }
            }
        }
    }
}

/// Effect row operations for row polymorphism
#[derive(Debug, Clone)]
pub struct EffectRow {
    /// Present effects
    pub present: HashMap<Symbol, Effect>,
    /// Absent effects (for row constraints)
    pub absent: HashSet<Symbol>,
    /// Tail variable
    pub tail: Option<RowVar>,
}

impl EffectRow {
    pub fn empty() -> Self {
        EffectRow {
            present: HashMap::new(),
            absent: HashSet::new(),
            tail: None,
        }
    }
    
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.present.insert(effect.name, effect);
        self
    }
    
    pub fn without_effect(mut self, effect: Symbol) -> Self {
        self.absent.insert(effect);
        self
    }
    
    pub fn with_tail(mut self, tail: RowVar) -> Self {
        self.tail = Some(tail);
        self
    }
    
    /// Check if this row lacks a specific effect
    pub fn lacks(&self, effect: Symbol) -> bool {
        self.absent.contains(&effect) && !self.present.contains_key(&effect)
    }
    
    /// Extend this row with another effect
    pub fn extend(&self, effect: Effect) -> StdResult<EffectRow, String> {
        if self.present.contains_key(&effect.name) {
            return Err(format!("Effect {} already present in row", effect.name));
        }
        
        if self.absent.contains(&effect.name) {
            return Err(format!("Effect {} is marked as absent in row", effect.name));
        }
        
        let mut result = self.clone();
        result.present.insert(effect.name, effect);
        Ok(result)
    }
}

/// Built-in effect definitions
pub fn create_builtin_effects() -> HashMap<Symbol, EffectDefinition> {
    use x_parser::symbol::symbols;
    let mut effects = HashMap::new();
    
    // IO Effect
    let io_operations = {
        let mut ops = HashMap::new();
        ops.insert(
            symbols::PRINT(),
            OperationSignature {
                name: symbols::PRINT(),
                type_params: Vec::new(),
                params: vec![Type::Con(symbols::STRING())],
                return_type: Type::Con(symbols::UNIT_TYPE()),
                resumption_type: Type::Con(symbols::UNIT_TYPE()),
                span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
            },
        );
        ops.insert(
            symbols::READ(),
            OperationSignature {
                name: symbols::READ(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Type::Con(symbols::STRING()),
                resumption_type: Type::Con(symbols::STRING()),
                span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
            },
        );
        ops
    };
    
    effects.insert(
        symbols::IO(),
        EffectDefinition {
            name: symbols::IO(),
            operations: io_operations,
            span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
        },
    );
    
    // State Effect
    let state_operations = {
        let mut ops = HashMap::new();
        let state_var = TypeVar(0);
        
        ops.insert(
            symbols::GET(),
            OperationSignature {
                name: symbols::GET(),
                type_params: vec![state_var],
                params: Vec::new(),
                return_type: Type::Var(state_var),
                resumption_type: Type::Var(state_var),
                span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
            },
        );
        ops.insert(
            symbols::PUT(),
            OperationSignature {
                name: symbols::PUT(),
                type_params: vec![state_var],
                params: vec![Type::Var(state_var)],
                return_type: Type::Con(symbols::UNIT_TYPE()),
                resumption_type: Type::Con(symbols::UNIT_TYPE()),
                span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
            },
        );
        ops
    };
    
    effects.insert(
        symbols::STATE(),
        EffectDefinition {
            name: symbols::STATE(),
            operations: state_operations,
            span: Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0)),
        },
    );
    
    effects
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::symbol::Symbol;
    
    #[test]
    fn test_effect_context_creation() {
        let ctx = EffectContext::new();
        assert!(ctx.effects.is_empty());
        assert!(ctx.handlers.is_empty());
    }
    
    #[test]
    fn test_effect_subset() {
        let empty = EffectSet::Empty;
        let io_effect = EffectSet::Row {
            effects: vec![Effect {
                name: Symbol::intern("IO"),
                operations: Vec::new(),
            }],
            tail: None,
        };
        
        assert!(empty.is_subset_of(&io_effect));
        assert!(!io_effect.is_subset_of(&empty));
    }
    
    #[test]
    fn test_effect_contains() {
        let io_symbol = Symbol::intern("IO");
        let io_effect = EffectSet::Row {
            effects: vec![Effect {
                name: io_symbol,
                operations: Vec::new(),
            }],
            tail: None,
        };
        
        assert!(io_effect.contains_effect(io_symbol));
        assert!(!io_effect.contains_effect(Symbol::intern("State")));
    }
    
    #[test]
    fn test_effect_merge() {
        let io_symbol = Symbol::intern("IO");
        let state_symbol = Symbol::intern("State");
        
        let io_effect = EffectSet::Row {
            effects: vec![Effect {
                name: io_symbol,
                operations: Vec::new(),
            }],
            tail: None,
        };
        
        let state_effect = EffectSet::Row {
            effects: vec![Effect {
                name: state_symbol,
                operations: Vec::new(),
            }],
            tail: None,
        };
        
        let merged = io_effect.merge(&state_effect);
        
        assert!(merged.contains_effect(io_symbol));
        assert!(merged.contains_effect(state_symbol));
    }
    
    #[test]
    fn test_builtin_effects() {
        let effects = create_builtin_effects();
        assert!(effects.contains_key(&Symbol::intern("IO")));
        assert!(effects.contains_key(&Symbol::intern("State")));
    }
}