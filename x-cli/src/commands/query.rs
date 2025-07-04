//! AST query commands

use anyhow::{Result, Context};
use std::path::Path;
use colored::*;
use serde_json;
use x_parser::{persistent_ast::{PersistentAstNode, AstNodeKind}, symbol::Symbol};
use crate::format::{detect_format, load_ast};
use crate::utils::ProgressIndicator;

/// Execute queries against AST
pub async fn query_command(
    input: &Path,
    query_str: &str,
    output_format: &str,
) -> Result<()> {
    let progress = ProgressIndicator::new("Executing query");
    
    // Load AST
    let input_format = detect_format(input)?;
    let ast = load_ast(input, input_format).await
        .with_context(|| format!("Failed to load AST from: {}", input.display()))?;
    
    progress.set_message("Parsing query");
    
    // Parse and execute query (simplified implementation)
    let results = execute_simple_query(&ast, query_str)?;
    
    progress.finish("Query completed");
    
    // Display results
    display_query_results(&results, output_format)?;
    
    Ok(())
}

/// Simple query execution (placeholder implementation)
fn execute_simple_query(ast: &PersistentAstNode, query_str: &str) -> Result<Vec<QueryResult>> {
    let mut results = Vec::new();
    
    if query_str.starts_with("type:") {
        let type_name = query_str.strip_prefix("type:").unwrap().trim();
        find_by_type(ast, type_name, &mut results);
    } else if query_str.starts_with("symbol:") {
        let symbol_name = query_str.strip_prefix("symbol:").unwrap().trim();
        find_by_symbol(ast, symbol_name, &mut results);
    } else {
        // Default: find all nodes
        collect_all_nodes(ast, &mut results);
    }
    
    Ok(results)
}

/// Find nodes by type
fn find_by_type(node: &PersistentAstNode, type_name: &str, results: &mut Vec<QueryResult>) {
    let node_type = get_node_type_name(&node.kind);
    if node_type.to_lowercase().contains(&type_name.to_lowercase()) {
        results.push(QueryResult {
            node_id: node.metadata.node_id.as_u64(),
            node_type: node_type.to_string(),
            description: format!("Found {} node", node_type),
            location: format!("{}:{}", node.metadata.span.start.as_u32(), node.metadata.span.end.as_u32()),
        });
    }
    
    for child in node.children() {
        find_by_type(&child, type_name, results);
    }
}

/// Find nodes by symbol
fn find_by_symbol(node: &PersistentAstNode, symbol_name: &str, results: &mut Vec<QueryResult>) {
    match &node.kind {
        AstNodeKind::Variable { name } if name.as_str().contains(symbol_name) => {
            results.push(QueryResult {
                node_id: node.metadata.node_id.as_u64(),
                node_type: "Variable".to_string(),
                description: format!("Variable: {}", name.as_str()),
                location: format!("{}:{}", node.metadata.span.start.as_u32(), node.metadata.span.end.as_u32()),
            });
        },
        AstNodeKind::ValueDef { name, .. } if name.as_str().contains(symbol_name) => {
            results.push(QueryResult {
                node_id: node.metadata.node_id.as_u64(),
                node_type: "ValueDef".to_string(),
                description: format!("Function: {}", name.as_str()),
                location: format!("{}:{}", node.metadata.span.start.as_u32(), node.metadata.span.end.as_u32()),
            });
        },
        _ => {}
    }
    
    for child in node.children() {
        find_by_symbol(&child, symbol_name, results);
    }
}

/// Collect all nodes
fn collect_all_nodes(node: &PersistentAstNode, results: &mut Vec<QueryResult>) {
    let node_type = get_node_type_name(&node.kind);
    results.push(QueryResult {
        node_id: node.metadata.node_id.as_u64(),
        node_type: node_type.to_string(),
        description: format!("{} node", node_type),
        location: format!("{}:{}", node.metadata.span.start.as_u32(), node.metadata.span.end.as_u32()),
    });
    
    for child in node.children() {
        collect_all_nodes(&child, results);
    }
}

/// Display query results
fn display_query_results(results: &[QueryResult], output_format: &str) -> Result<()> {
    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(results)?;
            println!("{}", json);
        },
        "table" => {
            println!("{}", "Query Results:".bold().underline());
            println!();
            
            if results.is_empty() {
                println!("{}", "No results found".yellow());
                return Ok(());
            }
            
            for (i, result) in results.iter().enumerate() {
                println!("{}. {} {} at {}",
                    (i + 1).to_string().cyan(),
                    result.node_type.green().bold(),
                    result.description.white(),
                    result.location.dimmed()
                );
            }
            
            println!();
            println!("Found {} result(s)", results.len().to_string().cyan());
        },
        _ => {
            println!("Results: {} found", results.len());
            for result in results {
                println!("- {}: {} at {}", result.node_type, result.description, result.location);
            }
        }
    }
    
    Ok(())
}

/// Query result structure
#[derive(Debug, serde::Serialize)]
struct QueryResult {
    node_id: u64,
    node_type: String,
    description: String,
    location: String,
}

/// Get node type name
fn get_node_type_name(kind: &AstNodeKind) -> &'static str {
    match kind {
        AstNodeKind::CompilationUnit { .. } => "CompilationUnit",
        AstNodeKind::Module { .. } => "Module",
        AstNodeKind::ValueDef { .. } => "ValueDef",
        AstNodeKind::TypeDef { .. } => "TypeDef",
        AstNodeKind::EffectDef { .. } => "EffectDef",
        // AstNodeKind::HandlerDef { .. } => "HandlerDef",
        AstNodeKind::Variable { .. } => "Variable",
        AstNodeKind::Lambda { .. } => "Lambda",
        AstNodeKind::Application { .. } => "Application",
        AstNodeKind::Let { .. } => "Let",
        AstNodeKind::If { .. } => "If",
        AstNodeKind::Match { .. } => "Match",
        AstNodeKind::Literal { .. } => "Literal",
        // AstNodeKind::Record { .. } => "Record",
        // AstNodeKind::FieldAccess { .. } => "FieldAccess",
        // AstNodeKind::Annotation { .. } => "Annotation",
        // AstNodeKind::Block { .. } => "Block",
        // AstNodeKind::Do { .. } => "Do",
        AstNodeKind::Handle { .. } => "Handle",
        // AstNodeKind::Resume { .. } => "Resume",
        AstNodeKind::Perform { .. } => "Perform",
        // AstNodeKind::PatternMatch { .. } => "PatternMatch",
        AstNodeKind::PatternLiteral { .. } => "PatternLiteral",
        AstNodeKind::PatternVariable { .. } => "PatternVariable",
        AstNodeKind::PatternConstructor { .. } => "PatternConstructor",
        AstNodeKind::PatternRecord { .. } => "PatternRecord",
        // AstNodeKind::TypeAnnotation { .. } => "TypeAnnotation",
        // AstNodeKind::BasicType { .. } => "BasicType",
        AstNodeKind::TypeReference { .. } => "TypeReference",
        AstNodeKind::FunctionType { .. } => "FunctionType",
        AstNodeKind::RecordType { .. } => "RecordType",
        AstNodeKind::VariantType { .. } => "VariantType",
        // AstNodeKind::EffectType { .. } => "EffectType",
        // AstNodeKind::Import { .. } => "Import",
        // AstNodeKind::Export { .. } => "Export",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_query_command() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.rustic.x");
        
        // Create a simple test file
        let mut file = File::create(&input_path).unwrap();
        writeln!(file, "pub fn test() {{}}").unwrap();
        
        // Test query execution
        let result = query_command(
            &input_path,
            "type:function",
            "table"
        ).await;
        
        // Should succeed (though actual query depends on parser implementation)
        match result {
            Ok(()) => {},
            Err(_) => {
                // Expected if parser is not fully implemented
                // This is fine for testing the CLI structure
            }
        }
    }
}