//! Simple benchmark comparing Binary AST vs Text AST performance

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
    },
};

/// Create a simple test AST
fn create_test_ast() -> CompilationUnit {
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(100));
    
    // Create a simple function: let f = fun x -> x + 1
    let body = Expr::App(
        Box::new(Expr::Var(Symbol::intern("+"), span)),
        vec![
            Expr::Var(Symbol::intern("x"), span),
            Expr::Literal(Literal::Integer(1), span),
        ],
        span,
    );
    
    let lambda = Expr::Lambda {
        parameters: vec![Pattern::Variable(Symbol::intern("x"), span)],
        body: Box::new(body),
        span,
    };
    
    let value_def = ValueDef {
        name: Symbol::intern("f"),
        type_annotation: None,
        parameters: Vec::new(),
        body: lambda,
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    };
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("Test"), span),
        exports: None,
        imports: Vec::new(),
        items: vec![Item::ValueDef(value_def)],
        span,
    };
    
    CompilationUnit { module, span }
}

/// Test binary serialization performance
fn bench_binary_operations(c: &mut Criterion) {
    let ast = create_test_ast();
    
    // Serialization
    c.bench_function("binary_serialize", |b| {
        b.iter(|| {
            let mut serializer = BinarySerializer::with_type_checking();
            black_box(serializer.serialize_compilation_unit(&ast).unwrap())
        })
    });
    
    // Prepare binary data for deserialization test
    let mut serializer = BinarySerializer::with_type_checking();
    let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
    
    // Deserialization
    c.bench_function("binary_deserialize", |b| {
        b.iter(|| {
            let mut deserializer = BinaryDeserializer::new(binary_data.clone()).unwrap();
            black_box(deserializer.deserialize_compilation_unit().unwrap())
        })
    });
    
    // Binary type checking
    c.bench_function("binary_type_check", |b| {
        b.iter(|| {
            let mut checker = BinaryTypeChecker::new();
            black_box(checker.check_binary_compilation_unit(&binary_data).unwrap())
        })
    });
}

/// Test traditional AST operations
fn bench_traditional_operations(c: &mut Criterion) {
    let ast = create_test_ast();
    
    // Traditional type checking
    c.bench_function("traditional_type_check", |b| {
        b.iter(|| {
            let mut ctx = InferenceContext::new();
            for item in &ast.module.items {
                if let Item::ValueDef(value_def) = item {
                    black_box(ctx.infer_expr(&value_def.body).unwrap());
                }
            }
        })
    });
}

/// Memory size comparison
fn bench_memory_size(c: &mut Criterion) {
    let ast = create_test_ast();
    
    c.bench_function("binary_size", |b| {
        b.iter(|| {
            let mut serializer = BinarySerializer::with_type_checking();
            let binary_data = serializer.serialize_compilation_unit(&ast).unwrap();
            black_box(binary_data.len())
        })
    });
    
    c.bench_function("ast_size_estimate", |b| {
        b.iter(|| {
            // Rough estimate of AST memory usage by counting nodes
            let mut count = 0;
            for item in &ast.module.items {
                if let Item::ValueDef(value_def) = item {
                    count += count_ast_nodes(&value_def.body);
                }
            }
            black_box(count * std::mem::size_of::<Expr>())
        })
    });
}

/// Count AST nodes recursively
fn count_ast_nodes(expr: &Expr) -> usize {
    match expr {
        Expr::Literal(_, _) => 1,
        Expr::Var(_, _) => 1,
        Expr::App(func, args, _) => {
            1 + count_ast_nodes(func) + args.iter().map(count_ast_nodes).sum::<usize>()
        }
        Expr::Lambda { body, .. } => 1 + count_ast_nodes(body),
        Expr::Let { value, body, .. } => 1 + count_ast_nodes(value) + count_ast_nodes(body),
        Expr::If { condition, then_branch, else_branch, .. } => {
            1 + count_ast_nodes(condition) + count_ast_nodes(then_branch) + count_ast_nodes(else_branch)
        }
        _ => 1,
    }
}

criterion_group!(
    benches,
    bench_binary_operations,
    bench_traditional_operations,
    bench_memory_size
);

criterion_main!(benches);