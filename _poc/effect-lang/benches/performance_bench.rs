//! Performance benchmarks for x Language components

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use effect_lang::{
    analysis::{lexer::Lexer, parser::parse},
    core::{
        binary::{BinarySerializer, BinaryDeserializer},
        diff::BinaryAstDiffer,
        span::FileId,
        symbol::Symbol,
    },
};

fn create_test_program(size: usize) -> String {
    let mut program = String::from("module BenchTest\n");
    
    for i in 0..size {
        program.push_str(&format!("let var{} = fun x{} -> x{} + {}\n", i, i, i, i));
    }
    
    program
}

fn benchmark_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    for size in [10, 50, 100, 500].iter() {
        let program = create_test_program(*size);
        
        group.bench_with_input(
            BenchmarkId::new("tokenize", size),
            &program,
            |b, program| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(program), FileId::new(0));
                    black_box(lexer.tokenize().unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    for size in [10, 50, 100].iter() {
        let program = create_test_program(*size);
        
        group.bench_with_input(
            BenchmarkId::new("parse", size),
            &program,
            |b, program| {
                b.iter(|| {
                    black_box(parse(black_box(program), FileId::new(0)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    for size in [10, 50, 100].iter() {
        let program = create_test_program(*size);
        let ast = parse(&program, FileId::new(0)).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut serializer = BinarySerializer::new();
                    black_box(serializer.serialize_compilation_unit(black_box(ast)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    
    for size in [10, 50, 100].iter() {
        let program = create_test_program(*size);
        let ast = parse(&program, FileId::new(0)).unwrap();
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("deserialize", size),
            &binary_data,
            |b, binary_data| {
                b.iter(|| {
                    let mut deserializer = BinaryDeserializer::new(black_box(binary_data.clone()));
                    black_box(deserializer.deserialize_compilation_unit().unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");
    
    for size in [10, 25, 50].iter() {
        let program1 = create_test_program(*size);
        let mut program2 = create_test_program(*size);
        // Make a small change
        program2.push_str("let extra = 999\n");
        
        let ast1 = parse(&program1, FileId::new(0)).unwrap();
        let ast2 = parse(&program2, FileId::new(0)).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("diff_asts", size),
            &(ast1, ast2),
            |b, (ast1, ast2)| {
                b.iter(|| {
                    let mut differ = BinaryAstDiffer::new();
                    black_box(differ.diff_compilation_units(black_box(ast1), black_box(ast2)).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_symbol_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbols");
    
    // Benchmark symbol interning
    group.bench_function("intern_new", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            black_box(Symbol::intern(&format!("symbol_{}", counter)))
        })
    });
    
    // Benchmark symbol lookup (existing symbols)
    let test_symbol = Symbol::intern("test_symbol");
    group.bench_function("intern_existing", |b| {
        b.iter(|| {
            black_box(Symbol::intern("test_symbol"))
        })
    });
    
    group.finish();
}

fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    
    for size in [10, 25, 50].iter() {
        let program = create_test_program(*size);
        
        group.bench_with_input(
            BenchmarkId::new("full_pipeline", size),
            &program,
            |b, program| {
                b.iter(|| {
                    // Full pipeline: parse -> serialize -> deserialize
                    let ast = parse(black_box(program), FileId::new(0)).unwrap();
                    let mut serializer = BinarySerializer::new();
                    let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
                    let mut deserializer = BinaryDeserializer::new(binary_data);
                    black_box(deserializer.deserialize_compilation_unit().unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_content_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_hash");
    
    for size in [10, 50, 100, 500].iter() {
        let program = create_test_program(*size);
        let ast = parse(&program, FileId::new(0)).unwrap();
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("hash", size),
            &binary_data,
            |b, binary_data| {
                b.iter(|| {
                    black_box(BinarySerializer::content_hash(black_box(binary_data)))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_realistic_programs(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic");
    
    let programs = vec![
        ("simple", r#"
module Simple
let id = fun x -> x
let add = fun x y -> x + y
"#),
        ("fibonacci", r#"
module Fibonacci
let fib = fun n ->
  if n <= 1 then n
  else fib (n - 1) + fib (n - 2)

let fib_list = fun n ->
  let rec loop = fun i acc ->
    if i > n then acc
    else loop (i + 1) (fib i :: acc)
  in loop 0 []
"#),
        ("data_structures", r#"
module DataStructures
type List a = Nil | Cons a (List a)

let map = fun f xs -> match xs with
  | Nil -> Nil
  | Cons x rest -> Cons (f x) (map f rest)

let filter = fun pred xs -> match xs with
  | Nil -> Nil
  | Cons x rest -> if pred x then Cons x (filter pred rest) else filter pred rest

let fold = fun f acc xs -> match xs with
  | Nil -> acc
  | Cons x rest -> fold f (f acc x) rest

type Tree a = Leaf | Node a (Tree a) (Tree a)

let tree_map = fun f tree -> match tree with
  | Leaf -> Leaf
  | Node value left right -> Node (f value) (tree_map f left) (tree_map f right)
"#),
    ];
    
    for (name, program) in programs {
        group.bench_function(&format!("parse_{}", name), |b| {
            b.iter(|| {
                black_box(parse(black_box(program), FileId::new(0)).unwrap())
            })
        });
        
        let ast = parse(program, FileId::new(0)).unwrap();
        group.bench_function(&format!("serialize_{}", name), |b| {
            b.iter(|| {
                let mut serializer = BinarySerializer::new();
                black_box(serializer.serialize_compilation_unit(black_box(&ast)).unwrap())
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_lexer,
    benchmark_parser,
    benchmark_serialization,
    benchmark_deserialization,
    benchmark_diff,
    benchmark_symbol_interning,
    benchmark_end_to_end,
    benchmark_content_hashing,
    benchmark_realistic_programs,
);

criterion_main!(benches);