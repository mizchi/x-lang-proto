//! Hash command - compute and display content hashes for definitions

use anyhow::{Result, Context};
use clap::Args;
use std::path::PathBuf;
use std::fs;
use x_parser::{Parser, FileId, content_hash, metadata::{ContentHash, MetadataRepository, DefinitionMetadata}};

/// Compute content hashes for definitions
#[derive(Debug, Args)]
pub struct HashArgs {
    /// Input file path
    input: PathBuf,
    
    /// Show all definitions (not just specified ones)
    #[arg(short, long)]
    all: bool,
    
    /// Names of specific definitions to hash
    #[arg(short, long, value_delimiter = ',')]
    names: Vec<String>,
    
    /// Output format (text, json)
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
    
    /// Show definition content
    #[arg(short = 's', long)]
    show_content: bool,
}

pub async fn run(args: HashArgs) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;
    
    // Parse the file
    let file_id = FileId::new(0);
    let mut parser = Parser::new(&source, file_id)?;
    let ast = parser.parse()
        .with_context(|| "Failed to parse input file")?;
    
    // Build metadata repository
    let mut metadata_repo = MetadataRepository::new();
    
    // Process all definitions
    for item in &ast.module.items {
        match item {
            x_parser::ast::Item::ValueDef(def) => {
                let hash = content_hash::hash_value_def(def);
                let metadata = DefinitionMetadata {
                    name: def.name,
                    hash: ContentHash(hash.clone()),
                    dependencies: x_parser::dependency::DependencyManager::extract_dependencies_from_def(def),
                    source_info: None,
                    documentation: extract_doc_string(&def.documentation),
                    type_signature: format_type_signature(&def.type_annotation),
                    is_exported: matches!(def.visibility, x_parser::ast::Visibility::Public),
                };
                metadata_repo.store_definition(metadata);
            }
            _ => {
                // Skip other item types for now
            }
        }
    }
    
    // Filter definitions to show
    let definitions_to_show = if args.all {
        metadata_repo.all_hashes()
    } else if !args.names.is_empty() {
        args.names.iter()
            .map(|n| ContentHash(n.clone()))
            .collect()
    } else {
        // Show all if no specific names given
        metadata_repo.all_hashes()
    };
    
    // Display results
    match args.format.as_str() {
        "json" => display_json(&metadata_repo, &definitions_to_show)?,
        _ => display_text(&metadata_repo, &definitions_to_show, args.show_content)?,
    }
    
    Ok(())
}

fn display_text(
    repo: &MetadataRepository,
    hashes: &Vec<ContentHash>,
    show_content: bool
) -> Result<()> {
    use colored::*;
    
    println!("{}", "Content Hashes:".bold().underline());
    println!();
    
    for hash in hashes {
        if let Some(metadata) = repo.lookup_by_hash(hash) {
            println!("{} {}", 
                metadata.name.as_str().cyan().bold(),
                format!("({})", if metadata.is_exported { "public" } else { "private" }).dimmed()
            );
            
            println!("  {} {}", "Hash:".dimmed(), hash.0.yellow());
            
            if let Some(type_sig) = &metadata.type_signature {
                println!("  {} {}", "Type:".dimmed(), type_sig.green());
            }
            
            if !metadata.dependencies.is_empty() {
                let deps: Vec<_> = metadata.dependencies.iter()
                    .map(|s| s.as_str())
                    .collect();
                println!("  {} {}", "Deps:".dimmed(), deps.join(", ").blue());
            }
            
            if let Some(doc) = &metadata.documentation {
                println!("  {} {}", "Docs:".dimmed(), doc.dimmed());
            }
            
            if show_content {
                if let Some(names) = repo.get_names_for_hash(hash) {
                    let aliases: Vec<_> = names.iter()
                        .filter(|n| **n != metadata.name)
                        .map(|n| n.as_str())
                        .collect();
                    if !aliases.is_empty() {
                        println!("  {} {}", "Also:".dimmed(), aliases.join(", ").dimmed());
                    }
                }
            }
            
            println!();
        }
    }
    
    Ok(())
}

fn display_json(
    repo: &MetadataRepository,
    hashes: &Vec<ContentHash>
) -> Result<()> {
    use serde_json::json;
    
    let mut results = Vec::new();
    
    for hash in hashes {
        if let Some(metadata) = repo.lookup_by_hash(hash) {
            let entry = json!({
                "name": metadata.name.as_str(),
                "hash": hash.0,
                "type": metadata.type_signature,
                "dependencies": metadata.dependencies.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
                "documentation": metadata.documentation,
                "exported": metadata.is_exported,
            });
            results.push(entry);
        }
    }
    
    let output = json!({
        "definitions": results
    });
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    
    Ok(())
}

fn extract_doc_string(doc: &Option<x_parser::ast::Documentation>) -> Option<String> {
    doc.as_ref().map(|d| d.doc_comment.content.clone())
}

fn format_type_signature(type_ann: &Option<x_parser::ast::Type>) -> Option<String> {
    type_ann.as_ref().map(|t| format_type(t))
}

fn format_type(ty: &x_parser::ast::Type) -> String {
    match ty {
        x_parser::ast::Type::Var(name, _) => name.as_str().to_string(),
        x_parser::ast::Type::Con(name, _) => name.as_str().to_string(),
        x_parser::ast::Type::App(func, args, _) => {
            format!("{} {}", 
                format_type(func),
                args.iter().map(format_type).collect::<Vec<_>>().join(" ")
            )
        }
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
        _ => "<complex type>".to_string(),
    }
}