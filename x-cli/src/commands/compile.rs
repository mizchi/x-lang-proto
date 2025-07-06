//! Compilation commands

use anyhow::{Result, Context};
use std::path::Path;
use colored::*;
use crate::utils::{ProgressIndicator, print_success};
use x_compiler::compile;

pub async fn compile_command(input: &Path, target: &str, output: &Path) -> Result<()> {
    let progress = ProgressIndicator::new("Compiling");
    
    println!("Compiling {} to {}", input.display(), target.cyan());
    println!("Output directory: {}", output.display());
    
    progress.set_message("Reading source file");
    let source = tokio::fs::read_to_string(input)
        .await
        .with_context(|| format!("Failed to read source file: {}", input.display()))?;
    
    progress.set_message(&format!("Compiling to {}", target));
    
    // Use default compiler configuration
    let config = x_compiler::config::CompilerConfig::default();
    let result = compile(&source, target, output.to_path_buf(), config)
        .with_context(|| format!("Failed to compile to {}", target))?;
    
    progress.finish("Compilation completed");
    
    // Display results
    if !result.diagnostics.is_empty() {
        println!("\nDiagnostics:");
        for diagnostic in &result.diagnostics {
            match diagnostic.severity {
                x_compiler::backend::DiagnosticSeverity::Error => {
                    println!("  {} {}", "Error:".red().bold(), diagnostic.message);
                }
                x_compiler::backend::DiagnosticSeverity::Warning => {
                    println!("  {} {}", "Warning:".yellow().bold(), diagnostic.message);
                }
                x_compiler::backend::DiagnosticSeverity::Info => {
                    println!("  {} {}", "Info:".blue().bold(), diagnostic.message);
                }
            }
        }
        println!();
    }
    
    // Display generated files
    println!("Generated {} files:", result.files.len());
    for file_path in result.files.keys() {
        println!("  {}", file_path.display().to_string().green());
    }
    
    print_success(&format!("Successfully compiled to {}", target));
    
    Ok(())
}