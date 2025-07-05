//! Abstract backend interface for code generation

use x_parser::{CompilationUnit, Module, Span, Symbol};
use x_checker::{Type, TypeScheme};
use crate::{CompilerError, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Compilation target specification
#[derive(Debug, Clone)]
pub struct CompilationTarget {
    pub name: String,
    pub file_extension: String,
    pub supports_modules: bool,
    pub supports_effects: bool,
    pub supports_gc: bool,
}

/// Code generation options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    pub target: CompilationTarget,
    pub output_dir: PathBuf,
    pub source_maps: bool,
    pub debug_info: bool,
    pub optimization_level: u8,
    pub emit_types: bool,
}

/// Result of code generation
#[derive(Debug)]
pub struct CodegenResult {
    pub files: HashMap<PathBuf, String>,
    pub source_maps: HashMap<PathBuf, String>,
    pub diagnostics: Vec<CodegenDiagnostic>,
    pub metadata: CodegenMetadata,
}

/// Code generation diagnostic
#[derive(Debug, Clone)]
pub struct CodegenDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Metadata about the generated code
#[derive(Debug)]
pub struct CodegenMetadata {
    pub target_info: CompilationTarget,
    pub generated_files: usize,
    pub total_size: usize,
    pub compilation_time: std::time::Duration,
}

/// Abstract code generation backend
pub trait CodegenBackend {
    /// Get the target information for this backend
    fn target_info(&self) -> CompilationTarget;
    
    /// Check if this backend supports the given features
    fn supports_feature(&self, feature: &str) -> bool;
    
    /// Generate code for a compilation unit
    fn generate_code(
        &mut self,
        cu: &CompilationUnit,
        type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<CodegenResult>;
    
    /// Generate code for a module
    fn generate_module(
        &mut self,
        module: &Module,
        type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<String>;
    
    /// Generate runtime support code
    fn generate_runtime(&self, options: &CodegenOptions) -> Result<String>;
    
    /// Optimize generated code
    fn optimize_code(&self, code: &str, level: u8) -> Result<String> {
        // Default: no optimization
        Ok(code.to_string())
    }
    
    /// Validate the generated code
    fn validate_output(&self, code: &str) -> Vec<CodegenDiagnostic> {
        // Default: no validation
        Vec::new()
    }
}

/// Factory for creating backend instances
pub struct BackendFactory;

impl BackendFactory {
    /// Create a backend for the specified target
    pub fn create_backend(target: &str) -> Result<Box<dyn CodegenBackend>> {
        match target {
            "typescript" | "ts" => {
                Ok(Box::new(crate::typescript::TypeScriptBackend::new()))
            }
            "wasm-gc" | "wasm" => {
                Ok(Box::new(crate::wasm_gc::WasmGCBackend::new()))
            }
            "wasm-component" | "component" => {
                Ok(Box::new(crate::wasm_component::WasmComponentBackend::new()))
            }
            "wit" => {
                Ok(Box::new(crate::wit_backend::WitBackend::new()))
            }
            _ => Err(CompilerError::InvalidTarget {
                target: target.to_string(),
            }),
        }
    }
    
    /// List all available backends
    pub fn available_backends() -> Vec<&'static str> {
        vec!["typescript", "wasm-gc", "wasm-component", "wit"]
    }
}

/// Utility functions for code generation
pub mod utils {
    use x_parser::Symbol;
    
    /// Sanitize a symbol name for the target language
    pub fn sanitize_identifier(symbol: Symbol, target: &str) -> String {
        let name = symbol.as_str();
        match target {
            "typescript" => sanitize_typescript_identifier(name),
            "wasm-gc" => sanitize_wasm_identifier(name),
            _ => name.to_string(),
        }
    }
    
    fn sanitize_typescript_identifier(name: &str) -> String {
        // Handle TypeScript reserved keywords and naming conventions
        match name {
            "class" | "interface" | "type" | "namespace" | "module" => {
                format!("{}_", name)
            }
            _ => name.replace("-", "_").replace("?", "_q").replace("!", "_e"),
        }
    }
    
    fn sanitize_wasm_identifier(name: &str) -> String {
        // WebAssembly identifier rules
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
    
    /// Generate a unique name to avoid conflicts
    pub fn unique_name(base: &str, taken: &std::collections::HashSet<String>) -> String {
        if !taken.contains(base) {
            return base.to_string();
        }
        
        for i in 1.. {
            let candidate = format!("{}_{}", base, i);
            if !taken.contains(&candidate) {
                return candidate;
            }
        }
        
        unreachable!()
    }
}