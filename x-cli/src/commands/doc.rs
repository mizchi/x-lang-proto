//! Documentation and semantic analysis command
//! 
//! Provides AI-friendly semantic summaries of code structure

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use x_parser::{parse_source, FileId, SyntaxStyle};
use x_checker::TypeChecker;

#[derive(Debug, Args)]
pub struct DocCommand {
    /// Path to analyze
    #[arg(default_value = ".")]
    path: PathBuf,
    
    /// Output format
    #[arg(long, value_enum, default_value = "summary")]
    format: OutputFormat,
    
    /// Include private items
    #[arg(long)]
    include_private: bool,
    
    /// Maximum depth for nested structures
    #[arg(long, default_value = "3")]
    max_depth: usize,
    
    /// Filter by symbol name
    #[arg(long)]
    filter: Option<String>,
    
    /// Show only specific kinds of symbols
    #[arg(long)]
    kind: Vec<SymbolKind>,
    
    /// Show function list
    #[arg(long)]
    functions: bool,
    
    /// Show dependency tree for a specific function
    #[arg(long)]
    deps: Option<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Summary,
    Json,
    Tree,
    Semantic,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Serialize, Deserialize)]
enum SymbolKind {
    Module,
    Function,
    Type,
    Effect,
    Handler,
    Test,
    Interface,
}

/// Semantic symbol information - AI-friendly representation
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSymbol {
    /// Unique identifier for the symbol
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Fully qualified path
    pub path: Vec<String>,
    
    /// Symbol kind
    pub kind: SymbolKind,
    
    /// Type signature (if applicable)
    pub type_signature: Option<String>,
    
    /// Documentation
    pub doc: Option<String>,
    
    /// Effects used/handled
    pub effects: Vec<String>,
    
    /// Direct dependencies (symbols referenced)
    pub dependencies: Vec<String>,
    
    /// Nested symbols
    pub children: Vec<SemanticSymbol>,
    
    /// Semantic properties
    pub properties: SymbolProperties,
    
    /// Source location (AST node reference)
    pub ast_ref: AstReference,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolProperties {
    pub is_pure: bool,
    pub is_exported: bool,
    pub is_generic: bool,
    pub has_effects: bool,
    pub complexity_score: u32,
    pub test_coverage: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AstReference {
    /// File path
    pub file: String,
    
    /// AST node path (e.g., "module.items[2].body.arms[0]")
    pub node_path: String,
    
    /// Node type
    pub node_type: String,
    
    /// Stable hash of the node content
    pub content_hash: String,
}

/// Module summary for AI consumption
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub name: String,
    pub exports: Vec<ExportSummary>,
    pub imports: Vec<ImportSummary>,
    pub internal_symbols: Vec<SemanticSymbol>,
    pub effect_graph: EffectGraph,
    pub dependency_graph: DependencyGraph,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSummary {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSummary {
    pub module: String,
    pub symbols: Vec<String>,
    pub is_qualified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectGraph {
    /// Effects defined in this module
    pub defined_effects: Vec<String>,
    
    /// Effects used by functions
    pub effect_usage: Vec<(String, Vec<String>)>,
    
    /// Effect handlers and what they handle
    pub handlers: Vec<(String, Vec<String>)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Internal dependencies (symbol -> [dependencies])
    pub internal: Vec<(String, Vec<String>)>,
    
    /// External dependencies (symbol -> [external symbols])
    pub external: Vec<(String, Vec<String>)>,
}

impl DocCommand {
    pub fn run(self) -> Result<()> {
        // Discover x files
        let files = discover_x_files(&self.path)?;
        
        let mut all_summaries = Vec::new();
        
        for file_path in files {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            
            let file_id = FileId(0);
            let compilation_unit = parse_source(&content, file_id, SyntaxStyle::SExpression)?;
            
            // Type check for better semantic information
            let mut type_checker = TypeChecker::new();
            let check_result = type_checker.check_compilation_unit(&compilation_unit);
            
            // Generate semantic summary
            let summary = generate_module_summary(
                &compilation_unit,
                &check_result,
                &file_path,
                self.include_private,
                self.max_depth,
            )?;
            
            all_summaries.push(summary);
        }
        
        // Handle special modes
        if self.functions {
            print_function_list(&all_summaries);
            return Ok(());
        }
        
        if let Some(function_name) = &self.deps {
            print_dependency_tree(&all_summaries, function_name)?;
            return Ok(());
        }
        
        // Output based on format
        match self.format {
            OutputFormat::Summary => print_human_summary(&all_summaries),
            OutputFormat::Json => print_json_summary(&all_summaries)?,
            OutputFormat::Tree => print_tree_summary(&all_summaries),
            OutputFormat::Semantic => print_semantic_summary(&all_summaries)?,
        }
        
        Ok(())
    }
}

fn discover_x_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if path.is_file() && path.extension().map_or(false, |ext| ext == "x") {
        files.push(path.clone());
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "x") {
                files.push(path);
            }
        }
    }
    
    Ok(files)
}

fn generate_module_summary(
    compilation_unit: &x_parser::CompilationUnit,
    check_result: &x_checker::CheckResult,
    file_path: &PathBuf,
    include_private: bool,
    max_depth: usize,
) -> Result<ModuleSummary> {
    use x_parser::{Item, Visibility};
    
    let module = &compilation_unit.module;
    let mut exports = Vec::new();
    let mut internal_symbols = Vec::new();
    let mut effect_graph = EffectGraph {
        defined_effects: Vec::new(),
        effect_usage: Vec::new(),
        handlers: Vec::new(),
    };
    
    // Process module items
    for (index, item) in module.items.iter().enumerate() {
        let ast_ref = AstReference {
            file: file_path.to_string_lossy().to_string(),
            node_path: format!("module.items[{}]", index),
            node_type: match item {
                Item::TypeDef(_) => "TypeDef",
                Item::ValueDef(_) => "ValueDef",
                Item::EffectDef(_) => "EffectDef",
                Item::HandlerDef(_) => "HandlerDef",
                Item::ModuleTypeDef(_) => "ModuleTypeDef",
                Item::InterfaceDef(_) => "InterfaceDef",
                Item::TestDef(_) => "TestDef",
            }.to_string(),
            content_hash: calculate_content_hash(item),
        };
        
        match item {
            Item::ValueDef(def) => {
                if should_include(&def.visibility, include_private) {
                    let symbol = SemanticSymbol {
                        id: format!("{}.{}", module.name.to_string(), def.name.as_str()),
                        name: def.name.as_str().to_string(),
                        path: vec![module.name.to_string(), def.name.as_str().to_string()],
                        kind: SymbolKind::Function,
                        type_signature: def.type_annotation.as_ref().map(|t| format!("{:?}", t)),
                        doc: def.documentation.as_ref().map(|doc| format_documentation(doc)),
                        effects: extract_effects_from_type(&def.type_annotation),
                        dependencies: extract_dependencies_from_expr(&def.body),
                        children: Vec::new(),
                        properties: SymbolProperties {
                            is_pure: def.purity == x_parser::Purity::Pure,
                            is_exported: is_exported(&def.visibility),
                            is_generic: def.type_annotation.as_ref().map_or(false, has_type_params),
                            has_effects: !extract_effects_from_type(&def.type_annotation).is_empty(),
                            complexity_score: calculate_complexity(&def.body),
                            test_coverage: None,
                        },
                        ast_ref,
                    };
                    
                    if is_exported(&def.visibility) {
                        exports.push(ExportSummary {
                            name: def.name.as_str().to_string(),
                            kind: SymbolKind::Function,
                            signature: symbol.type_signature.clone(),
                            doc: symbol.doc.clone(),
                        });
                    } else {
                        internal_symbols.push(symbol);
                    }
                }
            }
            Item::TestDef(def) => {
                if include_private || is_exported(&def.visibility) {
                    let symbol = SemanticSymbol {
                        id: format!("{}.{}", module.name.to_string(), def.name.as_str()),
                        name: def.name.as_str().to_string(),
                        path: vec![module.name.to_string(), def.name.as_str().to_string()],
                        kind: SymbolKind::Test,
                        type_signature: Some("() -> Bool".to_string()),
                        doc: def.documentation.as_ref()
                            .map(|doc| format_documentation(doc))
                            .or_else(|| def.description.clone()),
                        effects: extract_effects_from_expr(&def.body),
                        dependencies: extract_dependencies_from_expr(&def.body),
                        children: Vec::new(),
                        properties: SymbolProperties {
                            is_pure: false, // Tests may have side effects
                            is_exported: is_exported(&def.visibility),
                            is_generic: false,
                            has_effects: true,
                            complexity_score: calculate_complexity(&def.body),
                            test_coverage: Some(1.0), // Tests provide coverage
                        },
                        ast_ref,
                    };
                    internal_symbols.push(symbol);
                }
            }
            Item::EffectDef(def) => {
                effect_graph.defined_effects.push(def.name.as_str().to_string());
                // Process effect definition...
            }
            _ => {} // Handle other item types
        }
    }
    
    // Build dependency graph
    let dependency_graph = build_dependency_graph(&internal_symbols, &exports);
    
    Ok(ModuleSummary {
        name: module.name.to_string(),
        exports,
        imports: extract_imports(&module.imports),
        internal_symbols,
        effect_graph,
        dependency_graph,
    })
}

// Helper functions (stubs for now)
fn should_include(visibility: &x_parser::Visibility, include_private: bool) -> bool {
    eprintln!("Checking visibility: {:?}, include_private: {}", visibility, include_private);
    include_private || !matches!(visibility, x_parser::Visibility::Private)
}

fn is_exported(visibility: &x_parser::Visibility) -> bool {
    !matches!(visibility, x_parser::Visibility::Private)
}

fn extract_doc_comment(_span: &x_parser::span::Span) -> Option<String> {
    // TODO: Extract doc comments from source span
    // For now, documentation is extracted directly from AST nodes
    None
}

fn format_documentation(doc: &x_parser::Documentation) -> String {
    let mut result = String::new();
    
    // Format main content
    if !doc.doc_comment.content.is_empty() {
        result.push_str(&doc.doc_comment.content);
    }
    
    // Add attributes if present
    if !doc.doc_comment.attributes.is_empty() {
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str("Attributes:\n");
        for (key, value) in &doc.doc_comment.attributes {
            result.push_str(&format!("  {}: ", key));
            match value {
                x_parser::DocAttributeValue::String(s) => result.push_str(s),
                x_parser::DocAttributeValue::Number(n) => result.push_str(&n.to_string()),
                x_parser::DocAttributeValue::Boolean(b) => result.push_str(&b.to_string()),
                x_parser::DocAttributeValue::List(items) => {
                    result.push('[');
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 { result.push_str(", "); }
                        result.push_str(item);
                    }
                    result.push(']');
                },
                x_parser::DocAttributeValue::TypedParam { type_info, description } => {
                    result.push_str(&format!("{{{}}}", type_info));
                    if !description.is_empty() {
                        result.push_str(" ");
                        result.push_str(description);
                    }
                },
                x_parser::DocAttributeValue::Object(map) => {
                    result.push_str("{ ");
                    for (i, (k, v)) in map.iter().enumerate() {
                        if i > 0 { result.push_str(", "); }
                        result.push_str(&format!("{}: {:?}", k, v));
                    }
                    result.push_str(" }");
                },
            }
            result.push('\n');
        }
    }
    
    result.trim().to_string()
}

fn extract_effects_from_type(type_ann: &Option<x_parser::Type>) -> Vec<String> {
    // TODO: Extract effect information from type
    Vec::new()
}

fn extract_effects_from_expr(expr: &x_parser::Expr) -> Vec<String> {
    // TODO: Extract effects from expression
    Vec::new()
}

fn extract_dependencies_from_expr(expr: &x_parser::Expr) -> Vec<String> {
    use x_parser::Expr;
    let mut deps = Vec::new();
    
    fn collect_deps(expr: &Expr, deps: &mut Vec<String>) {
        match expr {
            Expr::Var(name, _) => {
                let name_str = name.as_str();
                if !deps.contains(&name_str.to_string()) {
                    deps.push(name_str.to_string());
                }
            }
            Expr::App(func, args, _) => {
                collect_deps(func, deps);
                for arg in args {
                    collect_deps(arg, deps);
                }
            }
            Expr::Lambda { body, .. } => {
                collect_deps(body, deps);
            }
            Expr::Let { value, body, .. } => {
                collect_deps(value, deps);
                collect_deps(body, deps);
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                collect_deps(condition, deps);
                collect_deps(then_branch, deps);
                collect_deps(else_branch, deps);
            }
            Expr::Match { scrutinee, arms, .. } => {
                collect_deps(scrutinee, deps);
                for arm in arms {
                    collect_deps(&arm.body, deps);
                }
            }
            Expr::Do { statements, .. } => {
                for stmt in statements {
                    match stmt {
                        x_parser::DoStatement::Let { expr, .. } => collect_deps(expr, deps),
                        x_parser::DoStatement::Expr(expr) => collect_deps(expr, deps),
                        x_parser::DoStatement::Bind { expr, .. } => collect_deps(expr, deps),
                    }
                }
            }
            Expr::Handle { expr, .. } => {
                collect_deps(expr, deps);
                // TODO: Process handlers
            }
            Expr::Resume { value, .. } => {
                collect_deps(value, deps);
            }
            Expr::Perform { args, .. } => {
                for arg in args {
                    collect_deps(arg, deps);
                }
            }
            Expr::Ann { expr, .. } => {
                collect_deps(expr, deps);
            }
            Expr::Literal(_, _) => {
                // No dependencies
            }
        }
    }
    
    collect_deps(expr, &mut deps);
    deps
}

fn has_type_params(typ: &x_parser::Type) -> bool {
    // TODO: Check for type parameters
    false
}

fn calculate_complexity(expr: &x_parser::Expr) -> u32 {
    // TODO: Calculate cyclomatic complexity
    1
}

fn calculate_content_hash(item: &x_parser::Item) -> String {
    // TODO: Calculate stable hash
    "TODO".to_string()
}

fn extract_imports(imports: &[x_parser::Import]) -> Vec<ImportSummary> {
    imports.iter().map(|import| {
        ImportSummary {
            module: import.module_path.to_string(),
            symbols: Vec::new(), // TODO: Extract from ImportKind
            is_qualified: matches!(import.kind, x_parser::ImportKind::Qualified),
        }
    }).collect()
}

fn build_dependency_graph(
    internal_symbols: &[SemanticSymbol],
    exports: &[ExportSummary],
) -> DependencyGraph {
    // TODO: Build actual dependency graph
    DependencyGraph {
        internal: Vec::new(),
        external: Vec::new(),
    }
}

fn print_human_summary(summaries: &[ModuleSummary]) {
    for summary in summaries {
        println!("Module: {}", summary.name);
        println!("  Exports: {}", summary.exports.len());
        for export in &summary.exports {
            println!("    - {} ({})", export.name, format!("{:?}", export.kind));
        }
        println!("  Internal symbols: {}", summary.internal_symbols.len());
        for symbol in &summary.internal_symbols {
            println!("    - {} ({})", symbol.name, format!("{:?}", symbol.kind));
            if let Some(doc) = &symbol.doc {
                println!("      Doc: {}", doc);
            }
        }
        println!();
    }
}

fn print_json_summary(summaries: &[ModuleSummary]) -> Result<()> {
    let json = serde_json::to_string_pretty(summaries)?;
    println!("{}", json);
    Ok(())
}

fn print_tree_summary(summaries: &[ModuleSummary]) {
    for summary in summaries {
        println!("📦 {}", summary.name);
        
        if !summary.exports.is_empty() {
            println!("├── 📤 Exports");
            for (i, export) in summary.exports.iter().enumerate() {
                let prefix = if i == summary.exports.len() - 1 { "└──" } else { "├──" };
                println!("│   {} {} {}", prefix, match export.kind {
                    SymbolKind::Function => "🔧",
                    SymbolKind::Type => "📐",
                    SymbolKind::Effect => "⚡",
                    _ => "📄",
                }, export.name);
            }
        }
        
        if !summary.internal_symbols.is_empty() {
            println!("└── 🔒 Internal");
            for (i, symbol) in summary.internal_symbols.iter().enumerate() {
                let prefix = if i == summary.internal_symbols.len() - 1 { "└──" } else { "├──" };
                println!("    {} {} {}", prefix, match symbol.kind {
                    SymbolKind::Function => "🔧",
                    SymbolKind::Test => "🧪",
                    _ => "📄",
                }, symbol.name);
            }
        }
        println!();
    }
}

fn print_semantic_summary(summaries: &[ModuleSummary]) -> Result<()> {
    // AI-optimized format
    let semantic_output = summaries.iter().map(|summary| {
        serde_json::json!({
            "module": summary.name,
            "api": summary.exports.iter().map(|e| {
                serde_json::json!({
                    "name": e.name,
                    "kind": e.kind,
                    "signature": e.signature,
                })
            }).collect::<Vec<_>>(),
            "ast_refs": summary.internal_symbols.iter().map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "ast": s.ast_ref,
                })
            }).collect::<Vec<_>>(),
        })
    }).collect::<Vec<_>>();
    
    println!("{}", serde_json::to_string_pretty(&semantic_output)?);
    Ok(())
}

fn print_function_list(summaries: &[ModuleSummary]) {
    println!("Functions:");
    println!("==========");
    
    for summary in summaries {
        println!("\nModule: {}", summary.name);
        
        // Collect functions from both internal symbols and exports
        let functions: Vec<_> = summary.internal_symbols.iter()
            .filter(|sym| matches!(sym.kind, SymbolKind::Function))
            .collect();
        
        // Also collect functions from exports if they're not already in internal_symbols
        let exported_functions: Vec<_> = summary.exports.iter()
            .filter(|exp| matches!(exp.kind, SymbolKind::Function))
            .collect();
        
        if functions.is_empty() && exported_functions.is_empty() {
            println!("  (no functions)");
        } else {
            // Print internal functions
            for func in functions {
                println!("  - {}", func.name);
                if let Some(sig) = &func.type_signature {
                    println!("    Type: {}", sig);
                }
                if !func.dependencies.is_empty() {
                    println!("    Deps: {}", func.dependencies.join(", "));
                }
            }
            
            // Print exported functions
            for exp in exported_functions {
                println!("  - {} (exported)", exp.name);
                if let Some(sig) = &exp.signature {
                    println!("    Type: {}", sig);
                }
            }
        }
    }
}

fn print_dependency_tree(summaries: &[ModuleSummary], function_name: &str) -> Result<()> {
    // Find the function
    let mut target_func = None;
    let mut all_functions = std::collections::HashMap::new();
    
    for summary in summaries {
        for sym in &summary.internal_symbols {
            if matches!(sym.kind, SymbolKind::Function) {
                all_functions.insert(sym.name.clone(), sym);
                if sym.name == function_name {
                    target_func = Some(sym);
                }
            }
        }
    }
    
    if let Some(func) = target_func {
        println!("Dependency tree for '{}':", func.name);
        println!("========================");
        print_deps_recursive(&func.name, &func.dependencies, &all_functions, 0, &mut Vec::new());
    } else {
        println!("Function '{}' not found", function_name);
    }
    
    Ok(())
}

fn print_deps_recursive(
    name: &str,
    deps: &[String],
    all_functions: &std::collections::HashMap<String, &SemanticSymbol>,
    depth: usize,
    visited: &mut Vec<String>,
) {
    let indent = "  ".repeat(depth);
    
    if visited.contains(&name.to_string()) {
        println!("{}└─ {} (circular)", indent, name);
        return;
    }
    
    visited.push(name.to_string());
    
    println!("{}└─ {}", indent, name);
    
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let prefix = if is_last { "└─" } else { "├─" };
        
        if let Some(dep_func) = all_functions.get(dep) {
            print_deps_recursive(dep, &dep_func.dependencies, all_functions, depth + 1, visited);
        } else {
            println!("{}  {} {} (external)", indent, prefix, dep);
        }
    }
    
    visited.pop();
}