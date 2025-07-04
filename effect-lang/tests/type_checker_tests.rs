//! Type checker tests for x Language
//! 
//! These tests verify the correctness of:
//! - Type inference
//! - Effect inference
//! - Unification
//! - Constraint solving

use effect_lang::analysis::{
    types::{
        Type, EffectSet, TypeVar, EffectVar, TypeScheme, Constraint, 
        TypeEnv, VarGen, Unifier, Substitution, Effect, Operation
    },
    inference::InferenceContext,
    unification::ConstraintSolver,
    effects::{EffectContext, HandlerInfo},
};
use effect_lang::core::{
    ast::{Literal, Parameter},
    symbol::Symbol,
    span::{Span, FileId, ByteOffset},
};

fn test_span() -> Span {
    Span::new(
        FileId::new(0),
        ByteOffset::new(0),
        ByteOffset::new(10),
    )
}

// Helper to create analysis types
fn int_type() -> Type {
    Type::Con(Symbol::intern("Int"))
}

fn string_type() -> Type {
    Type::Con(Symbol::intern("String"))
}

fn bool_type() -> Type {
    Type::Con(Symbol::intern("Bool"))
}

#[test]
fn test_literal_inference() {
    let mut ctx = InferenceContext::new();
    
    // Test integer literal
    let int_lit = Literal::Integer(42);
    let result = ctx.infer_literal(&int_lit).unwrap();
    assert!(matches!(result.typ, Type::Con(_)));
    assert!(matches!(result.effects, EffectSet::Empty));
    
    // Test string literal
    let str_lit = Literal::String("hello".to_string());
    let result = ctx.infer_literal(&str_lit).unwrap();
    assert!(matches!(result.typ, Type::Con(_)));
    
    // Test boolean literal
    let bool_lit = Literal::Bool(true);
    let result = ctx.infer_literal(&bool_lit).unwrap();
    assert!(matches!(result.typ, Type::Con(_)));
    
    // Test unit literal
    let unit_lit = Literal::Unit;
    let result = ctx.infer_literal(&unit_lit).unwrap();
    assert!(matches!(result.typ, Type::Con(_)));
}

#[test]
fn test_variable_inference() {
    let mut ctx = InferenceContext::new();
    
    // Add a variable to the environment
    let var_name = Symbol::intern("x");
    let scheme = TypeScheme {
        type_vars: vec![],
        effect_vars: vec![],
        constraints: vec![],
        body: int_type(),
    };
    ctx.env.insert_var(var_name, scheme);
    
    // Test variable lookup
    let result = ctx.infer_var(var_name).unwrap();
    assert!(matches!(result.typ, Type::Con(_)));
    
    // Test unbound variable
    let unbound = ctx.infer_var(Symbol::intern("y"));
    assert!(unbound.is_err());
}

#[test]
fn test_type_unification() {
    let mut unifier = Unifier::new();
    
    // Test basic unification
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
    
    // Try to unify a with (a -> a) - should fail due to occurs check
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
        params: vec![int_type()],
        return_type: Box::new(string_type()),
        effects: EffectSet::Empty,
    };
    
    let fun2 = Type::Fun {
        params: vec![int_type()],
        return_type: Box::new(string_type()),
        effects: EffectSet::Empty,
    };
    
    // These should unify successfully
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
    
    // Effect should unify with itself
    unifier.unify_effects(io_effect.clone(), io_effect).unwrap();
}

#[test]
fn test_mgu() {
    let var_a = TypeVar(0);
    let int_type = int_type();
    
    let subst = Unifier::mgu(&Type::Var(var_a), &int_type).unwrap();
    assert_eq!(subst.lookup_type(var_a), Some(&int_type));
}

#[test]
fn test_unifiable() {
    let var_a = TypeVar(0);
    let int_type = int_type();
    let string_type = string_type();
    
    assert!(Unifier::unifiable(&Type::Var(var_a), &int_type));
    assert!(!Unifier::unifiable(&int_type, &string_type));
}

#[test]
fn test_effect_context() {
    let mut ctx = EffectContext::new();
    
    // Test empty context
    assert!(!ctx.is_effect_handled(Symbol::intern("IO")));
    
    // Add handler
    let handler_info = HandlerInfo {
        effect: Symbol::intern("IO"),
        operations: std::collections::HashMap::new(),
        return_clause: None,
        handled_type: Type::Con(Symbol::intern("Unit")),
        result_type: Type::Con(Symbol::intern("Unit")),
    };
    
    ctx.push_handler(handler_info);
    assert!(ctx.is_effect_handled(Symbol::intern("IO")));
    
    // Remove handler
    ctx.pop_handler();
    assert!(!ctx.is_effect_handled(Symbol::intern("IO")));
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
    
    // Empty is subset of any effect set
    assert!(empty.is_subset_of(&io_effect));
    assert!(!io_effect.is_subset_of(&empty));
    
    // Effect set is subset of itself
    assert!(io_effect.is_subset_of(&io_effect));
}

#[test]
fn test_effect_contains() {
    let io_symbol = Symbol::intern("IO");
    let state_symbol = Symbol::intern("State");
    
    let io_effect = EffectSet::Row {
        effects: vec![Effect {
            name: io_symbol,
            operations: Vec::new(),
        }],
        tail: None,
    };
    
    assert!(io_effect.contains_effect(io_symbol));
    assert!(!io_effect.contains_effect(state_symbol));
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
fn test_constraint_solver() {
    let env = TypeEnv::new();
    let mut solver = ConstraintSolver::new(env);
    
    // Add a simple constraint
    let constraint = Constraint::Class {
        class: Symbol::intern("Eq"),
        types: vec![int_type()],
    };
    
    solver.add_constraints(vec![constraint]);
    
    // For now, this should fail because we don't have instance definitions
    let result = solver.solve();
    assert!(result.is_err()); // Expected since no instances are defined
}

#[test]
fn test_type_scheme_instantiation() {
    let mut ctx = InferenceContext::new();
    
    // Create polymorphic type scheme: ∀a. a -> a
    let type_var = TypeVar(0);
    let scheme = TypeScheme {
        type_vars: vec![type_var],
        effect_vars: vec![],
        constraints: vec![],
        body: Type::Fun {
            params: vec![Type::Var(type_var)],
            return_type: Box::new(Type::Var(type_var)),
            effects: EffectSet::Empty,
        },
    };
    
    // Instantiate should give fresh variables
    let (typ1, _effects1) = ctx.instantiate(&scheme);
    let (typ2, _effects2) = ctx.instantiate(&scheme);
    
    // Both should be function types but with different type variables
    match (&typ1, &typ2) {
        (Type::Fun { params: p1, return_type: r1, .. },
         Type::Fun { params: p2, return_type: r2, .. }) => {
            // Same structure but different variables
            assert_eq!(p1[0], **r1);
            assert_eq!(p2[0], **r2);
            // Different instantiations should have different variables
            assert_ne!(p1[0], p2[0]);
        }
        _ => panic!("Expected function types"),
    }
}

#[test]
fn test_generalization() {
    let mut ctx = InferenceContext::new();
    
    // Create a type with free variables
    let type_var = TypeVar(0);
    let typ = Type::Fun {
        params: vec![Type::Var(type_var)],
        return_type: Box::new(Type::Var(type_var)),
        effects: EffectSet::Empty,
    };
    
    let scheme = ctx.generalize(&typ, &EffectSet::Empty);
    
    // Should generalize the free variable
    assert!(scheme.type_vars.contains(&type_var));
}

#[test]
fn test_substitution_composition() {
    let var_a = TypeVar(0);
    let var_b = TypeVar(1);
    let int_type = int_type();
    
    let mut subst1 = Substitution::new();
    subst1.insert_type(var_a, Type::Var(var_b));
    
    let mut subst2 = Substitution::new();
    subst2.insert_type(var_b, int_type.clone());
    
    let composed = subst1.compose(&subst2);
    
    // Composing should resolve the chain: a -> b -> Int becomes a -> Int
    let final_type = Type::Var(var_a).apply_subst(&composed);
    assert_eq!(final_type, int_type);
}

#[test]
fn test_free_variables() {
    let var_a = TypeVar(0);
    let var_b = TypeVar(1);
    let var_c = TypeVar(2);
    
    // Type: a -> (b, c)
    let typ = Type::Fun {
        params: vec![Type::Var(var_a)],
        return_type: Box::new(Type::Tuple(vec![
            Type::Var(var_b),
            Type::Var(var_c),
        ])),
        effects: EffectSet::Empty,
    };
    
    let free_vars = typ.free_vars();
    assert!(free_vars.contains(&var_a));
    assert!(free_vars.contains(&var_b));
    assert!(free_vars.contains(&var_c));
    assert_eq!(free_vars.len(), 3);
}

#[test]
fn test_type_environment() {
    let mut env = TypeEnv::new();
    
    // Test built-in types are present
    assert!(env.lookup_type_con(Symbol::intern("Int")).is_some());
    assert!(env.lookup_type_con(Symbol::intern("String")).is_some());
    assert!(env.lookup_type_con(Symbol::intern("Bool")).is_some());
    
    // Test adding new variable
    let scheme = TypeScheme {
        type_vars: vec![],
        effect_vars: vec![],
        constraints: vec![],
        body: int_type(),
    };
    env.insert_var(Symbol::intern("x"), scheme.clone());
    
    assert_eq!(env.lookup_var(Symbol::intern("x")), Some(&scheme));
}

#[test]
fn test_var_generator() {
    let mut gen = VarGen::new();
    
    let var1 = gen.fresh_type_var();
    let var2 = gen.fresh_type_var();
    let effect1 = gen.fresh_effect_var();
    let effect2 = gen.fresh_effect_var();
    
    // Should generate unique variables
    assert_ne!(var1, var2);
    assert_ne!(effect1, effect2);
}

#[test]
fn test_tuple_type() {
    let tuple_type = Type::Tuple(vec![
        int_type(),
        string_type(),
        bool_type(),
    ]);
    
    // Check that the tuple has 3 elements
    match tuple_type {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 3);
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_record_type() {
    let record_type = Type::Record(vec![
        (Symbol::intern("x"), int_type()),
        (Symbol::intern("y"), string_type()),
    ]);
    
    // Check that the record has 2 fields
    match record_type {
        Type::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, Symbol::intern("x"));
            assert_eq!(fields[1].0, Symbol::intern("y"));
        }
        _ => panic!("Expected record type"),
    }
}

#[test]
fn test_forall_type() {
    let forall_type = Type::Forall {
        type_vars: vec![TypeVar(0), TypeVar(1)],
        effect_vars: vec![EffectVar(0)],
        body: Box::new(Type::Fun {
            params: vec![Type::Var(TypeVar(0))],
            return_type: Box::new(Type::Var(TypeVar(1))),
            effects: EffectSet::Var(EffectVar(0)),
        }),
    };
    
    // Check the forall structure
    match forall_type {
        Type::Forall { type_vars, effect_vars, body } => {
            assert_eq!(type_vars.len(), 2);
            assert_eq!(effect_vars.len(), 1);
            assert!(matches!(**body, Type::Fun { .. }));
        }
        _ => panic!("Expected forall type"),
    }
}

#[test]
fn test_recursive_type() {
    // Test recursive type: μα. (Int, α)
    let rec_var = TypeVar(0);
    let rec_body = Type::Tuple(vec![
        int_type(),
        Type::Var(rec_var),
    ]);
    
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(rec_body),
    };
    
    // Check the recursive type structure
    match rec_type {
        Type::Rec { var, body } => {
            assert_eq!(var, rec_var);
            match *body {
                Type::Tuple(types) => {
                    assert_eq!(types.len(), 2);
                    assert!(matches!(types[1], Type::Var(_)));
                }
                _ => panic!("Expected tuple in recursive type body"),
            }
        }
        _ => panic!("Expected recursive type"),
    }
}

#[test]
fn test_recursive_type_unfold() {
    // Test recursive type: μα. (Int, α)
    let rec_var = TypeVar(0);
    let rec_body = Type::Tuple(vec![
        int_type(),
        Type::Var(rec_var),
    ]);
    
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(rec_body),
    };
    
    // Unfold the recursive type
    let unfolded = rec_type.unfold_rec();
    
    // Should get: (Int, μα. (Int, α))
    match unfolded {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 2);
            assert!(matches!(types[0], Type::Con(_))); // Int
            assert!(matches!(types[1], Type::Rec { .. })); // The recursive type itself
        }
        _ => panic!("Expected tuple after unfolding"),
    }
}

#[test]
fn test_recursive_type_unification() {
    let mut unifier = Unifier::new();
    
    // Create two identical recursive types: μα. (Int, α)
    let rec_var1 = TypeVar(0);
    let rec_var2 = TypeVar(1);
    
    let rec_type1 = Type::Rec {
        var: rec_var1,
        body: Box::new(Type::Tuple(vec![
            int_type(),
            Type::Var(rec_var1),
        ])),
    };
    
    let rec_type2 = Type::Rec {
        var: rec_var2,
        body: Box::new(Type::Tuple(vec![
            int_type(),
            Type::Var(rec_var2),
        ])),
    };
    
    // These should unify successfully
    unifier.unify_types(rec_type1, rec_type2).unwrap();
}

#[test]
fn test_recursive_type_free_vars() {
    let rec_var = TypeVar(0);
    let free_var = TypeVar(1);
    
    // μα. (α, β) where β is free
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Tuple(vec![
            Type::Var(rec_var),    // bound variable
            Type::Var(free_var),   // free variable
        ])),
    };
    
    let free_vars = rec_type.free_vars();
    
    // Should only contain the free variable, not the bound one
    assert!(free_vars.contains(&free_var));
    assert!(!free_vars.contains(&rec_var));
    assert_eq!(free_vars.len(), 1);
}

#[test]
fn test_recursive_type_substitution() {
    let rec_var = TypeVar(0);
    let free_var = TypeVar(1);
    
    // μα. (α, β) where β is free
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Tuple(vec![
            Type::Var(rec_var),    // bound variable
            Type::Var(free_var),   // free variable
        ])),
    };
    
    // Substitute β with Int
    let mut subst = Substitution::new();
    subst.insert_type(free_var, int_type());
    
    let substituted = rec_type.apply_subst(&subst);
    
    match substituted {
        Type::Rec { var, body } => {
            assert_eq!(var, rec_var); // bound variable unchanged
            match *body {
                Type::Tuple(types) => {
                    assert_eq!(types.len(), 2);
                    assert!(matches!(types[0], Type::Var(_))); // still bound variable
                    assert!(matches!(types[1], Type::Con(_))); // substituted to Int
                }
                _ => panic!("Expected tuple in substituted recursive type"),
            }
        }
        _ => panic!("Expected recursive type after substitution"),
    }
}

#[test]
fn test_recursive_list_type() {
    // List α = μβ. Nil | Cons α β
    // This represents a more complex recursive type
    let list_var = TypeVar(0);  // α (element type)
    let rec_var = TypeVar(1);   // β (recursive variable)
    
    let nil_variant = (Symbol::intern("Nil"), vec![]);
    let cons_variant = (Symbol::intern("Cons"), vec![
        Type::Var(list_var),  // element
        Type::Var(rec_var),   // rest of list
    ]);
    
    let list_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Variant(vec![nil_variant, cons_variant])),
    };
    
    // Check the structure
    match list_type {
        Type::Rec { var, body } => {
            assert_eq!(var, rec_var);
            match *body {
                Type::Variant(variants) => {
                    assert_eq!(variants.len(), 2);
                    assert_eq!(variants[0].0, Symbol::intern("Nil"));
                    assert_eq!(variants[1].0, Symbol::intern("Cons"));
                    assert_eq!(variants[1].1.len(), 2);
                }
                _ => panic!("Expected variant in list type"),
            }
        }
        _ => panic!("Expected recursive type for list"),
    }
}