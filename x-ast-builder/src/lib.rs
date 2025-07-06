//! AST Builder API for x Language
//! 
//! This module provides a fluent API for programmatically constructing
//! x Language AST nodes without writing source code.

use x_parser::ast::*;
use x_parser::{Symbol, Span, FileId, span::ByteOffset};

pub mod builder;
pub mod dsl;

pub use builder::*;
pub use dsl::*;

/// Main AST builder context
pub struct AstBuilder {
    file_id: FileId,
    current_offset: u32,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            file_id: FileId::new(0),
            current_offset: 0,
        }
    }
    
    pub fn with_file_id(mut self, file_id: FileId) -> Self {
        self.file_id = file_id;
        self
    }
    
    /// Create a new module builder
    pub fn module(&mut self, name: &str) -> ModuleBuilder {
        ModuleBuilder::new(self, name)
    }
    
    /// Create a new expression builder
    pub fn expr(&mut self) -> ExprBuilder {
        ExprBuilder::new(self)
    }
    
    /// Create a new type builder
    pub fn typ(&mut self) -> TypeBuilder {
        TypeBuilder::new(self)
    }
    
    /// Create a new pattern builder
    pub fn pattern(&mut self) -> PatternBuilder {
        PatternBuilder::new(self)
    }
    
    /// Create a span for the current position
    pub fn span(&mut self) -> Span {
        let start = self.current_offset;
        self.current_offset += 1;
        Span::new(
            self.file_id,
            ByteOffset::new(start),
            ByteOffset::new(self.current_offset),
        )
    }
    
    /// Create a span with specific length
    pub fn span_with_len(&mut self, len: u32) -> Span {
        let start = self.current_offset;
        self.current_offset += len;
        Span::new(
            self.file_id,
            ByteOffset::new(start),
            ByteOffset::new(self.current_offset),
        )
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Example usage demonstrating the AST builder API
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_module_construction() {
        let mut builder = AstBuilder::new();
        
        // Build: module Main let x = 42
        let module = builder.module("Main")
            .value("x", |e| e.int(42))
            .build();
        
        assert_eq!(module.name.segments[0].as_str(), "Main");
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_function_construction() {
        let mut builder = AstBuilder::new();
        
        // Build: let add = fun x y -> x + y
        let module = builder.module("Math")
            .function("add", vec!["x", "y"], |e| {
                e.binop("+", |e| e.var("x"), |e| e.var("y"))
            })
            .build();
        
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_complex_expression() {
        let mut builder = AstBuilder::new();
        
        // Build: if x > 0 then x * 2 else 0
        let expr = builder.expr()
            .if_then_else(
                |e| e.binop(">", |e| e.var("x"), |e| e.int(0)),
                |e| e.binop("*", |e| e.var("x"), |e| e.int(2)),
                |e| e.int(0)
            )
            .build();
        
        match expr {
            Expr::If { .. } => (),
            _ => panic!("Expected If expression"),
        }
    }
}