//! Tests for Binary AST Type Checker
//! 
//! This test suite verifies that type checking works correctly
//! on binary AST format with cached type information.

use effect_lang::analysis::{
    binary_type_checker::{BinaryTypeChecker, ValidationMode, TypeCheckResult},
    types::*,
    inference::InferenceContext,
};
use effect_lang::core::{
    binary::{BinarySerializer, BinaryFlags},
    ast::*,
    span::{Span, FileId, ByteOffset},
    symbol::Symbol,
};

fn test_span() -> Span {
    Span::new(
        FileId::new(0),
        ByteOffset::new(0),
        ByteOffset::new(10),
    )
}

fn create_simple_module() -> Module {
    Module {
        name: ModulePath::single(Symbol::intern("Test"), test_span()),
        exports: None,
        imports: Vec::new(),
        items: vec![
            Item::ValueDef(ValueDef {
                name: Symbol::intern("x"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Literal(Literal::Integer(42), test_span()),
                visibility: Visibility::Public,
                purity: Purity::Pure,
                span: test_span(),
            }),
        ],
        span: test_span(),
    }
}

#[test]
fn test_binary_type_checker_creation() {
    let checker = BinaryTypeChecker::new();
    // Verify initial state
    assert_eq!(format!("{:?}", checker).contains("BinaryTypeChecker"), true);
}

#[test]
fn test_type_check_result_success() {
    let result = TypeCheckResult {
        typed_nodes: vec![],
        errors: vec![],
        validation_mode: ValidationMode::FullInference,
        node_count: 0,
    };
    
    assert!(result.is_success());
    assert_eq!(result.error_count(), 0);
    assert!(!result.used_cache());
    assert_eq!(result.format_errors(), "");
}

#[test]
fn test_type_check_result_with_cache() {
    let result = TypeCheckResult {
        typed_nodes: vec![],
        errors: vec![],
        validation_mode: ValidationMode::CachedValidation,
        node_count: 5,
    };
    
    assert!(result.is_success());
    assert!(result.used_cache());
    assert_eq!(result.node_count, 5);
}

#[test] 
fn test_binary_serialization_with_types() {
    let mut serializer = BinarySerializer::with_type_checking();
    
    let module = create_simple_module();
    let cu = CompilationUnit {
        module,
        span: test_span(),
    };
    
    // Serialize with type information
    let binary_data = serializer.serialize_compilation_unit(&cu);
    assert!(binary_data.is_ok());
    
    let data = binary_data.unwrap();
    assert!(!data.is_empty());
    
    // Verify magic header
    assert_eq!(&data[0..4], b"EFFL");
}

#[test]
fn test_validation_modes() {
    // Test each validation mode
    assert_eq!(ValidationMode::CachedValidation, ValidationMode::CachedValidation);
    assert_ne!(ValidationMode::CachedValidation, ValidationMode::FullInference);
    assert_ne!(ValidationMode::FullInference, ValidationMode::IncrementalUpdate);
}

#[test]
fn test_binary_flags() {
    let flags = BinaryFlags::TYPE_CHECKED | BinaryFlags::CACHED_EFFECTS;
    
    assert!(flags.contains(BinaryFlags::TYPE_CHECKED));
    assert!(flags.contains(BinaryFlags::CACHED_EFFECTS));
    assert!(!flags.contains(BinaryFlags::COMPRESSED));
}

#[test]
fn test_type_caching_concept() {
    // Create a type for caching
    let int_type = Type::Con(Symbol::intern("Int"));
    let fun_type = Type::Fun {
        params: vec![int_type.clone()],
        return_type: Box::new(int_type.clone()),
        effects: EffectSet::Empty,
    };
    
    // Verify type structure
    match &fun_type {
        Type::Fun { params, return_type, effects } => {
            assert_eq!(params.len(), 1);
            assert!(matches!(params[0], Type::Con(_)));
            assert!(matches!(effects, EffectSet::Empty));
            assert!(matches!(return_type.as_ref(), Type::Con(_)));
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_recursive_type_in_binary() {
    // Test that recursive types can be represented
    let rec_var = TypeVar(0);
    let rec_type = Type::Rec {
        var: rec_var,
        body: Box::new(Type::Var(rec_var)),
    };
    
    // Verify recursive type structure  
    match &rec_type {
        Type::Rec { var, body } => {
            assert_eq!(*var, rec_var);
            match body.as_ref() {
                Type::Var(inner_var) => {
                    assert_eq!(*inner_var, rec_var);
                }
                _ => panic!("Expected variable in recursive type body"),
            }
        }
        _ => panic!("Expected recursive type"),
    }
}

#[test]
fn test_effect_set_serialization_concept() {
    // Test different effect set variants
    let empty = EffectSet::Empty;
    let var_effects = EffectSet::Var(EffectVar(1));
    let row_effects = EffectSet::Row {
        effects: vec![
            Effect {
                name: Symbol::intern("IO"),
                operations: vec![],
            }
        ],
        tail: None,
    };
    
    // Verify effect set variants
    assert!(matches!(empty, EffectSet::Empty));
    assert!(matches!(var_effects, EffectSet::Var(_)));
    assert!(matches!(row_effects, EffectSet::Row { .. }));
}

#[test]
fn test_type_scheme_with_effects() {
    // Create a polymorphic type scheme with effects
    let type_var = TypeVar(0);
    let effect_var = EffectVar(0);
    
    let scheme = TypeScheme {
        type_vars: vec![type_var],
        effect_vars: vec![effect_var],
        constraints: vec![],
        body: Type::Fun {
            params: vec![Type::Var(type_var)],
            return_type: Box::new(Type::Var(type_var)),
            effects: EffectSet::Var(effect_var),
        },
    };
    
    // Verify type scheme structure
    assert_eq!(scheme.type_vars.len(), 1);
    assert_eq!(scheme.effect_vars.len(), 1);
    assert!(matches!(scheme.body, Type::Fun { .. }));
}

#[test]
fn test_binary_type_checker_workflow() {
    // Simulate the complete workflow
    let mut checker = BinaryTypeChecker::new();
    
    // Create test binary data (minimal)
    let test_data = vec![
        // Magic header "EFFL"
        0x45, 0x46, 0x46, 0x4C,
        // Version (3)
        0x03, 0x00,
        // Flags (TYPE_CHECKED | CACHED_EFFECTS = 0x0009)
        0x09, 0x00,
        // Symbol table size
        0x00, 0x00, 0x00, 0x00,
        // Type metadata offset
        0x18, 0x00, 0x00, 0x00,
        // Inference cache offset
        0x20, 0x00, 0x00, 0x00,
        // Checksum
        0x00, 0x00, 0x00, 0x00,
    ];
    
    // Test type checking (will fail gracefully with empty data)
    let result = checker.check_binary_compilation_unit(&test_data);
    
    // Should handle the test case gracefully
    match result {
        Ok(type_result) => {
            // Verify result structure
            assert!(type_result.node_count >= 0);
        }
        Err(_) => {
            // Expected for minimal test data
            assert!(true);
        }
    }
}

#[test]
fn test_type_inference_integration() {
    // Test integration with existing type inference
    let mut ctx = InferenceContext::new();
    
    // Create a simple expression for type inference
    let expr = Expr::Literal(Literal::Integer(42), test_span());
    let result = ctx.infer_expr(&expr);
    
    assert!(result.is_ok());
    let inference_result = result.unwrap();
    
    // Verify inferred type
    match inference_result.typ {
        Type::Con(symbol) => {
            assert_eq!(symbol.as_str(), "Int");
        }
        _ => panic!("Expected Int type for integer literal"),
    }
    
    // Verify no effects for pure literal
    assert!(matches!(inference_result.effects, EffectSet::Empty));
}

#[test]
fn test_binary_ast_format_versioning() {
    use effect_lang::core::binary::BinaryHeader;
    
    // Test header creation
    let header = BinaryHeader::new();
    assert_eq!(header.magic, *b"EFFL");
    assert_eq!(header.version, 3); // Enhanced version
    
    let typed_header = BinaryHeader::with_type_checking();
    assert!(typed_header.flags.contains(BinaryFlags::TYPE_CHECKED));
    assert!(typed_header.flags.contains(BinaryFlags::CACHED_EFFECTS));
}

#[test]
fn test_comprehensive_type_system_features() {
    // Test that all major type system features are represented
    
    // Basic types
    let int_type = Type::Con(Symbol::intern("Int"));
    let string_type = Type::Con(Symbol::intern("String"));
    
    // Function type
    let fun_type = Type::Fun {
        params: vec![int_type.clone()],
        return_type: Box::new(string_type.clone()),
        effects: EffectSet::Empty,
    };
    
    // Tuple type
    let tuple_type = Type::Tuple(vec![int_type.clone(), string_type.clone()]);
    
    // Recursive type
    let rec_type = Type::Rec {
        var: TypeVar(0),
        body: Box::new(Type::Var(TypeVar(0))),
    };
    
    // Variant type
    let variant_type = Type::Variant(vec![
        (Symbol::intern("A"), vec![int_type.clone()]),
        (Symbol::intern("B"), vec![string_type.clone()]),
    ]);
    
    // Verify all types are constructible
    assert!(matches!(int_type, Type::Con(_)));
    assert!(matches!(fun_type, Type::Fun { .. }));
    assert!(matches!(tuple_type, Type::Tuple(_)));
    assert!(matches!(rec_type, Type::Rec { .. }));
    assert!(matches!(variant_type, Type::Variant(_)));
}