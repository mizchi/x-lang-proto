//! Multi-syntax support for x Language
//! 
//! This module provides support for multiple textual representations of the same AST.
//! Users can parse and pretty-print code in different syntactic styles while maintaining
//! the same underlying semantic structure.

pub mod sexp;
pub mod printer;
pub mod converter;

use crate::{ast::*, span::FileId};
use crate::error::{ParseError as Error, Result};
use std::fmt;

/// Supported syntax styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxStyle {
    /// S-expression syntax (Lisp-like)
    SExp,
}

impl fmt::Display for SyntaxStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxStyle::SExp => write!(f, "sexp"),
        }
    }
}

impl std::str::FromStr for SyntaxStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sexp" | "sexpr" | "lisp" => Ok(SyntaxStyle::SExp),
            _ => Err(Error::Parse {
                message: format!("Unknown syntax style: {s}"),
            }),
        }
    }
}

/// Configuration for parsing and printing
#[derive(Debug, Clone)]
pub struct SyntaxConfig {
    pub style: SyntaxStyle,
    pub indent_size: usize,
    pub use_tabs: bool,
    pub max_line_length: usize,
    pub preserve_comments: bool,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        SyntaxConfig {
            style: SyntaxStyle::SExp,
            indent_size: 2,
            use_tabs: false,
            max_line_length: 100,
            preserve_comments: true,
        }
    }
}

/// Universal parser interface for all syntax styles
pub trait SyntaxParser {
    /// Parse source code into AST
    fn parse(&mut self, input: &str, file_id: FileId) -> Result<CompilationUnit>;
    
    /// Parse expression from string (for REPL/testing)
    fn parse_expression(&mut self, input: &str, file_id: FileId) -> Result<Expr>;
    
    /// Get the syntax style this parser handles
    fn syntax_style(&self) -> SyntaxStyle;
}

/// Universal printer interface for all syntax styles
pub trait SyntaxPrinter {
    /// Print AST to source code
    fn print(&self, ast: &CompilationUnit, config: &SyntaxConfig) -> Result<String>;
    
    /// Print expression to string (for REPL/testing)
    fn print_expression(&self, expr: &Expr, config: &SyntaxConfig) -> Result<String>;
    
    /// Print type to string
    fn print_type(&self, typ: &Type, config: &SyntaxConfig) -> Result<String>;
    
    /// Get the syntax style this printer handles
    fn syntax_style(&self) -> SyntaxStyle;
}

/// Multi-syntax facade that coordinates between different parsers and printers
pub struct MultiSyntax {
    parsers: std::collections::HashMap<SyntaxStyle, Box<dyn SyntaxParser>>,
    printers: std::collections::HashMap<SyntaxStyle, Box<dyn SyntaxPrinter>>,
}

impl MultiSyntax {
    pub fn new() -> Self {
        MultiSyntax {
            parsers: std::collections::HashMap::new(),
            printers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a parser for a specific syntax style
    pub fn register_parser(&mut self, parser: Box<dyn SyntaxParser>) {
        let style = parser.syntax_style();
        self.parsers.insert(style, parser);
    }
    
    /// Register a printer for a specific syntax style
    pub fn register_printer(&mut self, printer: Box<dyn SyntaxPrinter>) {
        let style = printer.syntax_style();
        self.printers.insert(style, printer);
    }
    
    /// Parse code in the specified syntax style
    pub fn parse(&mut self, input: &str, style: SyntaxStyle, file_id: FileId) -> Result<CompilationUnit> {
        match self.parsers.get_mut(&style) {
            Some(parser) => parser.parse(input, file_id),
            None => Err(Error::Parse {
                message: format!("No parser registered for syntax style: {style}"),
            }),
        }
    }
    
    /// Parse expression in the specified syntax style
    pub fn parse_expression(&mut self, input: &str, style: SyntaxStyle, file_id: FileId) -> Result<Expr> {
        match self.parsers.get_mut(&style) {
            Some(parser) => parser.parse_expression(input, file_id),
            None => Err(Error::Parse {
                message: format!("No parser registered for syntax style: {style}"),
            }),
        }
    }
    
    /// Print AST in the specified syntax style
    pub fn print(&self, ast: &CompilationUnit, config: &SyntaxConfig) -> Result<String> {
        match self.printers.get(&config.style) {
            Some(printer) => printer.print(ast, config),
            None => Err(Error::Parse {
                message: format!("No printer registered for syntax style: {}", config.style),
            }),
        }
    }
    
    /// Print expression in the specified syntax style
    pub fn print_expression(&self, expr: &Expr, config: &SyntaxConfig) -> Result<String> {
        match self.printers.get(&config.style) {
            Some(printer) => printer.print_expression(expr, config),
            None => Err(Error::Parse {
                message: format!("No printer registered for syntax style: {}", config.style),
            }),
        }
    }
    
    /// Convert code from one syntax style to another
    pub fn convert(&mut self, input: &str, from: SyntaxStyle, to: SyntaxStyle, file_id: FileId) -> Result<String> {
        // Parse with source syntax
        let ast = self.parse(input, from, file_id)?;
        
        // Print with target syntax
        let config = SyntaxConfig {
            style: to,
            ..Default::default()
        };
        self.print(&ast, &config)
    }
    
    /// Get list of supported syntax styles
    pub fn supported_styles(&self) -> Vec<SyntaxStyle> {
        self.parsers.keys().copied().collect()
    }
}

impl Default for MultiSyntax {
    fn default() -> Self {
        let mut multi = MultiSyntax::new();
        
        // Register all parsers and printers
        multi.register_parser(Box::new(sexp::SExpParser::new()));
        multi.register_printer(Box::new(sexp::SExpPrinter::new()));
        
        multi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_syntax_style_parsing() {
        assert_eq!("sexp".parse::<SyntaxStyle>().unwrap(), SyntaxStyle::SExp);
        
        assert!("unknown".parse::<SyntaxStyle>().is_err());
    }

    #[test]
    fn test_syntax_config_default() {
        let config = SyntaxConfig::default();
        assert_eq!(config.style, SyntaxStyle::SExp);
        assert_eq!(config.indent_size, 2);
        assert!(!config.use_tabs);
    }

    #[test]
    fn test_multi_syntax_registration() {
        let mut multi = MultiSyntax::new();
        assert_eq!(multi.supported_styles().len(), 0);
        
        multi.register_parser(Box::new(sexp::SExpParser::new()));
        assert_eq!(multi.supported_styles().len(), 1);
        assert!(multi.supported_styles().contains(&SyntaxStyle::SExp));
    }
}