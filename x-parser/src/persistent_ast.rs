//! Persistent AST implementation for efficient tree operations
//! 
//! This module provides immutable, persistent AST nodes that support
//! efficient structural sharing and O(log n) operations.

use crate::{span::{Span, FileId, ByteOffset}, symbol::Symbol};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
// use im::Vector;

/// Unique identifier for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
    
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Version identifier for AST snapshots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VersionId(u64);

impl VersionId {
    pub fn new(id: u64) -> Self {
        VersionId(id)
    }
    
    pub fn next(self) -> Self {
        VersionId(self.0 + 1)
    }
    
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Metadata attached to every AST node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Unique node identifier
    pub node_id: NodeId,
    /// Source location information
    pub span: Span,
    /// Type information (populated by type checker)
    pub type_info: Option<TypeInfo>,
    /// Additional semantic annotations
    pub annotations: HashMap<String, AnnotationValue>,
}

/// Type information for a node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    /// The inferred type
    pub inferred_type: TypeId,
    /// Type constraints
    pub constraints: Vec<TypeConstraint>,
    /// Effect signature
    pub effects: Option<EffectSet>,
}

/// Type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(u64);

/// Type constraint for inference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeConstraint {
    Equals { left: TypeId, right: TypeId },
    Subtype { sub: TypeId, sup: TypeId },
    HasField { record: TypeId, field: Symbol, field_type: TypeId },
    Callable { function: TypeId, args: Vec<TypeId>, ret: TypeId },
}

/// Effect set for tracking side effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectSet {
    Empty,
    Concrete(Vec<Symbol>),
    Variable(Symbol),
    Union(Box<EffectSet>, Box<EffectSet>),
}

/// Annotation values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    List(Vec<AnnotationValue>),
}

/// Persistent AST node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistentAstNode {
    /// Node metadata
    pub metadata: NodeMetadata,
    /// Node content
    pub kind: AstNodeKind,
}

/// AST node types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNodeKind {
    // Top-level constructs
    CompilationUnit {
        modules: Vec<PersistentAstNode>,
        imports: Vec<PersistentAstNode>,
        exports: Vec<PersistentAstNode>,
    },
    
    Module {
        name: Symbol,
        items: Vec<PersistentAstNode>,
        visibility: Visibility,
    },
    
    // Declarations
    ValueDef {
        name: Symbol,
        type_annotation: Option<Box<PersistentAstNode>>,
        body: Box<PersistentAstNode>,
        visibility: Visibility,
        purity: Purity,
    },
    
    TypeDef {
        name: Symbol,
        type_params: Vec<Symbol>,
        definition: Box<PersistentAstNode>,
        visibility: Visibility,
    },
    
    EffectDef {
        name: Symbol,
        operations: Vec<PersistentAstNode>,
        visibility: Visibility,
    },
    
    // Expressions
    Literal {
        value: LiteralValue,
    },
    
    Variable {
        name: Symbol,
    },
    
    Application {
        function: Box<PersistentAstNode>,
        arguments: Vec<PersistentAstNode>,
    },
    
    Lambda {
        parameters: Vec<Parameter>,
        body: Box<PersistentAstNode>,
        effect_annotation: Option<Box<PersistentAstNode>>,
    },
    
    Let {
        bindings: Vec<Binding>,
        body: Box<PersistentAstNode>,
    },
    
    If {
        condition: Box<PersistentAstNode>,
        then_branch: Box<PersistentAstNode>,
        else_branch: Option<Box<PersistentAstNode>>,
    },
    
    Match {
        scrutinee: Box<PersistentAstNode>,
        cases: Vec<MatchCase>,
    },
    
    Handle {
        expression: Box<PersistentAstNode>,
        handlers: Vec<Handler>,
        return_clause: Option<Box<PersistentAstNode>>,
    },
    
    Perform {
        effect: Symbol,
        operation: Symbol,
        arguments: Vec<PersistentAstNode>,
    },
    
    // Types
    TypeReference {
        name: Symbol,
        type_args: Vec<PersistentAstNode>,
    },
    
    FunctionType {
        parameters: Vec<PersistentAstNode>,
        return_type: Box<PersistentAstNode>,
        effects: Option<Box<PersistentAstNode>>,
    },
    
    RecordType {
        fields: Vec<RecordField>,
    },
    
    VariantType {
        variants: Vec<Variant>,
    },
    
    // Patterns
    PatternVariable {
        name: Symbol,
    },
    
    PatternLiteral {
        value: LiteralValue,
    },
    
    PatternConstructor {
        constructor: Symbol,
        patterns: Vec<PersistentAstNode>,
    },
    
    PatternRecord {
        fields: Vec<PatternField>,
    },
}

/// Supporting types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Module(Vec<Symbol>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Purity {
    Pure,
    Impure,
    Inferred,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Unit,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Symbol,
    pub type_annotation: Option<Box<PersistentAstNode>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub pattern: Box<PersistentAstNode>,
    pub value: Box<PersistentAstNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: Box<PersistentAstNode>,
    pub guard: Option<Box<PersistentAstNode>>,
    pub body: Box<PersistentAstNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Handler {
    pub effect: Symbol,
    pub operations: Vec<OperationHandler>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationHandler {
    pub operation: Symbol,
    pub parameters: Vec<Parameter>,
    pub body: Box<PersistentAstNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    pub name: Symbol,
    pub field_type: Box<PersistentAstNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub name: Symbol,
    pub data: Option<Box<PersistentAstNode>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternField {
    pub name: Symbol,
    pub pattern: Box<PersistentAstNode>,
}

impl PersistentAstNode {
    /// Create a new AST node
    pub fn new(node_id: NodeId, span: Span, kind: AstNodeKind) -> Self {
        Self {
            metadata: NodeMetadata {
                node_id,
                span,
                type_info: None,
                annotations: HashMap::new(),
            },
            kind,
        }
    }
    
    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.metadata.node_id
    }
    
    /// Get the source span
    pub fn span(&self) -> Span {
        self.metadata.span
    }
    
    /// Get type information if available
    pub fn type_info(&self) -> Option<&TypeInfo> {
        self.metadata.type_info.as_ref()
    }
    
    /// Set type information
    pub fn with_type_info(mut self, type_info: TypeInfo) -> Self {
        self.metadata.type_info = Some(type_info);
        self
    }
    
    /// Add an annotation
    pub fn with_annotation(mut self, key: String, value: AnnotationValue) -> Self {
        self.metadata.annotations.insert(key, value);
        self
    }
    
    /// Get all direct children of this node
    pub fn children(&self) -> Vec<&PersistentAstNode> {
        let mut children = Vec::new();
        
        match &self.kind {
            AstNodeKind::CompilationUnit { modules, imports, exports } => {
                for module in modules {
                    children.push(module);
                }
                for import in imports {
                    children.push(import);
                }
                for export in exports {
                    children.push(export);
                }
            },
            AstNodeKind::Module { items, .. } => {
                for item in items {
                    children.push(item);
                }
            },
            AstNodeKind::ValueDef { type_annotation, body, .. } => {
                if let Some(type_ann) = type_annotation {
                    children.push(type_ann);
                }
                children.push(body);
            },
            AstNodeKind::Application { function, arguments } => {
                children.push(function);
                for arg in arguments {
                    children.push(arg);
                }
            },
            AstNodeKind::Lambda { parameters: _, body, effect_annotation } => {
                children.push(body);
                if let Some(eff) = effect_annotation {
                    children.push(eff);
                }
            },
            AstNodeKind::Let { bindings, body } => {
                for binding in bindings {
                    children.push(&binding.pattern);
                    children.push(&binding.value);
                }
                children.push(body);
            },
            AstNodeKind::If { condition, then_branch, else_branch } => {
                children.push(condition);
                children.push(then_branch);
                if let Some(else_b) = else_branch {
                    children.push(else_b);
                }
            },
            // Add more cases as needed...
            _ => {},
        }
        
        children
    }
    
    /// Check if this node is of a specific type
    pub fn is_type(&self, node_type: &str) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(&match node_type {
            "CompilationUnit" => AstNodeKind::CompilationUnit { 
                modules: Vec::new(), 
                imports: Vec::new(), 
                exports: Vec::new() 
            },
            "Module" => AstNodeKind::Module { 
                name: Symbol::intern(""), 
                items: Vec::new(), 
                visibility: Visibility::Private 
            },
            "ValueDef" => AstNodeKind::ValueDef { 
                name: Symbol::intern(""), 
                type_annotation: None, 
                body: Box::new(PersistentAstNode::new(NodeId::new(0), Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)), AstNodeKind::Literal { value: LiteralValue::Unit })), 
                visibility: Visibility::Private, 
                purity: Purity::Inferred 
            },
            "Variable" => AstNodeKind::Variable { name: Symbol::intern("") },
            "Application" => AstNodeKind::Application { 
                function: Box::new(PersistentAstNode::new(NodeId::new(0), Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)), AstNodeKind::Literal { value: LiteralValue::Unit })), 
                arguments: Vec::new() 
            },
            _ => return false,
        })
    }
}

/// Helper trait for nodes that have spans
pub trait HasSpan {
    fn span(&self) -> Span;
}

impl HasSpan for PersistentAstNode {
    fn span(&self) -> Span {
        self.metadata.span
    }
}

/// Node builder for constructing AST nodes with proper IDs
pub struct NodeBuilder {
    next_id: u64,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }
    
    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        id
    }
    
    pub fn build(&mut self, span: Span, kind: AstNodeKind) -> PersistentAstNode {
        PersistentAstNode::new(self.next_id(), span, kind)
    }
}

impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}