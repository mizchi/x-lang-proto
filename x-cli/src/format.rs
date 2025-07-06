//! Format detection and conversion utilities

use anyhow::{Result, Context, bail};
use std::path::Path;
use std::fs;
use x_parser::{
    persistent_ast::{PersistentAstNode, NodeBuilder, AstNodeKind, Visibility},
    span::{Span, FileId},
    symbol::Symbol,
    SyntaxStyle,
};

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Binary AST format (.x)
    Binary,
    /// Rust-like syntax (.rustic.x)
    Rustic,
    /// OCaml-like syntax (.ocaml.x)
    OCaml,
    /// S-expression syntax (.lisp.x)
    SExpression,
    /// Haskell-like syntax (.haskell.x)
    Haskell,
    /// JSON representation (.json)
    Json,
}

impl Format {
    /// Parse format from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "binary" | "bin" | "x" => Ok(Format::Binary),
            "rustic" | "rust" => Ok(Format::Rustic),
            "ocaml" | "ml" => Ok(Format::OCaml),
            "sexp" | "lisp" | "s-expression" => Ok(Format::SExpression),
            "haskell" | "hs" => Ok(Format::Haskell),
            "json" => Ok(Format::Json),
            _ => bail!("Unknown format: {}", s),
        }
    }
    
    /// Get default file extension for this format
    pub fn default_extension(&self) -> &'static str {
        match self {
            Format::Binary => "x",
            Format::Rustic => "rustic.x",
            Format::OCaml => "ocaml.x",
            Format::SExpression => "lisp.x",
            Format::Haskell => "haskell.x",
            Format::Json => "json",
        }
    }
    
    /// Get syntax style for text formats
    pub fn syntax_style(&self) -> Option<SyntaxStyle> {
        match self {
            Format::Rustic => Some(SyntaxStyle::RustLike),
            Format::OCaml => Some(SyntaxStyle::OCaml),
            Format::SExpression => Some(SyntaxStyle::SExpression),
            Format::Haskell => Some(SyntaxStyle::Haskell),
            Format::Binary | Format::Json => None,
        }
    }
}

/// Detect format from file path
pub fn detect_format(path: &Path) -> Result<Format> {
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.ends_with(".x") && path_str.matches('.').count() == 1 {
        Ok(Format::Binary)
    } else if path_str.ends_with(".rustic.x") {
        Ok(Format::Rustic)
    } else if path_str.ends_with(".ocaml.x") || path_str.ends_with(".ml.x") {
        Ok(Format::OCaml)
    } else if path_str.ends_with(".lisp.x") || path_str.ends_with(".sexp.x") {
        Ok(Format::SExpression)
    } else if path_str.ends_with(".haskell.x") || path_str.ends_with(".hs.x") {
        Ok(Format::Haskell)
    } else if path_str.ends_with(".json") {
        Ok(Format::Json)
    } else {
        bail!("Cannot detect format from file extension: {}", path.display());
    }
}

/// Load AST from file
pub async fn load_ast(path: &Path, format: Format) -> Result<PersistentAstNode> {
    let content = fs::read(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    match format {
        Format::Binary => load_binary_ast(&content),
        Format::Json => load_json_ast(&content),
        Format::Rustic | Format::OCaml | Format::SExpression | Format::Haskell => {
            load_text_ast(&content, format)
        }
    }
}

/// Save AST to file
pub async fn save_ast(path: &Path, ast: &PersistentAstNode, format: Format) -> Result<()> {
    let content = match format {
        Format::Binary => save_binary_ast(ast)?,
        Format::Json => save_json_ast(&ast)?,
        Format::Rustic | Format::OCaml | Format::SExpression | Format::Haskell => {
            save_text_ast(&ast, format)?
        }
    };
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    
    Ok(())
}

/// Load binary AST format
fn load_binary_ast(content: &[u8]) -> Result<PersistentAstNode> {
    // Check magic number
    if content.len() < 4 {
        bail!("File too short to be a valid x Language binary file");
    }
    
    let magic = &content[0..4];
    if magic != x_parser::binary::MAGIC_NUMBER {
        bail!("Invalid magic number. This is not a valid x Language binary file");
    }
    
    // Use the binary deserializer from x_parser
    let mut deserializer = x_parser::binary::BinaryDeserializer::new(content.to_vec())
        .context("Failed to create binary deserializer")?;
    
    let compilation_unit = deserializer.deserialize_compilation_unit()
        .context("Failed to deserialize compilation unit")?;
    
    // Convert AST to PersistentAstNode
    convert_ast_to_persistent(&compilation_unit)
}

/// Save binary AST format
fn save_binary_ast(ast: &PersistentAstNode) -> Result<Vec<u8>> {
    // Convert PersistentAstNode to AST
    let compilation_unit = convert_persistent_to_ast(ast)
        .context("Failed to convert PersistentAstNode to AST")?;
    
    // Use the binary serializer from x_parser
    let mut serializer = x_parser::binary::BinarySerializer::new();
    
    let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
        .context("Failed to serialize compilation unit to binary")?;
    
    Ok(binary_data)
}

/// Load JSON AST format
fn load_json_ast(content: &[u8]) -> Result<PersistentAstNode> {
    let json_str = std::str::from_utf8(content)
        .context("Invalid UTF-8 in JSON file")?;
    
    let ast: PersistentAstNode = serde_json::from_str(json_str)
        .context("Failed to parse JSON AST")?;
    
    Ok(ast)
}

/// Save JSON AST format
fn save_json_ast(ast: &PersistentAstNode) -> Result<Vec<u8>> {
    let json_str = serde_json::to_string_pretty(ast)
        .context("Failed to serialize AST to JSON")?;
    
    Ok(json_str.into_bytes())
}

/// Load text AST format
fn load_text_ast(content: &[u8], format: Format) -> Result<PersistentAstNode> {
    let _text = std::str::from_utf8(content)
        .context("Invalid UTF-8 in text file")?;
    
    let _syntax_style = format.syntax_style()
        .context("Format does not support text parsing")?;
    
    // TODO: Use actual parser
    // For now, create a simple placeholder AST based on the content
    let mut builder = NodeBuilder::new();
    
    let module = builder.build(
        Span::new(FileId::new(0), x_parser::span::ByteOffset::new(0), x_parser::span::ByteOffset::new(0)),
        AstNodeKind::Module {
            name: Symbol::intern("main"),
            items: Vec::new(),
            visibility: Visibility::Public,
        },
    );
    
    Ok(builder.build(
        Span::new(FileId::new(0), x_parser::span::ByteOffset::new(0), x_parser::span::ByteOffset::new(0)),
        AstNodeKind::CompilationUnit {
            modules: vec![module],
            imports: Vec::new(),
            exports: Vec::new(),
        },
    ))
}

/// Save text AST format
fn save_text_ast(ast: &PersistentAstNode, format: Format) -> Result<Vec<u8>> {
    let syntax_style = format.syntax_style()
        .context("Format does not support text output")?;
    
    // TODO: Implement actual AST to text conversion
    // For now, generate a simple placeholder
    let text = match syntax_style {
        SyntaxStyle::RustLike => generate_rust_like_text(ast),
        SyntaxStyle::OCaml => generate_ocaml_text(ast),
        SyntaxStyle::SExpression => generate_sexp_text(ast),
        SyntaxStyle::Haskell => generate_haskell_text(ast),
    };
    
    Ok(text.into_bytes())
}

/// Generate Rust-like syntax text (placeholder)
fn generate_rust_like_text(ast: &PersistentAstNode) -> String {
    format!("// Generated from AST node: {:?}\n// TODO: Implement text generation\npub fn main() {{\n    println!(\"Hello from x Language!\");\n}}\n", ast.id())
}

/// Generate OCaml syntax text (placeholder)
fn generate_ocaml_text(ast: &PersistentAstNode) -> String {
    format!("(* Generated from AST node: {:?} *)\n(* TODO: Implement text generation *)\nlet main () = print_endline \"Hello from x Language!\"\n", ast.id())
}

/// Generate S-expression text (placeholder)
fn generate_sexp_text(ast: &PersistentAstNode) -> String {
    format!(";; Generated from AST node: {:?}\n;; TODO: Implement text generation\n(def main () (print-line \"Hello from x Language!\"))\n", ast.id())
}

/// Generate Haskell syntax text (placeholder)
fn generate_haskell_text(ast: &PersistentAstNode) -> String {
    format!("-- Generated from AST node: {:?}\n-- TODO: Implement text generation\nmain :: IO ()\nmain = putStrLn \"Hello from x Language!\"\n", ast.id())
}

/// Convert AST to PersistentAstNode
fn convert_ast_to_persistent(cu: &x_parser::ast::CompilationUnit) -> Result<PersistentAstNode> {
    let mut builder = NodeBuilder::new();
    
    // Create module nodes
    let mut modules = Vec::new();
    let module_node = builder.build(
        convert_span(&cu.module.span),
        AstNodeKind::Module {
            name: cu.module.name.segments.first().copied().unwrap_or(Symbol::intern("main")),
            items: Vec::new(), // TODO: Convert items
            visibility: Visibility::Public,
        },
    );
    modules.push(module_node);
    
    Ok(builder.build(
        convert_span(&cu.span),
        AstNodeKind::CompilationUnit {
            modules,
            imports: Vec::new(), // TODO: Convert imports
            exports: Vec::new(), // TODO: Convert exports
        },
    ))
}

/// Convert PersistentAstNode to AST
fn convert_persistent_to_ast(ast: &PersistentAstNode) -> Result<x_parser::ast::CompilationUnit> {
    use x_parser::ast::*;
    
    
    match &ast.kind {
        AstNodeKind::CompilationUnit { modules, imports: _, exports: _ } => {
            // Convert the first module (assuming single module for now)
            let persistent_module = modules.first()
                .context("CompilationUnit must have at least one module")?;
            
            let module = convert_persistent_module_to_ast(persistent_module)?;
            
            Ok(CompilationUnit {
                module,
                span: convert_persistent_span_to_ast(&ast.span()),
            })
        }
        _ => bail!("Expected CompilationUnit, got {:?}", ast.kind),
    }
}

/// Convert PersistentAstNode module to AST Module
fn convert_persistent_module_to_ast(module_ast: &PersistentAstNode) -> Result<x_parser::ast::Module> {
    use x_parser::ast::*;
    
    match &module_ast.kind {
        AstNodeKind::Module { name, items, visibility: _ } => {
            let mut ast_items = Vec::new();
            
            // Convert each item
            for item in items {
                ast_items.push(convert_persistent_item_to_ast(&item)?);
            }
            
            Ok(Module {
                name: ModulePath::new(
                    vec![*name],
                    convert_persistent_span_to_ast(&module_ast.span())
                ),
                documentation: None,
                exports: None,
                imports: Vec::new(),
                items: ast_items,
                span: convert_persistent_span_to_ast(&module_ast.span()),
            })
        }
        _ => bail!("Expected Module, got {:?}", module_ast.kind),
    }
}

/// Convert PersistentAstNode item to AST Item
fn convert_persistent_item_to_ast(item_ast: &PersistentAstNode) -> Result<x_parser::ast::Item> {
    use x_parser::ast::*;
    
    match &item_ast.kind {
        AstNodeKind::ValueDef { name, type_annotation, body, visibility, purity } => {
            let ast_body = convert_persistent_expr_to_ast(body)?;
            let ast_type_annotation = if let Some(type_ann) = type_annotation {
                Some(Box::new(convert_persistent_type_to_ast(&type_ann)?))
            } else {
                None
            };
            
            Ok(Item::ValueDef(ValueDef {
                name: name.clone(),
                documentation: None,
                type_annotation: ast_type_annotation.map(|t| *t),
                parameters: Vec::new(), // TODO: Extract from lambda if needed
                body: ast_body,
                visibility: convert_persistent_visibility_to_ast(&visibility),
                purity: convert_persistent_purity_to_ast(&purity),
                span: convert_persistent_span_to_ast(&item_ast.span()),
            }))
        }
        _ => bail!("Unsupported item type: {:?}", item_ast.kind),
    }
}

/// Convert PersistentAstNode expression to AST Expr
fn convert_persistent_expr_to_ast(expr_ast: &PersistentAstNode) -> Result<x_parser::ast::Expr> {
    use x_parser::ast::*;
    
    match &expr_ast.kind {
        AstNodeKind::Lambda { parameters, body, effect_annotation: _ } => {
            let ast_body = Box::new(convert_persistent_expr_to_ast(&body)?);
            let mut ast_parameters = Vec::new();
            
            for param in parameters {
                ast_parameters.push(convert_persistent_pattern_to_ast_from_param(&param)?);
            }
            
            Ok(Expr::Lambda {
                parameters: ast_parameters,
                body: ast_body,
                span: convert_persistent_span_to_ast(&expr_ast.span()),
            })
        }
        AstNodeKind::Application { function, arguments } => {
            let ast_function = Box::new(convert_persistent_expr_to_ast(&function)?);
            let mut ast_arguments = Vec::new();
            
            for arg in arguments {
                ast_arguments.push(convert_persistent_expr_to_ast(&arg)?);
            }
            
            Ok(Expr::App(
                ast_function,
                ast_arguments,
                convert_persistent_span_to_ast(&expr_ast.span())
            ))
        }
        AstNodeKind::Variable { name } => {
            Ok(Expr::Var(
                name.clone(),
                convert_persistent_span_to_ast(&expr_ast.span())
            ))
        }
        AstNodeKind::Literal { value } => {
            let ast_literal = match value {
                x_parser::persistent_ast::LiteralValue::Integer(n) => Literal::Integer(n.clone()),
                x_parser::persistent_ast::LiteralValue::Float(f) => Literal::Float(f.clone()),
                x_parser::persistent_ast::LiteralValue::String(s) => Literal::String(s.clone()),
                x_parser::persistent_ast::LiteralValue::Boolean(b) => Literal::Bool(b.clone()),
                x_parser::persistent_ast::LiteralValue::Unit => Literal::Unit,
                x_parser::persistent_ast::LiteralValue::Char(c) => Literal::String(c.to_string()),
            };
            
            Ok(Expr::Literal(
                ast_literal,
                convert_persistent_span_to_ast(&expr_ast.span())
            ))
        }
        AstNodeKind::Let { bindings, body } => {
            if let Some(binding) = bindings.first() {
                let pattern = convert_persistent_pattern_to_ast(&binding.pattern)?;
                let value = Box::new(convert_persistent_expr_to_ast(&binding.value)?);
                let body_expr = Box::new(convert_persistent_expr_to_ast(&body)?);
                
                Ok(Expr::Let {
                    pattern,
                    type_annotation: None,
                    value,
                    body: body_expr,
                    span: convert_persistent_span_to_ast(&expr_ast.span()),
                })
            } else {
                bail!("Let expression must have at least one binding")
            }
        }
        _ => bail!("Unsupported expression type: {:?}", expr_ast.kind),
    }
}

/// Convert PersistentAstNode pattern to AST Pattern
fn convert_persistent_pattern_to_ast(pattern_ast: &PersistentAstNode) -> Result<x_parser::ast::Pattern> {
    use x_parser::ast::*;
    
    match &pattern_ast.kind {
        AstNodeKind::PatternVariable { name } => {
            Ok(Pattern::Variable(
                name.clone(),
                convert_persistent_span_to_ast(&pattern_ast.span())
            ))
        }
        _ => bail!("Unsupported pattern type: {:?}", pattern_ast.kind),
    }
}

/// Convert Parameter to AST Pattern (helper function)
fn convert_persistent_pattern_to_ast_from_param(param: &x_parser::persistent_ast::Parameter) -> Result<x_parser::ast::Pattern> {
    use x_parser::ast::*;
    use x_parser::span::{FileId, ByteOffset};
    
    Ok(Pattern::Variable(
        param.name,
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1))
    ))
}

/// Convert PersistentAstNode type to AST Type
fn convert_persistent_type_to_ast(type_ast: &PersistentAstNode) -> Result<x_parser::ast::Type> {
    use x_parser::ast::*;
    
    match &type_ast.kind {
        AstNodeKind::TypeReference { name, type_args } => {
            if type_args.is_empty() {
                Ok(Type::Con(
                    name.clone(),
                    convert_persistent_span_to_ast(&type_ast.span())
                ))
            } else {
                let mut ast_args = Vec::new();
                for arg in type_args {
                    ast_args.push(convert_persistent_type_to_ast(&arg)?);
                }
                
                Ok(Type::App(
                    Box::new(Type::Con(name.clone(), convert_persistent_span_to_ast(&type_ast.span()))),
                    ast_args,
                    convert_persistent_span_to_ast(&type_ast.span())
                ))
            }
        }
        AstNodeKind::FunctionType { parameters, return_type, effects: _ } => {
            // For simplicity, convert to nested function types
            let mut result_type = convert_persistent_type_to_ast(&return_type)?;
            
            // Build function type right-to-left
            for param_type in parameters.iter().rev() {
                let param_ast_type = convert_persistent_type_to_ast(param_type)?;
                result_type = Type::Fun {
                    params: vec![param_ast_type],
                    return_type: Box::new(result_type),
                    effects: x_parser::ast::EffectSet::empty(convert_persistent_span_to_ast(&type_ast.span())),
                    span: convert_persistent_span_to_ast(&type_ast.span())
                };
            }
            
            Ok(result_type)
        }
        _ => bail!("Unsupported type: {:?}", type_ast.kind),
    }
}

/// Convert PersistentAstNode span to AST span
fn convert_persistent_span_to_ast(span: &Span) -> x_parser::span::Span {
    x_parser::span::Span::new(
        x_parser::span::FileId::new(span.file_id.as_u32()),
        span.start,
        span.end
    )
}

/// Convert persistent visibility to AST visibility
fn convert_persistent_visibility_to_ast(visibility: &x_parser::persistent_ast::Visibility) -> x_parser::ast::Visibility {
    match visibility {
        x_parser::persistent_ast::Visibility::Public => x_parser::ast::Visibility::Public,
        x_parser::persistent_ast::Visibility::Private => x_parser::ast::Visibility::Private,
        x_parser::persistent_ast::Visibility::Crate => x_parser::ast::Visibility::Crate,
        x_parser::persistent_ast::Visibility::Module(_) => x_parser::ast::Visibility::Public, // Default fallback
    }
}

/// Convert persistent purity to AST purity
fn convert_persistent_purity_to_ast(purity: &x_parser::persistent_ast::Purity) -> x_parser::ast::Purity {
    match purity {
        x_parser::persistent_ast::Purity::Pure => x_parser::ast::Purity::Pure,
        x_parser::persistent_ast::Purity::Impure => x_parser::ast::Purity::Impure,
        x_parser::persistent_ast::Purity::Inferred => x_parser::ast::Purity::Inferred,
    }
}

/// Convert span from AST to persistent format
fn convert_span(span: &x_parser::span::Span) -> Span {
    Span {
        file_id: FileId::new(span.file_id.as_u32()),
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detection() {
        assert_eq!(detect_format(Path::new("test.x")).unwrap(), Format::Binary);
        assert_eq!(detect_format(Path::new("test.rustic.x")).unwrap(), Format::Rustic);
        assert_eq!(detect_format(Path::new("test.ocaml.x")).unwrap(), Format::OCaml);
        assert_eq!(detect_format(Path::new("test.lisp.x")).unwrap(), Format::SExpression);
        assert_eq!(detect_format(Path::new("test.haskell.x")).unwrap(), Format::Haskell);
        assert_eq!(detect_format(Path::new("test.json")).unwrap(), Format::Json);
    }
    
    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from_str("binary").unwrap(), Format::Binary);
        assert_eq!(Format::from_str("rustic").unwrap(), Format::Rustic);
        assert_eq!(Format::from_str("ocaml").unwrap(), Format::OCaml);
        assert_eq!(Format::from_str("lisp").unwrap(), Format::SExpression);
        assert_eq!(Format::from_str("haskell").unwrap(), Format::Haskell);
        assert_eq!(Format::from_str("json").unwrap(), Format::Json);
    }
    
    #[test]
    fn test_default_extension() {
        assert_eq!(Format::Binary.default_extension(), "x");
        assert_eq!(Format::Rustic.default_extension(), "rustic.x");
        assert_eq!(Format::OCaml.default_extension(), "ocaml.x");
        assert_eq!(Format::SExpression.default_extension(), "lisp.x");
        assert_eq!(Format::Haskell.default_extension(), "haskell.x");
        assert_eq!(Format::Json.default_extension(), "json");
    }
}