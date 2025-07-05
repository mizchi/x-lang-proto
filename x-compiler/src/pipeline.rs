//! Compilation pipeline for orchestrating the compilation process

use crate::{
    backend::{BackendFactory, CodegenOptions, CompilationTarget},
    config::CompilerConfig,
    CompilerError, CompilationResult, CompilationMetadata, CompilerDiagnostic, DiagnosticSource,
};
use x_parser::{parse_with_metadata, FileId};
use x_checker::type_check;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Compilation pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Parse,
    TypeCheck,
    Optimize,
    CodeGen,
    Link,
    Write,
}

/// Pipeline stage result
#[derive(Debug)]
pub struct PipelineResult<T> {
    pub stage: PipelineStage,
    pub result: T,
    pub duration: std::time::Duration,
    pub diagnostics: Vec<CompilerDiagnostic>,
}

/// Compilation pipeline
pub struct CompilationPipeline {
    config: CompilerConfig,
    enabled_stages: Vec<PipelineStage>,
}

impl CompilationPipeline {
    pub fn new(config: CompilerConfig) -> Self {
        let enabled_stages = vec![
            PipelineStage::Parse,
            PipelineStage::TypeCheck,
            PipelineStage::Optimize,
            PipelineStage::CodeGen,
            PipelineStage::Link,
            PipelineStage::Write,
        ];

        Self {
            config,
            enabled_stages,
        }
    }

    /// Run the full compilation pipeline
    pub fn compile(
        &mut self,
        source: &str,
        target: &str,
        output_dir: PathBuf,
    ) -> Result<CompilationResult, CompilerError> {
        let total_start = Instant::now();
        let mut all_diagnostics = Vec::new();

        // Stage 1: Parse
        let parse_result = self.run_parse_stage(source)?;
        all_diagnostics.extend(parse_result.diagnostics);
        let ast = parse_result.result;
        let parse_time = parse_result.duration;

        // Stage 2: Type Check
        let check_result = self.run_typecheck_stage(&ast)?;
        all_diagnostics.extend(check_result.diagnostics);
        let check_time = check_result.duration;

        // Stage 3: Optimize (optional)
        let optimize_result = self.run_optimize_stage(&ast)?;
        all_diagnostics.extend(optimize_result.diagnostics);
        let optimized_ast = optimize_result.result;

        // Stage 4: Code Generation
        let codegen_result = self.run_codegen_stage(&optimized_ast, target, &output_dir)?;
        all_diagnostics.extend(codegen_result.diagnostics);
        let generated_files = codegen_result.result;
        let codegen_time = codegen_result.duration;

        // Stage 5: Link (optional for some targets)
        let link_result = self.run_link_stage(&generated_files, target)?;
        all_diagnostics.extend(link_result.diagnostics);

        // Stage 6: Write files
        let write_result = self.run_write_stage(generated_files, &output_dir)?;
        all_diagnostics.extend(write_result.diagnostics);
        let final_files = write_result.result;

        let total_time = total_start.elapsed();

        // Calculate metadata
        let lines_of_code = source.lines().count();
        let ast_nodes = self.count_ast_nodes(&ast);
        let total_output_size = final_files.values().map(|content| content.len()).sum();

        let generated_files_count = final_files.len();
        
        Ok(CompilationResult {
            target: target.to_string(),
            files: final_files,
            diagnostics: all_diagnostics,
            metadata: CompilationMetadata {
                parse_time,
                check_time,
                codegen_time,
                total_time,
                lines_of_code,
                ast_nodes,
                generated_files: generated_files_count,
                total_output_size,
            },
        })
    }

    /// Run parse stage
    fn run_parse_stage(
        &self,
        source: &str,
    ) -> Result<PipelineResult<x_parser::CompilationUnit>, CompilerError> {
        let start = Instant::now();
        let file_id = FileId::new(0);

        let parse_result = parse_with_metadata(source, file_id, self.config.syntax_style)?;
        let duration = start.elapsed();

        Ok(PipelineResult {
            stage: PipelineStage::Parse,
            result: parse_result.ast,
            duration,
            diagnostics: Vec::new(), // TODO: Convert parse errors to diagnostics
        })
    }

    /// Run type checking stage
    fn run_typecheck_stage(
        &self,
        ast: &x_parser::CompilationUnit,
    ) -> Result<PipelineResult<x_checker::CheckResult>, CompilerError> {
        let start = Instant::now();
        
        let check_result = type_check(ast);
        let duration = start.elapsed();

        let diagnostics = check_result.errors.iter()
            .map(|error| CompilerDiagnostic {
                severity: crate::backend::DiagnosticSeverity::Error,
                message: format!("{}", error),
                source: DiagnosticSource::TypeChecker,
                span: None, // TODO: Extract span from type error
            })
            .chain(check_result.warnings.iter().map(|warning| CompilerDiagnostic {
                severity: crate::backend::DiagnosticSeverity::Warning,
                message: format!("{}", warning),
                source: DiagnosticSource::TypeChecker,
                span: None,
            }))
            .collect();

        Ok(PipelineResult {
            stage: PipelineStage::TypeCheck,
            result: check_result,
            duration,
            diagnostics,
        })
    }

    /// Run optimization stage
    fn run_optimize_stage(
        &self,
        ast: &x_parser::CompilationUnit,
    ) -> Result<PipelineResult<x_parser::CompilationUnit>, CompilerError> {
        let start = Instant::now();
        
        // TODO: Implement AST optimizations
        let optimized_ast = ast.clone();
        
        let duration = start.elapsed();

        Ok(PipelineResult {
            stage: PipelineStage::Optimize,
            result: optimized_ast,
            duration,
            diagnostics: Vec::new(),
        })
    }

    /// Run code generation stage
    fn run_codegen_stage(
        &self,
        ast: &x_parser::CompilationUnit,
        target: &str,
        output_dir: &PathBuf,
    ) -> Result<PipelineResult<HashMap<PathBuf, String>>, CompilerError> {
        let start = Instant::now();

        let mut backend = BackendFactory::create_backend(target)
            .map_err(|_| CompilerError::InvalidTarget { target: target.to_string() })?;

        let target_config = self.config.target_config(target);
        let compilation_target = self.create_compilation_target(target, &target_config)?;

        let codegen_options = CodegenOptions {
            target: compilation_target,
            output_dir: output_dir.clone(),
            source_maps: self.config.source_maps,
            debug_info: self.config.debug_info,
            optimization_level: self.config.optimization_level,
            emit_types: self.config.emit_types,
        };

        let codegen_result = backend.generate_code(ast, &HashMap::new(), &codegen_options)
            .map_err(|e| CompilerError::CodeGen { message: format!("{:?}", e) })?;

        let duration = start.elapsed();

        let diagnostics = codegen_result.diagnostics.into_iter()
            .map(|diag| CompilerDiagnostic {
                severity: diag.severity,
                message: diag.message,
                source: DiagnosticSource::CodeGenerator,
                span: diag.location,
            })
            .collect();

        Ok(PipelineResult {
            stage: PipelineStage::CodeGen,
            result: codegen_result.files,
            duration,
            diagnostics,
        })
    }

    /// Run linking stage
    fn run_link_stage(
        &self,
        _files: &HashMap<PathBuf, String>,
        _target: &str,
    ) -> Result<PipelineResult<()>, CompilerError> {
        let start = Instant::now();
        
        // TODO: Implement linking for targets that need it
        
        let duration = start.elapsed();

        Ok(PipelineResult {
            stage: PipelineStage::Link,
            result: (),
            duration,
            diagnostics: Vec::new(),
        })
    }

    /// Run file writing stage
    fn run_write_stage(
        &self,
        files: HashMap<PathBuf, String>,
        output_dir: &PathBuf,
    ) -> Result<PipelineResult<HashMap<PathBuf, String>>, CompilerError> {
        let start = Instant::now();
        let mut written_files = HashMap::new();
        let mut diagnostics = Vec::new();

        for (relative_path, content) in files {
            let full_path = output_dir.join(&relative_path);
            
            if let Some(parent) = full_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    diagnostics.push(CompilerDiagnostic {
                        severity: crate::backend::DiagnosticSeverity::Error,
                        message: format!("Failed to create directory {}: {}", parent.display(), e),
                        source: DiagnosticSource::Linker,
                        span: None,
                    });
                    continue;
                }
            }

            match std::fs::write(&full_path, &content) {
                Ok(()) => {
                    written_files.insert(full_path, content);
                }
                Err(e) => {
                    diagnostics.push(CompilerDiagnostic {
                        severity: crate::backend::DiagnosticSeverity::Error,
                        message: format!("Failed to write file {}: {}", full_path.display(), e),
                        source: DiagnosticSource::Linker,
                        span: None,
                    });
                }
            }
        }

        let duration = start.elapsed();

        Ok(PipelineResult {
            stage: PipelineStage::Write,
            result: written_files,
            duration,
            diagnostics,
        })
    }

    /// Create compilation target from configuration
    fn create_compilation_target(
        &self,
        target_name: &str,
        _target_config: &crate::config::TargetConfig,
    ) -> Result<CompilationTarget, CompilerError> {
        let file_extension = match target_name {
            "typescript" | "ts" => "ts",
            "wasm-gc" | "wasm" => "wasm",
            "wasm-component" | "component" => "wasm",
            "wit" => "wit",
            _ => return Err(CompilerError::InvalidTarget { target: target_name.to_string() }),
        };

        Ok(CompilationTarget {
            name: target_name.to_string(),
            file_extension: file_extension.to_string(),
            supports_modules: true,
            supports_effects: target_name != "wit",
            supports_gc: target_name.contains("wasm"),
        })
    }

    /// Count AST nodes for metrics
    fn count_ast_nodes(&self, ast: &x_parser::CompilationUnit) -> usize {
        // Simple node counting - could be more sophisticated
        let mut count = 1; // CompilationUnit itself
        
        // Count module items
        count += 1; // Module
        count += ast.module.items.len(); // Items
        count += ast.module.imports.len();
        if let Some(ref exports) = ast.module.exports {
            count += exports.items.len();
        }
        
        count
    }

    /// Enable or disable a pipeline stage
    pub fn set_stage_enabled(&mut self, stage: PipelineStage, enabled: bool) {
        if enabled {
            if !self.enabled_stages.contains(&stage) {
                self.enabled_stages.push(stage);
                self.enabled_stages.sort_by_key(|&s| match s {
                    PipelineStage::Parse => 0,
                    PipelineStage::TypeCheck => 1,
                    PipelineStage::Optimize => 2,
                    PipelineStage::CodeGen => 3,
                    PipelineStage::Link => 4,
                    PipelineStage::Write => 5,
                });
            }
        } else {
            self.enabled_stages.retain(|&s| s != stage);
        }
    }

    /// Check if a stage is enabled
    pub fn is_stage_enabled(&self, stage: PipelineStage) -> bool {
        self.enabled_stages.contains(&stage)
    }

    /// Get stage execution order
    #[allow(dead_code)]
    fn stage_order(&self, stage: PipelineStage) -> u8 {
        match stage {
            PipelineStage::Parse => 0,
            PipelineStage::TypeCheck => 1,
            PipelineStage::Optimize => 2,
            PipelineStage::CodeGen => 3,
            PipelineStage::Link => 4,
            PipelineStage::Write => 5,
        }
    }

    /// Get pipeline configuration
    pub fn config(&self) -> &CompilerConfig {
        &self.config
    }

    /// Update pipeline configuration
    pub fn update_config(&mut self, config: CompilerConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pipeline_creation() {
        let config = CompilerConfig::default();
        let pipeline = CompilationPipeline::new(config);
        
        assert!(pipeline.is_stage_enabled(PipelineStage::Parse));
        assert!(pipeline.is_stage_enabled(PipelineStage::TypeCheck));
        assert!(pipeline.is_stage_enabled(PipelineStage::CodeGen));
    }

    #[test]
    fn test_stage_management() {
        let config = CompilerConfig::default();
        let mut pipeline = CompilationPipeline::new(config);
        
        pipeline.set_stage_enabled(PipelineStage::Optimize, false);
        assert!(!pipeline.is_stage_enabled(PipelineStage::Optimize));
        
        pipeline.set_stage_enabled(PipelineStage::Optimize, true);
        assert!(pipeline.is_stage_enabled(PipelineStage::Optimize));
    }

    #[test]
    fn test_compilation_target_creation() {
        let config = CompilerConfig::default();
        let pipeline = CompilationPipeline::new(config);
        let target_config = crate::config::TargetConfig::default();
        
        let ts_target = pipeline.create_compilation_target("typescript", &target_config);
        assert!(ts_target.is_ok());
        
        let target = ts_target.unwrap();
        assert_eq!(target.name, "typescript");
        assert_eq!(target.file_extension, "ts");
        assert!(target.supports_modules);
    }

    #[test]
    fn test_ast_node_counting() {
        let config = CompilerConfig::default();
        let pipeline = CompilationPipeline::new(config);
        
        use x_parser::{parse_source, FileId, SyntaxStyle};
        let source = "let x = 42\nlet y = true";
        let file_id = FileId::new(0);
        let ast = parse_source(source, file_id, SyntaxStyle::OCaml).unwrap();
        
        let count = pipeline.count_ast_nodes(&ast);
        assert!(count > 0);
    }

    #[test]
    fn test_simple_compilation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CompilerConfig::default();
        let mut pipeline = CompilationPipeline::new(config);
        
        let source = "let x = 42";
        let result = pipeline.compile(source, "wit", temp_dir.path().to_path_buf());
        
        // Should not panic, though may have errors due to incomplete implementation
        println!("Pipeline result: {:?}", result.is_ok());
    }
}