//! High-performance indexing system for AST nodes
//! 
//! This module provides multiple specialized indices for different query patterns:
//! - Type-based lookup
//! - Symbol resolution
//! - Position-based queries  
//! - Dependency tracking

use crate::query::{AstQuery, QueryResult};
use x_parser::{
    persistent_ast::{PersistentAstNode, NodeId, AstNodeKind},
    span::Span,
    symbol::Symbol,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use im::OrdSet;

/// Collection of all indices for fast AST queries
#[derive(Debug, Clone)]
pub struct IndexCollection {
    /// Index nodes by their type
    pub type_index: TypeIndex,
    /// Index symbols and their references
    pub symbol_index: SymbolIndex,
    /// Index nodes by position for spatial queries
    pub position_index: PositionIndex,
    /// Index dependency relationships
    pub dependency_index: DependencyIndex,
    /// Index for parent-child relationships
    pub hierarchy_index: HierarchyIndex,
}

impl Default for IndexCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexCollection {
    pub fn new() -> Self {
        Self {
            type_index: TypeIndex::new(),
            symbol_index: SymbolIndex::new(),
            position_index: PositionIndex::new(),
            dependency_index: DependencyIndex::new(),
            hierarchy_index: HierarchyIndex::new(),
        }
    }
    
    /// Rebuild all indices from an AST
    pub fn rebuild_from_ast(&mut self, ast: &PersistentAstNode) {
        self.type_index.clear();
        self.symbol_index.clear();
        self.position_index.clear();
        self.dependency_index.clear();
        self.hierarchy_index.clear();
        
        self.index_node_recursive(ast, None);
    }
    
    /// Incrementally update indices for a node and its subtree
    pub fn update_node(&mut self, node: &PersistentAstNode, parent: Option<NodeId>) {
        self.remove_node_recursive(node);
        self.index_node_recursive(node, parent);
    }
    
    /// Remove a node and its subtree from all indices
    pub fn remove_node_recursive(&mut self, node: &PersistentAstNode) {
        let node_id = node.id();
        
        // Remove from all indices
        self.type_index.remove_node(node_id);
        self.symbol_index.remove_node(node_id);
        self.position_index.remove_node(node_id);
        self.dependency_index.remove_node(node_id);
        self.hierarchy_index.remove_node(node_id);
        
        // Recursively remove children
        for child in node.children() {
            self.remove_node_recursive(child);
        }
    }
    
    /// Index a node and its subtree
    fn index_node_recursive(&mut self, node: &PersistentAstNode, parent: Option<NodeId>) {
        let node_id = node.id();
        
        // Index in all relevant indices
        self.type_index.index_node(node);
        self.symbol_index.index_node(node);
        self.position_index.index_node(node);
        self.dependency_index.index_node(node);
        self.hierarchy_index.index_node(node, parent);
        
        // Recursively index children
        for child in node.children() {
            self.index_node_recursive(child, Some(node_id));
        }
    }
    
    /// Execute a query using the appropriate index
    pub fn execute_query(&self, query: &AstQuery) -> QueryResult {
        match query {
            AstQuery::FindByType { node_type } => {
                self.type_index.find_by_type(node_type)
            },
            AstQuery::FindReferences { symbol } => {
                self.symbol_index.find_references(*symbol)
            },
            AstQuery::FindDefinition { symbol } => {
                self.symbol_index.find_definition(*symbol)
            },
            AstQuery::GetParent { node_id } => {
                self.hierarchy_index.get_parent(*node_id)
            },
            AstQuery::GetChildren { node_id } => {
                self.hierarchy_index.get_children(*node_id)
            },
            AstQuery::NodesInRange { start, end } => {
                self.position_index.find_in_range(*start, *end)
            },
            AstQuery::ContainingNode { position } => {
                self.position_index.find_containing(*position)
            },
            AstQuery::FindDependencies { node_id } => {
                self.dependency_index.find_dependencies(*node_id)
            },
            AstQuery::FindDependents { node_id } => {
                self.dependency_index.find_dependents(*node_id)
            },
            _ => QueryResult::empty(),
        }
    }
}

/// Index for fast type-based lookups
#[derive(Debug, Clone)]
pub struct TypeIndex {
    /// Map from type name to set of node IDs
    type_to_nodes: HashMap<String, OrdSet<NodeId>>,
    /// Map from node ID to its type name
    node_to_type: HashMap<NodeId, String>,
}

impl Default for TypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeIndex {
    pub fn new() -> Self {
        Self {
            type_to_nodes: HashMap::new(),
            node_to_type: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.type_to_nodes.clear();
        self.node_to_type.clear();
    }
    
    pub fn index_node(&mut self, node: &PersistentAstNode) {
        let node_id = node.id();
        let type_name = self.get_type_name(&node.kind);
        
        self.node_to_type.insert(node_id, type_name.clone());
        let _ = self.type_to_nodes
            .entry(type_name)
            .or_default()
            .update(node_id);
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(type_name) = self.node_to_type.remove(&node_id) {
            if let Some(mut nodes) = self.type_to_nodes.get(&type_name).cloned() {
                nodes = nodes.without(&node_id);
                if nodes.is_empty() {
                    self.type_to_nodes.remove(&type_name);
                } else {
                    self.type_to_nodes.insert(type_name, nodes);
                }
            }
        }
    }
    
    pub fn find_by_type(&self, type_name: &str) -> QueryResult {
        match self.type_to_nodes.get(type_name) {
            Some(node_ids) => QueryResult::new(node_ids.iter().cloned().collect()),
            None => QueryResult::empty(),
        }
    }
    
    fn get_type_name(&self, kind: &AstNodeKind) -> String {
        match kind {
            AstNodeKind::CompilationUnit { .. } => "CompilationUnit".to_string(),
            AstNodeKind::Module { .. } => "Module".to_string(),
            AstNodeKind::ValueDef { .. } => "ValueDef".to_string(),
            AstNodeKind::TypeDef { .. } => "TypeDef".to_string(),
            AstNodeKind::EffectDef { .. } => "EffectDef".to_string(),
            AstNodeKind::Literal { .. } => "Literal".to_string(),
            AstNodeKind::Variable { .. } => "Variable".to_string(),
            AstNodeKind::Application { .. } => "Application".to_string(),
            AstNodeKind::Lambda { .. } => "Lambda".to_string(),
            AstNodeKind::Let { .. } => "Let".to_string(),
            AstNodeKind::If { .. } => "If".to_string(),
            AstNodeKind::Match { .. } => "Match".to_string(),
            AstNodeKind::Handle { .. } => "Handle".to_string(),
            AstNodeKind::Perform { .. } => "Perform".to_string(),
            AstNodeKind::TypeReference { .. } => "TypeReference".to_string(),
            AstNodeKind::FunctionType { .. } => "FunctionType".to_string(),
            AstNodeKind::RecordType { .. } => "RecordType".to_string(),
            AstNodeKind::VariantType { .. } => "VariantType".to_string(),
            AstNodeKind::PatternVariable { .. } => "PatternVariable".to_string(),
            AstNodeKind::PatternLiteral { .. } => "PatternLiteral".to_string(),
            AstNodeKind::PatternConstructor { .. } => "PatternConstructor".to_string(),
            AstNodeKind::PatternRecord { .. } => "PatternRecord".to_string(),
        }
    }
}

/// Index for symbol resolution and reference finding
#[derive(Debug, Clone)]
pub struct SymbolIndex {
    /// Map from symbol to its definition node
    definitions: HashMap<Symbol, NodeId>,
    /// Map from symbol to set of reference nodes
    references: HashMap<Symbol, OrdSet<NodeId>>,
    /// Map from node to symbols it defines
    node_definitions: HashMap<NodeId, OrdSet<Symbol>>,
    /// Map from node to symbols it references
    node_references: HashMap<NodeId, OrdSet<Symbol>>,
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            references: HashMap::new(),
            node_definitions: HashMap::new(),
            node_references: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.references.clear();
        self.node_definitions.clear();
        self.node_references.clear();
    }
    
    pub fn index_node(&mut self, node: &PersistentAstNode) {
        let node_id = node.id();
        
        // Extract symbols defined and referenced by this node
        let (defined_symbols, referenced_symbols) = self.extract_symbols(&node.kind);
        
        // Index definitions
        for symbol in &defined_symbols {
            self.definitions.insert(*symbol, node_id);
            let _ = self.node_definitions
                .entry(node_id)
                .or_default()
                .update(*symbol);
        }
        
        // Index references
        for symbol in &referenced_symbols {
            let _ = self.references
                .entry(*symbol)
                .or_default()
                .update(node_id);
            let _ = self.node_references
                .entry(node_id)
                .or_default()
                .update(*symbol);
        }
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        // Remove definitions
        if let Some(defined_symbols) = self.node_definitions.remove(&node_id) {
            for symbol in defined_symbols {
                self.definitions.remove(&symbol);
            }
        }
        
        // Remove references
        if let Some(referenced_symbols) = self.node_references.remove(&node_id) {
            for symbol in referenced_symbols {
                if let Some(mut refs) = self.references.get(&symbol).cloned() {
                    refs = refs.without(&node_id);
                    if refs.is_empty() {
                        self.references.remove(&symbol);
                    } else {
                        self.references.insert(symbol, refs);
                    }
                }
            }
        }
    }
    
    pub fn find_definition(&self, symbol: Symbol) -> QueryResult {
        match self.definitions.get(&symbol) {
            Some(node_id) => QueryResult::new(vec![*node_id]),
            None => QueryResult::empty(),
        }
    }
    
    pub fn find_references(&self, symbol: Symbol) -> QueryResult {
        match self.references.get(&symbol) {
            Some(node_ids) => QueryResult::new(node_ids.iter().cloned().collect()),
            None => QueryResult::empty(),
        }
    }
    
    fn extract_symbols(&self, kind: &AstNodeKind) -> (OrdSet<Symbol>, OrdSet<Symbol>) {
        let mut defined = OrdSet::new();
        let mut referenced = OrdSet::new();
        
        match kind {
            AstNodeKind::ValueDef { name, .. } => {
                defined = defined.update(*name);
            },
            AstNodeKind::TypeDef { name, .. } => {
                defined = defined.update(*name);
            },
            AstNodeKind::EffectDef { name, .. } => {
                defined = defined.update(*name);
            },
            AstNodeKind::Variable { name } => {
                referenced = referenced.update(*name);
            },
            AstNodeKind::TypeReference { name, .. } => {
                referenced = referenced.update(*name);
            },
            AstNodeKind::PatternVariable { name } => {
                defined = defined.update(*name);
            },
            AstNodeKind::Perform { effect, operation, .. } => {
                referenced = referenced.update(*effect);
                referenced = referenced.update(*operation);
            },
            _ => {},
        }
        
        (defined, referenced)
    }
}

/// Index for position-based spatial queries
#[derive(Debug, Clone)]
pub struct PositionIndex {
    /// Tree structure for efficient range queries
    intervals: BTreeMap<u32, BTreeMap<u32, OrdSet<NodeId>>>,
    /// Map from node to its span
    node_spans: HashMap<NodeId, Span>,
}

impl Default for PositionIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionIndex {
    pub fn new() -> Self {
        Self {
            intervals: BTreeMap::new(),
            node_spans: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.intervals.clear();
        self.node_spans.clear();
    }
    
    pub fn index_node(&mut self, node: &PersistentAstNode) {
        let node_id = node.id();
        let span = node.span();
        
        self.node_spans.insert(node_id, span);
        
        let start = span.start.as_u32();
        let end = span.end.as_u32();
        
        let _ = self.intervals
            .entry(start)
            .or_default()
            .entry(end)
            .or_default()
            .update(node_id);
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(span) = self.node_spans.remove(&node_id) {
            let start = span.start.as_u32();
            let end = span.end.as_u32();
            
            if let Some(end_map) = self.intervals.get_mut(&start) {
                if let Some(mut nodes) = end_map.get(&end).cloned() {
                    nodes = nodes.without(&node_id);
                    if nodes.is_empty() {
                        end_map.remove(&end);
                        if end_map.is_empty() {
                            self.intervals.remove(&start);
                        }
                    } else {
                        end_map.insert(end, nodes);
                    }
                }
            }
        }
    }
    
    pub fn find_in_range(&self, start: Position, end: Position) -> QueryResult {
        let mut result = Vec::new();
        let start_offset = start.offset;
        let end_offset = end.offset;
        
        for (&span_start, end_map) in self.intervals.range(..=end_offset) {
            if span_start > end_offset {
                break;
            }
            
            for (&span_end, nodes) in end_map {
                if span_start <= end_offset && span_end >= start_offset {
                    result.extend(nodes.iter());
                }
            }
        }
        
        QueryResult::new(result)
    }
    
    pub fn find_containing(&self, position: Position) -> QueryResult {
        let offset = position.offset;
        let mut result = Vec::new();
        
        for (&span_start, end_map) in self.intervals.range(..=offset) {
            for (&span_end, nodes) in end_map {
                if span_start <= offset && offset <= span_end {
                    result.extend(nodes.iter());
                }
            }
        }
        
        QueryResult::new(result)
    }
}

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

/// Index for tracking dependencies between nodes
#[derive(Debug, Clone)]
pub struct DependencyIndex {
    /// Map from node to nodes it depends on
    dependencies: HashMap<NodeId, OrdSet<NodeId>>,
    /// Map from node to nodes that depend on it
    dependents: HashMap<NodeId, OrdSet<NodeId>>,
}

impl Default for DependencyIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyIndex {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
    }
    
    pub fn index_node(&mut self, node: &PersistentAstNode) {
        // Extract dependencies based on node type
        let dependencies = self.extract_dependencies(node);
        let node_id = node.id();
        
        for dep in dependencies {
            self.add_dependency(node_id, dep);
        }
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        // Remove as dependent
        if let Some(deps) = self.dependencies.remove(&node_id) {
            for dep in deps {
                if let Some(mut dependents) = self.dependents.get(&dep).cloned() {
                    dependents = dependents.without(&node_id);
                    if dependents.is_empty() {
                        self.dependents.remove(&dep);
                    } else {
                        self.dependents.insert(dep, dependents);
                    }
                }
            }
        }
        
        // Remove as dependency
        if let Some(dependents) = self.dependents.remove(&node_id) {
            for dependent in dependents {
                if let Some(mut deps) = self.dependencies.get(&dependent).cloned() {
                    deps = deps.without(&node_id);
                    if deps.is_empty() {
                        self.dependencies.remove(&dependent);
                    } else {
                        self.dependencies.insert(dependent, deps);
                    }
                }
            }
        }
    }
    
    pub fn add_dependency(&mut self, dependent: NodeId, dependency: NodeId) {
        let _ = self.dependencies
            .entry(dependent)
            .or_default()
            .update(dependency);
        
        let _ = self.dependents
            .entry(dependency)
            .or_default()
            .update(dependent);
    }
    
    pub fn find_dependencies(&self, node_id: NodeId) -> QueryResult {
        match self.dependencies.get(&node_id) {
            Some(deps) => QueryResult::new(deps.iter().cloned().collect()),
            None => QueryResult::empty(),
        }
    }
    
    pub fn find_dependents(&self, node_id: NodeId) -> QueryResult {
        match self.dependents.get(&node_id) {
            Some(deps) => QueryResult::new(deps.iter().cloned().collect()),
            None => QueryResult::empty(),
        }
    }
    
    fn extract_dependencies(&self, _node: &PersistentAstNode) -> OrdSet<NodeId> {
        // TODO: Implement dependency extraction based on semantic analysis
        OrdSet::new()
    }
}

/// Index for parent-child hierarchy
#[derive(Debug, Clone)]
pub struct HierarchyIndex {
    /// Map from child to parent
    parent_map: HashMap<NodeId, NodeId>,
    /// Map from parent to children
    children_map: HashMap<NodeId, OrdSet<NodeId>>,
}

impl Default for HierarchyIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchyIndex {
    pub fn new() -> Self {
        Self {
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.parent_map.clear();
        self.children_map.clear();
    }
    
    pub fn index_node(&mut self, node: &PersistentAstNode, parent: Option<NodeId>) {
        let node_id = node.id();
        
        if let Some(parent_id) = parent {
            self.parent_map.insert(node_id, parent_id);
            let _ = self.children_map
                .entry(parent_id)
                .or_default()
                .update(node_id);
        }
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        // Remove from parent's children
        if let Some(parent_id) = self.parent_map.remove(&node_id) {
            if let Some(mut children) = self.children_map.get(&parent_id).cloned() {
                children = children.without(&node_id);
                if children.is_empty() {
                    self.children_map.remove(&parent_id);
                } else {
                    self.children_map.insert(parent_id, children);
                }
            }
        }
        
        // Remove children
        self.children_map.remove(&node_id);
    }
    
    pub fn get_parent(&self, node_id: NodeId) -> QueryResult {
        match self.parent_map.get(&node_id) {
            Some(parent) => QueryResult::new(vec![*parent]),
            None => QueryResult::empty(),
        }
    }
    
    pub fn get_children(&self, node_id: NodeId) -> QueryResult {
        match self.children_map.get(&node_id) {
            Some(children) => QueryResult::new(children.iter().cloned().collect()),
            None => QueryResult::empty(),
        }
    }
}