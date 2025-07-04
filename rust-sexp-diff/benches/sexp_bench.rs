use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sexp_diff::{
    parser::parse,
    serializer::{serialize, deserialize},
    diff::StructuralDiff,
    hash::ContentHash,
    sexp::{SExp, Atom},
};

fn create_test_expressions() -> Vec<(&'static str, SExp)> {
    vec![
        (
            "simple_atom",
            SExp::Atom(Atom::Integer(42)),
        ),
        (
            "simple_list",
            SExp::List(vec![
                SExp::Symbol("+".to_string()),
                SExp::Atom(Atom::Integer(1)),
                SExp::Atom(Atom::Integer(2)),
            ]),
        ),
        (
            "factorial",
            SExp::List(vec![
                SExp::Symbol("defun".to_string()),
                SExp::Symbol("factorial".to_string()),
                SExp::List(vec![SExp::Symbol("n".to_string())]),
                SExp::List(vec![
                    SExp::Symbol("if".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("=".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::Atom(Atom::Integer(0)),
                    ]),
                    SExp::Atom(Atom::Integer(1)),
                    SExp::List(vec![
                        SExp::Symbol("*".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("factorial".to_string()),
                            SExp::List(vec![
                                SExp::Symbol("-".to_string()),
                                SExp::Symbol("n".to_string()),
                                SExp::Atom(Atom::Integer(1)),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ),
        (
            "complex_module",
            SExp::List(vec![
                SExp::Symbol("module".to_string()),
                SExp::Symbol("math".to_string()),
                SExp::List(vec![
                    SExp::Symbol("export".to_string()),
                    SExp::Symbol("factorial".to_string()),
                    SExp::Symbol("fibonacci".to_string()),
                ]),
                SExp::List(vec![
                    SExp::Symbol("defun".to_string()),
                    SExp::Symbol("factorial".to_string()),
                    SExp::List(vec![SExp::Symbol("n".to_string())]),
                    SExp::List(vec![
                        SExp::Symbol("if".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("=".to_string()),
                            SExp::Symbol("n".to_string()),
                            SExp::Atom(Atom::Integer(0)),
                        ]),
                        SExp::Atom(Atom::Integer(1)),
                        SExp::List(vec![
                            SExp::Symbol("*".to_string()),
                            SExp::Symbol("n".to_string()),
                            SExp::List(vec![
                                SExp::Symbol("factorial".to_string()),
                                SExp::List(vec![
                                    SExp::Symbol("-".to_string()),
                                    SExp::Symbol("n".to_string()),
                                    SExp::Atom(Atom::Integer(1)),
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
                SExp::List(vec![
                    SExp::Symbol("defun".to_string()),
                    SExp::Symbol("fibonacci".to_string()),
                    SExp::List(vec![SExp::Symbol("n".to_string())]),
                    SExp::List(vec![
                        SExp::Symbol("if".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("<".to_string()),
                            SExp::Symbol("n".to_string()),
                            SExp::Atom(Atom::Integer(2)),
                        ]),
                        SExp::Symbol("n".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("+".to_string()),
                            SExp::List(vec![
                                SExp::Symbol("fibonacci".to_string()),
                                SExp::List(vec![
                                    SExp::Symbol("-".to_string()),
                                    SExp::Symbol("n".to_string()),
                                    SExp::Atom(Atom::Integer(1)),
                                ]),
                            ]),
                            SExp::List(vec![
                                SExp::Symbol("fibonacci".to_string()),
                                SExp::List(vec![
                                    SExp::Symbol("-".to_string()),
                                    SExp::Symbol("n".to_string()),
                                    SExp::Atom(Atom::Integer(2)),
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ),
    ]
}

fn bench_parsing(c: &mut Criterion) {
    let test_strings = vec![
        ("simple_atom", "42"),
        ("simple_list", "(+ 1 2)"),
        ("factorial", "(defun factorial (n) (if (= n 0) 1 (* n (factorial (- n 1)))))"),
        ("complex_module", r#"
            (module math
              (export factorial fibonacci)
              
              (defun factorial (n)
                (if (= n 0)
                    1
                    (* n (factorial (- n 1)))))
              
              (defun fibonacci (n)
                (if (< n 2)
                    n
                    (+ (fibonacci (- n 1))
                       (fibonacci (- n 2))))))
        "#),
    ];

    for (name, input) in test_strings {
        c.bench_with_input(
            BenchmarkId::new("parse", name),
            input,
            |b, input| {
                b.iter(|| parse(black_box(input)).unwrap());
            },
        );
    }
}

fn bench_serialization(c: &mut Criterion) {
    let test_expressions = create_test_expressions();

    for (name, sexp) in test_expressions {
        c.bench_with_input(
            BenchmarkId::new("serialize", name),
            &sexp,
            |b, sexp| {
                b.iter(|| serialize(black_box(sexp)).unwrap());
            },
        );
    }
}

fn bench_deserialization(c: &mut Criterion) {
    let test_expressions = create_test_expressions();

    for (name, sexp) in test_expressions {
        let serialized = serialize(&sexp).unwrap();
        c.bench_with_input(
            BenchmarkId::new("deserialize", name),
            &serialized,
            |b, data| {
                b.iter(|| deserialize(black_box(data)).unwrap());
            },
        );
    }
}

fn bench_hashing(c: &mut Criterion) {
    let test_expressions = create_test_expressions();

    for (name, sexp) in test_expressions {
        c.bench_with_input(
            BenchmarkId::new("hash", name),
            &sexp,
            |b, sexp| {
                b.iter(|| ContentHash::hash(black_box(sexp)));
            },
        );
    }
}

fn bench_diff(c: &mut Criterion) {
    let factorial1 = SExp::List(vec![
        SExp::Symbol("defun".to_string()),
        SExp::Symbol("factorial".to_string()),
        SExp::List(vec![SExp::Symbol("n".to_string())]),
        SExp::List(vec![
            SExp::Symbol("if".to_string()),
            SExp::List(vec![
                SExp::Symbol("=".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::Atom(Atom::Integer(0)),
            ]),
            SExp::Atom(Atom::Integer(1)),
            SExp::List(vec![
                SExp::Symbol("*".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::List(vec![
                    SExp::Symbol("factorial".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("-".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::Atom(Atom::Integer(1)),
                    ]),
                ]),
            ]),
        ]),
    ]);

    let factorial2 = SExp::List(vec![
        SExp::Symbol("defun".to_string()),
        SExp::Symbol("factorial".to_string()),
        SExp::List(vec![SExp::Symbol("n".to_string())]),
        SExp::List(vec![
            SExp::Symbol("if".to_string()),
            SExp::List(vec![
                SExp::Symbol("<=".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::Atom(Atom::Integer(1)),
            ]),
            SExp::Atom(Atom::Integer(1)),
            SExp::List(vec![
                SExp::Symbol("*".to_string()),
                SExp::Symbol("n".to_string()),
                SExp::List(vec![
                    SExp::Symbol("factorial".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("-".to_string()),
                        SExp::Symbol("n".to_string()),
                        SExp::Atom(Atom::Integer(1)),
                    ]),
                ]),
            ]),
        ]),
    ]);

    let diff_engine = StructuralDiff::new();

    c.bench_function("diff_identical", |b| {
        b.iter(|| diff_engine.diff(black_box(&factorial1), black_box(&factorial1)));
    });

    c.bench_function("diff_different", |b| {
        b.iter(|| diff_engine.diff(black_box(&factorial1), black_box(&factorial2)));
    });
}

fn bench_round_trip(c: &mut Criterion) {
    let test_expressions = create_test_expressions();

    for (name, sexp) in test_expressions {
        c.bench_with_input(
            BenchmarkId::new("round_trip", name),
            &sexp,
            |b, sexp| {
                b.iter(|| {
                    let serialized = serialize(black_box(sexp)).unwrap();
                    let _deserialized = deserialize(black_box(&serialized)).unwrap();
                });
            },
        );
    }
}

criterion_group!(
    benches,
    bench_parsing,
    bench_serialization,
    bench_deserialization,
    bench_hashing,
    bench_diff,
    bench_round_trip
);
criterion_main!(benches);