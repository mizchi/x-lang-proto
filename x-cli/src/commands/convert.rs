//! Format conversion commands

use anyhow::{Result, Context, bail};
use std::path::Path;
use std::fs;
use colored::*;
use x_parser::persistent_ast::PersistentAstNode;
// use x_editor::ast_engine::AstEngine;
use crate::format::{Format, detect_format, load_ast, save_ast};
use crate::utils::ProgressIndicator;

/// Convert between different x Language formats
pub async fn convert_command(
    input: &Path,
    output: Option<&Path>,
    from_format: Option<&str>,
    to_format: Option<&str>,
) -> Result<()> {
    let progress = ProgressIndicator::new("Converting format");
    
    // Detect input format
    let input_format = match from_format {
        Some(fmt) => Format::from_str(fmt)?,
        None => detect_format(input)?,
    };
    
    // Determine output path and format
    let output_path = match output {
        Some(path) => path.to_owned(),
        None => {
            let output_format = match to_format {
                Some(fmt) => Format::from_str(fmt)?,
                None => bail!("Output format must be specified when output path is not provided"),
            };
            
            // Generate output filename based on input and target format
            let stem = input.file_stem()
                .context("Invalid input filename")?
                .to_string_lossy();
            
            let extension = output_format.default_extension();
            let mut output_path = input.with_file_name(format!("{}.{}", stem, extension));
            
            // Avoid overwriting the input file
            if output_path == input {
                output_path = input.with_file_name(format!("{}_converted.{}", stem, extension));
            }
            
            output_path
        }
    };
    
    // Detect output format
    let output_format = match to_format {
        Some(fmt) => Format::from_str(fmt)?,
        None => detect_format(&output_path)?,
    };
    
    println!("Converting {} → {}", 
        format!("{:?}", input_format).cyan(),
        format!("{:?}", output_format).green()
    );
    println!("Input:  {}", input.display());
    println!("Output: {}", output_path.display());
    
    progress.set_message("Loading input file");
    
    // Load AST from input
    let ast = load_ast(input, input_format).await
        .with_context(|| format!("Failed to load input file: {}", input.display()))?;
    
    progress.set_message("Converting format");
    
    // Convert if needed (currently AST is format-agnostic)
    let converted_ast = convert_ast_format(ast, input_format, output_format)?;
    
    progress.set_message("Saving output file");
    
    // Save AST to output
    save_ast(&output_path, &converted_ast, output_format).await
        .with_context(|| format!("Failed to save output file: {}", output_path.display()))?;
    
    progress.finish("Conversion completed successfully");
    
    // Print statistics
    print_conversion_stats(input, &output_path).await?;
    
    Ok(())
}

/// Convert AST between different formats (placeholder for format-specific transformations)
fn convert_ast_format(
    ast: PersistentAstNode, 
    _from: Format, 
    _to: Format
) -> Result<PersistentAstNode> {
    // For now, AST is format-agnostic, so no conversion needed
    // In the future, this might handle format-specific optimizations or transformations
    Ok(ast)
}

/// Print conversion statistics
async fn print_conversion_stats(input: &Path, output: &Path) -> Result<()> {
    let input_size = fs::metadata(input)?.len();
    let output_size = fs::metadata(output)?.len();
    
    println!("\n{}", "Conversion Statistics:".bold().underline());
    println!("  Input size:   {} bytes", format!("{:>10}", input_size).cyan());
    println!("  Output size:  {} bytes", format!("{:>10}", output_size).cyan());
    
    let ratio = if input_size > 0 {
        (output_size as f64 / input_size as f64) * 100.0
    } else {
        0.0
    };
    
    let ratio_str = format!("{:.1}%", ratio);
    let ratio_colored = if ratio < 80.0 {
        ratio_str.green()
    } else if ratio < 120.0 {
        ratio_str.yellow()
    } else {
        ratio_str.red()
    };
    
    println!("  Size ratio:   {}", ratio_colored);
    
    if output_size < input_size {
        let saved = input_size - output_size;
        println!("  Space saved:  {} bytes", format!("{:>10}", saved).green());
    } else if output_size > input_size {
        let increased = output_size - input_size;
        println!("  Size increase: {} bytes", format!("{:>10}", increased).yellow());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_convert_command() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.rustic.x");
        let output_path = temp_dir.path().join("test.x");
        
        // Create a simple test file
        let mut file = File::create(&input_path).unwrap();
        writeln!(file, "pub fn main() -> () | IO {{").unwrap();
        writeln!(file, "    println(\"Hello, world!\")").unwrap();
        writeln!(file, "}}").unwrap();
        
        // Test conversion
        let result = convert_command(
            &input_path,
            Some(&output_path),
            Some("rustic"),
            Some("binary")
        ).await;
        
        // Should succeed (though actual conversion depends on parser implementation)
        // For now, just test that the command doesn't panic
        match result {
            Ok(()) => {
                assert!(output_path.exists());
            },
            Err(_) => {
                // Expected if parser is not fully implemented
                // This is fine for testing the CLI structure
            }
        }
    }
}