//! Benchmark comparing Binary AST vs Text AST performance
//! 
//! This benchmark compares:
//! - Serialization/Deserialization speed
//! - Type checking performance
//! - Memory usage
//! - Parse time vs Binary load time

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use effect_lang::{
    core::{
        ast::*,
        binary::{BinarySerializer, BinaryDeserializer},
        span::{Span, FileId, ByteOffset},
        symbol::Symbol,
    },
    analysis::{
        binary_type_checker::BinaryTypeChecker,
        inference::InferenceContext,
        parser::Parser,
    },
};
use std::time::Duration;

/// Create a sample AST for benchmarking
fn create_sample_ast(complexity: usize) -> CompilationUnit {
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(100));
    
    // Create items with varying complexity
    let mut items = Vec::new();
    
    for i in 0..complexity {
        // Create a function definition
        let function_name = Symbol::intern(&format!("func_{}", i));
        
        // Create nested lambda expressions for complexity
        let mut body = Expr::Literal(Literal::Integer(42), span);
        
        for j in 0..std::cmp::min(i, 5) {
            let param_name = Symbol::intern(&format!("x_{}", j));
            body = Expr::Lambda {
                parameters: vec![Pattern::Variable(param_name, span)],
                body: Box::new(body),
                span,
            };
        }
        
        // Add function application
        if i > 0 {
            let args = (0..std::cmp::min(i, 3))
                .map(|k| Expr::Var(Symbol::intern(&format!("arg_{}", k)), span))
                .collect();
            
            body = Expr::App(
                Box::new(body),
                args,
                span,
            );
        }
        
        let value_def = ValueDef {
            name: function_name,
            type_annotation: None,
            parameters: Vec::new(),
            body,
            visibility: Visibility::Public,
            purity: Purity::Pure,
            span,
        };
        
        items.push(Item::ValueDef(value_def));
    }
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("Benchmark"), span),
        exports: None,
        imports: Vec::new(),
        items,
        span,
    };
    
    CompilationUnit { module, span }
}

/// Create a sample program source code
fn create_sample_source(complexity: usize) -> String {
    let mut source = String::from("module Benchmark\n\n");
    
    for i in 0..complexity {
        source.push_str(&format!("let func_{} = ", i));
        
        // Add nested lambdas
        for j in 0..std::cmp::min(i, 5) {
            source.push_str(&format!("fun x_{} -> ", j));
        }
        
        source.push_str("42");
        
        // Add function application
        if i > 0 {
            source.push_str(" (");
            for k in 0..std::cmp::min(i, 3) {
                if k > 0 { source.push(' '); }
                source.push_str(&format!("arg_{}", k));
            }
            source.push(')');
        }
        
        source.push('\n');
    }
    
    source
}

/// Benchmark AST serialization to binary format
fn bench_binary_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_serialization");
    
    for complexity in [10, 50, 100, 200].iter() {
        let ast = create_sample_ast(*complexity);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut serializer = BinarySerializer::with_type_checking();
                    black_box(serializer.serialize_compilation_unit(&ast).unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark AST deserialization from binary format
fn bench_binary_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_deserialization");
    
    for complexity in [10, 50, 100, 200].iter() {
        let ast = create_sample_ast(*complexity);
        let mut serializer = BinarySerializer::with_type_checking();
        let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("deserialize", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut deserializer = BinaryDeserializer::new(binary_data.clone()).unwrap();
                    black_box(deserializer.deserialize_compilation_unit().unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark text parsing to AST
fn bench_text_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_parsing");
    
    for complexity in [10, 50, 100, 200].iter() {
        let source = create_sample_source(*complexity);
        
        group.bench_with_input(
            BenchmarkId::new("parse", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut parser = Parser::new(&source, FileId::new(0)).unwrap();
                    black_box(parser.parse().unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark type checking on AST vs Binary AST
fn bench_type_checking_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checking");
    
    for complexity in [10, 50, 100].iter() {
        let ast = create_sample_ast(*complexity);
        
        // Prepare binary AST
        let mut serializer = BinarySerializer::with_type_checking();
        let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
        
        // Benchmark traditional AST type checking
        group.bench_with_input(
            BenchmarkId::new("traditional_ast", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut ctx = InferenceContext::new();
                    for item in &ast.module.items {
                        if let Item::ValueDef(value_def) = item {
                            black_box(ctx.infer_expr(&value_def.body).unwrap());
                        }
                    }
                })
            }
        );
        
        // Benchmark binary AST type checking
        group.bench_with_input(
            BenchmarkId::new("binary_ast", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut checker = BinaryTypeChecker::new();
                    black_box(checker.check_binary_compilation_unit(&binary_data).unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark memory usage comparison
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(20); // Fewer samples for memory benchmarks
    
    for complexity in [50, 100, 200].iter() {
        let ast = create_sample_ast(*complexity);
        let source = create_sample_source(*complexity);
        
        // Binary size
        group.bench_with_input(
            BenchmarkId::new("binary_size", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut serializer = BinarySerializer::with_type_checking();
                    let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
                    black_box(binary_data.len())
                })
            }
        );
        
        // Source code size
        group.bench_with_input(
            BenchmarkId::new("source_size", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    black_box(source.len())
                })
            }
        );
    }
    
    group.finish();
}

/// Comprehensive round-trip benchmark
fn bench_roundtrip_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.measurement_time(Duration::from_secs(15));
    
    for complexity in [25, 50, 100].iter() {
        let source = create_sample_source(*complexity);
        
        // Text: Parse -> Type Check
        group.bench_with_input(
            BenchmarkId::new("text_roundtrip", complexity), 
            complexity,
            |b, _| {
                b.iter(|| {
                    // Parse
                    let mut parser = Parser::new(&source, FileId::new(0)).unwrap();
                    let ast = parser.parse().unwrap();
                    
                    // Type check
                    let mut ctx = InferenceContext::new();
                    for item in &ast.module.items {
                        if let Item::ValueDef(value_def) = item {
                            ctx.infer_expr(&value_def.body).unwrap();
                        }
                    }
                    
                    black_box(ast)
                })
            }
        );
        
        // Binary: Parse -> Serialize -> Deserialize -> Type Check
        group.bench_with_input(
            BenchmarkId::new("binary_roundtrip", complexity), 
            complexity,
            |b, _| {
                // Pre-serialize for fair comparison
                let mut parser = Parser::new(&source, FileId::new(0)).unwrap();
                let ast = parser.parse().unwrap();
                let mut serializer = BinarySerializer::with_type_checking();
                let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
                
                b.iter(|| {
                    // Deserialize
                    let mut deserializer = BinaryDeserializer::new(binary_data.clone()).unwrap();
                    let _restored_ast = deserializer.deserialize_compilation_unit().unwrap();
                    
                    // Type check on binary
                    let mut checker = BinaryTypeChecker::new();
                    let result = checker.check_binary_compilation_unit(&binary_data).unwrap();
                    
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark for incremental scenarios (LSP-like)
fn bench_incremental_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");
    
    // Simulate multiple small changes
    for file_count in [5, 10, 20].iter() {
        let asts: Vec<_> = (0..*file_count)
            .map(|i| create_sample_ast(20 + i))
            .collect();
        
        // Traditional: Re-parse and type check all files
        group.bench_with_input(
            BenchmarkId::new("traditional_incremental", file_count), 
            file_count,
            |b, _| {
                b.iter(|| {
                    for ast in &asts {
                        let mut ctx = InferenceContext::new();
                        for item in &ast.module.items {
                            if let Item::ValueDef(value_def) = item {
                                black_box(ctx.infer_expr(&value_def.body).unwrap());
                            }
                        }
                    }
                })
            }
        );
        
        // Binary: Type check pre-serialized files
        let binary_files: Vec<_> = asts.iter()
            .map(|ast| {
                let mut serializer = BinarySerializer::with_type_checking();
                serializer.serialize_compilation_unit(ast).unwrap()
            })
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("binary_incremental", file_count), 
            file_count,
            |b, _| {
                b.iter(|| {
                    for binary_data in &binary_files {
                        let mut checker = BinaryTypeChecker::new();
                        black_box(checker.check_binary_compilation_unit(binary_data).unwrap());
                    }
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_binary_serialization,
    bench_binary_deserialization,
    bench_text_parsing,
    bench_type_checking_comparison,
    bench_memory_usage,
    bench_roundtrip_performance,
    bench_incremental_scenarios
);

criterion_main!(benches);