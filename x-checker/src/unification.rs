//! Unification and constraint solving for x Language
//! 
//! This module implements:
//! - Type unification with occurs check
//! - Effect unification with row polymorphism
//! - Constraint solving for type classes
//! - Substitution composition and application

use crate::analysis::types::*;
use crate::analysis::effects::*;
use crate::core::symbol::Symbol;
use crate::{Error, Result};

use std::collections::{HashMap, HashSet, VecDeque};

/// Unification engine
#[derive(Debug, Clone)]
pub struct Unifier {
    /// Current substitution being built
    substitution: Substitution,
    
    /// Constraints to be solved
    constraints: VecDeque<UnificationConstraint>,
    
    /// Solved constraints
    solved: Vec<Constraint>,
}

/// Internal constraints for unification
#[derive(Debug, Clone)]
enum UnificationConstraint {
    /// Type equality constraint
    Unify(Type, Type),
    
    /// Effect equality constraint
    UnifyEffect(EffectSet, EffectSet),
    
    /// Row constraint (lacks label)
    RowLacks(EffectSet, Symbol),
    
    /// Type class constraint
    Class(Symbol, Vec<Type>),
}

impl Unifier {
    pub fn new() -> Self {
        Unifier {
            substitution: Substitution::new(),
            constraints: VecDeque::new(),
            solved: Vec::new(),
        }
    }
    
    /// Add a type unification constraint
    pub fn unify_types(&mut self, t1: Type, t2: Type) -> Result<()> {
        self.constraints.push_back(UnificationConstraint::Unify(t1, t2));
        self.solve_constraints()
    }
    
    /// Add an effect unification constraint
    pub fn unify_effects(&mut self, e1: EffectSet, e2: EffectSet) -> Result<()> {
        self.constraints.push_back(UnificationConstraint::UnifyEffect(e1, e2));
        self.solve_constraints()
    }
    
    /// Add a row constraint
    pub fn add_row_constraint(&mut self, row: EffectSet, lacks: Symbol) -> Result<()> {
        self.constraints.push_back(UnificationConstraint::RowLacks(row, lacks));
        self.solve_constraints()
    }
    
    /// Get the current substitution
    pub fn get_substitution(&self) -> &Substitution {
        &self.substitution
    }
    
    /// Solve all pending constraints
    fn solve_constraints(&mut self) -> Result<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            self.solve_constraint(constraint)?;
        }
        Ok(())
    }
    
    /// Solve a single constraint
    fn solve_constraint(&mut self, constraint: UnificationConstraint) -> Result<()> {
        match constraint {
            UnificationConstraint::Unify(t1, t2) => self.unify_types_impl(t1, t2),
            UnificationConstraint::UnifyEffect(e1, e2) => self.unify_effects_impl(e1, e2),
            UnificationConstraint::RowLacks(row, label) => self.solve_row_lacks(row, label),
            UnificationConstraint::Class(class, types) => self.solve_class_constraint(class, types),
        }
    }
    
    /// Core type unification implementation
    fn unify_types_impl(&mut self, t1: Type, t2: Type) -> Result<()> {
        let t1 = t1.apply_subst(&self.substitution);
        let t2 = t2.apply_subst(&self.substitution);
        
        match (t1, t2) {
            // Same variables unify trivially
            (Type::Var(v1), Type::Var(v2)) if v1 == v2 => Ok(()),
            
            // Variable unification
            (Type::Var(var), typ) => self.unify_var(var, typ),
            (typ, Type::Var(var)) => self.unify_var(var, typ),
            
            // Constructor unification
            (Type::Con(n1), Type::Con(n2)) if n1 == n2 => Ok(()),
            
            // Application unification
            (Type::App(c1, args1), Type::App(c2, args2)) => {
                self.unify_types_impl(*c1, *c2)?;
                if args1.len() != args2.len() {
                    return Err(Error::Type {
                        message: format!(
                            "Type application arity mismatch: {} vs {}",
                            args1.len(),
                            args2.len()
                        ),
                    });
                }
                for (a1, a2) in args1.into_iter().zip(args2.into_iter()) {
                    self.unify_types_impl(a1, a2)?;
                }
                Ok(())
            }
            
            // Function type unification
            (Type::Fun { params: p1, return_type: r1, effects: e1 },
             Type::Fun { params: p2, return_type: r2, effects: e2 }) => {
                if p1.len() != p2.len() {
                    return Err(Error::Type {
                        message: format!(
                            "Function arity mismatch: {} vs {}",
                            p1.len(),
                            p2.len()
                        ),
                    });
                }
                
                for (param1, param2) in p1.into_iter().zip(p2.into_iter()) {
                    self.unify_types_impl(param1, param2)?;
                }
                
                self.unify_types_impl(*r1, *r2)?;
                self.unify_effects_impl(e1, e2)?;
                Ok(())
            }
            
            // Record type unification
            (Type::Record(fields1), Type::Record(fields2)) => {
                if fields1.len() != fields2.len() {
                    return Err(Error::Type {
                        message: format!(
                            "Record field count mismatch: {} vs {}",
                            fields1.len(),
                            fields2.len()
                        ),
                    });
                }
                
                // Sort fields for comparison
                let mut sorted1 = fields1;
                let mut sorted2 = fields2;
                sorted1.sort_by_key(|(name, _)| *name);
                sorted2.sort_by_key(|(name, _)| *name);
                
                for ((name1, type1), (name2, type2)) in sorted1.into_iter().zip(sorted2.into_iter()) {
                    if name1 != name2 {
                        return Err(Error::Type {
                            message: format!("Record field mismatch: {} vs {}", name1, name2),
                        });
                    }
                    self.unify_types_impl(type1, type2)?;
                }
                Ok(())
            }
            
            // Tuple unification
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(Error::Type {
                        message: format!(
                            "Tuple length mismatch: {} vs {}",
                            types1.len(),
                            types2.len()
                        ),
                    });
                }
                
                for (t1, t2) in types1.into_iter().zip(types2.into_iter()) {
                    self.unify_types_impl(t1, t2)?;
                }
                Ok(())
            }
            
            // Forall unification (alpha conversion)
            (Type::Forall { type_vars: vars1, body: body1, .. },
             Type::Forall { type_vars: vars2, body: body2, .. }) => {
                if vars1.len() != vars2.len() {
                    return Err(Error::Type {
                        message: "Forall variable count mismatch".to_string(),
                    });
                }
                
                // Rename variables in body2 to match body1
                let mut rename_subst = Substitution::new();
                for (&var1, &var2) in vars1.iter().zip(vars2.iter()) {
                    rename_subst.insert_type(var2, Type::Var(var1));
                }
                
                let renamed_body2 = body2.apply_subst(&rename_subst);
                self.unify_types_impl(*body1, renamed_body2)
            }
            
            // Hole unifies with anything
            (Type::Hole, _) | (_, Type::Hole) => Ok(()),
            
            // Recursive type unification
            (Type::Rec { var: v1, body: b1 }, Type::Rec { var: v2, body: b2 }) => {
                // Rename variables in one recursive type to match the other
                let mut rename_subst = Substitution::new();
                rename_subst.insert_type(v2, Type::Var(v1));
                let renamed_b2 = b2.apply_subst(&rename_subst);
                self.unify_types_impl(*b1.clone(), renamed_b2)
            }
            
            // Mismatch
            (t1, t2) => Err(Error::Type {
                message: format!("Cannot unify {} with {}", t1, t2),
            }),
        }
    }
    
    /// Unify a variable with a type
    fn unify_var(&mut self, var: TypeVar, typ: Type) -> Result<()> {
        // Occurs check
        if typ.free_vars().contains(&var) {
            return Err(Error::Type {
                message: format!("Occurs check failed: {} occurs in {}", Type::Var(var), typ),
            });
        }
        
        // Add to substitution
        self.substitution.insert_type(var, typ);
        Ok(())
    }
    
    /// Core effect unification implementation
    fn unify_effects_impl(&mut self, e1: EffectSet, e2: EffectSet) -> Result<()> {
        let e1 = e1.apply_subst(&self.substitution);
        let e2 = e2.apply_subst(&self.substitution);
        
        match (e1, e2) {
            // Empty effects unify
            (EffectSet::Empty, EffectSet::Empty) => Ok(()),
            
            // Variable unification
            (EffectSet::Var(v1), EffectSet::Var(v2)) if v1 == v2 => Ok(()),
            (EffectSet::Var(var), effects) => self.unify_effect_var(var, effects),
            (effects, EffectSet::Var(var)) => self.unify_effect_var(var, effects),
            
            // Row unification
            (EffectSet::Row { effects: e1, tail: t1 },
             EffectSet::Row { effects: e2, tail: t2 }) => {
                self.unify_effect_rows(e1, t1, e2, t2)
            }
            
            // Empty with non-empty
            (EffectSet::Empty, EffectSet::Row { effects, .. }) if effects.is_empty() => Ok(()),
            (EffectSet::Row { effects, .. }, EffectSet::Empty) if effects.is_empty() => Ok(()),
            
            // Mismatch
            (e1, e2) => Err(Error::Effect {
                message: format!("Cannot unify effect sets {} with {}", e1, e2),
            }),
        }
    }
    
    /// Unify an effect variable with an effect set
    fn unify_effect_var(&mut self, var: EffectVar, effects: EffectSet) -> Result<()> {
        // Check for occurs check in effects
        if self.effect_var_occurs_in(var, &effects) {
            return Err(Error::Effect {
                message: format!("Occurs check failed in effects: {:?} occurs in {:?}", var, effects),
            });
        }
        
        self.substitution.insert_effect(var, effects);
        Ok(())
    }
    
    /// Check if an effect variable occurs in an effect set
    fn effect_var_occurs_in(&self, var: EffectVar, effects: &EffectSet) -> bool {
        match effects {
            EffectSet::Empty => false,
            EffectSet::Var(v) => *v == var,
            EffectSet::Row { tail: Some(tail), .. } => self.effect_var_occurs_in(var, tail),
            EffectSet::Row { tail: None, .. } => false,
        }
    }
    
    /// Unify effect rows
    fn unify_effect_rows(
        &mut self,
        effects1: Vec<Effect>,
        tail1: Option<Box<EffectSet>>,
        effects2: Vec<Effect>,
        tail2: Option<Box<EffectSet>>,
    ) -> Result<()> {
        // Find common effects
        let mut remaining1 = effects1;
        let mut remaining2 = effects2;
        
        // Remove common effects
        remaining1.retain(|e1| {
            let found_index = remaining2.iter().position(|e2| e1.name == e2.name);
            if let Some(index) = found_index {
                remaining2.remove(index);
                false // Remove this effect from remaining1
            } else {
                true // Keep this effect
            }
        });
        
        // Handle remaining effects and tails
        match (remaining1.is_empty(), remaining2.is_empty(), &tail1, &tail2) {
            // Both empty, unify tails
            (true, true, Some(t1), Some(t2)) => self.unify_effects_impl(*t1.clone(), *t2.clone()),
            (true, true, None, None) => Ok(()),
            
            // One side has remaining effects, other must have compatible tail
            (false, true, tail1_opt, Some(t2)) => {
                let remaining_set = EffectSet::Row {
                    effects: remaining1,
                    tail: tail1_opt.clone(),
                };
                self.unify_effects_impl(remaining_set, *t2.clone())
            }
            (true, false, Some(t1), tail2_opt) => {
                let remaining_set = EffectSet::Row {
                    effects: remaining2,
                    tail: tail2_opt.clone(),
                };
                self.unify_effects_impl(*t1.clone(), remaining_set)
            }
            
            _ => Err(Error::Effect {
                message: "Effect row unification failed".to_string(),
            }),
        }
    }
    
    /// Solve row lacks constraint
    fn solve_row_lacks(&mut self, row: EffectSet, lacks: Symbol) -> Result<()> {
        let row = row.apply_subst(&self.substitution);
        
        match row {
            EffectSet::Empty => Ok(()), // Empty row lacks everything
            EffectSet::Var(_) => {
                // Add constraint to be checked later
                self.constraints.push_back(UnificationConstraint::RowLacks(row, lacks));
                Ok(())
            }
            EffectSet::Row { effects, tail } => {
                // Check that the effect is not present
                if effects.iter().any(|e| e.name == lacks) {
                    return Err(Error::Effect {
                        message: format!("Row constraint violation: row contains {}", lacks),
                    });
                }
                
                // Recursively check tail
                if let Some(tail) = tail {
                    self.solve_row_lacks(*tail, lacks)
                } else {
                    Ok(())
                }
            }
        }
    }
    
    /// Solve type class constraint
    fn solve_class_constraint(&mut self, class: Symbol, types: Vec<Type>) -> Result<()> {
        // Apply current substitution to types
        let types: Vec<Type> = types.into_iter()
            .map(|t| t.apply_subst(&self.substitution))
            .collect();
        
        // For now, just store the constraint as solved
        // In a full implementation, this would check instance resolution
        self.solved.push(Constraint::Class { class, types });
        Ok(())
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced unification utilities
impl Unifier {
    /// Most general unifier for two types
    pub fn mgu(t1: &Type, t2: &Type) -> Result<Substitution> {
        let mut unifier = Unifier::new();
        unifier.unify_types(t1.clone(), t2.clone())?;
        Ok(unifier.substitution)
    }
    
    /// Most general unifier for effect sets
    pub fn mgu_effects(e1: &EffectSet, e2: &EffectSet) -> Result<Substitution> {
        let mut unifier = Unifier::new();
        unifier.unify_effects(e1.clone(), e2.clone())?;
        Ok(unifier.substitution)
    }
    
    /// Check if two types are unifiable
    pub fn unifiable(t1: &Type, t2: &Type) -> bool {
        Self::mgu(t1, t2).is_ok()
    }
    
    /// Match a type against a pattern (one-way unification)
    pub fn match_type(pattern: &Type, target: &Type) -> Result<Substitution> {
        let mut unifier = Unifier::new();
        unifier.match_type_impl(pattern.clone(), target.clone())?;
        Ok(unifier.substitution)
    }
    
    fn match_type_impl(&mut self, pattern: Type, target: Type) -> Result<()> {
        match pattern {
            Type::Var(var) => {
                self.substitution.insert_type(var, target);
                Ok(())
            }
            Type::Con(name1) => match target {
                Type::Con(name2) if name1 == name2 => Ok(()),
                _ => Err(Error::Type {
                    message: format!("Cannot match {} with {}", pattern, target),
                }),
            },
            Type::App(ref con1, ref args1) => match target {
                Type::App(con2, args2) => {
                    self.match_type_impl((**con1).clone(), (*con2).clone())?;
                    if args1.len() != args2.len() {
                        return Err(Error::Type {
                            message: "Arity mismatch in type application".to_string(),
                        });
                    }
                    for (arg1, arg2) in args1.into_iter().zip(args2.into_iter()) {
                        self.match_type_impl(arg1.clone(), arg2.clone())?;
                    }
                    Ok(())
                }
                _ => Err(Error::Type {
                    message: format!("Cannot match {} with {}", pattern, target),
                }),
            },
            _ => {
                // For other patterns, fall back to regular unification
                self.unify_types_impl(pattern, target)
            }
        }
    }
}

/// Constraint propagation and simplification
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    /// Constraints to solve
    constraints: Vec<Constraint>,
    
    /// Type environment for instance resolution
    type_env: TypeEnv,
    
    /// Current substitution
    substitution: Substitution,
}

impl ConstraintSolver {
    pub fn new(type_env: TypeEnv) -> Self {
        ConstraintSolver {
            constraints: Vec::new(),
            type_env,
            substitution: Substitution::new(),
        }
    }
    
    /// Add constraints to solve
    pub fn add_constraints(&mut self, constraints: Vec<Constraint>) {
        self.constraints.extend(constraints);
    }
    
    /// Solve all constraints
    pub fn solve(&mut self) -> Result<Substitution> {
        while !self.constraints.is_empty() {
            let constraint = self.constraints.remove(0);
            self.solve_constraint(constraint)?;
        }
        Ok(self.substitution.clone())
    }
    
    fn solve_constraint(&mut self, constraint: Constraint) -> Result<()> {
        match constraint {
            Constraint::Class { class, types } => {
                self.solve_class_instance(class, types)
            }
            Constraint::Effect { effect, type_ } => {
                self.solve_effect_constraint(effect, type_)
            }
            Constraint::Row { lacks, row } => {
                self.solve_row_constraint(lacks, row)
            }
        }
    }
    
    fn solve_class_instance(&mut self, class: Symbol, types: Vec<Type>) -> Result<()> {
        // Apply current substitution
        let types: Vec<Type> = types.into_iter()
            .map(|t| t.apply_subst(&self.substitution))
            .collect();
        
        // Look up instances in environment
        if let Some(instances) = self.type_env.instances.get(&class) {
            for instance in instances {
                if let Ok(subst) = self.match_instance(&instance.head, &types) {
                    // Found matching instance, add its constraints
                    self.substitution = self.substitution.compose(&subst);
                    for constraint in &instance.constraints {
                        self.constraints.push(constraint.clone());
                    }
                    return Ok(());
                }
            }
        }
        
        Err(Error::Type {
            message: format!("No instance found for {} {:?}", class, types),
        })
    }
    
    fn match_instance(&self, instance_head: &Constraint, types: &[Type]) -> Result<Substitution> {
        match instance_head {
            Constraint::Class { class: _, types: instance_types } => {
                if instance_types.len() != types.len() {
                    return Err(Error::Type {
                        message: "Instance arity mismatch".to_string(),
                    });
                }
                
                let mut unifier = Unifier::new();
                for (instance_type, target_type) in instance_types.iter().zip(types.iter()) {
                    unifier.match_type_impl(instance_type.clone(), target_type.clone())?;
                }
                Ok(unifier.substitution)
            }
            _ => Err(Error::Type {
                message: "Invalid instance head".to_string(),
            }),
        }
    }
    
    fn solve_effect_constraint(&mut self, _effect: Effect, _type_: Type) -> Result<()> {
        // Check that the type supports the effect
        // This is a simplified implementation
        Ok(())
    }
    
    fn solve_row_constraint(&mut self, lacks: Symbol, row: EffectSet) -> Result<()> {
        let row = row.apply_subst(&self.substitution);
        
        // Check that the row doesn't contain the lacks label
        if row.contains_effect(lacks) {
            Err(Error::Effect {
                message: format!("Row constraint violation: {} present in row", lacks),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::symbol::Symbol;
    
    #[test]
    fn test_variable_unification() {
        let mut unifier = Unifier::new();
        let var_a = TypeVar(0);
        let int_type = Type::Con(Symbol::intern("Int"));
        
        unifier.unify_types(Type::Var(var_a), int_type.clone()).unwrap();
        
        let subst = unifier.get_substitution();
        assert_eq!(subst.lookup_type(var_a), Some(&int_type));
    }
    
    #[test]
    fn test_occurs_check() {
        let mut unifier = Unifier::new();
        let var_a = TypeVar(0);
        
        // Try to unify a with (a -> a)
        let recursive_type = Type::Fun {
            params: vec![Type::Var(var_a)],
            return_type: Box::new(Type::Var(var_a)),
            effects: EffectSet::Empty,
        };
        
        let result = unifier.unify_types(Type::Var(var_a), recursive_type);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_function_unification() {
        let mut unifier = Unifier::new();
        
        let fun1 = Type::Fun {
            params: vec![Type::Con(Symbol::intern("Int"))],
            return_type: Box::new(Type::Con(Symbol::intern("String"))),
            effects: EffectSet::Empty,
        };
        
        let fun2 = Type::Fun {
            params: vec![Type::Con(Symbol::intern("Int"))],
            return_type: Box::new(Type::Con(Symbol::intern("String"))),
            effects: EffectSet::Empty,
        };
        
        unifier.unify_types(fun1, fun2).unwrap();
    }
    
    #[test]
    fn test_effect_unification() {
        let mut unifier = Unifier::new();
        
        let io_effect = EffectSet::Row {
            effects: vec![Effect {
                name: Symbol::intern("IO"),
                operations: Vec::new(),
            }],
            tail: None,
        };
        
        unifier.unify_effects(io_effect.clone(), io_effect).unwrap();
    }
    
    #[test]
    fn test_mgu() {
        let var_a = TypeVar(0);
        let int_type = Type::Con(Symbol::intern("Int"));
        
        let subst = Unifier::mgu(&Type::Var(var_a), &int_type).unwrap();
        assert_eq!(subst.lookup_type(var_a), Some(&int_type));
    }
    
    #[test]
    fn test_unifiable() {
        let var_a = TypeVar(0);
        let int_type = Type::Con(Symbol::intern("Int"));
        let string_type = Type::Con(Symbol::intern("String"));
        
        assert!(Unifier::unifiable(&Type::Var(var_a), &int_type));
        assert!(!Unifier::unifiable(&int_type, &string_type));
    }
}