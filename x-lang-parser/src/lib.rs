//! x Language Parser
//! 
//! This crate provides parsing and lexical analysis functionality for x Language.
//! It supports multiple syntax styles (OCaml, S-expression, Haskell, Rust-like)
//! and handles conversion from text to AST representation.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod span;
pub mod symbol;
pub mod token;
pub mod binary;
pub mod error;

// Re-export core types
pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
pub use span::{Span, FileId};
pub use symbol::Symbol;
pub use token::{Token, TokenKind};
pub use error::{ParseError, Result};

/// Parse source code in the specified syntax style
pub fn parse_source(source: &str, file_id: FileId, syntax_style: SyntaxStyle) -> Result<CompilationUnit> {
    let mut parser = Parser::new(source, file_id)?;
    parser.set_syntax_style(syntax_style);
    parser.parse()
}

/// Syntax styles supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxStyle {
    OCaml,
    SExpression,
    Haskell,
    RustLike,
}

impl Default for SyntaxStyle {
    fn default() -> Self {
        SyntaxStyle::OCaml
    }
}

/// Parse result containing AST and metadata
#[derive(Debug)]
pub struct ParseResult {
    pub ast: CompilationUnit,
    pub syntax_style: SyntaxStyle,
    pub file_id: FileId,
    pub source_hash: u64,
    pub parse_time: std::time::Duration,
}

/// Parse source with detailed result information
pub fn parse_with_metadata(source: &str, file_id: FileId, syntax_style: SyntaxStyle) -> Result<ParseResult> {
    let start_time = std::time::Instant::now();
    let ast = parse_source(source, file_id, syntax_style)?;
    let parse_time = start_time.elapsed();
    
    // Calculate source hash for caching
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let source_hash = hasher.finish();
    
    Ok(ParseResult {
        ast,
        syntax_style,
        file_id,
        source_hash,
        parse_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let source = "let x = 42";
        let file_id = FileId::new(0);
        let result = parse_source(source, file_id, SyntaxStyle::OCaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_syntax_styles() {
        let ocaml_source = "let x = 42";
        let sexp_source = "(let x 42)";
        
        let file_id = FileId::new(0);
        
        let ocaml_result = parse_source(ocaml_source, file_id, SyntaxStyle::OCaml);
        let sexp_result = parse_source(sexp_source, file_id, SyntaxStyle::SExpression);
        
        assert!(ocaml_result.is_ok());
        assert!(sexp_result.is_ok());
    }

    #[test]
    fn test_parse_with_metadata() {
        let source = "let x = 42";
        let file_id = FileId::new(0);
        let result = parse_with_metadata(source, file_id, SyntaxStyle::OCaml);
        
        assert!(result.is_ok());
        let parse_result = result.unwrap();
        assert_eq!(parse_result.syntax_style, SyntaxStyle::OCaml);
        assert_eq!(parse_result.file_id, file_id);
        assert!(parse_result.parse_time.as_nanos() > 0);
    }
}