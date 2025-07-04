//! Advanced AST querying and navigation system

use x_parser::{
    persistent_ast::{PersistentAstNode, NodeId},
    symbol::Symbol,
};
use crate::index_system::Position;
use serde::{Deserialize, Serialize};
use im::Vector;

/// Advanced query types for AST exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstQuery {
    // Basic structural queries
    FindByType { node_type: String },
    FindByPath { path: Vec<usize> },
    FindByPattern { pattern: QueryPattern },
    GetChildren { node_id: NodeId },
    GetParent { node_id: NodeId },
    GetSiblings { node_id: NodeId },
    
    // Symbol-based queries
    FindReferences { symbol: Symbol },
    FindDefinition { symbol: Symbol },
    FindUsages { symbol: Symbol },
    FindTypeReferences { type_name: String },
    
    // Position-based queries
    NodesInRange { start: Position, end: Position },
    ContainingNode { position: Position },
    NodesAtLine { line: u32 },
    
    // Semantic queries
    FindDependencies { node_id: NodeId },
    FindDependents { node_id: NodeId },
    FindCallSites { function: Symbol },
    FindOverrides { method: Symbol },
    
    // Complex composite queries
    And { queries: Vec<AstQuery> },
    Or { queries: Vec<AstQuery> },
    Filter { base: Box<AstQuery>, predicate: QueryPredicate },
    Map { base: Box<AstQuery>, transform: QueryTransform },
    
    // Navigation queries
    AncestorsOfType { node_id: NodeId, node_type: String },
    DescendantsOfType { node_id: NodeId, node_type: String },
    NextSiblingOfType { node_id: NodeId, node_type: String },
    PreviousSiblingOfType { node_id: NodeId, node_type: String },
}

/// Advanced pattern matching for AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPattern {
    /// Match any node
    Any,
    /// Match nodes with specific type
    Type(String),
    /// Match nodes with specific value
    Value(String),
    /// Match nodes with specific symbol name
    Symbol(Symbol),
    /// Match nodes that are children of another pattern
    Child(Box<QueryPattern>),
    /// Match nodes that are descendants of another pattern
    Descendant(Box<QueryPattern>),
    /// Match nodes that satisfy all patterns
    And(Vec<QueryPattern>),
    /// Match nodes that satisfy any pattern
    Or(Vec<QueryPattern>),
    /// Match nodes that don't satisfy a pattern
    Not(Box<QueryPattern>),
    /// Match nodes with specific annotation
    HasAnnotation { key: String, value: Option<String> },
    /// Match nodes with specific type annotation
    HasType { type_name: String },
    /// Match nodes with specific effect annotation
    HasEffect { effect_name: String },
    /// Match nodes at specific depth
    AtDepth { depth: usize },
    /// Match nodes in specific position range
    InRange { start: Position, end: Position },
}

/// Predicate for filtering query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPredicate {
    /// Filter by node type
    IsType(String),
    /// Filter by symbol presence
    HasSymbol(Symbol),
    /// Filter by annotation presence
    HasAnnotation(String),
    /// Filter by type information
    HasTypeInfo,
    /// Filter by position
    InPosition { start: Position, end: Position },
    /// Filter by custom condition
    Custom(String), // JavaScript-like expression
    /// Combine predicates
    And(Vec<QueryPredicate>),
    Or(Vec<QueryPredicate>),
    Not(Box<QueryPredicate>),
}

/// Transform operations for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryTransform {
    /// Get parent nodes
    GetParents,
    /// Get child nodes
    GetChildren,
    /// Get all descendants
    GetDescendants,
    /// Get ancestors of specific type
    GetAncestorsOfType(String),
    /// Extract symbol names
    ExtractSymbols,
    /// Extract type information
    ExtractTypes,
    /// Extract source positions
    ExtractPositions,
}

/// Query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Matched node IDs
    pub nodes: Vector<NodeId>,
    /// Execution metadata
    pub metadata: QueryMetadata,
}

/// Query execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Number of nodes examined
    pub nodes_examined: u64,
    /// Whether the query was satisfied from cache
    pub from_cache: bool,
    /// Query complexity score
    pub complexity_score: f64,
}

/// Node selector for precise targeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelector {
    /// Path to the node
    pub path: Vec<usize>,
    /// Optional type constraint
    pub node_type: Option<String>,
    /// Optional value constraint
    pub value_constraint: Option<String>,
    /// Optional position constraint
    pub position_constraint: Option<Position>,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(nodes: Vec<NodeId>) -> Self {
        Self {
            nodes: Vector::from(nodes),
            metadata: QueryMetadata {
                execution_time_us: 0,
                nodes_examined: 0,
                from_cache: false,
                complexity_score: 0.0,
            },
        }
    }
    
    /// Create an empty query result
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }
    
    /// Create result with metadata
    pub fn with_metadata(nodes: Vec<NodeId>, metadata: QueryMetadata) -> Self {
        Self {
            nodes: Vector::from(nodes),
            metadata,
        }
    }
    
    /// Get node IDs as a vector
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.iter().cloned().collect()
    }
    
    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    /// Get the number of results
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Get the first result if any
    pub fn first(&self) -> Option<NodeId> {
        self.nodes.get(0).copied()
    }
    
    /// Combine two query results
    pub fn union(&self, other: &QueryResult) -> QueryResult {
        let mut combined_nodes = self.nodes.clone();
        for node in &other.nodes {
            if !combined_nodes.contains(node) {
                combined_nodes = combined_nodes.push_back(*node);
            }
        }
        
        QueryResult {
            nodes: combined_nodes,
            metadata: QueryMetadata {
                execution_time_us: self.metadata.execution_time_us + other.metadata.execution_time_us,
                nodes_examined: self.metadata.nodes_examined + other.metadata.nodes_examined,
                from_cache: self.metadata.from_cache && other.metadata.from_cache,
                complexity_score: self.metadata.complexity_score + other.metadata.complexity_score,
            },
        }
    }
    
    /// Intersect two query results
    pub fn intersection(&self, other: &QueryResult) -> QueryResult {
        let intersection_nodes: Vector<NodeId> = self.nodes
            .iter()
            .filter(|node| other.nodes.contains(node))
            .cloned()
            .collect();
        
        QueryResult {
            nodes: intersection_nodes,
            metadata: QueryMetadata {
                execution_time_us: self.metadata.execution_time_us + other.metadata.execution_time_us,
                nodes_examined: self.metadata.nodes_examined + other.metadata.nodes_examined,
                from_cache: self.metadata.from_cache && other.metadata.from_cache,
                complexity_score: self.metadata.complexity_score + other.metadata.complexity_score,
            },
        }
    }
    
    /// Apply a transform to the result
    pub fn transform(&self, _transform: &QueryTransform) -> QueryResult {
        // TODO: Implement query transforms
        self.clone()
    }
    
    /// Filter the result with a predicate
    pub fn filter(&self, _predicate: &QueryPredicate) -> QueryResult {
        // TODO: Implement query filtering
        self.clone()
    }
}

impl NodeSelector {
    /// Create a new node selector
    pub fn new(path: Vec<usize>) -> Self {
        Self {
            path,
            node_type: None,
            value_constraint: None,
            position_constraint: None,
        }
    }
    
    /// Add type constraint
    pub fn with_type(mut self, node_type: String) -> Self {
        self.node_type = Some(node_type);
        self
    }
    
    /// Add value constraint
    pub fn with_value(mut self, value: String) -> Self {
        self.value_constraint = Some(value);
        self
    }
    
    /// Add position constraint
    pub fn with_position(mut self, position: Position) -> Self {
        self.position_constraint = Some(position);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::persistent_ast::NodeId;

    #[test]
    fn test_query_result_creation() {
        let result = QueryResult::new(vec![NodeId::new(1), NodeId::new(2)]);
        assert_eq!(result.len(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_query_result_union() {
        let result1 = QueryResult::new(vec![NodeId::new(1), NodeId::new(2)]);
        let result2 = QueryResult::new(vec![NodeId::new(2), NodeId::new(3)]);
        let union = result1.union(&result2);
        
        assert_eq!(union.len(), 3);
        assert!(union.node_ids().contains(&NodeId::new(1)));
        assert!(union.node_ids().contains(&NodeId::new(2)));
        assert!(union.node_ids().contains(&NodeId::new(3)));
    }

    #[test]
    fn test_query_result_intersection() {
        let result1 = QueryResult::new(vec![NodeId::new(1), NodeId::new(2)]);
        let result2 = QueryResult::new(vec![NodeId::new(2), NodeId::new(3)]);
        let intersection = result1.intersection(&result2);
        
        assert_eq!(intersection.len(), 1);
        assert_eq!(intersection.first(), Some(NodeId::new(2)));
    }

    #[test]
    fn test_node_selector_builder() {
        let selector = NodeSelector::new(vec![0, 1])
            .with_type("ValueDef".to_string())
            .with_value("test".to_string());
        
        assert_eq!(selector.path, vec![0, 1]);
        assert_eq!(selector.node_type, Some("ValueDef".to_string()));
        assert_eq!(selector.value_constraint, Some("test".to_string()));
    }
}