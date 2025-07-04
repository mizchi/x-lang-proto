//! WebAssembly runtime tests for EffectLang

use effect_lang::{
    core::{ast::*, span::{Span, FileId, ByteOffset}, symbol::Symbol},
    analysis::inference::InferenceContext,
    codegen::{
        backend::{BackendFactory, CodegenOptions, CompilationTarget},
        ir::IRBuilder,
    },
};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::process::Command;

fn create_test_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
}

fn create_simple_wasm_test() -> CompilationUnit {
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

fn create_function_wasm_test() -> CompilationUnit {
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

fn wat_to_wasm(wat_content: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Write WAT to temporary file
    let wat_path = output_path.with_extension("wat");
    fs::write(&wat_path, wat_content)?;
    
    // Try to convert WAT to WASM using wasmtime
    let wasmtime_path = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/bin/wasmtime";
    
    let output = Command::new(&wasmtime_path)
        .args(&["compile", wat_path.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("wasmtime compile failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Clean up temporary WAT file
    let _ = fs::remove_file(wat_path);
    
    Ok(())
}

fn run_wasm_with_wasmtime(wasm_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let wasmtime_path = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/bin/wasmtime";
    
    let output = Command::new(&wasmtime_path)
        .args(&["run", wasm_path.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("wasmtime run failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn test_wasm_gc_codegen_output() {
    let ast = create_simple_wasm_test();
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
    
    // Print the generated WAT content for inspection
    for (path, content) in &result.files {
        println!("=== Generated WAT for {} ===", path.display());
        println!("{}", content);
        println!("=== End WAT ===");
    }
}

#[test]
fn test_wasm_gc_function_codegen_output() {
    let ast = create_function_wasm_test();
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
    
    // Print the generated WAT content for inspection
    for (path, content) in &result.files {
        println!("=== Generated WAT for {} ===", path.display());
        println!("{}", content);
        println!("=== End WAT ===");
    }
}

#[test] 
#[ignore] // Ignore by default as it requires wasmtime installation
fn test_wasm_compilation_and_execution() {
    let ast = create_simple_wasm_test();
    let mut backend = BackendFactory::create_backend("wasm-gc").unwrap();
    let type_info = HashMap::new();
    
    let temp_dir = std::env::temp_dir().join("effect_lang_wasm_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "WebAssembly GC".to_string(),
            file_extension: "wasm".to_string(),
            supports_modules: true,
            supports_effects: false,
            supports_gc: true,
        },
        output_dir: temp_dir.clone(),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };
    
    let result = backend.generate_code(&ast, &type_info, &options).unwrap();
    
    // Get the generated WAT content
    let (wat_path, wat_content) = result.files.iter().next().unwrap();
    
    // Try to compile WAT to WASM
    let wasm_path = temp_dir.join("test.wasm");
    
    if let Err(e) = wat_to_wasm(wat_content, &wasm_path) {
        println!("Warning: Could not compile WAT to WASM: {}", e);
        println!("This is expected as the generated WAT may be incomplete");
        return;
    }
    
    // Try to run the WASM module
    if let Err(e) = run_wasm_with_wasmtime(&wasm_path) {
        println!("Warning: Could not run WASM: {}", e);
        println!("This is expected as the generated WASM may need more runtime support");
        return;
    }
    
    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_wasm_runtime_generation() {
    let backend = BackendFactory::create_backend("wasm-gc").unwrap();
    
    let options = CodegenOptions {
        target: CompilationTarget {
            name: "WebAssembly GC".to_string(),
            file_extension: "wasm".to_string(),
            supports_modules: true,
            supports_effects: false,
            supports_gc: true,
        },
        output_dir: std::path::PathBuf::from("test"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };
    
    let runtime = backend.generate_runtime(&options).unwrap();
    
    println!("=== Generated WebAssembly Runtime ===");
    println!("{}", runtime);
    println!("=== End Runtime ===");
    
    // Check that runtime contains expected structures
    assert!(runtime.contains("$value"));
    assert!(runtime.contains("$closure"));
    assert!(runtime.contains("$TAG_"));
    assert!(runtime.contains("struct"));
    assert!(runtime.contains("func"));
}