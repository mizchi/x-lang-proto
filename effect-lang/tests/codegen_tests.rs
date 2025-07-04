//! Code generation tests for different backends

use effect_lang::{
    core::{ast::*, span::{Span, FileId, ByteOffset}, symbol::Symbol},
    analysis::inference::InferenceContext,
    codegen::{
        backend::{BackendFactory, CodegenOptions, CompilationTarget},
        ir::IRBuilder,
    },
};
use std::collections::HashMap;

fn create_test_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
}

fn create_simple_ast() -> CompilationUnit {
    let span = create_test_span();
    
    // Create: let x = 42
    let value_def = ValueDef {
        name: Symbol::intern("x"),
        type_annotation: None,
        parameters: Vec::new(),
        body: Expr::Literal(Literal::Integer(42), span),
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

fn create_function_ast() -> CompilationUnit {
    let span = create_test_span();
    
    // Create: let add = fun x y -> x + y
    let body = Expr::App(
        Box::new(Expr::Var(Symbol::intern("+"), span)),
        vec![
            Expr::Var(Symbol::intern("x"), span),
            Expr::Var(Symbol::intern("y"), span),
        ],
        span,
    );
    
    let lambda = Expr::Lambda {
        parameters: vec![
            Pattern::Variable(Symbol::intern("x"), span),
            Pattern::Variable(Symbol::intern("y"), span),
        ],
        body: Box::new(body),
        span,
    };
    
    let value_def = ValueDef {
        name: Symbol::intern("add"),
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

#[test]
fn test_ir_builder() {
    let ast = create_simple_ast();
    let mut ir_builder = IRBuilder::new();
    
    let ir = ir_builder.build_ir(&ast).unwrap();
    
    assert_eq!(ir.modules.len(), 1);
    assert_eq!(ir.modules[0].name, Symbol::intern("Test"));
    assert_eq!(ir.modules[0].constants.len(), 1);
    assert_eq!(ir.modules[0].constants[0].name, Symbol::intern("x"));
}

#[test]
fn test_typescript_backend_creation() {
    let backend = BackendFactory::create_backend("typescript").unwrap();
    let info = backend.target_info();
    
    assert_eq!(info.name, "TypeScript");
    assert_eq!(info.file_extension, "ts");
    assert!(info.supports_modules);
    assert!(info.supports_effects);
    assert!(info.supports_gc);
}

#[test]
fn test_wasm_gc_backend_creation() {
    let backend = BackendFactory::create_backend("wasm-gc").unwrap();
    let info = backend.target_info();
    
    assert_eq!(info.name, "WebAssembly GC");
    assert_eq!(info.file_extension, "wasm");
    assert!(info.supports_modules);
    assert!(info.supports_gc);
}

#[test]
fn test_typescript_codegen() {
    let ast = create_simple_ast();
    let mut backend = BackendFactory::create_backend("typescript").unwrap();
    let type_info = HashMap::new();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "TypeScript".to_string(),
            file_extension: "ts".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: true,
    };
    
    let result = backend.generate_code(&ast, &type_info, &options).unwrap();
    
    assert!(!result.files.is_empty());
    assert!(result.metadata.generated_files > 0);
    
    // Check that generated TypeScript contains our constant
    let files: Vec<_> = result.files.iter().collect();
    
    // Check that runtime is generated
    assert!(files.iter().any(|(_, content)| content.contains("EffectContext")));
    
    // Check that module file contains our constant
    assert!(files.iter().any(|(_, content)| content.contains("42")));
}

#[test]
fn test_typescript_function_codegen() {
    let ast = create_function_ast();
    let mut backend = BackendFactory::create_backend("typescript").unwrap();
    let type_info = HashMap::new();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "TypeScript".to_string(),
            file_extension: "ts".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: true,
    };
    
    let result = backend.generate_code(&ast, &type_info, &options).unwrap();
    
    let files: Vec<_> = result.files.iter().collect();
    
    // Check for function-related code in any of the generated files
    assert!(files.iter().any(|(_, content)| content.contains("add")));
    assert!(files.iter().any(|(_, content)| content.contains("function") || content.contains("=>")));
}

#[test]
fn test_wasm_gc_codegen() {
    let ast = create_simple_ast();
    let mut backend = BackendFactory::create_backend("wasm-gc").unwrap();
    let type_info = HashMap::new();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "WebAssembly GC".to_string(),
            file_extension: "wasm".to_string(),
            supports_modules: true,
            supports_effects: false,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };
    
    let result = backend.generate_code(&ast, &type_info, &options).unwrap();
    
    assert!(!result.files.is_empty());
    
    // Check that generated WebAssembly contains module structure
    let files: Vec<_> = result.files.iter().collect();
    
    // Check that WebAssembly module is generated
    assert!(files.iter().any(|(_, content)| content.contains("(module")));
    // Check for constants or basic structure
    assert!(files.iter().any(|(_, content)| content.contains("42") || content.contains("global")));
}

#[test]
fn test_backend_features() {
    let ts_backend = BackendFactory::create_backend("typescript").unwrap();
    let wasm_backend = BackendFactory::create_backend("wasm-gc").unwrap();
    
    // TypeScript should support all features
    assert!(ts_backend.supports_feature("modules"));
    assert!(ts_backend.supports_feature("types"));
    assert!(ts_backend.supports_feature("effects"));
    assert!(ts_backend.supports_feature("gc"));
    
    // WebAssembly GC has different capabilities
    assert!(wasm_backend.supports_feature("gc"));
    assert!(wasm_backend.supports_feature("structs"));
    assert!(!wasm_backend.supports_feature("effects")); // Needs special handling
}

#[test]
fn test_runtime_generation() {
    let ts_backend = BackendFactory::create_backend("typescript").unwrap();
    let wasm_backend = BackendFactory::create_backend("wasm-gc").unwrap();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "Test".to_string(),
            file_extension: "test".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };
    
    // TypeScript runtime
    let ts_runtime = ts_backend.generate_runtime(&options).unwrap();
    assert!(ts_runtime.contains("EffectContext"));
    assert!(ts_runtime.contains("curry"));
    assert!(ts_runtime.contains("MatchError"));
    
    // WebAssembly GC runtime
    let wasm_runtime = wasm_backend.generate_runtime(&options).unwrap();
    assert!(wasm_runtime.contains("$value"));
    assert!(wasm_runtime.contains("$closure"));
    assert!(wasm_runtime.contains("$TAG_"));
}

#[test]
fn test_available_backends() {
    let backends = BackendFactory::available_backends();
    
    assert!(backends.contains(&"typescript"));
    assert!(backends.contains(&"wasm-gc"));
    assert_eq!(backends.len(), 2);
}

#[test]
fn test_unknown_backend() {
    let result = BackendFactory::create_backend("unknown");
    assert!(result.is_err());
}

#[test]
fn test_codegen_with_types() {
    let ast = create_simple_ast();
    
    // Run type inference
    let mut inference_ctx = InferenceContext::new();
    let mut type_info = HashMap::new();
    
    for item in &ast.module.items {
        if let Item::ValueDef(value_def) = item {
            if let Ok(inference_result) = inference_ctx.infer_expr(&value_def.body) {
                let type_scheme = inference_ctx.generalize(&inference_result.typ, &inference_result.effects);
                type_info.insert(value_def.name, type_scheme);
            }
        }
    }
    
    // Generate code with type information
    let mut backend = BackendFactory::create_backend("typescript").unwrap();
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "TypeScript".to_string(),
            file_extension: "ts".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: true,
    };
    
    let result = backend.generate_code(&ast, &type_info, &options).unwrap();
    
    // Check that the result has no error diagnostics
    let has_errors = result.diagnostics.iter().any(|d| {
        matches!(d.severity, effect_lang::codegen::backend::DiagnosticSeverity::Error)
    });
    assert!(!has_errors, "Code generation should not have errors");
    assert!(result.metadata.compilation_time.as_nanos() > 0);
}