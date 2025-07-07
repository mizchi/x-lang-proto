use criterion::{black_box, criterion_group, criterion_main, Criterion};
use x_parser::{parse_source, FileId, SyntaxStyle};

fn benchmark_parse_simple(c: &mut Criterion) {
    let source = r#"
module Benchmark

let add = fun x y -> x + y
let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            let file_id = FileId::new(0);
            parse_source(black_box(source), file_id, SyntaxStyle::Haskell)
        })
    });
}

fn benchmark_parse_complex(c: &mut Criterion) {
    let source = r#"
module ComplexBenchmark

type Tree a =
  | Leaf
  | Node a (Tree a) (Tree a)

let rec map_tree = fun f tree ->
  match tree with
  | Leaf -> Leaf
  | Node v left right ->
    Node (f v) (map_tree f left) (map_tree f right)

let rec fold_tree = fun f acc tree ->
  match tree with
  | Leaf -> acc
  | Node v left right ->
    let acc' = f acc v in
    let acc'' = fold_tree f acc' left in
    fold_tree f acc'' right

effect State {
  get : () -> int
  put : int -> ()
}

let stateful_computation = fun () ->
  let x = perform State.get () in
  perform State.put (x + 1);
  let y = perform State.get () in
  perform State.put (y * 2);
  perform State.get ()
"#;

    c.bench_function("parse_complex", |b| {
        b.iter(|| {
            let file_id = FileId::new(0);
            parse_source(black_box(source), file_id, SyntaxStyle::Haskell)
        })
    });
}

fn benchmark_syntax_styles(c: &mut Criterion) {
    let ocaml_source = "module Test\n\nlet main = fun () -> 42";
    
    c.bench_function("parse_ocaml_style", |b| {
        b.iter(|| {
            let file_id = FileId::new(0);
            parse_source(black_box(ocaml_source), file_id, SyntaxStyle::Haskell)
        })
    });
}

criterion_group!(benches, benchmark_parse_simple, benchmark_parse_complex, benchmark_syntax_styles);
criterion_main!(benches);