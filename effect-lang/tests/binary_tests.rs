//! Binary serialization tests for EffectLang

use effect_lang::{
    analysis::parser::parse,
    core::{
        ast::*,
        binary::{BinarySerializer, BinaryDeserializer},
        span::FileId,
        symbol::Symbol,
    },
};

fn roundtrip_test(input: &str) -> CompilationUnit {
    let file_id = FileId::new(0);
    let original = parse(input, file_id).expect("Should parse successfully");
    
    let mut serializer = BinarySerializer::new();
    let binary_data = serializer.serialize_compilation_unit(&original)
        .expect("Should serialize successfully");
    
    let mut deserializer = BinaryDeserializer::new(binary_data);
    let deserialized = deserializer.deserialize_compilation_unit()
        .expect("Should deserialize successfully");
    
    deserialized
}

#[test]
fn test_simple_module_roundtrip() {
    let input = r#"
module Test
let x = 42
"#;
    let cu = roundtrip_test(input);
    assert_eq!(cu.module.name.segments[0], Symbol::intern("Test"));
    assert_eq!(cu.module.items.len(), 1);
    
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("x"));
            match &def.body {
                Expr::Literal(Literal::Integer(42), _) => {},
                _ => panic!("Expected integer literal 42"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_function_roundtrip() {
    let input = r#"
module Test
let add = fun x y -> x + y
"#;
    let cu = roundtrip_test(input);
    
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("add"));
            match &def.body {
                Expr::Lambda { params, .. } => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[0].name, Symbol::intern("x"));
                    assert_eq!(params[1].name, Symbol::intern("y"));
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_literals_roundtrip() {
    let input = r#"
module Test
let int_val = 42
let float_val = 3.14
let string_val = "hello"
let bool_val = true
let unit_val = ()
"#;
    let cu = roundtrip_test(input);
    assert_eq!(cu.module.items.len(), 5);
    
    // Check each literal type
    let items = &cu.module.items;
    
    match &items[0] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Literal(Literal::Integer(42), _) => {},
                _ => panic!("Expected integer 42"),
            }
        }
        _ => panic!("Expected value def"),
    }
    
    match &items[1] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Literal(Literal::Float(f), _) if (*f - 3.14).abs() < f64::EPSILON => {},
                _ => panic!("Expected float 3.14"),
            }
        }
        _ => panic!("Expected value def"),
    }
    
    match &items[2] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Literal(Literal::String(s), _) if s == "hello" => {},
                _ => panic!("Expected string 'hello'"),
            }
        }
        _ => panic!("Expected value def"),
    }
    
    match &items[3] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Literal(Literal::Bool(true), _) => {},
                _ => panic!("Expected bool true"),
            }
        }
        _ => panic!("Expected value def"),
    }
    
    match &items[4] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Literal(Literal::Unit, _) => {},
                _ => panic!("Expected unit literal"),
            }
        }
        _ => panic!("Expected value def"),
    }
}

#[test]
fn test_complex_expression_roundtrip() {
    let input = r#"
module Test
let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;
    let cu = roundtrip_test(input);
    
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("factorial"));
            match &def.body {
                Expr::Lambda { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    
                    // Check that the if expression was preserved
                    match body.as_ref() {
                        Expr::If { .. } => {},
                        _ => panic!("Expected if expression"),
                    }
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_type_annotations_roundtrip() {
    let input = r#"
module Test
let x : Int = 42
let f : Int -> Int = fun x -> x + 1
"#;
    let cu = roundtrip_test(input);
    assert_eq!(cu.module.items.len(), 2);
    
    // Check type annotations are preserved
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert!(def.type_annotation.is_some());
            match def.type_annotation.as_ref().unwrap() {
                Type::Con(sym, _) => {
                    assert_eq!(*sym, Symbol::intern("Int"));
                }
                _ => panic!("Expected Int type constructor"),
            }
        }
        _ => panic!("Expected value definition"),
    }
    
    match &cu.module.items[1] {
        Item::ValueDef(def) => {
            assert!(def.type_annotation.is_some());
            match def.type_annotation.as_ref().unwrap() {
                Type::Fun { .. } => {},
                _ => panic!("Expected function type"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_imports_roundtrip() {
    let input = r#"
module Test
import Core.List
import Data.String as Str
let x = 42
"#;
    let cu = roundtrip_test(input);
    assert_eq!(cu.module.imports.len(), 2);
    
    // Check imports are preserved
    match &cu.module.imports[0] {
        Import::Simple { module_path, .. } => {
            assert_eq!(module_path.segments.len(), 2);
            assert_eq!(module_path.segments[0], Symbol::intern("Core"));
            assert_eq!(module_path.segments[1], Symbol::intern("List"));
        }
        _ => panic!("Expected simple import"),
    }
}

#[test]
fn test_binary_format_stability() {
    let input = r#"
module Test
let x = 42
"#;
    
    let file_id = FileId::new(0);
    let ast = parse(input, file_id).expect("Should parse");
    
    let mut serializer1 = BinarySerializer::new();
    let binary1 = serializer1.serialize_compilation_unit(&ast)
        .expect("Should serialize");
    
    let mut serializer2 = BinarySerializer::new();
    let binary2 = serializer2.serialize_compilation_unit(&ast)
        .expect("Should serialize");
    
    // Same AST should produce identical binary output
    assert_eq!(binary1, binary2);
}

#[test]
fn test_content_hashing() {
    let input1 = r#"
module Test
let x = 42
"#;
    
    let input2 = r#"
module Test
let x = 42
"#;
    
    let input3 = r#"
module Test
let x = 43
"#;
    
    let file_id = FileId::new(0);
    
    let ast1 = parse(input1, file_id).expect("Should parse");
    let ast2 = parse(input2, file_id).expect("Should parse");
    let ast3 = parse(input3, file_id).expect("Should parse");
    
    let mut serializer = BinarySerializer::new();
    
    let binary1 = serializer.serialize_compilation_unit(&ast1).expect("Should serialize");
    let binary2 = serializer.serialize_compilation_unit(&ast2).expect("Should serialize");
    let binary3 = serializer.serialize_compilation_unit(&ast3).expect("Should serialize");
    
    let hash1 = BinarySerializer::content_hash(&binary1);
    let hash2 = BinarySerializer::content_hash(&binary2);
    let hash3 = BinarySerializer::content_hash(&binary3);
    
    // Same content should have same hash
    assert_eq!(hash1, hash2);
    
    // Different content should have different hash
    assert_ne!(hash1, hash3);
}

#[test]
fn test_large_ast_performance() {
    // Create a moderately large AST
    let mut input = String::from("module LargeTest\n");
    for i in 0..100 {
        input.push_str(&format!("let var{} = {}\n", i, i));
    }
    
    let file_id = FileId::new(0);
    let ast = parse(&input, file_id).expect("Should parse large AST");
    
    // Measure serialization
    let start = std::time::Instant::now();
    let mut serializer = BinarySerializer::new();
    let binary_data = serializer.serialize_compilation_unit(&ast)
        .expect("Should serialize large AST");
    let serialize_time = start.elapsed();
    
    // Measure deserialization
    let start = std::time::Instant::now();
    let mut deserializer = BinaryDeserializer::new(binary_data);
    let _deserialized = deserializer.deserialize_compilation_unit()
        .expect("Should deserialize large AST");
    let deserialize_time = start.elapsed();
    
    // Should complete in reasonable time (< 100ms for this size)
    assert!(serialize_time.as_millis() < 100);
    assert!(deserialize_time.as_millis() < 100);
}

#[test]
fn test_compression_ratio() {
    let input = r#"
module Test
let fibonacci = fun n ->
  if n <= 1 then n
  else fibonacci (n - 1) + fibonacci (n - 2)

let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)

let map = fun f xs -> match xs with
  | [] -> []
  | x :: rest -> f x :: map f rest

let filter = fun pred xs -> match xs with
  | [] -> []
  | x :: rest -> if pred x then x :: filter pred rest else filter pred rest
"#;
    
    let file_id = FileId::new(0);
    let ast = parse(input, file_id).expect("Should parse");
    
    let mut serializer = BinarySerializer::new();
    let binary_data = serializer.serialize_compilation_unit(&ast)
        .expect("Should serialize");
    
    let source_size = input.len();
    let binary_size = binary_data.len();
    let compression_ratio = binary_size as f64 / source_size as f64;
    
    println!("Source size: {} bytes", source_size);
    println!("Binary size: {} bytes", binary_size);
    println!("Compression ratio: {:.2}x", compression_ratio);
    
    // Binary should be reasonably compact compared to source
    // (exact ratio depends on implementation, but should be competitive)
    assert!(binary_size > 0);
    assert!(compression_ratio < 10.0); // Binary shouldn't be 10x larger than source
}