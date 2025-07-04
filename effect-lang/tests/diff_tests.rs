//! Binary AST diff tests for x Language

use effect_lang::{
    analysis::parser::parse,
    core::{
        binary::{BinarySerializer, BinaryDeserializer},
        diff::{BinaryAstDiffer, DiffOp, format_diff},
        span::FileId,
    },
};

fn parse_and_serialize(input: &str) -> Vec<u8> {
    let file_id = FileId::new(0);
    let ast = parse(input, file_id).expect("Should parse");
    let mut serializer = BinarySerializer::new();
    serializer.serialize_compilation_unit(&ast).expect("Should serialize")
}

fn deserialize_and_diff(binary1: Vec<u8>, binary2: Vec<u8>) -> Vec<DiffOp> {
    let mut deserializer1 = BinaryDeserializer::new(binary1);
    let mut deserializer2 = BinaryDeserializer::new(binary2);
    
    let ast1 = deserializer1.deserialize_compilation_unit().expect("Should deserialize");
    let ast2 = deserializer2.deserialize_compilation_unit().expect("Should deserialize");
    
    let mut differ = BinaryAstDiffer::new();
    differ.diff_compilation_units(&ast1, &ast2).expect("Should diff")
}

#[test]
fn test_identical_modules_diff() {
    let input = r#"
module Test
let x = 42
"#;
    
    let binary1 = parse_and_serialize(input);
    let binary2 = parse_and_serialize(input);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Identical modules should produce Equal diff
    assert_eq!(diff_ops.len(), 1);
    match &diff_ops[0] {
        DiffOp::Equal { .. } => {},
        _ => panic!("Expected Equal diff for identical modules"),
    }
}

#[test]
fn test_simple_value_change() {
    let input1 = r#"
module Test
let x = 42
"#;
    
    let input2 = r#"
module Test
let x = 43
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect the change in value
    assert!(diff_ops.len() >= 1);
    
    // Should contain a Replace operation
    let has_replace = diff_ops.iter().any(|op| {
        matches!(op, DiffOp::Replace { .. })
    });
    assert!(has_replace, "Should contain a Replace operation");
}

#[test]
fn test_function_name_change() {
    let input1 = r#"
module Test
let add = fun x y -> x + y
"#;
    
    let input2 = r#"
module Test
let plus = fun x y -> x + y
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect function name change
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect the function name change");
}

#[test]
fn test_function_body_change() {
    let input1 = r#"
module Test
let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;
    
    let input2 = r#"
module Test
let factorial = fun n ->
  if n <= 0 then 1
  else n * factorial (n - 1)
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect the condition change (n <= 1 vs n <= 0)
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect the condition change");
}

#[test]
fn test_added_function() {
    let input1 = r#"
module Test
let x = 42
"#;
    
    let input2 = r#"
module Test
let x = 42
let y = 43
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect the addition
    assert!(diff_ops.len() >= 1);
    
    // For now, our diff might show this as a Replace operation
    // A more sophisticated diff would show Insert
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect the addition");
}

#[test]
fn test_type_annotation_change() {
    let input1 = r#"
module Test
let x : Int = 42
"#;
    
    let input2 = r#"
module Test
let x : Float = 42.0
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect type annotation and value changes
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect type and value changes");
}

#[test]
fn test_import_changes() {
    let input1 = r#"
module Test
import Core.List
let x = 42
"#;
    
    let input2 = r#"
module Test
import Core.List
import Data.String
let x = 42
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect import addition
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect import addition");
}

#[test]
fn test_complex_structural_change() {
    let input1 = r#"
module Test
let process = fun xs ->
  map (fun x -> x * 2) xs
"#;
    
    let input2 = r#"
module Test
let process = fun xs ->
  filter (fun x -> x > 0) (map (fun x -> x * 2) xs)
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect the structural change (wrapping in filter)
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect structural change");
}

#[test]
fn test_diff_formatting() {
    let input1 = r#"
module Test
let x = 42
"#;
    
    let input2 = r#"
module Test
let x = 43
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    let formatted = format_diff(&diff_ops);
    
    // Should produce readable diff output
    assert!(!formatted.is_empty());
    println!("Formatted diff:\n{}", formatted);
}

#[test]
fn test_no_spurious_differences() {
    // Same logical content but with different whitespace/formatting
    let input1 = r#"module Test
let x=42"#;
    
    let input2 = r#"
module Test
let x = 42
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should be equal since the AST structure is the same
    assert_eq!(diff_ops.len(), 1);
    match &diff_ops[0] {
        DiffOp::Equal { .. } => {},
        _ => panic!("Whitespace changes should not affect binary AST comparison"),
    }
}

#[test]
fn test_diff_performance() {
    // Create moderately complex programs
    let mut input1 = String::from("module Test1\n");
    let mut input2 = String::from("module Test2\n");
    
    for i in 0..50 {
        input1.push_str(&format!("let var{} = {}\n", i, i));
        input2.push_str(&format!("let var{} = {}\n", i, i + 1)); // Slight difference
    }
    
    let start = std::time::Instant::now();
    
    let binary1 = parse_and_serialize(&input1);
    let binary2 = parse_and_serialize(&input2);
    let _diff_ops = deserialize_and_diff(binary1, binary2);
    
    let diff_time = start.elapsed();
    
    // Should complete in reasonable time
    assert!(diff_time.as_millis() < 1000); // Less than 1 second
    
    println!("Diff time for 50 functions: {:?}", diff_time);
}

#[test]
fn test_empty_vs_non_empty() {
    let input1 = r#"
module Empty
"#;
    
    let input2 = r#"
module Test
let x = 42
"#;
    
    let binary1 = parse_and_serialize(input1);
    let binary2 = parse_and_serialize(input2);
    let diff_ops = deserialize_and_diff(binary1, binary2);
    
    // Should detect the difference between empty and non-empty modules
    assert!(diff_ops.len() >= 1);
    
    let has_change = diff_ops.iter().any(|op| {
        !matches!(op, DiffOp::Equal { .. })
    });
    assert!(has_change, "Should detect difference between empty and non-empty");
}

#[test]
fn test_hash_based_optimization() {
    let input = r#"
module Test
let x = 42
"#;
    
    let binary1 = parse_and_serialize(input);
    let binary2 = parse_and_serialize(input);
    
    // Same content should have same hash
    let hash1 = BinarySerializer::content_hash(&binary1);
    let hash2 = BinarySerializer::content_hash(&binary2);
    assert_eq!(hash1, hash2);
    
    // If hashes are equal, we could skip detailed diffing
    // This test verifies the hash-based optimization potential
    if hash1 == hash2 {
        let diff_ops = deserialize_and_diff(binary1, binary2);
        match &diff_ops[0] {
            DiffOp::Equal { .. } => {},
            _ => panic!("Equal hashes should imply equal content"),
        }
    }
}