//! Minimal test for recursive types - only types module

#[test]
fn test_recursive_type_basic() {
    use effect_lang::analysis::types::{Type, TypeVar};
    use effect_lang::core::symbol::Symbol;
    
    // Create a simple recursive type: μα. α
    let rec_var = TypeVar(0);
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Var(rec_var)),
    };
    
    // Test structure
    match rec_type {
        Type::Rec { var, body } => {
            assert_eq!(var, rec_var);
            match *body {
                Type::Var(v) => assert_eq!(v, rec_var),
                _ => panic!("Expected variable in recursive type body"),
            }
        }
        _ => panic!("Expected recursive type"),
    }
}

#[test]
fn test_recursive_type_unfold_simple() {
    use effect_lang::analysis::types::{Type, TypeVar};
    
    // Create recursive type: μα. α  
    let rec_var = TypeVar(0);
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Var(rec_var)),
    };
    
    // Unfold should return the original recursive type
    let unfolded = rec_type.unfold_rec();
    
    match unfolded {
        Type::Rec { var, body: _ } => {
            assert_eq!(var, rec_var);
        }
        _ => panic!("Expected recursive type after unfolding"),
    }
}

#[test]
fn test_recursive_type_free_vars_simple() {
    use effect_lang::analysis::types::{Type, TypeVar};
    
    // μα. α has no free variables
    let rec_var = TypeVar(0);
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Var(rec_var)),
    };
    
    let free_vars = rec_type.free_vars();
    assert!(free_vars.is_empty());
}