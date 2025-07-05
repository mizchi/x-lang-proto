//! x Language Parser
//! 
//! This crate provides parsing and lexical analysis functionality for x Language.
//! It supports multiple syntax styles (OCaml, S-expression, Haskell, Rust-like)
//! and handles conversion from text to AST representation.

pub mod ast;
pub mod persistent_ast;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod span;
pub mod symbol;
pub mod token;
pub mod binary;
pub mod error;

#[cfg(test)]
mod binary_tests;

// Re-export core types
pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
pub use crate::span::{Span, FileId};
pub use crate::symbol::Symbol;
pub use token::{Token, TokenKind};
pub use error::{ParseError, Result};

/// Parse source code in the specified syntax style
pub fn parse_source(source: &str, file_id: FileId, _syntax_style: SyntaxStyle) -> Result<CompilationUnit> {
    let mut parser = Parser::new(source, file_id)?;
    // TODO: Implement syntax style selection
    parser.parse()
}

/// Syntax styles supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
        let source = "module Main\n\nlet x = 42";
        let file_id = FileId::new(0);
        let result = parse_source(source, file_id, SyntaxStyle::OCaml);
        match result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_syntax_styles() {
        let ocaml_source = "module Main\n\nlet x = 42";
        let file_id = FileId::new(0);
        
        // Test OCaml style (currently the only implemented style)
        let ocaml_result = parse_source(ocaml_source, file_id, SyntaxStyle::OCaml);
        match ocaml_result {
            Ok(_) => {},
            Err(e) => panic!("OCaml parse failed: {:?}", e),
        }
        
        // TODO: Enable when S-expression parser is implemented
        // let sexp_source = "(module Main (let x 42))";
        // let sexp_result = parse_source(sexp_source, file_id, SyntaxStyle::SExpression);
        // match sexp_result {
        //     Ok(_) => {},
        //     Err(e) => panic!("S-expression parse failed: {:?}", e),
        // }
    }

    #[test]
    fn test_parse_with_metadata() {
        let source = "module Main\n\nlet x = 42";
        let file_id = FileId::new(0);
        let result = parse_with_metadata(source, file_id, SyntaxStyle::OCaml);
        
        match result {
            Ok(parse_result) => {
                assert_eq!(parse_result.syntax_style, SyntaxStyle::OCaml);
                assert_eq!(parse_result.file_id, file_id);
                assert!(parse_result.parse_time.as_nanos() > 0);
            },
            Err(e) => panic!("Parse with metadata failed: {:?}", e),
        }
    }
}