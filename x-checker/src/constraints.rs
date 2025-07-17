//! Constraint generation and solving for type inference

use crate::{
    types::{Type, TypeVar, TypeScheme, Effect, EffectSet},
};
use x_parser::{Symbol, Span};
use std::collections::HashMap;

/// Type constraint for constraint-based type inference
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// Two types must be equal
    Equal(Type, Type, Span),
    /// Type must be an instance of a type scheme
    Instance(Type, TypeScheme, Span),
    /// Type must have a specific structure
    HasField(Type, Symbol, Type, Span),
    /// Type must be callable with given arguments
    Callable(Type, Vec<Type>, Type, Span),
}

/// Effect constraint for effect system
#[derive(Debug, Clone, PartialEq)]
pub enum EffectConstraint {
    /// Expression must have at most these effects
    SubEffect(EffectSet, EffectSet, Span),
    /// Handler must handle these effects
    HandlesEffect(Symbol, Effect, Span),
    /// Effect must be available in context
    RequiresEffect(Effect, Span),
}

/// Constraint set for type inference
#[derive(Debug, Default)]
pub struct ConstraintSet {
    pub type_constraints: Vec<TypeConstraint>,
    pub effect_constraints: Vec<EffectConstraint>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a type constraint
    pub fn add_type_constraint(&mut self, constraint: TypeConstraint) {
        self.type_constraints.push(constraint);
    }

    /// Add an effect constraint
    pub fn add_effect_constraint(&mut self, constraint: EffectConstraint) {
        self.effect_constraints.push(constraint);
    }

    /// Add equality constraint between two types
    pub fn equal(&mut self, t1: Type, t2: Type, span: Span) {
        self.add_type_constraint(TypeConstraint::Equal(t1, t2, span));
    }

    /// Add instance constraint
    pub fn instance(&mut self, typ: Type, scheme: TypeScheme, span: Span) {
        self.add_type_constraint(TypeConstraint::Instance(typ, scheme, span));
    }

    /// Add field access constraint
    pub fn has_field(&mut self, record_type: Type, field: Symbol, field_type: Type, span: Span) {
        self.add_type_constraint(TypeConstraint::HasField(record_type, field, field_type, span));
    }

    /// Add function call constraint
    pub fn callable(&mut self, func_type: Type, arg_types: Vec<Type>, return_type: Type, span: Span) {
        self.add_type_constraint(TypeConstraint::Callable(func_type, arg_types, return_type, span));
    }

    /// Add effect subtyping constraint
    pub fn sub_effect(&mut self, subset: EffectSet, superset: EffectSet, span: Span) {
        self.add_effect_constraint(EffectConstraint::SubEffect(subset, superset, span));
    }

    /// Check if constraint set is empty
    pub fn is_empty(&self) -> bool {
        self.type_constraints.is_empty() && self.effect_constraints.is_empty()
    }

    /// Merge another constraint set into this one
    pub fn merge(&mut self, other: ConstraintSet) {
        self.type_constraints.extend(other.type_constraints);
        self.effect_constraints.extend(other.effect_constraints);
    }
}

/// Constraint solver for type inference
pub struct ConstraintSolver {
    substitution: HashMap<TypeVar, Type>,
    effect_substitution: HashMap<Symbol, EffectSet>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            substitution: HashMap::new(),
            effect_substitution: HashMap::new(),
        }
    }

    /// Solve a set of constraints
    pub fn solve(&mut self, constraints: &ConstraintSet) -> Result<Substitution, ConstraintError> {
        // Solve type constraints first
        for constraint in &constraints.type_constraints {
            self.solve_type_constraint(constraint)?;
        }

        // Then solve effect constraints
        for constraint in &constraints.effect_constraints {
            self.solve_effect_constraint(constraint)?;
        }

        Ok(Substitution {
            type_substitution: self.substitution.clone(),
            effect_substitution: self.effect_substitution.clone(),
        })
    }

    /// Solve a single type constraint
    fn solve_type_constraint(&mut self, constraint: &TypeConstraint) -> Result<(), ConstraintError> {
        match constraint {
            TypeConstraint::Equal(t1, t2, span) => {
                self.unify_types(t1, t2, *span)?;
            }
            TypeConstraint::Instance(typ, scheme, span) => {
                let instantiated = self.instantiate_scheme(scheme);
                self.unify_types(typ, &instantiated, *span)?;
            }
            TypeConstraint::HasField(record_type, field, field_type, span) => {
                self.solve_field_constraint(record_type, *field, field_type, *span)?;
            }
            TypeConstraint::Callable(func_type, arg_types, return_type, span) => {
                self.solve_callable_constraint(func_type, arg_types, return_type, *span)?;
            }
        }
        Ok(())
    }

    /// Solve a single effect constraint
    fn solve_effect_constraint(&mut self, constraint: &EffectConstraint) -> Result<(), ConstraintError> {
        match constraint {
            EffectConstraint::SubEffect(subset, superset, _span) => {
                // Check that subset is actually a subset of superset
                if !self.is_effect_subset(subset, superset) {
                    return Err(ConstraintError::EffectMismatch {
                        required: superset.clone(),
                        found: subset.clone(),
                    });
                }
            }
            EffectConstraint::HandlesEffect(handler, effect, _span) => {
                // Verify that handler can handle the given effect
                self.verify_handler_capability(*handler, effect)?;
            }
            EffectConstraint::RequiresEffect(effect, _span) => {
                // Ensure effect is available in current context
                self.ensure_effect_available(effect)?;
            }
        }
        Ok(())
    }

    /// Unify two types
    fn unify_types(&mut self, t1: &Type, t2: &Type, span: Span) -> Result<(), ConstraintError> {
        use crate::unification::Unifier;
        
        let mut unifier = Unifier::new();
        unifier.unify(t1, t2).map_err(|e| ConstraintError::UnificationFailed {
            t1: t1.clone(),
            t2: t2.clone(),
            span,
            message: format!("{e:?}"),
        })?;

        // Apply unifier substitutions
        let subst = unifier.get_substitution();
        for (var, typ) in subst.type_subst.iter() {
            self.substitution.insert(*var, typ.clone());
        }

        Ok(())
    }

    /// Instantiate a type scheme with fresh variables
    fn instantiate_scheme(&self, scheme: &TypeScheme) -> Type {
        // TODO: Implement proper scheme instantiation
        scheme.body.clone()
    }

    /// Solve field access constraint
    fn solve_field_constraint(
        &mut self,
        record_type: &Type,
        field: Symbol,
        _field_type: &Type,
        span: Span,
    ) -> Result<(), ConstraintError> {
        match record_type {
            Type::Con(_name) => {
                // Look up record type definition and check field
                // TODO: Implement proper record field checking
                Ok(())
            }
            Type::Var(_var) => {
                // Create a record type constraint for this variable
                // TODO: Implement row polymorphism
                Ok(())
            }
            _ => Err(ConstraintError::InvalidFieldAccess {
                record_type: record_type.clone(),
                field,
                span,
            }),
        }
    }

    /// Solve function call constraint
    fn solve_callable_constraint(
        &mut self,
        func_type: &Type,
        arg_types: &[Type],
        return_type: &Type,
        span: Span,
    ) -> Result<(), ConstraintError> {
        match func_type {
            Type::Fun { params, return_type: expected_return, .. } => {
                // Check parameter count
                if params.len() != arg_types.len() {
                    return Err(ConstraintError::ArityMismatch {
                        expected: params.len(),
                        found: arg_types.len(),
                        span,
                    });
                }

                // Unify parameter types
                for (param, arg) in params.iter().zip(arg_types.iter()) {
                    self.unify_types(param, arg, span)?;
                }

                // Unify return type
                self.unify_types(expected_return, return_type, span)?;
            }
            Type::Var(_) => {
                // Create function type constraint
                let function_type = Type::Fun {
                    params: arg_types.to_vec(),
                    return_type: Box::new(return_type.clone()),
                    effects: EffectSet::Empty,
                };
                self.unify_types(func_type, &function_type, span)?;
            }
            _ => {
                return Err(ConstraintError::NotCallable {
                    typ: func_type.clone(),
                    span,
                });
            }
        }
        Ok(())
    }

    /// Check if one effect set is a subset of another
    fn is_effect_subset(&self, subset: &EffectSet, superset: &EffectSet) -> bool {
        // TODO: Implement proper effect subset checking
        subset.is_subset_of(superset)
    }

    /// Verify handler capability
    fn verify_handler_capability(&self, _handler: Symbol, _effect: &Effect) -> Result<(), ConstraintError> {
        // TODO: Implement handler capability verification
        Ok(())
    }

    /// Ensure effect is available
    fn ensure_effect_available(&self, _effect: &Effect) -> Result<(), ConstraintError> {
        // TODO: Implement effect availability checking
        Ok(())
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of constraint solving
#[derive(Debug, Clone)]
pub struct Substitution {
    pub type_substitution: HashMap<TypeVar, Type>,
    pub effect_substitution: HashMap<Symbol, EffectSet>,
}

impl Substitution {
    pub fn empty() -> Self {
        Self {
            type_substitution: HashMap::new(),
            effect_substitution: HashMap::new(),
        }
    }

    /// Apply substitution to a type
    pub fn apply_to_type(&self, typ: &Type) -> Type {
        match typ {
            Type::Var(var) => {
                self.type_substitution.get(var).cloned().unwrap_or_else(|| typ.clone())
            }
            Type::Fun { params, return_type, effects } => {
                Type::Fun {
                    params: params.iter().map(|t| self.apply_to_type(t)).collect(),
                    return_type: Box::new(self.apply_to_type(return_type)),
                    effects: self.apply_to_effects(effects),
                }
            }
            Type::App(constructor, args) => {
                Type::App(
                    Box::new(self.apply_to_type(constructor)),
                    args.iter().map(|t| self.apply_to_type(t)).collect(),
                )
            }
            Type::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.apply_to_type(t)).collect())
            }
            _ => typ.clone(),
        }
    }

    /// Apply substitution to an effect set
    pub fn apply_to_effects(&self, effects: &EffectSet) -> EffectSet {
        // TODO: Implement effect substitution
        effects.clone()
    }

    /// Compose with another substitution
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut type_substitution = self.type_substitution.clone();
        for (var, typ) in &other.type_substitution {
            let applied_type = self.apply_to_type(typ);
            type_substitution.insert(*var, applied_type);
        }

        // Add other's bindings that don't conflict
        for (var, typ) in &other.type_substitution {
            if !type_substitution.contains_key(var) {
                type_substitution.insert(*var, typ.clone());
            }
        }

        let mut effect_substitution = self.effect_substitution.clone();
        for (sym, effects) in &other.effect_substitution {
            let applied_effects = self.apply_to_effects(effects);
            effect_substitution.insert(*sym, applied_effects);
        }

        Substitution {
            type_substitution,
            effect_substitution,
        }
    }
}

/// Constraint solving errors
#[derive(Debug, Clone)]
pub enum ConstraintError {
    UnificationFailed {
        t1: Type,
        t2: Type,
        span: Span,
        message: String,
    },
    EffectMismatch {
        required: EffectSet,
        found: EffectSet,
    },
    InvalidFieldAccess {
        record_type: Type,
        field: Symbol,
        span: Span,
    },
    ArityMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },
    NotCallable {
        typ: Type,
        span: Span,
    },
    HandlerMismatch {
        handler: Symbol,
        effect: Effect,
    },
    EffectNotAvailable {
        effect: Effect,
        span: Span,
    },
}

impl std::fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintError::UnificationFailed { t1, t2, message, .. } => {
                write!(f, "Cannot unify types {t1:?} and {t2:?}: {message}")
            }
            ConstraintError::EffectMismatch { required, found } => {
                write!(f, "Effect mismatch: required {required:?}, found {found:?}")
            }
            ConstraintError::InvalidFieldAccess { record_type, field, .. } => {
                write!(f, "Invalid field access: type {:?} has no field {}", record_type, field.as_str())
            }
            ConstraintError::ArityMismatch { expected, found, .. } => {
                write!(f, "Arity mismatch: expected {expected} arguments, found {found}")
            }
            ConstraintError::NotCallable { typ, .. } => {
                write!(f, "Type {typ:?} is not callable")
            }
            ConstraintError::HandlerMismatch { handler, effect } => {
                write!(f, "Handler {} cannot handle effect {:?}", handler.as_str(), effect)
            }
            ConstraintError::EffectNotAvailable { effect, .. } => {
                write!(f, "Effect {effect:?} is not available in current context")
            }
        }
    }
}

impl std::error::Error for ConstraintError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeVar;
    use x_parser::{FileId, span::ByteOffset};

    #[test]
    fn test_constraint_set_creation() {
        let mut constraints = ConstraintSet::new();
        assert!(constraints.is_empty());

        let span = Span::new(FileId(u32::MAX), ByteOffset(0), ByteOffset(0));
        constraints.equal(Type::Con(Symbol::intern("Int")), Type::Con(Symbol::intern("Int")), span);
        assert!(!constraints.is_empty());
        assert_eq!(constraints.type_constraints.len(), 1);
    }

    #[test]
    fn test_constraint_solver() {
        let mut solver = ConstraintSolver::new();
        let constraints = ConstraintSet::new();
        
        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_substitution_composition() {
        let sub1 = Substitution::empty();
        let sub2 = Substitution::empty();
        
        let composed = sub1.compose(&sub2);
        assert!(composed.type_substitution.is_empty());
        assert!(composed.effect_substitution.is_empty());
    }

    #[test]
    fn test_type_substitution() {
        let mut type_sub = HashMap::new();
        type_sub.insert(TypeVar(0), Type::Con(Symbol::intern("Int")));
        
        let substitution = Substitution {
            type_substitution: type_sub,
            effect_substitution: HashMap::new(),
        };

        let var_type = Type::Var(TypeVar(0));
        let applied = substitution.apply_to_type(&var_type);
        
        assert_eq!(applied, Type::Con(Symbol::intern("Int")));
    }
}