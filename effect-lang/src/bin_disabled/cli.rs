//! x Language CLI tool for binary AST operations

use clap::{Parser, Subcommand};
use effect_lang::{
    analysis::parser::parse,
    core::{
        binary::{BinarySerializer, BinaryDeserializer},
        diff::{BinaryAstDiffer, format_diff},
        span::FileId,
    },
    Result,
};
use std::{
    fs,
    path::PathBuf,
    time::Instant,
};

#[derive(Parser)]
#[command(name = "effect-cli")]
#[command(about = "x Language CLI tool for binary AST operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse x Language source and serialize to binary AST
    Compile {
        /// Input source file (.eff)
        #[arg(short, long)]
        input: PathBuf,
        /// Output binary file (.eff.bin)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Show detailed timing information
        #[arg(long)]
        timing: bool,
    },
    /// Compare two binary AST files and show structural diff
    Diff {
        /// First file to compare
        file1: PathBuf,
        /// Second file to compare
        file2: PathBuf,
        /// Output format: text, json
        #[arg(long, default_value = "text")]
        format: String,
        /// Show detailed diff information
        #[arg(long)]
        verbose: bool,
    },
    /// Analyze binary AST file and show statistics
    Analyze {
        /// Binary AST file to analyze
        file: PathBuf,
        /// Show content hash
        #[arg(long)]
        hash: bool,
        /// Show size comparison with source
        #[arg(long)]
        size: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, output, timing } => {
            compile_command(input, output, timing)
        }
        Commands::Diff { file1, file2, format, verbose } => {
            diff_command(file1, file2, format, verbose)
        }
        Commands::Analyze { file, hash, size } => {
            analyze_command(file, hash, size)
        }
    }
}

fn compile_command(input: PathBuf, output: Option<PathBuf>, timing: bool) -> Result<()> {
    let start_time = Instant::now();
    
    // Read source file
    let source = fs::read_to_string(&input)?;
    let parse_start = Instant::now();
    
    // Parse AST
    let ast = parse(&source, FileId::new(0))?;
    let parse_time = parse_start.elapsed();
    
    if timing {
        println!("Parse time: {:?}", parse_time);
    }
    
    let serialize_start = Instant::now();
    
    // Serialize to binary
    let mut serializer = BinarySerializer::new();
    let binary_data = serializer.serialize_compilation_unit(&ast)?;
    let serialize_time = serialize_start.elapsed();
    
    if timing {
        println!("Serialize time: {:?}", serialize_time);
        println!("Total time: {:?}", start_time.elapsed());
    }
    
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("eff.bin");
        path
    });
    
    // Write binary file
    fs::write(&output_path, &binary_data)?;
    
    // Calculate content hash
    let content_hash = BinarySerializer::content_hash(&binary_data);
    
    println!("Compiled {} to {}", input.display(), output_path.display());
    println!("Binary size: {} bytes", binary_data.len());
    println!("Content hash: {}", content_hash);
    
    // Size comparison
    let source_size = source.len();
    let compression_ratio = binary_data.len() as f64 / source_size as f64;
    println!("Compression ratio: {:.2}x (source: {} bytes)", compression_ratio, source_size);
    
    Ok(())
}

fn diff_command(file1: PathBuf, file2: PathBuf, format: String, verbose: bool) -> Result<()> {
    let start_time = Instant::now();
    
    // Read binary files
    let data1 = fs::read(&file1)?;
    let data2 = fs::read(&file2)?;
    
    // Deserialize ASTs
    let mut deserializer1 = BinaryDeserializer::new(data1);
    let mut deserializer2 = BinaryDeserializer::new(data2);
    
    let ast1 = deserializer1.deserialize_compilation_unit()?;
    let ast2 = deserializer2.deserialize_compilation_unit()?;
    
    // Compute diff
    let mut differ = BinaryAstDiffer::new();
    let diff_ops = differ.diff_compilation_units(&ast1, &ast2)?;
    
    let diff_time = start_time.elapsed();
    
    if verbose {
        println!("Diff computed in {:?}", diff_time);
        println!("Number of diff operations: {}", diff_ops.len());
        println!();
    }
    
    // Output diff
    match format.as_str() {
        "json" => {
            // For now, just output structured text
            // In a full implementation, this would be proper JSON
            println!("[JSON format not yet implemented]");
            println!("{}", format_diff(&diff_ops));
        }
        "text" | _ => {
            println!("Diff between {} and {}:", file1.display(), file2.display());
            println!("{}", format_diff(&diff_ops));
        }
    }
    
    Ok(())
}

fn analyze_command(file: PathBuf, show_hash: bool, show_size: bool) -> Result<()> {
    // Read binary file
    let binary_data = fs::read(&file)?;
    
    // Deserialize AST
    let mut deserializer = BinaryDeserializer::new(binary_data.clone());
    let ast = deserializer.deserialize_compilation_unit()?;
    
    println!("Analysis of {}", file.display());
    println!("Binary size: {} bytes", binary_data.len());
    
    if show_hash {
        let content_hash = BinarySerializer::content_hash(&binary_data);
        println!("Content hash: {}", content_hash);
    }
    
    if show_size {
        // Try to find corresponding source file
        let source_path = if file.extension().and_then(|s| s.to_str()) == Some("bin") {
            let mut path = file.clone();
            path.set_extension("eff");
            path
        } else {
            let mut path = file.clone();
            path.set_extension("eff");
            path
        };
        
        if source_path.exists() {
            let source_size = fs::metadata(&source_path)?.len();
            let compression_ratio = binary_data.len() as f64 / source_size as f64;
            println!("Source size: {} bytes", source_size);
            println!("Compression ratio: {:.2}x", compression_ratio);
        }
    }
    
    // AST statistics
    println!();
    println!("AST Statistics:");
    println!("Module: {}", ast.module.name.to_string());
    println!("Items: {}", ast.module.items.len());
    println!("Imports: {}", ast.module.imports.len());
    
    Ok(())
}