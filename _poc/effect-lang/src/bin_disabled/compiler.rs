//! x Language compiler CLI
//! 
//! Multi-target compiler for x Language supporting TypeScript and WebAssembly GC

use effect_lang::{
    core::{ast::*, span::FileId},
    analysis::{parser::Parser, inference::InferenceContext, types::TypeScheme},
    codegen::{
        backend::{BackendFactory, CodegenOptions, CompilationTarget},
        Target, TypeScriptModuleSystem, WasmOptLevel, GCStrategy,
        CompilationContext,
    },
    Error, Result,
};
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::{info, warn, error};

#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Output directory
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile x Language source to target language
    Compile {
        /// Input source file
        input: PathBuf,
        
        /// Compilation target
        #[arg(short, long, value_enum, default_value = "typescript")]
        target: CompilationTargetArg,
        
        /// TypeScript module system (for TypeScript target)
        #[arg(long, value_enum, default_value = "es2020")]
        module_system: ModuleSystemArg,
        
        /// Enable type emission (TypeScript)
        #[arg(long)]
        emit_types: bool,
        
        /// Enable strict mode (TypeScript)
        #[arg(long)]
        strict: bool,
        
        /// Optimization level (WebAssembly)
        #[arg(long, value_enum, default_value = "none")]
        optimization: OptimizationArg,
        
        /// GC strategy (WebAssembly)
        #[arg(long, value_enum, default_value = "conservative")]
        gc_strategy: GCStrategyArg,
        
        /// Enable debug information
        #[arg(long)]
        debug: bool,
        
        /// Enable source maps
        #[arg(long)]
        source_maps: bool,
    },
    
    /// Check x Language source for errors
    Check {
        /// Input source file
        input: PathBuf,
    },
    
    /// Show information about compilation targets
    Targets,
}

#[derive(Clone, ValueEnum)]
enum CompilationTargetArg {
    TypeScript,
    Ts,
    WebAssembly,
    Wasm,
    WasmGc,
    WasmComponent,
    Component,
    Wit,
}

#[derive(Clone, ValueEnum)]
enum ModuleSystemArg {
    Es2020,
    CommonJs,
    Amd,
    SystemJs,
}

#[derive(Clone, ValueEnum)]
enum OptimizationArg {
    None,
    Size,
    Speed,
    Aggressive,
}

#[derive(Clone, ValueEnum)]
enum GCStrategyArg {
    Conservative,
    Precise,
    Incremental,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    
    match &cli.command {
        Commands::Compile {
            input,
            target,
            module_system,
            emit_types,
            strict,
            optimization,
            gc_strategy,
            debug,
            source_maps,
        } => {
            compile_file(
                input,
                &cli.output,
                target,
                module_system,
                *emit_types,
                *strict,
                optimization,
                gc_strategy,
                *debug,
                *source_maps,
            )
        }
        Commands::Check { input } => check_file(input),
        Commands::Targets => show_targets(),
    }
}

fn compile_file(
    input: &Path,
    output: &Path,
    target: &CompilationTargetArg,
    module_system: &ModuleSystemArg,
    emit_types: bool,
    strict: bool,
    optimization: &OptimizationArg,
    gc_strategy: &GCStrategyArg,
    debug: bool,
    source_maps: bool,
) -> Result<()> {
    info!("Compiling {} to {:?}", input.display(), target);
    
    // Read source file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Parse { message: format!("Failed to read input file: {}", e) })?;
    
    // Parse source
    info!("Parsing source code...");
    let mut parser = Parser::new(&source, FileId::new(0))?;
    let ast = parser.parse()?;
    
    // Type check
    info!("Type checking...");
    let mut type_info = HashMap::new();
    let mut inference_ctx = InferenceContext::new();
    
    // Simplified type checking - in practice this would be more comprehensive
    for item in &ast.module.items {
        if let Item::ValueDef(value_def) = item {
            let inference_result = inference_ctx.infer_expr(&value_def.body)?;
            let type_scheme = inference_ctx.generalize(&inference_result.typ, &inference_result.effects);
            type_info.insert(value_def.name, type_scheme);
        }
    }
    
    // Create compilation target
    let compilation_target = create_target(target, module_system, emit_types, strict, optimization, gc_strategy)?;
    
    // Create output directory
    fs::create_dir_all(output)
        .map_err(|e| Error::Parse { message: format!("Failed to create output directory: {}", e) })?;
    
    // Create backend
    let target_name = match target {
        CompilationTargetArg::TypeScript | CompilationTargetArg::Ts => "typescript",
        CompilationTargetArg::WebAssembly | CompilationTargetArg::Wasm | CompilationTargetArg::WasmGc => "wasm-gc",
        CompilationTargetArg::WasmComponent | CompilationTargetArg::Component => "wasm-component",
        CompilationTargetArg::Wit => "wit",
    };
    
    let mut backend = BackendFactory::create_backend(target_name)?;
    
    // Create codegen options
    let codegen_options = CodegenOptions {
        target: compilation_target,
        output_dir: output.to_path_buf(),
        source_maps,
        debug_info: debug,
        optimization_level: match optimization {
            OptimizationArg::None => 0,
            OptimizationArg::Size => 1,
            OptimizationArg::Speed => 2,
            OptimizationArg::Aggressive => 3,
        },
        emit_types,
    };
    
    // Generate code
    info!("Generating {} code...", target_name);
    let result = backend.generate_code(&ast, &type_info, &codegen_options)?;
    
    // Write output files
    for (file_path, content) in result.files {
        info!("Writing {}", file_path.display());
        fs::write(&file_path, content)
            .map_err(|e| Error::Parse { 
                message: format!("Failed to write output file {}: {}", file_path.display(), e) 
            })?;
    }
    
    // Write source maps if enabled
    if source_maps {
        for (file_path, source_map) in result.source_maps {
            let map_path = file_path.with_extension(format!("{}.map", 
                file_path.extension().unwrap_or_default().to_string_lossy()));
            info!("Writing source map {}", map_path.display());
            fs::write(&map_path, source_map)
                .map_err(|e| Error::Parse { 
                    message: format!("Failed to write source map {}: {}", map_path.display(), e) 
                })?;
        }
    }
    
    // Print diagnostics
    for diagnostic in &result.diagnostics {
        match diagnostic.severity {
            effect_lang::codegen::backend::DiagnosticSeverity::Error => {
                error!("{}", diagnostic.message);
            }
            effect_lang::codegen::backend::DiagnosticSeverity::Warning => {
                warn!("{}", diagnostic.message);
            }
            effect_lang::codegen::backend::DiagnosticSeverity::Info => {
                info!("{}", diagnostic.message);
            }
        }
    }
    
    // Print summary
    info!("Compilation completed successfully!");
    info!("Generated {} files ({} bytes) in {:?}",
          result.metadata.generated_files,
          result.metadata.total_size,
          result.metadata.compilation_time);
    
    Ok(())
}

fn check_file(input: &Path) -> Result<()> {
    info!("Type checking {}", input.display());
    
    // Read and parse
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Parse { message: format!("Failed to read input file: {}", e) })?;
    
    let mut parser = Parser::new(&source, FileId::new(0))?;
    let ast = parser.parse()?;
    
    // Type check
    let mut inference_ctx = InferenceContext::new();
    let mut errors = Vec::new();
    
    for item in &ast.module.items {
        if let Item::ValueDef(value_def) = item {
            match inference_ctx.infer_expr(&value_def.body) {
                Ok(result) => {
                    info!("✓ {} : {}", value_def.name.as_str(), format_type(&result.typ));
                }
                Err(e) => {
                    error!("✗ {} : {}", value_def.name.as_str(), e);
                    errors.push(e);
                }
            }
        }
    }
    
    if errors.is_empty() {
        info!("✓ Type checking completed successfully!");
        Ok(())
    } else {
        error!("✗ Type checking failed with {} errors", errors.len());
        Err(errors.into_iter().next().unwrap())
    }
}

fn show_targets() -> Result<()> {
    println!("Available compilation targets:");
    println!();
    
    println!("TypeScript (typescript, ts):");
    println!("  - Generates type-safe TypeScript code");
    println!("  - Supports all x Language features via async/await");
    println!("  - Module systems: ES2020, CommonJS, AMD, SystemJS");
    println!("  - Full type information emission");
    println!();
    
    println!("WebAssembly GC (wasm-gc, wasm):");
    println!("  - Generates WebAssembly GC bytecode");
    println!("  - Efficient functional programming with GC");
    println!("  - Optimization levels: none, size, speed, aggressive");
    println!("  - GC strategies: conservative, precise, incremental");
    println!();
    
    println!("WebAssembly Component (wasm-component, component):");
    println!("  - Generates WebAssembly Component Model compliant code");
    println!("  - Full support for interfaces, resources, and imports/exports");
    println!("  - Effect system integration with WASI");
    println!("  - Generates Rust source code for wit-bindgen");
    println!();
    
    println!("WIT (wit):");
    println!("  - Generates WebAssembly Interface Types definitions");
    println!("  - Language-agnostic interface specifications");
    println!("  - Compatible with wasm-tools and wit-bindgen");
    println!("  - Export x Language interfaces to other languages");
    println!();
    
    println!("Backend features:");
    for backend_name in BackendFactory::available_backends() {
        if let Ok(backend) = BackendFactory::create_backend(backend_name) {
            let info = backend.target_info();
            println!("  {}: modules={}, effects={}, gc={}",
                     info.name,
                     info.supports_modules,
                     info.supports_effects,
                     info.supports_gc);
        }
    }
    
    Ok(())
}

fn create_target(
    target: &CompilationTargetArg,
    module_system: &ModuleSystemArg,
    emit_types: bool,
    strict: bool,
    optimization: &OptimizationArg,
    gc_strategy: &GCStrategyArg,
) -> Result<CompilationTarget> {
    match target {
        CompilationTargetArg::TypeScript | CompilationTargetArg::Ts => {
            Ok(CompilationTarget {
                name: "TypeScript".to_string(),
                file_extension: "ts".to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: true,
            })
        }
        CompilationTargetArg::WebAssembly | CompilationTargetArg::Wasm | CompilationTargetArg::WasmGc => {
            Ok(CompilationTarget {
                name: "WebAssembly GC".to_string(),
                file_extension: "wasm".to_string(),
                supports_modules: true,
                supports_effects: false,
                supports_gc: true,
            })
        }
        CompilationTargetArg::WasmComponent | CompilationTargetArg::Component => {
            Ok(CompilationTarget {
                name: "WebAssembly Component".to_string(),
                file_extension: "wasm".to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: true,
            })
        }
        CompilationTargetArg::Wit => {
            Ok(CompilationTarget {
                name: "WIT".to_string(),
                file_extension: "wit".to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: false,
            })
        }
    }
}

fn format_type(typ: &effect_lang::analysis::types::Type) -> String {
    use effect_lang::analysis::types::Type;
    match typ {
        Type::Con(symbol) => symbol.as_str().to_string(),
        Type::Var(var) => format!("t{}", var.0),
        Type::Fun { params, return_type, .. } => {
            let param_strs: Vec<String> = params.iter().map(format_type).collect();
            format!("({}) -> {}", param_strs.join(", "), format_type(return_type))
        }
        Type::Tuple(types) => {
            let type_strs: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", type_strs.join(", "))
        }
        Type::Hole => "?".to_string(),
        _ => "unknown".to_string(),
    }
}