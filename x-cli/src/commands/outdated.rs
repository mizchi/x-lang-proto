//! Check for outdated dependencies in the codebase

use anyhow::{Result, Context};
use clap::Args;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use x_parser::{Parser, FileId};
use x_parser::versioning::Version;
use x_parser::dependency::DependencyManager;
use colored::*;
use crate::version_db::VersionDatabase;

/// Check for outdated dependencies
#[derive(Debug, Args)]
pub struct OutdatedArgs {
    /// Input file or directory
    input: PathBuf,
    /// Check all versions (not just latest stable)
    #[arg(short, long)]
    all: bool,
    /// Show detailed information
    #[arg(short, long)]
    detailed: bool,
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

pub async fn run(args: OutdatedArgs) -> Result<()> {
    let db = VersionDatabase::load_default()?;
    
    // Parse the input file
    let content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;
    
    let mut parser = Parser::new(&content, FileId::new(0))?;
    let compilation_unit = parser.parse()
        .with_context(|| format!("Failed to parse: {}", args.input.display()))?;
    
    let module = &compilation_unit.module;
    
    // Collect all function dependencies
    let mut outdated_deps = Vec::new();
    
    for item in &module.items {
        if let x_parser::ast::Item::ValueDef(def) = item {
            // Extract dependencies from the function
            let deps = DependencyManager::extract_dependencies_from_def(def);
            
            // Check each dependency
            for dep_name in deps {
                // Check imports for version specifications
                let imported_version = find_imported_version(module, &dep_name);
                
                // Get the latest version from the database
                if let Some(func_versions) = db.functions.get(dep_name.as_str()) {
                    if let Some(latest) = &func_versions.latest {
                        // Compare with imported version
                        if let Some(spec) = imported_version {
                            if !is_latest(&spec, latest) {
                                outdated_deps.push(OutdatedDependency {
                                    function_name: def.name,
                                    dependency_name: dep_name,
                                    current_version: spec.clone(),
                                    latest_version: latest.clone(),
                                    usage_count: count_usages(&def.body, &dep_name),
                                });
                            }
                        } else {
                            // No version specified, might be using an old version
                            outdated_deps.push(OutdatedDependency {
                                function_name: def.name,
                                dependency_name: dep_name,
                                current_version: "unspecified".to_string(),
                                latest_version: latest.clone(),
                                usage_count: count_usages(&def.body, &dep_name),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // Display results
    match args.format.as_str() {
        "json" => display_json(&outdated_deps)?,
        _ => display_text(&outdated_deps, args.detailed),
    }
    
    Ok(())
}

#[derive(Debug)]
struct OutdatedDependency {
    function_name: x_parser::symbol::Symbol,
    dependency_name: x_parser::symbol::Symbol,
    current_version: String,
    latest_version: Version,
    usage_count: usize,
}

fn find_imported_version(
    module: &x_parser::ast::Module, 
    dep_name: &x_parser::symbol::Symbol
) -> Option<String> {
    // Check module-level imports
    for import in &module.imports {
        // Check if this import contains the dependency
        if import.module_path.segments.last() == Some(dep_name) {
            return import.version_spec.clone();
        }
        
        // Check selective imports
        if let x_parser::ast::ImportKind::Selective(items) = &import.kind {
            for item in items {
                if &item.name == dep_name {
                    return item.version_spec.clone()
                        .or_else(|| import.version_spec.clone());
                }
            }
        }
    }
    
    None
}

fn is_latest(version_spec: &str, latest: &Version) -> bool {
    // Parse version spec and check if it includes the latest version
    if version_spec == "latest" {
        return true;
    }
    
    if version_spec.starts_with("^") {
        // Compatible version - check if latest is compatible
        if let Ok(base_version) = parse_version(&version_spec[1..]) {
            return latest.major == base_version.major && 
                   (latest.minor > base_version.minor || 
                    (latest.minor == base_version.minor && latest.patch >= base_version.patch));
        }
    } else if version_spec.starts_with("=") {
        // Exact version - check if it matches latest
        if let Ok(exact_version) = parse_version(&version_spec[1..]) {
            return latest == &exact_version;
        }
    }
    
    false
}

fn parse_version(s: &str) -> Result<Version> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid version format");
    }
    
    Ok(Version::new(
        parts[0].parse()?,
        parts[1].parse()?,
        parts[2].parse()?,
    ))
}

fn count_usages(expr: &x_parser::ast::Expr, name: &x_parser::symbol::Symbol) -> usize {
    use x_parser::ast::Expr;
    
    match expr {
        Expr::Var(var_name, _) if var_name == name => 1,
        Expr::App(func, args, _) => {
            count_usages(func, name) + 
            args.iter().map(|arg| count_usages(arg, name)).sum::<usize>()
        }
        Expr::Lambda { body, .. } => count_usages(body, name),
        Expr::Let { value, body, .. } => {
            count_usages(value, name) + count_usages(body, name)
        }
        Expr::If { condition, then_branch, else_branch, .. } => {
            count_usages(condition, name) + 
            count_usages(then_branch, name) + 
            count_usages(else_branch, name)
        }
        _ => 0,
    }
}

fn display_text(outdated: &[OutdatedDependency], detailed: bool) {
    if outdated.is_empty() {
        println!("{} All dependencies are up to date!", "✓".green());
        return;
    }
    
    println!("{}", "Outdated Dependencies:".bold().underline());
    println!();
    
    // Group by function
    let mut by_function: HashMap<x_parser::symbol::Symbol, Vec<&OutdatedDependency>> = HashMap::new();
    for dep in outdated {
        by_function.entry(dep.function_name).or_default().push(dep);
    }
    
    for (func_name, deps) in by_function {
        println!("{} {}:", "►".blue(), func_name.as_str().cyan());
        
        for dep in deps {
            let current = if dep.current_version == "unspecified" {
                dep.current_version.dimmed()
            } else {
                dep.current_version.yellow()
            };
            
            println!("    {} {} @ {} → {}",
                "•".dimmed(),
                dep.dependency_name.as_str(),
                current,
                format!("{}.{}.{}", 
                    dep.latest_version.major,
                    dep.latest_version.minor,
                    dep.latest_version.patch
                ).green()
            );
            
            if detailed {
                println!("      {} {} usage(s)",
                    "Uses:".dimmed(),
                    dep.usage_count
                );
            }
        }
        println!();
    }
    
    // Summary
    let total = outdated.len();
    let unspecified = outdated.iter()
        .filter(|d| d.current_version == "unspecified")
        .count();
    
    println!("{}", "Summary:".bold());
    println!("  {} {} outdated dependencies", 
        total.to_string().red(), 
        if total == 1 { "dependency is" } else { "dependencies are" }
    );
    
    if unspecified > 0 {
        println!("  {} {} without version specification", 
            unspecified.to_string().yellow(),
            if unspecified == 1 { "dependency" } else { "dependencies" }
        );
    }
}

fn display_json(outdated: &[OutdatedDependency]) -> Result<()> {
    let json = serde_json::json!({
        "outdated": outdated.iter().map(|dep| {
            serde_json::json!({
                "function": dep.function_name.as_str(),
                "dependency": dep.dependency_name.as_str(),
                "current_version": dep.current_version,
                "latest_version": {
                    "major": dep.latest_version.major,
                    "minor": dep.latest_version.minor,
                    "patch": dep.latest_version.patch,
                },
                "usage_count": dep.usage_count,
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "total_outdated": outdated.len(),
            "unspecified": outdated.iter()
                .filter(|d| d.current_version == "unspecified")
                .count(),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}