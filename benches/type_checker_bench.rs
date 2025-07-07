use criterion::{black_box, criterion_group, criterion_main, Criterion};
use x_parser::{parse_source, FileId, SyntaxStyle};
use x_checker::type_check;

fn benchmark_type_check_simple(c: &mut Criterion) {
    let source = r#"
module Benchmark

let add = fun x y -> x + y
let main = fun () -> add 5 3
"#;

    c.bench_function("type_check_simple", |b| {
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::Haskell).unwrap();
        
        b.iter(|| {
            type_check(black_box(&cu))
        })
    });
}

fn benchmark_type_check_recursive(c: &mut Criterion) {
    let source = r#"
module RecursiveBenchmark

let rec factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)

let rec fibonacci = fun n ->
  if n <= 1 then n
  else fibonacci (n - 1) + fibonacci (n - 2)

let main = fun () ->
  factorial 10 + fibonacci 10
"#;

    c.bench_function("type_check_recursive", |b| {
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::Haskell).unwrap();
        
        b.iter(|| {
            type_check(black_box(&cu))
        })
    });
}

fn benchmark_type_check_polymorphic(c: &mut Criterion) {
    let source = r#"
module PolymorphicBenchmark

let id = fun x -> x

let compose = fun f g x -> f (g x)

let map = fun f lst ->
  match lst with
  | [] -> []
  | h :: t -> f h :: map f t

let main = fun () ->
  let inc = fun x -> x + 1 in
  let double = fun x -> x * 2 in
  let inc_then_double = compose double inc in
  map inc_then_double [1; 2; 3; 4; 5]
"#;

    c.bench_function("type_check_polymorphic", |b| {
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::Haskell).unwrap();
        
        b.iter(|| {
            type_check(black_box(&cu))
        })
    });
}

criterion_group!(benches, benchmark_type_check_simple, benchmark_type_check_recursive, benchmark_type_check_polymorphic);
criterion_main!(benches);