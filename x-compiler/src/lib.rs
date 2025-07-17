//! x Language Compiler
//! 
//! This crate provides code generation and compilation functionality for x Language.
//! It supports multiple target languages and platforms including TypeScript, 
//! WebAssembly GC, and WebAssembly Component Model.

pub mod backend;
pub mod ir;
pub mod codegen_mod;
pub mod typescript;
pub mod wasm_gc;
pub mod wasm_component;
pub mod wit;
pub mod wit_backend;
pub mod utils;
pub mod pipeline;
pub mod config;

// Re-export main types
pub use backend::{
    CodegenBackend, BackendFactory, CompilationTarget, CodegenOptions, CodegenResult,
    CodegenDiagnostic, DiagnosticSeverity, CodegenMetadata,
};
pub use ir::{IR, IRBuilder};
pub use pipeline::{CompilationPipeline, PipelineStage, PipelineResult};
pub use config::{CompilerConfig, TargetConfig};

use x_parser::{CompilationUnit, SyntaxStyle};
use x_checker::{type_check, CheckResult};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CompilerError>;

/// High-level compilation function
pub fn compile(
    source: &str,
    target: &str,
    output_dir: PathBuf,
    config: CompilerConfig,
) -> Result<CompilationResult> {
    let mut pipeline = CompilationPipeline::new(config);
    pipeline.compile(source, target, output_dir)
}

/// Compilation result
#[derive(Debug)]
pub struct CompilationResult {
    pub target: String,
    pub files: std::collections::HashMap<PathBuf, String>,
    pub diagnostics: Vec<CompilerDiagnostic>,
    pub metadata: CompilationMetadata,
}

/// Compilation metadata
#[derive(Debug)]
pub struct CompilationMetadata {
    pub parse_time: std::time::Duration,
    pub check_time: std::time::Duration,
    pub codegen_time: std::time::Duration,
    pub total_time: std::time::Duration,
    pub lines_of_code: usize,
    pub ast_nodes: usize,
    pub generated_files: usize,
    pub total_output_size: usize,
}

/// Compiler diagnostic
#[derive(Debug, Clone)]
pub struct CompilerDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: DiagnosticSource,
    pub span: Option<x_parser::Span>,
}

/// Source of a diagnostic
#[derive(Debug, Clone)]
pub enum DiagnosticSource {
    Parser,
    TypeChecker,
    CodeGenerator,
    Linker,
    Optimizer,
}

/// Compiler errors
#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("Parse error: {0}")]
    Parse(#[from] x_parser::ParseError),

    #[error("Type checking failed: {message}")]
    TypeCheck { message: String },

    #[error("Code generation failed: {message}")]
    CodeGen { message: String },

    #[error("Invalid target: {target}")]
    InvalidTarget { target: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<String> for CompilerError {
    fn from(s: String) -> Self {
        CompilerError::Generic(s)
    }
}

impl From<x_checker::TypeError> for CompilerError {
    fn from(e: x_checker::TypeError) -> Self {
        CompilerError::TypeCheck { message: e.to_string() }
    }
}

impl From<&str> for CompilerError {
    fn from(s: &str) -> Self {
        CompilerError::Generic(s.to_string())
    }
}

impl From<std::fmt::Error> for CompilerError {
    fn from(e: std::fmt::Error) -> Self {
        CompilerError::Generic(format!("Formatting error: {e}"))
    }
}

/// Compiler builder for fluent configuration
pub struct CompilerBuilder {
    config: CompilerConfig,
}

impl CompilerBuilder {
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    pub fn syntax_style(mut self, style: SyntaxStyle) -> Self {
        self.config.syntax_style = style;
        self
    }

    pub fn optimization_level(mut self, level: u8) -> Self {
        self.config.optimization_level = level;
        self
    }

    pub fn debug_info(mut self, enabled: bool) -> Self {
        self.config.debug_info = enabled;
        self
    }

    pub fn source_maps(mut self, enabled: bool) -> Self {
        self.config.source_maps = enabled;
        self
    }

    pub fn target_config(mut self, target: &str, config: TargetConfig) -> Self {
        self.config.target_configs.insert(target.to_string(), config);
        self
    }

    pub fn build(self) -> Compiler {
        Compiler::new(self.config)
    }
}

impl Default for CompilerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main compiler interface
pub struct Compiler {
    config: CompilerConfig,
    pipeline: CompilationPipeline,
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        let pipeline = CompilationPipeline::new(config.clone());
        Self { config, pipeline }
    }

    /// Compile source code to target
    pub fn compile(
        &mut self,
        source: &str,
        target: &str,
        output_dir: PathBuf,
    ) -> Result<CompilationResult> {
        self.pipeline.compile(source, target, output_dir)
    }

    /// Compile file to target
    pub fn compile_file(
        &mut self,
        input_path: &PathBuf,
        target: &str,
        output_dir: PathBuf,
    ) -> Result<CompilationResult> {
        let source = std::fs::read_to_string(input_path)?;
        self.compile(&source, target, output_dir)
    }

    /// Get available targets
    pub fn available_targets(&self) -> Vec<String> {
        BackendFactory::available_backends()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        // TODO: Implement configuration validation
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &CompilerConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: CompilerConfig) {
        self.config = config.clone();
        self.pipeline = CompilationPipeline::new(config);
    }
}

/// Convenience functions
pub mod convenience {
    use super::*;

    /// Quick compilation with default settings
    pub fn compile_to_typescript(source: &str, output_dir: PathBuf) -> Result<CompilationResult> {
        let config = CompilerConfig::default();
        compile(source, "typescript", output_dir, config)
    }

    /// Quick compilation to WebAssembly Component Model
    pub fn compile_to_wasm_component(source: &str, output_dir: PathBuf) -> Result<CompilationResult> {
        let config = CompilerConfig::default();
        compile(source, "wasm-component", output_dir, config)
    }

    /// Quick compilation to WIT
    pub fn compile_to_wit(source: &str, output_dir: PathBuf) -> Result<CompilationResult> {
        let config = CompilerConfig::default();
        compile(source, "wit", output_dir, config)
    }

    /// Type check only
    pub fn type_check_source(source: &str) -> Result<CheckResult> {
        use x_parser::{parse_source, FileId};
        
        let file_id = FileId::new(0);
        let cu = parse_source(source, file_id, SyntaxStyle::default())?;
        Ok(type_check(&cu))
    }

    /// Parse only
    pub fn parse_source_only(source: &str, syntax_style: SyntaxStyle) -> Result<CompilationUnit> {
        use x_parser::{parse_source, FileId};
        
        let file_id = FileId::new(0);
        Ok(parse_source(source, file_id, syntax_style)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compiler_builder() {
        let compiler = CompilerBuilder::new()
            .syntax_style(SyntaxStyle::Haskell)
            .optimization_level(2)
            .debug_info(true)
            .build();

        assert_eq!(compiler.config.syntax_style, SyntaxStyle::Haskell);
        assert_eq!(compiler.config.optimization_level, 2);
        assert!(compiler.config.debug_info);
    }

    #[test]
    fn test_available_targets() {
        let compiler = Compiler::new(CompilerConfig::default());
        let targets = compiler.available_targets();
        
        assert!(targets.contains(&"typescript".to_string()));
        assert!(targets.contains(&"wasm-gc".to_string()));
        assert!(targets.contains(&"wit".to_string()));
    }

    #[test]
    fn test_convenience_functions() {
        let source = "let x = 42";
        
        // Test parsing
        let result = convenience::parse_source_only(source, SyntaxStyle::Haskell);
        assert!(result.is_ok());

        // Test type checking
        let result = convenience::type_check_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compiler_creation() {
        let config = CompilerConfig::default();
        let compiler = Compiler::new(config);
        
        assert_eq!(compiler.config.optimization_level, 0); // Default optimization
    }

    #[test]
    fn test_compile_simple() {
        let temp_dir = TempDir::new().unwrap();
        let source = "let x = 42";
        
        let result = convenience::compile_to_typescript(source, temp_dir.path().to_path_buf());
        // May fail due to incomplete implementation, but should not panic
        println!("Compilation result: {:?}", result);
    }
}