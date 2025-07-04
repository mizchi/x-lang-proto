//! Tests for visibility modifiers and WebAssembly Component Model features

use effect_lang::{
    MultiSyntax, SyntaxStyle, SyntaxConfig,
    core::{span::FileId, symbol::Symbol, ast::*},
};

#[test]
fn test_pub_visibility() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        pub let public_value = 42
        let private_value = 24
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse pub visibility");
    
    let cu = result.unwrap();
    assert_eq!(cu.module.items.len(), 2);
    
    // Check first item (public)
    if let Item::ValueDef(def) = &cu.module.items[0] {
        assert_eq!(def.name.as_str(), "public_value");
        assert!(matches!(def.visibility, Visibility::Public));
    } else {
        panic!("Expected ValueDef");
    }
    
    // Check second item (private)
    if let Item::ValueDef(def) = &cu.module.items[1] {
        assert_eq!(def.name.as_str(), "private_value");
        assert!(matches!(def.visibility, Visibility::Private));
    } else {
        panic!("Expected ValueDef");
    }
}

#[test]
fn test_pub_crate_visibility() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        pub(crate) type CrateType = Int
        pub(package) let package_value = 42
        pub(super) effect SuperEffect { get : Int }
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse pub(scope) visibility");
    
    let cu = result.unwrap();
    assert_eq!(cu.module.items.len(), 3);
    
    // Check crate visibility
    if let Item::TypeDef(def) = &cu.module.items[0] {
        assert!(matches!(def.visibility, Visibility::Crate));
    }
    
    // Check package visibility  
    if let Item::ValueDef(def) = &cu.module.items[1] {
        assert!(matches!(def.visibility, Visibility::Package));
    }
    
    // Check super visibility
    if let Item::EffectDef(def) = &cu.module.items[2] {
        assert!(matches!(def.visibility, Visibility::Super));
    }
}

#[test]
fn test_pub_in_path_visibility() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        pub(in Core.Types) let shared_value = 42
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse pub(in path) visibility");
    
    let cu = result.unwrap();
    assert_eq!(cu.module.items.len(), 1);
    
    if let Item::ValueDef(def) = &cu.module.items[0] {
        if let Visibility::InPath(path) = &def.visibility {
            assert_eq!(path.to_string(), "Core.Types");
        } else {
            panic!("Expected InPath visibility");
        }
    }
}

#[test]
fn test_interface_definition() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        interface "wasi:io/poll@0.2.0" {
            func poll-one (param i32) (result i32)
            type descriptor = i32
            resource stream {
                constructor create (param i32)
                static get-size (param i32) (result i64)
                read (param i32) (result i32)
            }
        }
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse interface definition");
    
    let cu = result.unwrap();
    assert_eq!(cu.module.items.len(), 1);
    
    if let Item::InterfaceDef(interface) = &cu.module.items[0] {
        assert_eq!(interface.name, "wasi:io/poll@0.2.0");
        assert_eq!(interface.items.len(), 3);
        
        // Check function item
        if let InterfaceItem::Func { name, signature, .. } = &interface.items[0] {
            assert_eq!(name.as_str(), "poll-one");
            assert_eq!(signature.params.len(), 1);
            assert_eq!(signature.results.len(), 1);
        }
        
        // Check type item
        if let InterfaceItem::Type { name, definition, .. } = &interface.items[1] {
            assert_eq!(name.as_str(), "descriptor");
            assert!(definition.is_some());
        }
        
        // Check resource item
        if let InterfaceItem::Resource { name, methods, .. } = &interface.items[2] {
            assert_eq!(name.as_str(), "stream");
            assert_eq!(methods.len(), 3);
            
            // Check constructor
            assert!(methods[0].is_constructor);
            assert_eq!(methods[0].name.as_str(), "create");
            
            // Check static method
            assert!(methods[1].is_static);
            assert_eq!(methods[1].name.as_str(), "get-size");
            
            // Check instance method
            assert!(!methods[2].is_constructor && !methods[2].is_static);
            assert_eq!(methods[2].name.as_str(), "read");
        }
    }
}

#[test]
fn test_wasm_types() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        interface "test:types" {
            func test-types (param i32 i64 f32 f64 funcref externref) (result v128)
        }
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse WebAssembly types");
    
    let cu = result.unwrap();
    if let Item::InterfaceDef(interface) = &cu.module.items[0] {
        if let InterfaceItem::Func { signature, .. } = &interface.items[0] {
            assert_eq!(signature.params.len(), 6);
            assert_eq!(signature.results.len(), 1);
            
            // Check all WebAssembly types are correctly parsed
            assert!(matches!(signature.params[0], WasmType::I32));
            assert!(matches!(signature.params[1], WasmType::I64));
            assert!(matches!(signature.params[2], WasmType::F32));
            assert!(matches!(signature.params[3], WasmType::F64));
            assert!(matches!(signature.params[4], WasmType::FuncRef));
            assert!(matches!(signature.params[5], WasmType::ExternRef));
            assert!(matches!(signature.results[0], WasmType::V128));
        }
    }
}

#[test]
fn test_visibility_printing() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        pub let public_func = 42
        pub(crate) type CrateType = String
        pub(super) effect SuperEffect { get : Int }
    "#;
    
    // Parse
    let cu = multi.parse(input, SyntaxStyle::OCaml, file_id).unwrap();
    
    // Print back
    let config = SyntaxConfig { style: SyntaxStyle::OCaml, ..Default::default() };
    let printed = multi.print(&cu, &config).unwrap();
    
    // Should contain visibility modifiers
    assert!(printed.contains("pub let"));
    assert!(printed.contains("pub(crate) type"));
    assert!(printed.contains("pub(super) effect"));
}

#[test]
fn test_interface_printing() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        interface "wasi:filesystem@0.2.0" {
            func open (param i32) (result i32)
            resource file {
                read (param i32) (result i32)
                write (param i32 i32) (result i32)
            }
        }
    "#;
    
    // Parse and print back
    let cu = multi.parse(input, SyntaxStyle::OCaml, file_id).unwrap();
    let config = SyntaxConfig { style: SyntaxStyle::OCaml, ..Default::default() };
    let printed = multi.print(&cu, &config).unwrap();
    
    // Should contain interface syntax
    assert!(printed.contains("interface \"wasi:filesystem@0.2.0\""));
    assert!(printed.contains("func open"));
    assert!(printed.contains("resource file"));
    assert!(printed.contains("(param i32)"));
    assert!(printed.contains("(result i32)"));
}

#[test]
fn test_mixed_visibility_and_interfaces() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    let input = r#"
        module Test
        
        pub interface "public:api@1.0" {
            func get-version (result i32)
        }
        
        interface "internal:api@1.0" {
            func debug-info (result i32)
        }
        
        pub(crate) let internal_helper = 42
    "#;
    
    let result = multi.parse(input, SyntaxStyle::OCaml, file_id);
    assert!(result.is_ok(), "Should parse mixed visibility and interfaces");
    
    let cu = result.unwrap();
    assert_eq!(cu.module.items.len(), 3);
    
    // First interface should be public (currently interfaces don't use visibility in our simple implementation)
    // Second interface should be private
    // Value should be pub(crate)
    if let Item::ValueDef(def) = &cu.module.items[2] {
        assert!(matches!(def.visibility, Visibility::Crate));
    }
}