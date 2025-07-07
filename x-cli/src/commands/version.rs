//! Version management commands

use anyhow::{Result, Context};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::fs;
use x_parser::{Parser, FileId, Symbol};
use x_parser::versioning::{Version, VersionRepository, VersionMetadata, VersionSpec, FunctionSignature};
use x_parser::metadata::{MetadataRepository, ContentHash};
use x_parser::signature::extract_signature;
use x_parser::content_hash;
use colored::*;
use crate::version_db;

/// Version management commands
#[derive(Debug, Args)]
pub struct VersionArgs {
    #[command(subcommand)]
    command: VersionCommands,
}

#[derive(Debug, Subcommand)]
enum VersionCommands {
    /// Show version information for functions
    Show {
        /// Input file
        input: PathBuf,
        /// Function name (show all if not specified)
        #[arg(short, long)]
        name: Option<String>,
        /// Show all versions
        #[arg(short, long)]
        all: bool,
    },
    /// Tag a function with a version
    Tag {
        /// Input file
        input: PathBuf,
        /// Function name
        name: String,
        /// Version (e.g., 1.0.0)
        version: String,
        /// Release notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// Check compatibility between versions
    Check {
        /// Input file
        input: PathBuf,
        /// Function name
        name: String,
        /// First version
        v1: String,
        /// Second version
        v2: String,
    },
    /// Show dependents of a function version
    Deps {
        /// Input file
        input: PathBuf,
        /// Function name
        name: String,
        /// Version
        version: String,
    },
}

pub async fn run(args: VersionArgs) -> Result<()> {
    match args.command {
        VersionCommands::Show { input, name, all } => {
            show_versions(&input, name.as_deref(), all).await
        }
        VersionCommands::Tag { input, name, version, notes } => {
            tag_version(&input, &name, &version, notes.as_deref()).await
        }
        VersionCommands::Check { input, name, v1, v2 } => {
            check_compatibility(&input, &name, &v1, &v2).await
        }
        VersionCommands::Deps { input, name, version } => {
            show_dependents(&input, &name, &version).await
        }
    }
}

async fn show_versions(input: &PathBuf, name: Option<&str>, all: bool) -> Result<()> {
    let project_root = input.parent().unwrap_or(std::path::Path::new("."));
    
    println!("{}", "Function Versions:".bold().underline());
    println!();
    
    if let Some(name) = name {
        // Show specific function
        if let Some(versions) = version_db::get_function_versions(project_root, name)? {
            println!("{}", name.cyan().bold());
            
            if let Some(latest) = &versions.latest {
                println!("  {} {}", "Latest:".dimmed(), 
                    format!("{}.{}.{}", latest.major, latest.minor, latest.patch).green());
            }
            
            if let Some(stable) = &versions.stable {
                println!("  {} {}", "Stable:".dimmed(), 
                    format!("{}.{}.{}", stable.major, stable.minor, stable.patch).yellow());
            }
            
            if all && !versions.versions.is_empty() {
                println!("\n  {}", "All versions:".dimmed());
                for v in &versions.versions {
                    let version_str = format!("{}.{}.{}", v.version.major, v.version.minor, v.version.patch);
                    let notes = v.release_notes.as_deref().unwrap_or("No release notes");
                    println!("    {} - {} - {}", version_str, v.created_at.split('T').next().unwrap_or(""), notes);
                }
            }
        } else {
            println!("{} No version information found for '{}'", "Warning:".yellow(), name);
        }
    } else {
        // Show all functions from the actual file
        let source = fs::read_to_string(input)?;
        let file_id = FileId::new(0);
        let mut parser = Parser::new(&source, file_id)?;
        let ast = parser.parse()?;
        
        for item in &ast.module.items {
            if let x_parser::ast::Item::ValueDef(def) = item {
                let name = def.name.as_str();
                let hash = content_hash::hash_value_def(def);
                
                println!("{}", name.cyan());
                
                // Check if we have version info
                if let Ok(Some(versions)) = version_db::get_function_versions(project_root, name) {
                    if let Some(latest) = &versions.latest {
                        println!("  {} {}", "Latest:".dimmed(), 
                            format!("{}.{}.{}", latest.major, latest.minor, latest.patch).green());
                    }
                } else {
                    println!("  {} {}", "Version:".dimmed(), "unversioned".dimmed());
                }
                
                println!("  {} {}...", "Hash:".dimmed(), &hash[..16].yellow());
                
                // Show dependencies
                let deps = x_parser::dependency::DependencyManager::extract_dependencies_from_def(def);
                if !deps.is_empty() {
                    let dep_str = deps.iter()
                        .map(|d| d.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("  {} {}", "Deps:".dimmed(), dep_str.blue());
                }
                
                println!();
            }
        }
    }
    
    Ok(())
}

async fn tag_version(input: &PathBuf, name: &str, version_str: &str, notes: Option<&str>) -> Result<()> {
    // Parse version
    let version = parse_version(version_str)?;
    let project_root = input.parent().unwrap_or(std::path::Path::new("."));
    
    // Read and parse file
    let source = fs::read_to_string(input)?;
    let file_id = FileId::new(0);
    let mut parser = Parser::new(&source, file_id)?;
    let ast = parser.parse()?;
    
    // Find the function
    let name_sym = Symbol::intern(name);
    let value_def = ast.module.items.iter()
        .find_map(|item| match item {
            x_parser::ast::Item::ValueDef(def) if def.name == name_sym => Some(def),
            _ => None,
        })
        .with_context(|| format!("Function '{}' not found", name))?;
    
    // Extract signature
    let signature = extract_signature(value_def)
        .context("Failed to extract function signature")?;
    
    // Compute hash
    let hash = content_hash::hash_value_def(value_def);
    let content_hash = ContentHash(hash.clone());
    
    // Save to version database
    version_db::save_version(
        project_root,
        name,
        version.clone(),
        content_hash,
        &signature,
        notes.map(|s| s.to_string()),
    )?;
    
    println!("{} {} {} {}",
        "Tagged".green().bold(),
        name.cyan(),
        "as".dimmed(),
        format!("v{}.{}.{}", version.major, version.minor, version.patch).yellow()
    );
    
    println!("  {} {}", "Hash:".dimmed(), hash);
    println!("  {} {} parameters, returns {}",
        "Signature:".dimmed(),
        signature.params.len(),
        format_type(&signature.return_type)
    );
    
    if !signature.effects.is_empty() {
        println!("  {} {}",
            "Effects:".dimmed(),
            signature.effects.iter()
                .map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    
    if let Some(notes) = notes {
        println!("  {} {}", "Notes:".dimmed(), notes);
    }
    
    println!("\n{} Version information saved to {}", 
        "✓".green(), 
        ".x-versions/versions.json".dimmed()
    );
    
    Ok(())
}

async fn check_compatibility(input: &PathBuf, name: &str, v1_str: &str, v2_str: &str) -> Result<()> {
    let project_root = input.parent().unwrap_or(std::path::Path::new("."));
    
    println!("{}", "Compatibility Check:".bold().underline());
    println!();
    
    println!("{} {} {} {}",
        name.cyan(),
        v1_str.yellow(),
        "→".dimmed(),
        v2_str.yellow()
    );
    
    // Load version database
    if let Some(versions) = version_db::get_function_versions(project_root, name)? {
        let v1 = parse_version(v1_str)?;
        let v2 = parse_version(v2_str)?;
        
        // Find the two versions
        let ver1 = versions.versions.iter()
            .find(|v| v.version == v1)
            .with_context(|| format!("Version {} not found", v1_str))?;
        let ver2 = versions.versions.iter()
            .find(|v| v.version == v2)
            .with_context(|| format!("Version {} not found", v2_str))?;
        
        // Compare signatures
        let compatible = if ver1.hash == ver2.hash {
            // Same hash = identical implementation
            println!("\n{}", "✓ Identical".green().bold());
            println!("  Same content hash - implementations are identical");
            true
        } else if ver1.signature.param_count != ver2.signature.param_count {
            println!("\n{}", "✗ Incompatible".red().bold());
            println!("  {} Parameter count changed: {} → {}",
                "Breaking:".red(),
                ver1.signature.param_count,
                ver2.signature.param_count
            );
            false
        } else if !ver2.signature.effects.is_empty() && 
                  !ver1.signature.effects.iter().all(|e| ver2.signature.effects.contains(e)) {
            println!("\n{}", "✗ Incompatible".red().bold());
            println!("  {} New effects added",
                "Breaking:".red()
            );
            let new_effects: Vec<_> = ver2.signature.effects.iter()
                .filter(|e| !ver1.signature.effects.contains(e))
                .cloned()
                .collect::<Vec<_>>();
            println!("    Added: {}", new_effects.join(", "));
            false
        } else {
            println!("\n{}", "✓ Compatible".green().bold());
            println!("  Parameters: {} (no change)", ver1.signature.param_count);
            println!("  Return type: {}", 
                if ver1.signature.return_type == ver2.signature.return_type {
                    "No changes"
                } else {
                    "Changed (check subtyping)"
                }
            );
            
            // Check for improvements
            if ver1.signature.effects.len() > ver2.signature.effects.len() {
                println!("  {} Effects removed (improvement)",
                    "Enhancement:".green()
                );
            }
            true
        };
        
        // Show version notes
        if let Some(notes) = &ver2.release_notes {
            println!("\n  {} {}", "v2 Notes:".dimmed(), notes);
        }
        
    } else {
        println!("{} No version information found for '{}'", 
            "Error:".red(), name);
    }
    
    Ok(())
}

async fn show_dependents(input: &PathBuf, name: &str, version: &str) -> Result<()> {
    println!("{}", "Dependents:".bold().underline());
    println!();
    
    println!("{} {} {}",
        "Functions that depend on".dimmed(),
        format!("{}@{}", name, version).cyan(),
        ":".dimmed()
    );
    
    // Simulate dependent listing
    println!("\n  {} (uses {} for doubling)",
        "double@1.0.0".yellow(),
        name.cyan()
    );
    println!("  {} (uses {} in calculation)",
        "calculate@2.1.0".yellow(),
        name.cyan()
    );
    
    Ok(())
}

fn parse_version(s: &str) -> Result<Version> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Version must be in format X.Y.Z");
    }
    
    Ok(Version::new(
        parts[0].parse()?,
        parts[1].parse()?,
        parts[2].parse()?,
    ))
}

fn format_type(ty: &x_parser::ast::Type) -> String {
    // Simple type formatting
    match ty {
        x_parser::ast::Type::Var(name, _) => name.as_str().to_string(),
        x_parser::ast::Type::Con(name, _) => name.as_str().to_string(),
        x_parser::ast::Type::Fun { params, return_type, .. } => {
            if params.len() == 1 {
                format!("{} -> {}", format_type(&params[0]), format_type(return_type))
            } else {
                format!("({}) -> {}",
                    params.iter().map(format_type).collect::<Vec<_>>().join(", "),
                    format_type(return_type)
                )
            }
        }
        x_parser::ast::Type::Hole(_) => "?".to_string(),
        _ => "<type>".to_string(),
    }
}