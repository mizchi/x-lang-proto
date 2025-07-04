//! Basic usage examples for the S-expression diff library

use sexp_diff::{
    parser::parse,
    serializer::{serialize, deserialize},
    diff::{StructuralDiff, DiffFormatter, DiffSummary},
    hash::ContentHash,
    sexp::{SExp, Atom},
    Result,
};

fn main() -> Result<()> {
    // Example 1: Parsing S-expressions
    println!("=== Example 1: Parsing ===");
    let input = "(defun factorial (n) (if (= n 0) 1 (* n (factorial (- n 1)))))";
    let sexp = parse(input)?;
    println!("Parsed: {}", sexp);
    println!("Pretty printed:\n{}", sexp.to_pretty_string(0));
    println!();

    // Example 2: Binary serialization
    println!("=== Example 2: Serialization ===");
    let binary = serialize(&sexp)?;
    println!("Binary size: {} bytes", binary.len());
    println!("Binary (hex): {}", hex::encode(&binary[..binary.len().min(32)]));
    
    let deserialized = deserialize(&binary)?;
    println!("Round-trip successful: {}", sexp == deserialized);
    println!();

    // Example 3: Content hashing
    println!("=== Example 3: Content Hashing ===");
    let hash = ContentHash::hash(&sexp);
    let short_hash = ContentHash::short_hash(&sexp);
    println!("Content hash: {}", short_hash);
    println!("Full hash: {}", hash);
    println!();

    // Example 4: Structural diff
    println!("=== Example 4: Structural Diff ===");
    let sexp1 = parse("(defun factorial (n) (if (= n 0) 1 (* n (factorial (- n 1)))))")?;
    let sexp2 = parse("(defun factorial (n) (if (<= n 1) 1 (* n (factorial (- n 1)))))")?;
    
    let diff_engine = StructuralDiff::new();
    let results = diff_engine.diff(&sexp1, &sexp2);
    
    let formatter = DiffFormatter::new();
    let output = formatter.format(&results);
    println!("Diff results:\n{}", output);
    
    let summary = DiffSummary::from_results(&results);
    println!("{}", summary);
    println!();

    // Example 5: Complex nested structures
    println!("=== Example 5: Complex Structures ===");
    let complex_sexp = SExp::List(vec![
        SExp::Symbol("module".to_string()),
        SExp::Symbol("math".to_string()),
        SExp::List(vec![
            SExp::Symbol("export".to_string()),
            SExp::Symbol("factorial".to_string()),
            SExp::Symbol("fibonacci".to_string()),
        ]),
        SExp::List(vec![
            SExp::Symbol("defstruct".to_string()),
            SExp::Symbol("point".to_string()),
            SExp::List(vec![
                SExp::Symbol("x".to_string()),
                SExp::Atom(Atom::Float(0.0)),
            ]),
            SExp::List(vec![
                SExp::Symbol("y".to_string()),
                SExp::Atom(Atom::Float(0.0)),
            ]),
        ]),
    ]);
    
    println!("Complex structure:");
    println!("{}", complex_sexp.to_pretty_string(0));
    
    let complex_binary = serialize(&complex_sexp)?;
    let complex_hash = ContentHash::short_hash(&complex_sexp);
    println!("Binary size: {} bytes", complex_binary.len());
    println!("Content hash: {}", complex_hash);
    println!();

    // Example 6: Performance comparison
    println!("=== Example 6: Performance Test ===");
    let iterations = 1000;
    
    // Parse performance
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = parse(input)?;
    }
    let parse_time = start.elapsed();
    println!("Parse {} iterations: {:?} ({:.2} µs/op)", 
             iterations, parse_time, parse_time.as_micros() as f64 / iterations as f64);
    
    // Serialize performance
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = serialize(&sexp)?;
    }
    let serialize_time = start.elapsed();
    println!("Serialize {} iterations: {:?} ({:.2} µs/op)", 
             iterations, serialize_time, serialize_time.as_micros() as f64 / iterations as f64);
    
    // Hash performance
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = ContentHash::hash(&sexp);
    }
    let hash_time = start.elapsed();
    println!("Hash {} iterations: {:?} ({:.2} µs/op)", 
             iterations, hash_time, hash_time.as_micros() as f64 / iterations as f64);

    Ok(())
}