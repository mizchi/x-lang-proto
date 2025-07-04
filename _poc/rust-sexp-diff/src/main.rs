//! Command-line interface for the S-expression diff tool

use clap::{Parser, Subcommand};
use sexp_diff::{
    parser::parse,
    serializer::{serialize, deserialize},
    diff::{StructuralDiff, DiffFormatter, DiffSummary},
    hash::ContentHash,
    Result,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "sexp-diff")]
#[command(about = "High-performance S-expression parser and structural diff tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an S-expression file and display the AST
    Parse {
        /// Input file path
        file: String,
        /// Show content hash
        #[arg(long)]
        hash: bool,
        /// Show binary representation
        #[arg(long)]
        binary: bool,
        /// Pretty print with indentation
        #[arg(long)]
        pretty: bool,
    },
    /// Compile S-expression to binary format
    Compile {
        /// Input S-expression file
        input: String,
        /// Output binary file (optional)
        output: Option<String>,
    },
    /// Compare two S-expression files
    Diff {
        /// First file
        file1: String,
        /// Second file
        file2: String,
        /// Show structural diff
        #[arg(long)]
        structural: bool,
        /// Compact output (changes only)
        #[arg(long)]
        compact: bool,
        /// Disable colors
        #[arg(long)]
        no_color: bool,
        /// Hide path information
        #[arg(long)]
        no_paths: bool,
        /// Include timing information
        #[arg(long)]
        time: bool,
    },
    /// Compare binary representations
    BinaryDiff {
        /// First file
        file1: String,
        /// Second file
        file2: String,
    },
    /// Benchmark parsing and diff performance
    Bench {
        /// Input file for benchmarking
        file: String,
        /// Number of iterations
        #[arg(long, default_value = "1000")]
        iterations: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, hash, binary, pretty } => {
            parse_command(&file, hash, binary, pretty)?;
        }
        Commands::Compile { input, output } => {
            compile_command(&input, output)?;
        }
        Commands::Diff { file1, file2, structural, compact, no_color, no_paths, time } => {
            diff_command(&file1, &file2, structural, compact, no_color, no_paths, time)?;
        }
        Commands::BinaryDiff { file1, file2 } => {
            binary_diff_command(&file1, &file2)?;
        }
        Commands::Bench { file, iterations } => {
            bench_command(&file, iterations)?;
        }
    }

    Ok(())
}

fn parse_command(file: &str, show_hash: bool, show_binary: bool, pretty: bool) -> Result<()> {
    let sexp = load_sexp_file(file)?;
    
    if pretty {
        println!("Parsed AST:");
        println!("{}", sexp.to_pretty_string(0));
    } else {
        println!("Parsed AST:");
        println!("{}", serde_json::to_string_pretty(&sexp).unwrap());
    }

    if show_hash {
        let hash = ContentHash::hash(&sexp);
        let short_hash = ContentHash::short_hash(&sexp);
        println!("\nContent Hash: {}", short_hash);
        println!("Full Hash: {}", hash);
    }

    if show_binary {
        let binary = serialize(&sexp)?;
        println!("\nBinary size: {} bytes", binary.len());
        println!("Binary (hex): {}", hex::encode(&binary[..binary.len().min(32)]));
        if binary.len() > 32 {
            println!("... (truncated)");
        }
    }

    Ok(())
}

fn compile_command(input: &str, output: Option<String>) -> Result<()> {
    let sexp = load_sexp_file(input)?;
    let binary = serialize(&sexp)?;
    
    let output_path = output.unwrap_or_else(|| {
        if input.ends_with(".s") {
            input.replace(".s", ".s.bin")
        } else {
            format!("{}.bin", input)
        }
    });

    fs::write(&output_path, &binary)?;
    let hash = ContentHash::short_hash(&sexp);
    
    println!("Compiled: {} -> {}", input, output_path);
    println!("Size: {} bytes", binary.len());
    println!("Content Hash: {}", hash);

    Ok(())
}

fn diff_command(
    file1: &str,
    file2: &str,
    structural: bool,
    compact: bool,
    no_color: bool,
    no_paths: bool,
    show_time: bool,
) -> Result<()> {
    let start_time = Instant::now();
    
    let sexp1 = load_sexp_file(file1)?;
    let sexp2 = load_sexp_file(file2)?;
    
    let parse_time = start_time.elapsed();
    let diff_start = Instant::now();

    println!("Comparing {} and {}:", file1, file2);
    println!();

    let diff_engine = StructuralDiff::new().include_unchanged(!compact);
    let results = diff_engine.diff(&sexp1, &sexp2);
    
    let diff_time = diff_start.elapsed();

    let formatter = DiffFormatter::new()
        .compact(compact)
        .no_color(no_color)
        .hide_paths(no_paths);
    
    let output = formatter.format(&results);
    print!("{}", output);

    let summary = DiffSummary::from_results(&results);
    println!("{}", summary);

    if show_time {
        println!("\nTiming:");
        println!("  Parse time: {:?}", parse_time);
        println!("  Diff time: {:?}", diff_time);
        println!("  Total time: {:?}", start_time.elapsed());
    }

    Ok(())
}

fn binary_diff_command(file1: &str, file2: &str) -> Result<()> {
    let sexp1 = load_sexp_file(file1)?;
    let sexp2 = load_sexp_file(file2)?;
    
    let binary1 = serialize(&sexp1)?;
    let binary2 = serialize(&sexp2)?;
    
    let hash1 = ContentHash::hash(&sexp1);
    let hash2 = ContentHash::hash(&sexp2);
    let short_hash1 = ContentHash::short_hash(&sexp1);
    let short_hash2 = ContentHash::short_hash(&sexp2);

    println!("{}:", file1);
    println!("  Size: {} bytes", binary1.len());
    println!("  Hash: {}", hash1);
    println!("  Content Hash: {}", short_hash1);

    println!("\n{}:", file2);
    println!("  Size: {} bytes", binary2.len());
    println!("  Hash: {}", hash2);
    println!("  Content Hash: {}", short_hash2);

    if hash1 == hash2 {
        println!("\n✓ Files are identical");
    } else {
        println!("\n✗ Files are different");
        let size_diff = binary2.len() as i64 - binary1.len() as i64;
        if size_diff > 0 {
            println!("Size difference: +{} bytes", size_diff);
        } else if size_diff < 0 {
            println!("Size difference: {} bytes", size_diff);
        }
    }

    Ok(())
}

fn bench_command(file: &str, iterations: usize) -> Result<()> {
    println!("Benchmarking with {} iterations...", iterations);
    
    let content = fs::read_to_string(file)?;
    
    // Parse benchmark
    let start = Instant::now();
    let mut last_sexp = None;
    for _ in 0..iterations {
        let sexp = parse(&content)?;
        last_sexp = Some(sexp);
    }
    let parse_time = start.elapsed();
    
    let sexp = last_sexp.unwrap();
    println!("Parse time: {:?} ({:.2} µs/op)", parse_time, parse_time.as_micros() as f64 / iterations as f64);
    
    // Serialization benchmark
    let start = Instant::now();
    let mut last_binary = None;
    for _ in 0..iterations {
        let binary = serialize(&sexp)?;
        last_binary = Some(binary);
    }
    let serialize_time = start.elapsed();
    
    let binary = last_binary.unwrap();
    println!("Serialize time: {:?} ({:.2} µs/op)", serialize_time, serialize_time.as_micros() as f64 / iterations as f64);
    
    // Deserialization benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = deserialize(&binary)?;
    }
    let deserialize_time = start.elapsed();
    
    println!("Deserialize time: {:?} ({:.2} µs/op)", deserialize_time, deserialize_time.as_micros() as f64 / iterations as f64);
    
    // Hash benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = ContentHash::hash(&sexp);
    }
    let hash_time = start.elapsed();
    
    println!("Hash time: {:?} ({:.2} µs/op)", hash_time, hash_time.as_micros() as f64 / iterations as f64);
    
    // Diff benchmark (against itself)
    let start = Instant::now();
    let diff_engine = StructuralDiff::new();
    for _ in 0..(iterations / 10).max(1) { // Fewer iterations for diff
        let _ = diff_engine.diff(&sexp, &sexp);
    }
    let diff_time = start.elapsed();
    let diff_iterations = (iterations / 10).max(1);
    
    println!("Diff time: {:?} ({:.2} µs/op)", diff_time, diff_time.as_micros() as f64 / diff_iterations as f64);
    
    println!("\nFile info:");
    println!("  Original size: {} bytes", content.len());
    println!("  Binary size: {} bytes ({:.1}% of original)", binary.len(), binary.len() as f64 / content.len() as f64 * 100.0);
    println!("  Content hash: {}", ContentHash::short_hash(&sexp));

    Ok(())
}

fn load_sexp_file(file_path: &str) -> Result<sexp_diff::SExp> {
    let path = Path::new(file_path);
    
    if file_path.ends_with(".bin") || file_path.ends_with(".s.bin") {
        // Load binary file
        let binary_data = fs::read(file_path)?;
        println!("Loaded from binary format");
        deserialize(&binary_data)
    } else {
        // Load text file
        let content = fs::read_to_string(file_path)?;
        parse(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_command() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.s");
        fs::write(&file_path, "(+ 1 2)").unwrap();
        
        // Test basic parsing
        parse_command(file_path.to_str().unwrap(), false, false, false).unwrap();
    }

    #[test]
    fn test_compile_command() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.s");
        let output_path = dir.path().join("test.s.bin");
        
        fs::write(&input_path, "(+ 1 2)").unwrap();
        
        compile_command(
            input_path.to_str().unwrap(),
            Some(output_path.to_str().unwrap().to_string())
        ).unwrap();
        
        assert!(output_path.exists());
    }
}