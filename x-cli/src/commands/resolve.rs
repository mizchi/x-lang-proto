//! Resolve content-addressed references to functions

use anyhow::{Result, Context};
use clap::Args;
use std::path::PathBuf;
use std::fs;
use x_parser::{Parser, FileId, Symbol, ast};
use x_parser::metadata::ContentHash;
use colored::*;
use crate::version_db;

/// Resolve content-addressed references
#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Input file or hash
    input: String,
    
    /// Show source code
    #[arg(short, long)]
    source: bool,
    
    /// Show metadata
    #[arg(short, long)]
    metadata: bool,
}

pub async fn run(args: ResolveArgs) -> Result<()> {
    // Check if input looks like a hash
    if args.input.len() == 64 && args.input.chars().all(|c| c.is_ascii_hexdigit()) {
        resolve_hash(&args.input, args.source, args.metadata).await
    } else {
        // Treat as file path
        resolve_file(&PathBuf::from(&args.input), args.source, args.metadata).await
    }
}

async fn resolve_hash(hash: &str, show_source: bool, show_metadata: bool) -> Result<()> {
    println!("{}", "Content Resolution:".bold().underline());
    println!();
    
    let project_root = std::path::Path::new(".");
    let db = version_db::load_db(project_root)?;
    
    // Find functions with this hash
    let mut found = false;
    for (_name, versions) in db.functions {
        for version in &versions.versions {
            if version.hash.0 == hash {
                found = true;
                println!("{} {} {}",
                    "Found:".green().bold(),
                    versions.name.cyan(),
                    format!("v{}.{}.{}", 
                        version.version.major, 
                        version.version.minor, 
                        version.version.patch
                    ).yellow()
                );
                
                if show_metadata {
                    println!("\n{}", "Metadata:".dimmed());
                    println!("  {} {}", "Created:".dimmed(), 
                        version.created_at.split('T').next().unwrap_or(""));
                    println!("  {} {} parameters",
                        "Signature:".dimmed(),
                        version.signature.param_count
                    );
                    if !version.signature.effects.is_empty() {
                        println!("  {} {}",
                            "Effects:".dimmed(),
                            version.signature.effects.join(", ")
                        );
                    }
                    if let Some(notes) = &version.release_notes {
                        println!("  {} {}", "Notes:".dimmed(), notes);
                    }
                }
                
                if show_source {
                    println!("\n{}", "Source:".dimmed());
                    println!("  {}", 
                        "(Source reconstruction from hash not yet implemented)".dimmed()
                    );
                }
                
                println!();
            }
        }
    }
    
    if !found {
        println!("{} No function found with hash: {}", 
            "Warning:".yellow(), 
            hash
        );
    }
    
    Ok(())
}

async fn resolve_file(input: &PathBuf, show_source: bool, show_metadata: bool) -> Result<()> {
    let source = fs::read_to_string(input)?;
    let file_id = FileId::new(0);
    let mut parser = Parser::new(&source, file_id)?;
    let ast = parser.parse()?;
    
    println!("{}", "File Content Hashes:".bold().underline());
    println!();
    
    for item in &ast.module.items {
        if let ast::Item::ValueDef(def) = item {
            let hash = x_parser::content_hash::hash_value_def(def);
            
            println!("{} -> {}",
                def.name.as_str().cyan(),
                hash.yellow()
            );
            
            if show_metadata {
                let deps = x_parser::dependency::DependencyManager::extract_dependencies_from_def(def);
                if !deps.is_empty() {
                    println!("  {} {}",
                        "Dependencies:".dimmed(),
                        deps.iter()
                            .map(|d| d.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                            .blue()
                    );
                }
            }
            
            if show_source {
                println!("  {} let {} = ...",
                    "Source:".dimmed(),
                    def.name.as_str()
                );
            }
            
            println!();
        }
    }
    
    Ok(())
}