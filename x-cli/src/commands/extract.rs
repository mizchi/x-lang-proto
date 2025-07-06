//! Extract command - extract dependencies and generate focused code
//! 
//! Inspired by Unison's ability to extract and work with specific definitions
//! and their dependencies.

use anyhow::{Result, Context, bail};
use clap::Args;
use std::path::PathBuf;
use std::fs;
use x_parser::{
    Parser, 
    dependency::{DependencyManager, DependencyCodeGenerator},
    symbol::Symbol,
    span::FileId,
    syntax::{SyntaxStyle, SyntaxConfig},
};

/// Extract dependencies for specific definitions
#[derive(Debug, Args)]
pub struct ExtractArgs {
    /// Input file path
    input: PathBuf,
    
    /// Names of definitions to extract (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    names: Vec<String>,
    
    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Include only direct dependencies
    #[arg(short = 'd', long)]
    direct_only: bool,
    
    /// Generate minimal code (no comments or formatting)
    #[arg(short = 'm', long)]
    minimal: bool,
    
    /// Show dependency tree instead of generating code
    #[arg(short = 't', long)]
    tree: bool,
    
    /// Check for circular dependencies
    #[arg(long)]
    check_cycles: bool,
    
    /// Generate with explicit imports at function level
    #[arg(short = 'i', long)]
    explicit_imports: bool,
}

pub async fn run(args: ExtractArgs) -> Result<()> {
    // Read input file
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read file: {}", args.input.display()))?;
    
    // Parse the file
    let file_id = FileId::new(0);
    let mut parser = Parser::new(&source, file_id)?;
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error details: {}", e);
            return Err(e).with_context(|| "Failed to parse input file");
        }
    };
    
    // Skip type checking for now
    let typed_ast = ast;
    
    // Build dependency graph
    let mut dep_manager = DependencyManager::new();
    
    // Add all definitions to the dependency manager
    for item in &typed_ast.module.items {
        match item {
            x_parser::ast::Item::ValueDef(def) => {
                let deps = DependencyManager::extract_dependencies_from_def(def);
                dep_manager.add_definition(def.name, deps);
            }
            x_parser::ast::Item::TypeDef(def) => {
                // Type definitions don't have expression dependencies
                dep_manager.add_definition(def.name, Default::default());
            }
            x_parser::ast::Item::EffectDef(def) => {
                dep_manager.add_definition(def.name, Default::default());
            }
            x_parser::ast::Item::TestDef(def) => {
                let deps = DependencyManager::extract_dependencies(&def.body);
                dep_manager.add_definition(def.name, deps);
            }
            _ => {
                // Skip other item types for now
            }
        }
    }
    
    // Compute transitive dependencies
    dep_manager.compute_transitive_deps();
    
    // Check for circular dependencies if requested
    if args.check_cycles {
        let cycles = dep_manager.find_circular_dependencies();
        if !cycles.is_empty() {
            eprintln!("Circular dependencies detected:");
            for cycle in cycles {
                eprintln!("  {}", cycle.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> "));
            }
            if !args.tree {
                bail!("Cannot extract code with circular dependencies");
            }
        }
    }
    
    // Convert string names to symbols
    let root_symbols: Vec<Symbol> = args.names.iter()
        .map(|name| Symbol::intern(name))
        .collect();
    
    // Validate that all requested names exist
    for name in &root_symbols {
        if dep_manager.get_all_dependencies(name).is_none() {
            bail!("Definition '{}' not found in {}", name.as_str(), args.input.display());
        }
    }
    
    if args.tree {
        // Display dependency tree
        display_dependency_tree(&dep_manager, &root_symbols, args.direct_only)?;
    } else {
        // Generate code with dependencies
        let generator = DependencyCodeGenerator::new(dep_manager);
        let ordered_defs = generator.generate_with_dependencies(&root_symbols);
        
        // Generate the output
        let output = generate_extracted_code(
            &typed_ast,
            &ordered_defs,
            args.explicit_imports,
            args.minimal
        )?;
        
        // Write output
        if let Some(output_path) = args.output {
            fs::write(&output_path, output)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            println!("Extracted {} definitions to {}", ordered_defs.len(), output_path.display());
        } else {
            print!("{}", output);
        }
    }
    
    Ok(())
}

fn display_dependency_tree(
    manager: &DependencyManager,
    roots: &[Symbol],
    direct_only: bool
) -> Result<()> {
    println!("Dependency tree:");
    
    for root in roots {
        print_tree_node(manager, *root, "", true, direct_only, &mut Default::default())?;
    }
    
    Ok(())
}

fn print_tree_node(
    manager: &DependencyManager,
    name: Symbol,
    prefix: &str,
    is_last: bool,
    direct_only: bool,
    visited: &mut std::collections::HashSet<Symbol>
) -> Result<()> {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, connector, name.as_str());
    
    if visited.contains(&name) {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        println!("{}(circular reference)", new_prefix);
        return Ok(());
    }
    
    visited.insert(name);
    
    if let Some(deps) = manager.get_all_dependencies(&name) {
        let deps_vec: Vec<_> = if direct_only {
            // Only show direct dependencies
            manager.definitions.get(&name)
                .map(|def| def.direct_deps.iter().cloned().collect())
                .unwrap_or_default()
        } else {
            deps.iter().cloned().collect()
        };
        
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        
        for (i, dep) in deps_vec.iter().enumerate() {
            let is_last_dep = i == deps_vec.len() - 1;
            print_tree_node(manager, *dep, &new_prefix, is_last_dep, direct_only, visited)?;
        }
    }
    
    visited.remove(&name);
    Ok(())
}

fn generate_extracted_code(
    ast: &x_parser::ast::CompilationUnit,
    ordered_defs: &[Symbol],
    explicit_imports: bool,
    minimal: bool
) -> Result<String> {
    use x_parser::syntax::{haskell::HaskellPrinter, SyntaxPrinter};
    
    let mut output = String::new();
    
    // Generate module header
    if !minimal {
        output.push_str(&format!("-- Extracted definitions: {}\n", 
            ordered_defs.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")));
        output.push_str(&format!("-- Total: {} definitions\n\n", ordered_defs.len()));
    }
    
    output.push_str(&format!("module {} where\n\n", ast.module.name.to_string()));
    
    // Create a set for quick lookup
    let def_set: std::collections::HashSet<_> = ordered_defs.iter().cloned().collect();
    
    // Generate imports if using explicit imports
    if explicit_imports {
        let mut imports = std::collections::HashSet::new();
        
        // Collect all external dependencies
        for name in ordered_defs {
            if let Some(item) = find_item_by_name(ast, *name) {
                if let x_parser::ast::Item::ValueDef(def) = item {
                    let deps = DependencyManager::extract_dependencies(&def.body);
                    for dep in deps {
                        if !def_set.contains(&dep) {
                            imports.insert(dep);
                        }
                    }
                }
            }
        }
        
        if !imports.is_empty() {
            output.push_str("-- External dependencies\n");
            for import in imports {
                output.push_str(&format!("import {}\n", import.as_str()));
            }
            output.push_str("\n");
        }
    }
    
    // Generate definitions in dependency order
    let printer = HaskellPrinter::new();
    let config = SyntaxConfig {
        style: SyntaxStyle::Haskell,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: !minimal,
    };
    
    for name in ordered_defs {
        if let Some(item) = find_item_by_name(ast, *name) {
            // Create a temporary AST with just this item
            let temp_ast = x_parser::ast::CompilationUnit {
                module: x_parser::ast::Module {
                    name: ast.module.name.clone(),
                    documentation: None,
                    exports: None,
                    imports: vec![],
                    items: vec![item.clone()],
                    span: ast.module.span.clone(),
                },
                span: ast.span.clone(),
            };
            
            let item_code = printer.print(&temp_ast, &config)?;
            
            // Extract just the definition part (skip module header)
            if let Some(start) = item_code.find("where") {
                let def_code = item_code[start + 5..].trim();
                if !minimal {
                    output.push_str(&format!("-- Definition: {}\n", name.as_str()));
                }
                output.push_str(def_code);
                output.push_str("\n\n");
            }
        }
    }
    
    Ok(output)
}

fn find_item_by_name(ast: &x_parser::ast::CompilationUnit, name: Symbol) -> Option<&x_parser::ast::Item> {
    ast.module.items.iter().find(|item| {
        match item {
            x_parser::ast::Item::ValueDef(def) => def.name == name,
            x_parser::ast::Item::TypeDef(def) => def.name == name,
            x_parser::ast::Item::EffectDef(def) => def.name == name,
            x_parser::ast::Item::TestDef(def) => def.name == name,
            _ => false,
        }
    })
}