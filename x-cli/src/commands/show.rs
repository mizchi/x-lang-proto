//! AST display and inspection commands

use anyhow::{Result, Context};
use std::path::Path;
use colored::*;
use serde_json;
use x_parser::persistent_ast::{PersistentAstNode, AstNodeKind, Purity};
use x_parser::syntax::{SyntaxConfig, SyntaxStyle};
use x_parser::syntax::ocaml::OCamlPrinter;
use x_parser::syntax::haskell::HaskellPrinter;
use x_parser::syntax::sexp::SExpPrinter;
use x_parser::syntax::SyntaxPrinter;
use crate::format::{detect_format, load_ast};
use crate::utils::ProgressIndicator;

/// Display AST information in various formats
pub async fn show_command(
    input: &Path,
    format: &str,
    depth: Option<usize>,
    show_types: bool,
    show_spans: bool,
) -> Result<()> {
    let progress = ProgressIndicator::new("Loading AST");
    
    // Load AST
    let input_format = detect_format(input)?;
    let ast = load_ast(input, input_format).await
        .with_context(|| format!("Failed to load AST from: {}", input.display()))?;
    
    progress.finish("AST loaded successfully");
    
    println!("File: {}", input.display().to_string().cyan());
    println!("Format: {:?}", input_format);
    println!();
    
    // Display AST based on requested format
    match format {
        "tree" => show_tree(&ast, depth, show_types, show_spans)?,
        "json" => show_json(&ast, depth)?,
        "summary" => show_summary(&ast)?,
        "compact" => show_compact(&ast, depth)?,
        "ocaml" => show_ocaml(&ast, depth)?,
        "haskell" => show_haskell(&ast, depth)?,
        "sexp" => show_sexp(&ast, depth)?,
        _ => {
            eprintln!("{} Unknown display format: {}", "Error:".red().bold(), format);
            eprintln!("Available formats: tree, json, summary, compact, ocaml, haskell, sexp");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Display AST as a tree structure
fn show_tree(
    ast: &PersistentAstNode, 
    max_depth: Option<usize>, 
    show_types: bool, 
    show_spans: bool
) -> Result<()> {
    println!("{}", "AST Tree:".bold().underline());
    print_tree_node(ast, 0, max_depth, show_types, show_spans, true);
    Ok(())
}

/// Recursively print AST nodes as a tree
fn print_tree_node(
    node: &PersistentAstNode,
    depth: usize,
    max_depth: Option<usize>,
    show_types: bool,
    show_spans: bool,
    is_last: bool,
) {
    // Check depth limit
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }
    
    // Print indentation
    for i in 0..depth {
        if i == depth - 1 {
            print!("{}", if is_last { "└── " } else { "├── " });
        } else {
            print!("│   ");
        }
    }
    
    // Print node type and basic info
    let node_type = get_node_type_name(&node.kind);
    print!("{}", node_type.green().bold());
    
    // Add node-specific information
    match &node.kind {
        AstNodeKind::ValueDef { name, visibility, purity, .. } => {
            print!(" {}", name.as_str().cyan());
            print!(" [{}]", format!("{:?}", visibility).yellow());
            if matches!(purity, Purity::Pure) {
                print!(" {}", "pure".blue());
            }
        },
        AstNodeKind::Variable { name } => {
            print!(" {}", name.as_str().cyan());
        },
        AstNodeKind::Literal { value } => {
            print!(" {}", format!("{:?}", value).magenta());
        },
        AstNodeKind::Module { name, visibility, .. } => {
            print!(" {}", name.as_str().cyan());
            print!(" [{}]", format!("{:?}", visibility).yellow());
        },
        _ => {}
    }
    
    // Show type information if requested
    if show_types {
        if let Some(type_info) = &node.metadata.type_info {
            print!(" : {}", format!("{:?}", type_info.inferred_type).blue());
        }
    }
    
    // Show span information if requested
    if show_spans {
        let span = node.metadata.span;
        print!(" @{}:{}-{}", 
            span.start.as_u32().to_string().dimmed(),
            span.start.as_u32().to_string().dimmed(),
            span.end.as_u32().to_string().dimmed()
        );
    }
    
    println!();
    
    // Print children
    let children = node.children();
    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == children.len() - 1;
        print_tree_node(child, depth + 1, max_depth, show_types, show_spans, is_last_child);
    }
}

/// Display AST as JSON
fn show_json(ast: &PersistentAstNode, max_depth: Option<usize>) -> Result<()> {
    println!("{}", "AST JSON:".bold().underline());
    
    let json_value = if let Some(depth) = max_depth {
        ast_to_json_with_depth(ast, depth)
    } else {
        serde_json::to_value(ast)?
    };
    
    let json_string = serde_json::to_string_pretty(&json_value)?;
    println!("{}", json_string);
    
    Ok(())
}

/// Convert AST to JSON with depth limitation
fn ast_to_json_with_depth(ast: &PersistentAstNode, max_depth: usize) -> serde_json::Value {
    if max_depth == 0 {
        return serde_json::json!({
            "type": get_node_type_name(&ast.kind),
            "id": ast.metadata.node_id.as_u64(),
            "truncated": true
        });
    }
    
    let value = serde_json::to_value(ast).unwrap_or(serde_json::Value::Null);
    
    // Recursively limit depth of children
    // This is a simplified implementation - real implementation would traverse the JSON structure
    value
}

/// Display summary statistics
fn show_summary(ast: &PersistentAstNode) -> Result<()> {
    println!("{}", "AST Summary:".bold().underline());
    
    let stats = collect_ast_stats(ast);
    
    println!("Total nodes: {}", stats.total_nodes.to_string().cyan());
    println!("Maximum depth: {}", stats.max_depth.to_string().cyan());
    
    println!("\nNode type distribution:");
    for (node_type, count) in &stats.node_type_counts {
        let percentage = (*count as f64 / stats.total_nodes as f64) * 100.0;
        println!("  {:<20} {:>6} ({:>5.1}%)", 
            node_type.green(),
            count.to_string().cyan(),
            percentage.to_string().yellow()
        );
    }
    
    if !stats.symbols.is_empty() {
        println!("\nSymbols defined: {}", stats.symbols.len().to_string().cyan());
        for symbol in &stats.symbols[..stats.symbols.len().min(10)] {
            println!("  {}", symbol.cyan());
        }
        if stats.symbols.len() > 10 {
            println!("  ... and {} more", (stats.symbols.len() - 10).to_string().dimmed());
        }
    }
    
    Ok(())
}

/// Display compact representation
fn show_compact(ast: &PersistentAstNode, max_depth: Option<usize>) -> Result<()> {
    println!("{}", "AST Compact:".bold().underline());
    print_compact_node(ast, 0, max_depth);
    Ok(())
}

/// Print compact node representation
fn print_compact_node(node: &PersistentAstNode, depth: usize, max_depth: Option<usize>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }
    
    let indent = "  ".repeat(depth);
    let node_type = get_node_type_name(&node.kind);
    
    match &node.kind {
        AstNodeKind::ValueDef { name, .. } => {
            println!("{}{}({})", indent, node_type.green(), name.as_str().cyan());
        },
        AstNodeKind::Variable { name } => {
            println!("{}{}({})", indent, node_type.green(), name.as_str().cyan());
        },
        AstNodeKind::Literal { value } => {
            println!("{}{}({:?})", indent, node_type.green(), value);
        },
        _ => {
            println!("{}{}", indent, node_type.green());
        }
    }
    
    for child in node.children() {
        print_compact_node(child, depth + 1, max_depth);
    }
}

/// Get human-readable node type name
fn get_node_type_name(kind: &AstNodeKind) -> &'static str {
    match kind {
        AstNodeKind::CompilationUnit { .. } => "CompilationUnit",
        AstNodeKind::Module { .. } => "Module",
        AstNodeKind::ValueDef { .. } => "ValueDef",
        AstNodeKind::TypeDef { .. } => "TypeDef",
        AstNodeKind::EffectDef { .. } => "EffectDef",
        AstNodeKind::Literal { .. } => "Literal",
        AstNodeKind::Variable { .. } => "Variable",
        AstNodeKind::Application { .. } => "Application",
        AstNodeKind::Lambda { .. } => "Lambda",
        AstNodeKind::Let { .. } => "Let",
        AstNodeKind::If { .. } => "If",
        AstNodeKind::Match { .. } => "Match",
        AstNodeKind::Handle { .. } => "Handle",
        AstNodeKind::Perform { .. } => "Perform",
        AstNodeKind::TypeReference { .. } => "TypeReference",
        AstNodeKind::FunctionType { .. } => "FunctionType",
        AstNodeKind::RecordType { .. } => "RecordType",
        AstNodeKind::VariantType { .. } => "VariantType",
        AstNodeKind::PatternVariable { .. } => "PatternVariable",
        AstNodeKind::PatternLiteral { .. } => "PatternLiteral",
        AstNodeKind::PatternConstructor { .. } => "PatternConstructor",
        AstNodeKind::PatternRecord { .. } => "PatternRecord",
    }
}

/// AST statistics
#[derive(Debug)]
struct AstStats {
    total_nodes: usize,
    max_depth: usize,
    node_type_counts: std::collections::HashMap<String, usize>,
    symbols: Vec<String>,
}

/// Collect statistics about the AST
fn collect_ast_stats(ast: &PersistentAstNode) -> AstStats {
    let mut stats = AstStats {
        total_nodes: 0,
        max_depth: 0,
        node_type_counts: std::collections::HashMap::new(),
        symbols: Vec::new(),
    };
    
    collect_stats_recursive(ast, 0, &mut stats);
    stats.symbols.sort();
    stats.symbols.dedup();
    
    stats
}

/// Recursively collect statistics
fn collect_stats_recursive(node: &PersistentAstNode, depth: usize, stats: &mut AstStats) {
    stats.total_nodes += 1;
    stats.max_depth = stats.max_depth.max(depth);
    
    let type_name = get_node_type_name(&node.kind).to_string();
    *stats.node_type_counts.entry(type_name).or_insert(0) += 1;
    
    // Collect symbols
    match &node.kind {
        AstNodeKind::ValueDef { name, .. } |
        AstNodeKind::Variable { name } |
        AstNodeKind::Module { name, .. } |
        AstNodeKind::TypeDef { name, .. } |
        AstNodeKind::EffectDef { name, .. } => {
            stats.symbols.push(name.as_str().to_string());
        },
        _ => {}
    }
    
    for child in node.children() {
        collect_stats_recursive(child, depth + 1, stats);
    }
}

/// Display AST in OCaml-style syntax
fn show_ocaml(ast: &PersistentAstNode, _max_depth: Option<usize>) -> Result<()> {
    println!("{}", "OCaml-style representation:".bold().underline());
    
    // Convert PersistentAstNode to regular AST for printing
    let compilation_unit = convert_persistent_to_ast(ast)?;
    
    let config = SyntaxConfig {
        style: SyntaxStyle::OCaml,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = OCamlPrinter::new();
    let output = printer.print(&compilation_unit, &config)?;
    
    println!("{}", output);
    Ok(())
}

/// Display AST in Haskell-style syntax
fn show_haskell(ast: &PersistentAstNode, _max_depth: Option<usize>) -> Result<()> {
    println!("{}", "Haskell-style representation:".bold().underline());
    
    // Convert PersistentAstNode to regular AST for printing
    let compilation_unit = convert_persistent_to_ast(ast)?;
    
    let config = SyntaxConfig {
        style: SyntaxStyle::Haskell,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = HaskellPrinter::new();
    let output = printer.print(&compilation_unit, &config)?;
    
    println!("{}", output);
    Ok(())
}

/// Display AST in S-expression syntax
fn show_sexp(ast: &PersistentAstNode, _max_depth: Option<usize>) -> Result<()> {
    println!("{}", "S-expression representation:".bold().underline());
    
    // Convert PersistentAstNode to regular AST for printing
    let compilation_unit = convert_persistent_to_ast(ast)?;
    
    let config = SyntaxConfig {
        style: SyntaxStyle::SExp,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = SExpPrinter::new();
    let output = printer.print(&compilation_unit, &config)?;
    
    println!("{}", output);
    Ok(())
}

/// Convert PersistentAstNode to regular CompilationUnit
/// This is a simplified conversion - a full implementation would reconstruct the entire AST
fn convert_persistent_to_ast(_ast: &PersistentAstNode) -> Result<x_parser::ast::CompilationUnit> {
    use x_parser::ast::*;
    use x_parser::symbol::Symbol;
    use x_parser::span::{Span, FileId, ByteOffset};
    
    // Create a minimal placeholder compilation unit
    // In a real implementation, this would traverse the persistent AST and reconstruct the full AST
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0));
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("Main"), span),
        exports: None,
        imports: Vec::new(),
        items: Vec::new(),
        span,
    };
    
    Ok(CompilationUnit {
        module,
        span,
    })
}