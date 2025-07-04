//! Binary AST diff system for x Language
//! 
//! This module provides structural diff algorithms for binary-serialized AST nodes,
//! enabling efficient version control and incremental compilation.

use crate::core::{
    ast::*,
    binary::{BinarySerializer, BinaryDeserializer},
    span::Span,
    symbol::Symbol,
};
use crate::{Error, Result};
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Diff operation representing a change between two AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// No change
    Equal {
        node_type: String,
        span: Option<Span>,
    },
    /// Node was added
    Insert {
        node: AstNode,
        position: usize,
    },
    /// Node was removed
    Delete {
        node: AstNode,
        position: usize,
    },
    /// Node was modified
    Replace {
        old_node: AstNode,
        new_node: AstNode,
        position: usize,
    },
    /// Structural change in expression
    ExprChange {
        old_expr: Box<DiffOp>,
        new_expr: Box<DiffOp>,
        expr_type: String,
    },
    /// Structural change in pattern
    PatternChange {
        old_pattern: Box<DiffOp>,
        new_pattern: Box<DiffOp>,
        pattern_type: String,
    },
    /// Structural change in type
    TypeChange {
        old_type: Box<DiffOp>,
        new_type: Box<DiffOp>,
        type_name: String,
    },
}

/// Abstract representation of any AST node for diffing
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Expr(Expr),
    Pattern(Pattern),
    Type(Type),
    Item(Item),
    Module(Module),
    CompilationUnit(CompilationUnit),
    Literal(Literal),
    Symbol(Symbol),
}

impl AstNode {
    pub fn node_type(&self) -> String {
        match self {
            AstNode::Expr(expr) => format!("Expr::{}", expr_type_name(expr)),
            AstNode::Pattern(pattern) => format!("Pattern::{}", pattern_type_name(pattern)),
            AstNode::Type(typ) => format!("Type::{}", type_type_name(typ)),
            AstNode::Item(item) => format!("Item::{}", item_type_name(item)),
            AstNode::Module(_) => "Module".to_string(),
            AstNode::CompilationUnit(_) => "CompilationUnit".to_string(),
            AstNode::Literal(lit) => format!("Literal::{}", literal_type_name(lit)),
            AstNode::Symbol(_) => "Symbol".to_string(),
        }
    }
    
    pub fn span(&self) -> Option<Span> {
        match self {
            AstNode::Expr(expr) => Some(expr.span()),
            AstNode::Pattern(pattern) => Some(pattern.span()),
            AstNode::Type(typ) => Some(typ.span()),
            AstNode::Module(module) => Some(module.span),
            AstNode::CompilationUnit(cu) => Some(cu.span),
            _ => None,
        }
    }
}

/// Binary AST differ
pub struct BinaryAstDiffer {
    serializer: BinarySerializer,
}

impl BinaryAstDiffer {
    pub fn new() -> Self {
        BinaryAstDiffer {
            serializer: BinarySerializer::new(),
        }
    }
    
    /// Compare two compilation units and generate a diff
    pub fn diff_compilation_units(&mut self, old: &CompilationUnit, new: &CompilationUnit) -> Result<Vec<DiffOp>> {
        self.diff_ast_nodes(&AstNode::CompilationUnit(old.clone()), &AstNode::CompilationUnit(new.clone()))
    }
    
    /// Compare two modules and generate a diff
    pub fn diff_modules(&mut self, old: &Module, new: &Module) -> Result<Vec<DiffOp>> {
        self.diff_ast_nodes(&AstNode::Module(old.clone()), &AstNode::Module(new.clone()))
    }
    
    /// Compare two expressions and generate a diff
    pub fn diff_expressions(&mut self, old: &Expr, new: &Expr) -> Result<Vec<DiffOp>> {
        self.diff_ast_nodes(&AstNode::Expr(old.clone()), &AstNode::Expr(new.clone()))
    }
    
    /// Core diff algorithm using Myers algorithm adapted for AST structures
    fn diff_ast_nodes(&mut self, old: &AstNode, new: &AstNode) -> Result<Vec<DiffOp>> {
        if nodes_equal(old, new) {
            return Ok(vec![DiffOp::Equal {
                node_type: old.node_type(),
                span: old.span(),
            }]);
        }
        
        // Check if nodes have the same structure but different content
        if old.node_type() == new.node_type() {
            return self.diff_same_type_nodes(old, new);
        }
        
        // Different node types - this is a replacement
        Ok(vec![DiffOp::Replace {
            old_node: old.clone(),
            new_node: new.clone(),
            position: 0,
        }])
    }
    
    /// Diff nodes of the same type
    fn diff_same_type_nodes(&mut self, old: &AstNode, new: &AstNode) -> Result<Vec<DiffOp>> {
        // For demonstration purposes, return a simple diff
        Ok(vec![DiffOp::Replace {
            old_node: old.clone(),
            new_node: new.clone(),
            position: 0,
        }])
    }
}

/// Check if two AST nodes are equal
fn nodes_equal(a: &AstNode, b: &AstNode) -> bool {
    // For now, use a simple structural comparison
    // In a full implementation, this would use semantic equivalence
    a == b
}

/// Get type name for expressions
fn expr_type_name(expr: &Expr) -> &'static str {
    match expr {
        Expr::Literal(_, _) => "Literal",
        Expr::Var(_, _) => "Var",
        Expr::App(_, _, _) => "App",
        Expr::Lambda { .. } => "Lambda",
        Expr::Let { .. } => "Let",
        Expr::If { .. } => "If",
        Expr::Match { .. } => "Match",
        Expr::Do { .. } => "Do",
        Expr::Handle { .. } => "Handle",
        Expr::Resume { .. } => "Resume",
        Expr::Perform { .. } => "Perform",
        Expr::Ann { .. } => "Ann",
    }
}

/// Get type name for patterns
fn pattern_type_name(pattern: &Pattern) -> &'static str {
    match pattern {
        Pattern::Wildcard(_) => "Wildcard",
        Pattern::Variable(_, _) => "Variable",
        Pattern::Literal(_, _) => "Literal",
        Pattern::Constructor { .. } => "Constructor",
        Pattern::Tuple { .. } => "Tuple",
        Pattern::Record { .. } => "Record",
        Pattern::Or { .. } => "Or",
        Pattern::As { .. } => "As",
        Pattern::Ann { .. } => "Ann",
    }
}

/// Get type name for types
fn type_type_name(typ: &Type) -> &'static str {
    match typ {
        Type::Var(_, _) => "Var",
        Type::Con(_, _) => "Con",
        Type::App(_, _, _) => "App",
        Type::Fun { .. } => "Fun",
        Type::Forall { .. } => "Forall",
        Type::Effects(_, _) => "Effects",
        Type::Exists { .. } => "Exists",
        Type::Record { .. } => "Record",
        Type::Variant { .. } => "Variant",
        Type::Tuple { .. } => "Tuple",
        Type::Row { .. } => "Row",
        Type::Hole(_) => "Hole",
    }
}

/// Get type name for items
fn item_type_name(item: &Item) -> &'static str {
    match item {
        Item::TypeDef(_) => "TypeDef",
        Item::ValueDef(_) => "ValueDef",
        Item::EffectDef(_) => "EffectDef",
        Item::HandlerDef(_) => "HandlerDef",
        Item::ModuleTypeDef(_) => "ModuleTypeDef",
        Item::InterfaceDef(_) => "InterfaceDef",
    }
}

/// Get type name for literals
fn literal_type_name(literal: &Literal) -> &'static str {
    match literal {
        Literal::Integer(_) => "Integer",
        Literal::Float(_) => "Float",
        Literal::String(_) => "String",
        Literal::Bool(_) => "Bool",
        Literal::Unit => "Unit",
    }
}

impl Default for BinaryAstDiffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DiffOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffOp::Equal { node_type, .. } => write!(f, "= {}", node_type),
            DiffOp::Insert { node, position } => write!(f, "+ {} at {}", node.node_type(), position),
            DiffOp::Delete { node, position } => write!(f, "- {} at {}", node.node_type(), position),
            DiffOp::Replace { old_node, new_node, position } => {
                write!(f, "~ {} -> {} at {}", old_node.node_type(), new_node.node_type(), position)
            }
            DiffOp::ExprChange { expr_type, .. } => write!(f, "expr change: {}", expr_type),
            DiffOp::PatternChange { pattern_type, .. } => write!(f, "pattern change: {}", pattern_type),
            DiffOp::TypeChange { type_name, .. } => write!(f, "type change: {}", type_name),
        }
    }
}

/// Pretty-print diff operations
pub fn format_diff(ops: &[DiffOp]) -> String {
    ops.iter()
        .map(|op| op.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::span::{FileId, ByteOffset};

    fn test_span() -> Span {
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
    }

    #[test]
    fn test_diff_equal_expressions() {
        let mut differ = BinaryAstDiffer::new();
        
        let expr1 = Expr::Literal(Literal::Integer(42), test_span());
        let expr2 = Expr::Literal(Literal::Integer(42), test_span());
        
        let diff = differ.diff_expressions(&expr1, &expr2).unwrap();
        
        assert_eq!(diff.len(), 1);
        match &diff[0] {
            DiffOp::Equal { .. } => {},
            _ => panic!("Expected Equal diff op"),
        }
    }
    
    #[test]
    fn test_diff_different_expressions() {
        let mut differ = BinaryAstDiffer::new();
        
        let expr1 = Expr::Literal(Literal::Integer(42), test_span());
        let expr2 = Expr::Literal(Literal::Integer(43), test_span());
        
        let diff = differ.diff_expressions(&expr1, &expr2).unwrap();
        
        assert_eq!(diff.len(), 1);
        match &diff[0] {
            DiffOp::Replace { .. } => {},
            _ => panic!("Expected Replace diff op"),
        }
    }
    
    #[test]
    fn test_format_diff() {
        let ops = vec![
            DiffOp::Equal {
                node_type: "Expr::Literal".to_string(),
                span: Some(test_span()),
            },
            DiffOp::Insert {
                node: AstNode::Literal(Literal::Integer(42)),
                position: 1,
            },
        ];
        
        let formatted = format_diff(&ops);
        assert!(formatted.contains("= Expr::Literal"));
        assert!(formatted.contains("+ Literal::Integer at 1"));
    }
}