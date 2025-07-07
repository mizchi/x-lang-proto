//! x Language compiler CLI

use clap::{Args, Parser, Subcommand};
use x_compiler::{
    CompilerBuilder, CompilerConfig, TargetConfig, config::presets, convenience,
};
use x_parser::SyntaxStyle;
use std::path::PathBuf;
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "x-lang")]
#[command(about = "x Language compiler with multi-target support")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile source code
    Compile(CompileArgs),
    
    /// Type check source code
    Check(CheckArgs),
    
    /// Parse source code and show AST
    Parse(ParseArgs),
    
    /// Show available targets
    Targets,
    
    /// Create default configuration file
    InitConfig {
        /// Output path for configuration file
        #[arg(short, long, default_value = "x-lang.toml")]
        output: PathBuf,
    },
    
    /// Validate configuration file
    ValidateConfig {
        /// Configuration file path
        path: PathBuf,
    },
}

#[derive(Args)]
struct CompileArgs {
    /// Input source file or directory
    input: PathBuf,
    
    /// Output directory
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
    
    /// Target language/platform
    #[arg(short, long, default_value = "typescript")]
    target: String,
    
    /// Syntax style for parsing
    #[arg(short, long, default_value = "ocaml")]
    syntax: String,
    
    /// Optimization level (0-3)
    #[arg(short = 'O', long, default_value = "0")]
    optimization: u8,
    
    /// Enable debug information
    #[arg(long)]
    debug: bool,
    
    /// Enable source maps
    #[arg(long)]
    source_maps: bool,
    
    /// Enable type emission
    #[arg(long)]
    emit_types: bool,
    
    /// Watch mode (recompile on changes)
    #[arg(short, long)]
    watch: bool,
}

#[derive(Args)]
struct CheckArgs {
    /// Input source file or directory
    input: PathBuf,
    
    /// Syntax style for parsing
    #[arg(short, long, default_value = "ocaml")]
    syntax: String,
}

#[derive(Args)]
struct ParseArgs {
    /// Input source file
    input: PathBuf,
    
    /// Syntax style for parsing
    #[arg(short, long, default_value = "ocaml")]
    syntax: String,
    
    /// Output format (json, debug, pretty)
    #[arg(short, long, default_value = "pretty")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    
    match cli.command {
        Commands::Compile(args) => {
            handle_compile(args, cli.config).await?;
        }
        Commands::Check(args) => {
            handle_check(args).await?;
        }
        Commands::Parse(args) => {
            handle_parse(args).await?;
        }
        Commands::Targets => {
            handle_targets().await?;
        }
        Commands::InitConfig { output } => {
            handle_init_config(output).await?;
        }
        Commands::ValidateConfig { path } => {
            handle_validate_config(path).await?;
        }
    }
    
    Ok(())
}

async fn handle_compile(args: CompileArgs, config_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Compiling {} to {}", args.input.display(), args.target);
    
    // Load configuration
    let mut config = if let Some(path) = config_path {
        CompilerConfig::from_file(&path)?
    } else {
        CompilerConfig::default()
    };
    
    // Override with CLI arguments
    config.syntax_style = parse_syntax_style(&args.syntax)?;
    config.optimization_level = args.optimization;
    config.debug_info = args.debug;
    config.source_maps = args.source_maps;
    config.emit_types = args.emit_types;
    
    // Set target-specific configuration
    let target_config = create_target_config(&args.target)?;
    config.set_target_config(&args.target, target_config);
    
    // Create compiler
    let mut compiler = CompilerBuilder::new()
        .syntax_style(config.syntax_style)
        .optimization_level(config.optimization_level)
        .debug_info(config.debug_info)
        .source_maps(config.source_maps)
        .target_config(&args.target, config.target_config(&args.target))
        .build();
    
    // Compile
    if args.input.is_file() {
        // Single file
        let result = compiler.compile_file(&args.input, &args.target, args.output.clone())?;
        
        info!("Compilation successful!");
        info!("  Parse time: {:?}", result.metadata.parse_time);
        info!("  Type check time: {:?}", result.metadata.check_time);
        info!("  Code generation time: {:?}", result.metadata.codegen_time);
        info!("  Total time: {:?}", result.metadata.total_time);
        info!("  Generated {} files", result.metadata.generated_files);
        
        // Show diagnostics
        for diagnostic in &result.diagnostics {
            match diagnostic.severity {
                x_compiler::backend::DiagnosticSeverity::Error => {
                    error!("{}", diagnostic.message);
                }
                x_compiler::backend::DiagnosticSeverity::Warning => {
                    warn!("{}", diagnostic.message);
                }
                x_compiler::backend::DiagnosticSeverity::Info => {
                    info!("{}", diagnostic.message);
                }
            }
        }
        
        if args.watch {
            info!("Watch mode not implemented yet");
        }
    } else {
        // Directory compilation not implemented yet
        return Err("Directory compilation not implemented yet".into());
    }
    
    Ok(())
}

async fn handle_check(args: CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
    info!("Type checking {}", args.input.display());
    
    let source = std::fs::read_to_string(&args.input)?;
    let syntax_style = parse_syntax_style(&args.syntax)?;
    
    // Parse and type check
    let _ast = convenience::parse_source_only(&source, syntax_style)?;
    let check_result = convenience::type_check_source(&source)?;
    
    if check_result.errors.is_empty() {
        info!("Type checking successful!");
        
        if !check_result.warnings.is_empty() {
            info!("Warnings:");
            for warning in &check_result.warnings {
                warn!("  {}", warning);
            }
        }
    } else {
        error!("Type checking failed:");
        for error in &check_result.errors {
            error!("  {}", error);
        }
        
        return Err("Type checking failed".into());
    }
    
    Ok(())
}

async fn handle_parse(args: ParseArgs) -> Result<(), Box<dyn std::error::Error>> {
    info!("Parsing {}", args.input.display());
    
    let source = std::fs::read_to_string(&args.input)?;
    let syntax_style = parse_syntax_style(&args.syntax)?;
    
    let ast = convenience::parse_source_only(&source, syntax_style)?;
    
    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&ast)?;
            println!("{}", json);
        }
        "debug" => {
            println!("{:#?}", ast);
        }
        "pretty" => {
            println!("Parse successful!");
            println!("Module: {}", ast.module.name.to_string());
            println!("Imports: {}", ast.module.imports.len());
            println!("Exports: {}", ast.module.exports.as_ref().map(|e| e.items.len()).unwrap_or(0));
            println!("Items: {}", ast.module.items.len());
        }
        _ => {
            return Err(format!("Unknown format: {}", args.format).into());
        }
    }
    
    Ok(())
}

async fn handle_targets() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = CompilerBuilder::new().build();
    let targets = compiler.available_targets();
    
    println!("Available targets:");
    for target in targets {
        println!("  {}", target);
    }
    
    Ok(())
}

async fn handle_init_config(output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating configuration file at {}", output.display());
    
    let mut config = CompilerConfig::default();
    
    // Add some default target configurations
    config.set_target_config("typescript", presets::typescript_dev());
    config.set_target_config("wasm-component", presets::wasm_component());
    config.set_target_config("wit", presets::wit_only());
    
    config.to_file(&output)?;
    
    info!("Configuration file created successfully!");
    
    Ok(())
}

async fn handle_validate_config(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Validating configuration file {}", path.display());
    
    let config = CompilerConfig::from_file(&path)?;
    config.validate()?;
    
    info!("Configuration file is valid!");
    
    Ok(())
}

fn parse_syntax_style(style: &str) -> Result<SyntaxStyle, Box<dyn std::error::Error>> {
    match style.to_lowercase().as_str() {
        "haskell" => Ok(SyntaxStyle::Haskell),
        "sexp" | "s-expression" => Ok(SyntaxStyle::SExpression),
        _ => Err(format!("Unknown syntax style: {}. Supported styles: haskell, sexp", style).into()),
    }
}

fn create_target_config(target: &str) -> Result<TargetConfig, Box<dyn std::error::Error>> {
    match target {
        "typescript" | "ts" => Ok(presets::typescript_dev()),
        "wasm-gc" => Ok(presets::wasm_dev()),
        "wasm-component" | "component" => Ok(presets::wasm_component()),
        "wit" => Ok(presets::wit_only()),
        _ => Ok(TargetConfig::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_syntax_style() {
        assert_eq!(parse_syntax_style("ocaml").unwrap(), SyntaxStyle::OCaml);
        assert_eq!(parse_syntax_style("sexp").unwrap(), SyntaxStyle::SExpression);
        assert_eq!(parse_syntax_style("haskell").unwrap(), SyntaxStyle::Haskell);
        assert_eq!(parse_syntax_style("rust").unwrap(), SyntaxStyle::RustLike);
        
        assert!(parse_syntax_style("invalid").is_err());
    }
    
    #[test]
    fn test_create_target_config() {
        let config = create_target_config("typescript").unwrap();
        assert!(config.enabled);
        
        let config = create_target_config("wasm-component").unwrap();
        assert!(config.enabled);
        
        let config = create_target_config("unknown").unwrap();
        assert!(config.enabled);
    }
}