//! Code generation backends for x Language
//! 
//! This module provides a unified interface for generating code
//! targeting different platforms and languages.


/// Supported compilation targets
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    TypeScript {
        module_system: TypeScriptModuleSystem,
        emit_types: bool,
        strict_mode: bool,
    },
    WebAssemblyGC {
        optimization_level: WasmOptLevel,
        debug_info: bool,
        gc_strategy: GCStrategy,
    },
    WebAssemblyComponent {
        optimization_level: WasmOptLevel,
        debug_info: bool,
        with_wit: bool,
    },
    WIT {
        package_name: Option<String>,
        generate_bindings: bool,
    },
}

/// TypeScript module system options
#[derive(Debug, Clone, PartialEq)]
pub enum TypeScriptModuleSystem {
    ES2020,
    CommonJS,
    AMD,
    SystemJS,
}

/// WebAssembly optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum WasmOptLevel {
    None,
    Size,
    Speed,
    Aggressive,
}

/// Garbage collection strategies for WebAssembly GC
#[derive(Debug, Clone, PartialEq)]
pub enum GCStrategy {
    Conservative,
    Precise,
    Incremental,
}

/// Compilation context shared across backends
#[derive(Debug)]
pub struct CompilationContext {
    pub target: Target,
    pub source_maps: bool,
    pub debug_info: bool,
    pub optimization_level: u8,
    pub output_directory: std::path::PathBuf,
}

impl CompilationContext {
    pub fn new(target: Target, output_dir: std::path::PathBuf) -> Self {
        CompilationContext {
            target,
            source_maps: true,
            debug_info: false,
            optimization_level: 1,
            output_directory: output_dir,
        }
    }
    
    pub fn with_debug(mut self) -> Self {
        self.debug_info = true;
        self
    }
    
    pub fn with_optimization(mut self, level: u8) -> Self {
        self.optimization_level = level;
        self
    }
}