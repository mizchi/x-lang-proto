//! Simple test for recursive types

use effect_lang::analysis::types::{Type, TypeVar};
use effect_lang::core::symbol::Symbol;

#[test]
fn test_recursive_type_creation() {
    // Test recursive type: μα. (Int, α)
    let rec_var = TypeVar(0);
    let int_type = Type::Con(Symbol::intern("Int"));
    
    let rec_body = Type::Tuple(vec![
        int_type,
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
    let int_type = Type::Con(Symbol::intern("Int"));
    
    let rec_body = Type::Tuple(vec![
        int_type,
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
fn test_recursive_type_display() {
    // Test recursive type display: μα. (Int, α)
    let rec_var = TypeVar(0);
    let int_type = Type::Con(Symbol::intern("Int"));
    
    let rec_body = Type::Tuple(vec![
        int_type,
        Type::Var(rec_var),
    ]);
    
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(rec_body),
    };
    
    let display_string = format!("{}", rec_type);
    println!("Recursive type display: {}", display_string);
    
    // Should contain μ and the variable name
    assert!(display_string.contains("μ"));
    assert!(display_string.contains("a0")); // TypeVar(0) displays as a0
}