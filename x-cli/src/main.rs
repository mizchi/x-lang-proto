//! x Language CLI - Direct AST manipulation and conversion tools
//! 
//! This CLI provides comprehensive tools for working with x Language's binary AST format,
//! including conversion, editing, querying, and analysis operations.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber;

mod commands;
mod config;
mod format;
mod interactive;
mod utils;

use commands::*;
use config::CliConfig;

/// x Language CLI - Direct AST manipulation and conversion tools
#[derive(Parser)]
#[command(name = "x")]
#[command(about = "x Language CLI for binary AST manipulation")]
#[command(version)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, global = true, default_value = "auto")]
    format: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new x Language project
    New {
        /// Project name
        name: String,
        /// Project directory (defaults to name)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    
    /// Convert between different formats
    Convert {
        /// Input file (.x, .rustic.x, .ocaml.x, etc.)
        input: PathBuf,
        /// Output file (format determined by extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source format (auto-detect if not specified)
        #[arg(long)]
        from: Option<String>,
        /// Target format (auto-detect from output extension if not specified)
        #[arg(long)]
        to: Option<String>,
    },
    
    /// Display AST information
    Show {
        /// Input file
        input: PathBuf,
        /// Display format (tree, json, summary, compact, ocaml, haskell, sexp)
        #[arg(short, long, default_value = "tree")]
        format: String,
        /// Maximum depth to display
        #[arg(short, long)]
        depth: Option<usize>,
        /// Show type information
        #[arg(long)]
        types: bool,
        /// Show spans
        #[arg(long)]
        spans: bool,
    },
    
    /// Query AST nodes
    Query {
        /// Input file
        input: PathBuf,
        /// Query expression
        query: String,
        /// Output format (json, table, tree)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Edit AST directly
    Edit {
        /// Input file
        input: PathBuf,
        /// Output file (defaults to input)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Edit commands file or inline command
        #[arg(short, long)]
        commands: Option<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    
    /// Rename symbols throughout the AST
    Rename {
        /// Input file
        input: PathBuf,
        /// Old symbol name
        from: String,
        /// New symbol name
        to: String,
        /// Output file (defaults to input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Extract method refactoring
    Extract {
        /// Input file
        input: PathBuf,
        /// Start position (line:column)
        start: String,
        /// End position (line:column)
        end: String,
        /// Method name
        name: String,
        /// Output file (defaults to input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Type check the AST
    Check {
        /// Input file or directory
        input: PathBuf,
        /// Show detailed type information
        #[arg(long)]
        detailed: bool,
        /// Check only (don't show types)
        #[arg(long)]
        quiet: bool,
    },
    
    /// Compile to target language
    Compile {
        /// Input file
        input: PathBuf,
        /// Target language (typescript, wasm, wasm-component)
        #[arg(short, long, default_value = "typescript")]
        target: String,
        /// Output directory
        #[arg(short, long, default_value = "./dist")]
        output: PathBuf,
    },
    
    /// Start interactive REPL
    Repl {
        /// Preload file
        #[arg(short, long)]
        preload: Option<PathBuf>,
        /// Syntax style for REPL
        #[arg(long, default_value = "rustic")]
        syntax: String,
    },
    
    /// Language server
    Lsp {
        /// Server mode (stdio, tcp)
        #[arg(long, default_value = "stdio")]
        mode: String,
        /// TCP port (for tcp mode)
        #[arg(long, default_value = "9257")]
        port: u16,
    },
    
    /// Analyze project statistics
    Stats {
        /// Input file or directory
        input: PathBuf,
        /// Output format (json, table)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.verbose)?;
    
    // Load configuration
    let _config = CliConfig::load(cli.config.as_deref())?;
    
    // Execute command
    let result = match cli.command {
        Commands::New { name, dir } => {
            new_command(&name, dir.as_deref()).await
        },
        Commands::Convert { input, output, from, to } => {
            convert_command(&input, output.as_deref(), from.as_deref(), to.as_deref()).await
        },
        Commands::Show { input, format, depth, types, spans } => {
            show_command(&input, &format, depth, types, spans).await
        },
        Commands::Query { input, query, format } => {
            query_command(&input, &query, &format).await
        },
        Commands::Edit { input, output, commands, interactive } => {
            edit_command(&input, output.as_deref(), commands.as_deref(), interactive).await
        },
        Commands::Rename { input: _, from: _, to: _, output: _ } => {
            // rename_command(&input, &from, &to, output.as_deref()).await
            println!("Rename command not yet implemented");
            Ok(())
        },
        Commands::Extract { input: _, start: _, end: _, name: _, output: _ } => {
            // extract_command(&input, &start, &end, &name, output.as_deref()).await
            println!("Extract command not yet implemented");
            Ok(())
        },
        Commands::Check { input, detailed, quiet } => {
            check_command(&input, detailed, quiet).await
        },
        Commands::Compile { input, target, output } => {
            compile_command(&input, &target, &output).await
        },
        Commands::Repl { preload, syntax } => {
            repl_command(preload.as_deref(), &syntax).await
        },
        Commands::Lsp { mode, port } => {
            lsp_command(&mode, port).await
        },
        Commands::Stats { input, format } => {
            stats_command(&input, &format).await
        },
    };
    
    match result {
        Ok(()) => {
            info!("Command completed successfully");
            Ok(())
        },
        Err(e) => {
            error!("Command failed: {}", e);
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn init_logging(verbose: bool) -> Result<()> {
    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
    
    Ok(())
}

#[allow(dead_code)]
fn print_banner() {
    println!("{}", r#"
 ╔═══════════════════════════════════════════════════════════════════════╗
 ║                          x Language CLI                              ║
 ║                  Direct AST Manipulation Tools                       ║
 ╚═══════════════════════════════════════════════════════════════════════╝
"#.cyan().bold());
}