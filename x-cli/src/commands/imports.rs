//! Extract and display import information with version specifications

use anyhow::{Result, Context};
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use std::fs;
use x_parser::{Parser, FileId};
use colored::*;

/// Extract and display import information
#[derive(Debug, Args)]
pub struct ImportsArgs {
    /// Input file or directory
    input: PathBuf,
    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
    /// Show transitive dependencies
    #[arg(short, long)]
    transitive: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Tree,
}

pub async fn run(args: ImportsArgs) -> Result<()> {
    let content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;
    
    let mut parser = Parser::new(&content, FileId::new(0))?;
    let compilation_unit = parser.parse()
        .with_context(|| format!("Failed to parse: {}", args.input.display()))?;
    
    let module = &compilation_unit.module;
    
    match args.format {
        OutputFormat::Text => display_imports_text(module),
        OutputFormat::Json => display_imports_json(module)?,
        OutputFormat::Tree => display_imports_tree(module),
    }
    
    Ok(())
}

fn display_imports_text(module: &x_parser::ast::Module) {
    println!("{}", "Module-level imports:".bold().underline());
    
    if module.imports.is_empty() {
        println!("  {} No imports", "○".dimmed());
    } else {
        for import in &module.imports {
            let module_name = import.module_path.to_string();
            let version_str = import.version_spec.as_ref()
                .map(|v| format!("@{}", v.green()))
                .unwrap_or_default();
            
            match &import.kind {
                x_parser::ast::ImportKind::Qualified => {
                    println!("  {} {}{}", 
                        "◆".cyan(), 
                        module_name.yellow(),
                        version_str
                    );
                }
                x_parser::ast::ImportKind::Selective(items) => {
                    println!("  {} {}{} {{ {} }}", 
                        "◆".cyan(),
                        module_name.yellow(),
                        version_str,
                        items.iter()
                            .map(|item| {
                                let item_version = item.version_spec.as_ref()
                                    .map(|v| format!("@{}", v))
                                    .unwrap_or_default();
                                format!("{}{}", item.name.as_str(), item_version)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                x_parser::ast::ImportKind::Wildcard => {
                    println!("  {} {}.*{}", 
                        "◆".cyan(),
                        module_name.yellow(),
                        version_str
                    );
                }
                x_parser::ast::ImportKind::Lazy => {
                    println!("  {} {} {}{}", 
                        "◆".cyan(),
                        "lazy".dimmed(),
                        module_name.yellow(),
                        version_str
                    );
                }
                _ => {}
            }
            
            if let Some(alias) = &import.alias {
                println!("    {} {}", "as".dimmed(), alias.as_str().italic());
            }
        }
    }
    
    println!("\n{}", "Function-level imports:".bold().underline());
    
    let mut function_imports = Vec::new();
    
    for item in &module.items {
        match item {
            x_parser::ast::Item::ValueDef(def) => {
                if !def.imports.is_empty() {
                    function_imports.push((def.name, &def.imports));
                }
            }
            x_parser::ast::Item::TestDef(def) => {
                if !def.imports.is_empty() {
                    function_imports.push((def.name, &def.imports));
                }
            }
            _ => {}
        }
    }
    
    if function_imports.is_empty() {
        println!("  {} No function-level imports", "○".dimmed());
    } else {
        for (func_name, imports) in function_imports {
            println!("  {} {}:", "►".blue(), func_name.as_str().cyan());
            for import in imports.iter() {
                let version_str = import.version_spec.as_ref()
                    .map(|v| format!("@{}", v.green()))
                    .unwrap_or_default();
                let alias_str = import.alias.as_ref()
                    .map(|a| format!(" as {}", a.as_str()))
                    .unwrap_or_default();
                println!("    - {}{}{}", 
                    import.name.as_str(),
                    version_str,
                    alias_str
                );
            }
        }
    }
}

fn display_imports_json(module: &x_parser::ast::Module) -> Result<()> {
    let json = serde_json::json!({
        "module": module.name.to_string(),
        "imports": module.imports.iter().map(|import| {
            serde_json::json!({
                "module": import.module_path.to_string(),
                "version": import.version_spec,
                "alias": import.alias.as_ref().map(|s| s.as_str()),
                "kind": format!("{:?}", import.kind),
            })
        }).collect::<Vec<_>>(),
        "function_imports": module.items.iter().filter_map(|item| {
            match item {
                x_parser::ast::Item::ValueDef(def) if !def.imports.is_empty() => {
                    Some(serde_json::json!({
                        "function": def.name.as_str(),
                        "imports": def.imports.iter().map(|imp| {
                            serde_json::json!({
                                "name": imp.name.as_str(),
                                "version": imp.version_spec,
                                "alias": imp.alias.as_ref().map(|s| s.as_str()),
                            })
                        }).collect::<Vec<_>>()
                    }))
                }
                _ => None
            }
        }).collect::<Vec<_>>()
    });
    
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn display_imports_tree(module: &x_parser::ast::Module) {
    println!("{}", module.name.to_string().bold());
    
    for (i, import) in module.imports.iter().enumerate() {
        let is_last = i == module.imports.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        
        let version_str = import.version_spec.as_ref()
            .map(|v| format!("@{}", v.green()))
            .unwrap_or_default();
        
        println!("{}{}{}", prefix, import.module_path.to_string().yellow(), version_str);
        
        if let x_parser::ast::ImportKind::Selective(items) = &import.kind {
            for (j, item) in items.iter().enumerate() {
                let item_is_last = j == items.len() - 1;
                let item_prefix = if is_last { "    " } else { "│   " };
                let item_branch = if item_is_last { "└── " } else { "├── " };
                
                let item_version = item.version_spec.as_ref()
                    .map(|v| format!("@{}", v.green()))
                    .unwrap_or_default();
                
                println!("{}{}{}{}", item_prefix, item_branch, item.name.as_str(), item_version);
            }
        }
    }
}