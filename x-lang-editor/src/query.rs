//! AST querying and navigation

use x_lang_parser::{CompilationUnit, AstNode, Module, Item, Expression};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query for finding nodes in an AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstQuery {
    /// Find nodes by type name
    FindByType(String),
    /// Find a node by its path
    FindByPath(Vec<usize>),
    /// Find nodes matching a pattern
    FindByPattern(QueryPattern),
    /// Get children of a node
    GetChildren(Vec<usize>),
}

/// Pattern for matching AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPattern {
    /// Match any node
    Any,
    /// Match nodes with specific type
    Type(String),
    /// Match nodes with specific value
    Value(String),
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
}

/// Result of a query operation
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Single node result
    Single(QueryNode),
    /// Multiple nodes result
    Multiple(Vec<QueryNode>),
    /// No results found
    None,
}

/// A node found by a query
#[derive(Debug, Clone)]
pub struct QueryNode {
    /// Path to the node in the AST
    pub path: Vec<usize>,
    /// The actual node
    pub node: AstNode,
    /// Parent path (if any)
    pub parent_path: Option<Vec<usize>>,
    /// Children paths
    pub children_paths: Vec<Vec<usize>>,
}

/// AST query engine
#[derive(Debug, Default)]
pub struct QueryEngine {
    /// Cache for query results
    cache: HashMap<String, QueryResult>,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a query against an AST
    pub fn execute(
        &mut self,
        ast: &CompilationUnit,
        query: &AstQuery,
    ) -> QueryResult {
        // Check cache first
        let cache_key = self.query_cache_key(query);
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return cached_result.clone();
        }

        let result = match query {
            AstQuery::FindByType(type_name) => {
                self.find_by_type(ast, type_name)
            }
            AstQuery::FindByPath(path) => {
                self.find_by_path(ast, path)
            }
            AstQuery::FindByPattern(pattern) => {
                self.find_by_pattern(ast, pattern)
            }
            AstQuery::GetChildren(path) => {
                self.get_children(ast, path)
            }
        };

        // Cache the result
        self.cache.insert(cache_key, result.clone());
        result
    }

    /// Find nodes by type
    fn find_by_type(&self, ast: &CompilationUnit, type_name: &str) -> QueryResult {
        let mut results = Vec::new();
        self.traverse_and_collect(
            ast,
            &mut results,
            &mut vec![],
            |node, _path| self.node_matches_type(node, type_name)
        );
        
        if results.is_empty() {
            QueryResult::None
        } else {
            QueryResult::Multiple(results)
        }
    }

    /// Find node by path
    fn find_by_path(&self, ast: &CompilationUnit, path: &[usize]) -> QueryResult {
        if let Some(node) = self.navigate_to_path(ast, path) {
            let parent_path = if path.is_empty() {
                None
            } else {
                Some(path[..path.len() - 1].to_vec())
            };
            
            let children_paths = self.get_children_paths(ast, path);
            
            QueryResult::Single(QueryNode {
                path: path.to_vec(),
                node,
                parent_path,
                children_paths,
            })
        } else {
            QueryResult::None
        }
    }

    /// Find nodes by pattern
    fn find_by_pattern(&self, ast: &CompilationUnit, pattern: &QueryPattern) -> QueryResult {
        let mut results = Vec::new();
        self.traverse_and_collect(
            ast,
            &mut results,
            &mut vec![],
            |node, path| self.node_matches_pattern(ast, node, path, pattern)
        );
        
        if results.is_empty() {
            QueryResult::None
        } else {
            QueryResult::Multiple(results)
        }
    }

    /// Get children of a node
    fn get_children(&self, ast: &CompilationUnit, path: &[usize]) -> QueryResult {
        let children_paths = self.get_children_paths(ast, path);
        let mut children = Vec::new();
        
        for child_path in children_paths {
            if let Some(node) = self.navigate_to_path(ast, &child_path) {
                children.push(QueryNode {
                    path: child_path.clone(),
                    node,
                    parent_path: Some(path.to_vec()),
                    children_paths: self.get_children_paths(ast, &child_path),
                });
            }
        }
        
        if children.is_empty() {
            QueryResult::None
        } else {
            QueryResult::Multiple(children)
        }
    }

    /// Traverse AST and collect matching nodes
    fn traverse_and_collect<F>(
        &self,
        ast: &CompilationUnit,
        results: &mut Vec<QueryNode>,
        current_path: &mut Vec<usize>,
        predicate: F,
    ) where
        F: Fn(&AstNode, &[usize]) -> bool + Copy,
    {
        // Check compilation unit
        let cu_node = AstNode::CompilationUnit(ast.clone());
        if predicate(&cu_node, current_path) {
            results.push(QueryNode {
                path: current_path.clone(),
                node: cu_node,
                parent_path: None,
                children_paths: self.get_children_paths(ast, current_path),
            });
        }

        // Traverse modules
        for (i, module) in ast.modules.iter().enumerate() {
            current_path.push(i);
            let module_node = AstNode::Module(module.clone());
            
            if predicate(&module_node, current_path) {
                results.push(QueryNode {
                    path: current_path.clone(),
                    node: module_node,
                    parent_path: Some(current_path[..current_path.len() - 1].to_vec()),
                    children_paths: self.get_children_paths(ast, current_path),
                });
            }
            
            // Traverse items
            for (j, item) in module.items.iter().enumerate() {
                current_path.push(j);
                let item_node = AstNode::Item(item.clone());
                
                if predicate(&item_node, current_path) {
                    results.push(QueryNode {
                        path: current_path.clone(),
                        node: item_node,
                        parent_path: Some(current_path[..current_path.len() - 1].to_vec()),
                        children_paths: self.get_children_paths(ast, current_path),
                    });
                }
                
                current_path.pop();
            }
            
            current_path.pop();
        }
    }

    /// Check if a node matches a specific type
    fn node_matches_type(&self, node: &AstNode, type_name: &str) -> bool {
        let node_type = match node {
            AstNode::CompilationUnit(_) => "CompilationUnit",
            AstNode::Module(_) => "Module",
            AstNode::Item(_) => "Item",
            AstNode::Expression(_) => "Expression",
            AstNode::Statement(_) => "Statement",
            AstNode::Literal(_) => "Literal",
        };
        
        node_type == type_name
    }

    /// Check if a node matches a pattern
    fn node_matches_pattern(
        &self,
        ast: &CompilationUnit,
        node: &AstNode,
        path: &[usize],
        pattern: &QueryPattern,
    ) -> bool {
        match pattern {
            QueryPattern::Any => true,
            QueryPattern::Type(type_name) => self.node_matches_type(node, type_name),
            QueryPattern::Value(value) => self.node_matches_value(node, value),
            QueryPattern::Child(child_pattern) => {
                // Check if any child matches the pattern
                let children_paths = self.get_children_paths(ast, path);
                children_paths.iter().any(|child_path| {
                    if let Some(child_node) = self.navigate_to_path(ast, child_path) {
                        self.node_matches_pattern(ast, &child_node, child_path, child_pattern)
                    } else {
                        false
                    }
                })
            }
            QueryPattern::Descendant(desc_pattern) => {
                // Check if any descendant matches the pattern (recursive)
                self.has_descendant_matching(ast, path, desc_pattern)
            }
            QueryPattern::And(patterns) => {
                patterns.iter().all(|p| self.node_matches_pattern(ast, node, path, p))
            }
            QueryPattern::Or(patterns) => {
                patterns.iter().any(|p| self.node_matches_pattern(ast, node, path, p))
            }
            QueryPattern::Not(pattern) => {
                !self.node_matches_pattern(ast, node, path, pattern)
            }
        }
    }

    /// Check if a node matches a specific value
    fn node_matches_value(&self, _node: &AstNode, _value: &str) -> bool {
        // TODO: Implement value matching based on node content
        false
    }

    /// Check if a node has a descendant matching a pattern
    fn has_descendant_matching(
        &self,
        ast: &CompilationUnit,
        path: &[usize],
        pattern: &QueryPattern,
    ) -> bool {
        let children_paths = self.get_children_paths(ast, path);
        
        for child_path in children_paths {
            if let Some(child_node) = self.navigate_to_path(ast, &child_path) {
                if self.node_matches_pattern(ast, &child_node, &child_path, pattern) {
                    return true;
                }
                
                // Recursively check descendants
                if self.has_descendant_matching(ast, &child_path, pattern) {
                    return true;
                }
            }
        }
        
        false
    }

    /// Navigate to a specific path in the AST
    fn navigate_to_path(&self, ast: &CompilationUnit, path: &[usize]) -> Option<AstNode> {
        if path.is_empty() {
            return Some(AstNode::CompilationUnit(ast.clone()));
        }

        if path.len() == 1 {
            let module_index = path[0];
            if module_index < ast.modules.len() {
                return Some(AstNode::Module(ast.modules[module_index].clone()));
            }
        } else if path.len() == 2 {
            let module_index = path[0];
            let item_index = path[1];
            
            if module_index < ast.modules.len() {
                let module = &ast.modules[module_index];
                if item_index < module.items.len() {
                    return Some(AstNode::Item(module.items[item_index].clone()));
                }
            }
        }

        None
    }

    /// Get paths to all children of a node
    fn get_children_paths(&self, ast: &CompilationUnit, path: &[usize]) -> Vec<Vec<usize>> {
        let mut children = Vec::new();
        
        if path.is_empty() {
            // Children of CompilationUnit are modules
            for i in 0..ast.modules.len() {
                children.push(vec![i]);
            }
        } else if path.len() == 1 {
            // Children of Module are items
            let module_index = path[0];
            if module_index < ast.modules.len() {
                let module = &ast.modules[module_index];
                for j in 0..module.items.len() {
                    children.push(vec![module_index, j]);
                }
            }
        }
        // TODO: Add more levels as needed
        
        children
    }

    /// Generate cache key for a query
    fn query_cache_key(&self, query: &AstQuery) -> String {
        format!("{:?}", query)
    }

    /// Clear the query cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl NodeSelector {
    /// Create a new node selector
    pub fn new(path: Vec<usize>) -> Self {
        Self {
            path,
            node_type: None,
            value_constraint: None,
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

    /// Check if a node matches this selector
    pub fn matches(&self, node: &AstNode) -> bool {
        if let Some(expected_type) = &self.node_type {
            let node_type = match node {
                AstNode::CompilationUnit(_) => "CompilationUnit",
                AstNode::Module(_) => "Module",
                AstNode::Item(_) => "Item",
                AstNode::Expression(_) => "Expression",
                AstNode::Statement(_) => "Statement",
                AstNode::Literal(_) => "Literal",
            };
            
            if node_type != expected_type {
                return false;
            }
        }
        
        // TODO: Add value constraint checking
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lang_parser::{parse_source, FileId, SyntaxStyle};

    #[test]
    fn test_query_engine_creation() {
        let engine = QueryEngine::new();
        assert!(engine.cache.is_empty());
    }

    #[test]
    fn test_find_by_type() {
        let mut engine = QueryEngine::new();
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
        
        let query = AstQuery::FindByType("Module".to_string());
        let result = engine.execute(&ast, &query);
        
        match result {
            QueryResult::Multiple(nodes) => {
                assert!(!nodes.is_empty());
            }
            _ => panic!("Expected multiple results"),
        }
    }

    #[test]
    fn test_find_by_path() {
        let mut engine = QueryEngine::new();
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
        
        let query = AstQuery::FindByPath(vec![0]);
        let result = engine.execute(&ast, &query);
        
        match result {
            QueryResult::Single(_) => {
                // Success
            }
            _ => panic!("Expected single result"),
        }
    }

    #[test]
    fn test_node_selector() {
        let selector = NodeSelector::new(vec![0, 1])
            .with_type("Item".to_string());
        
        assert_eq!(selector.path, vec![0, 1]);
        assert_eq!(selector.node_type, Some("Item".to_string()));
    }

    #[test]
    fn test_query_patterns() {
        let pattern = QueryPattern::And(vec![
            QueryPattern::Type("Item".to_string()),
            QueryPattern::Not(Box::new(QueryPattern::Value("test".to_string())))
        ]);
        
        // Test pattern creation
        match pattern {
            QueryPattern::And(patterns) => {
                assert_eq!(patterns.len(), 2);
            }
            _ => panic!("Expected And pattern"),
        }
    }
}