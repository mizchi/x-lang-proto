//! Direct AST editing operations without text representation

use crate::operations::{EditOperation, InsertOperation, DeleteOperation, ReplaceOperation, MoveOperation};
use crate::query::{AstQuery, QueryResult, NodeSelector};
use crate::validation::{ValidationResult, ValidationError};
use x_lang_parser::{CompilationUnit, AstNode, Module, Item, Expression, Statement, Literal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// AST Editor for direct manipulation of syntax trees
#[derive(Debug)]
pub struct AstEditor {
    /// Track changes for undo/redo functionality
    change_history: Vec<EditOperation>,
    /// Validation cache
    validation_cache: HashMap<String, ValidationResult>,
}

impl AstEditor {
    /// Create a new AST editor
    pub fn new() -> Self {
        Self {
            change_history: Vec::new(),
            validation_cache: HashMap::new(),
        }
    }

    /// Apply an edit operation to the AST
    pub fn apply_operation(
        &mut self,
        ast: &mut CompilationUnit,
        operation: EditOperation,
    ) -> Result<EditResult, EditError> {
        // Validate operation before applying
        self.validate_operation(ast, &operation)?;
        
        // Apply the operation
        let result = match operation {
            EditOperation::Insert(ref op) => self.apply_insert(ast, op)?,
            EditOperation::Delete(ref op) => self.apply_delete(ast, op)?,
            EditOperation::Replace(ref op) => self.apply_replace(ast, op)?,
            EditOperation::Move(ref op) => self.apply_move(ast, op)?,
        };
        
        // Record the operation for history
        self.change_history.push(operation);
        
        // Clear validation cache
        self.validation_cache.clear();
        
        Ok(result)
    }

    /// Apply insert operation
    fn apply_insert(
        &mut self,
        ast: &mut CompilationUnit,
        operation: &InsertOperation,
    ) -> Result<EditResult, EditError> {
        let target = self.navigate_to_path_mut(ast, &operation.path)?;
        
        match target {
            AstTarget::ModuleItems(items) => {
                let index = operation.path.last().copied().unwrap_or(items.len());
                if let AstNode::Item(item) = operation.node.clone() {
                    items.insert(index, item);
                    Ok(EditResult::Inserted { 
                        path: operation.path.clone(),
                        node_id: self.generate_node_id(),
                    })
                } else {
                    Err(EditError::InvalidNodeType {
                        expected: "Item".to_string(),
                        found: format!("{:?}", operation.node),
                    })
                }
            }
            AstTarget::Expressions(expressions) => {
                let index = operation.path.last().copied().unwrap_or(expressions.len());
                if let AstNode::Expression(expr) = operation.node.clone() {
                    expressions.insert(index, expr);
                    Ok(EditResult::Inserted { 
                        path: operation.path.clone(),
                        node_id: self.generate_node_id(),
                    })
                } else {
                    Err(EditError::InvalidNodeType {
                        expected: "Expression".to_string(),
                        found: format!("{:?}", operation.node),
                    })
                }
            }
            _ => Err(EditError::InvalidInsertTarget {
                path: operation.path.clone(),
            }),
        }
    }

    /// Apply delete operation
    fn apply_delete(
        &mut self,
        ast: &mut CompilationUnit,
        operation: &DeleteOperation,
    ) -> Result<EditResult, EditError> {
        let target = self.navigate_to_path_mut(ast, &operation.path)?;
        
        match target {
            AstTarget::ModuleItems(items) => {
                let index = operation.path.last().copied().unwrap_or(0);
                if index < items.len() {
                    let removed = items.remove(index);
                    Ok(EditResult::Deleted { 
                        path: operation.path.clone(),
                        removed_node: AstNode::Item(removed),
                    })
                } else {
                    Err(EditError::PathNotFound {
                        path: operation.path.clone(),
                    })
                }
            }
            AstTarget::Expressions(expressions) => {
                let index = operation.path.last().copied().unwrap_or(0);
                if index < expressions.len() {
                    let removed = expressions.remove(index);
                    Ok(EditResult::Deleted { 
                        path: operation.path.clone(),
                        removed_node: AstNode::Expression(removed),
                    })
                } else {
                    Err(EditError::PathNotFound {
                        path: operation.path.clone(),
                    })
                }
            }
            _ => Err(EditError::InvalidDeleteTarget {
                path: operation.path.clone(),
            }),
        }
    }

    /// Apply replace operation
    fn apply_replace(
        &mut self,
        ast: &mut CompilationUnit,
        operation: &ReplaceOperation,
    ) -> Result<EditResult, EditError> {
        let target = self.navigate_to_path_mut(ast, &operation.path)?;
        
        match target {
            AstTarget::ModuleItems(items) => {
                let index = operation.path.last().copied().unwrap_or(0);
                if index < items.len() {
                    if let AstNode::Item(new_item) = operation.new_node.clone() {
                        let old_item = std::mem::replace(&mut items[index], new_item);
                        Ok(EditResult::Replaced { 
                            path: operation.path.clone(),
                            old_node: AstNode::Item(old_item),
                            new_node: operation.new_node.clone(),
                        })
                    } else {
                        Err(EditError::InvalidNodeType {
                            expected: "Item".to_string(),
                            found: format!("{:?}", operation.new_node),
                        })
                    }
                } else {
                    Err(EditError::PathNotFound {
                        path: operation.path.clone(),
                    })
                }
            }
            AstTarget::Expressions(expressions) => {
                let index = operation.path.last().copied().unwrap_or(0);
                if index < expressions.len() {
                    if let AstNode::Expression(new_expr) = operation.new_node.clone() {
                        let old_expr = std::mem::replace(&mut expressions[index], new_expr);
                        Ok(EditResult::Replaced { 
                            path: operation.path.clone(),
                            old_node: AstNode::Expression(old_expr),
                            new_node: operation.new_node.clone(),
                        })
                    } else {
                        Err(EditError::InvalidNodeType {
                            expected: "Expression".to_string(),
                            found: format!("{:?}", operation.new_node),
                        })
                    }
                } else {
                    Err(EditError::PathNotFound {
                        path: operation.path.clone(),
                    })
                }
            }
            _ => Err(EditError::InvalidReplaceTarget {
                path: operation.path.clone(),
            }),
        }
    }

    /// Apply move operation
    fn apply_move(
        &mut self,
        ast: &mut CompilationUnit,
        operation: &MoveOperation,
    ) -> Result<EditResult, EditError> {
        // First, extract the node from source
        let source_target = self.navigate_to_path_mut(ast, &operation.source_path)?;
        let node_to_move = match source_target {
            AstTarget::ModuleItems(items) => {
                let index = operation.source_path.last().copied().unwrap_or(0);
                if index < items.len() {
                    AstNode::Item(items.remove(index))
                } else {
                    return Err(EditError::PathNotFound {
                        path: operation.source_path.clone(),
                    });
                }
            }
            AstTarget::Expressions(expressions) => {
                let index = operation.source_path.last().copied().unwrap_or(0);
                if index < expressions.len() {
                    AstNode::Expression(expressions.remove(index))
                } else {
                    return Err(EditError::PathNotFound {
                        path: operation.source_path.clone(),
                    });
                }
            }
            _ => return Err(EditError::InvalidMoveSource {
                path: operation.source_path.clone(),
            }),
        };

        // Then, insert at destination
        let dest_target = self.navigate_to_path_mut(ast, &operation.dest_path)?;
        match dest_target {
            AstTarget::ModuleItems(items) => {
                let index = operation.dest_path.last().copied().unwrap_or(items.len());
                if let AstNode::Item(item) = node_to_move {
                    items.insert(index, item);
                } else {
                    return Err(EditError::InvalidNodeType {
                        expected: "Item".to_string(),
                        found: format!("{:?}", node_to_move),
                    });
                }
            }
            AstTarget::Expressions(expressions) => {
                let index = operation.dest_path.last().copied().unwrap_or(expressions.len());
                if let AstNode::Expression(expr) = node_to_move {
                    expressions.insert(index, expr);
                } else {
                    return Err(EditError::InvalidNodeType {
                        expected: "Expression".to_string(),
                        found: format!("{:?}", node_to_move),
                    });
                }
            }
            _ => return Err(EditError::InvalidMoveDestination {
                path: operation.dest_path.clone(),
            }),
        }

        Ok(EditResult::Moved {
            source_path: operation.source_path.clone(),
            dest_path: operation.dest_path.clone(),
        })
    }

    /// Query the AST
    pub fn query(
        &self,
        ast: &CompilationUnit,
        query: AstQuery,
    ) -> Result<QueryResult, EditError> {
        match query {
            AstQuery::FindByType(node_type) => {
                let mut results = Vec::new();
                self.find_nodes_by_type(ast, &node_type, &mut results, &mut vec![]);
                Ok(QueryResult::Multiple(results))
            }
            AstQuery::FindByPath(path) => {
                let node = self.navigate_to_path(ast, &path)?;
                Ok(QueryResult::Single(node))
            }
            AstQuery::FindByPattern(pattern) => {
                let mut results = Vec::new();
                self.find_nodes_by_pattern(ast, &pattern, &mut results, &mut vec![]);
                Ok(QueryResult::Multiple(results))
            }
            AstQuery::GetChildren(path) => {
                let children = self.get_children(ast, &path)?;
                Ok(QueryResult::Multiple(children))
            }
        }
    }

    /// Get available operations for a node at the given path
    pub fn get_available_operations(
        &self,
        ast: &CompilationUnit,
        path: &[usize],
    ) -> Result<Vec<EditOperation>, EditError> {
        let target = self.navigate_to_path(ast, path)?;
        let mut operations = Vec::new();

        // Always allow delete if the node exists
        operations.push(EditOperation::Delete(DeleteOperation {
            path: path.to_vec(),
        }));

        // Add replace operations based on node type
        match target {
            AstTarget::ModuleItems(_) => {
                // Can replace with any item
                operations.push(EditOperation::Replace(ReplaceOperation {
                    path: path.to_vec(),
                    new_node: AstNode::Item(self.create_placeholder_item()),
                }));
            }
            AstTarget::Expressions(_) => {
                // Can replace with any expression
                operations.push(EditOperation::Replace(ReplaceOperation {
                    path: path.to_vec(),
                    new_node: AstNode::Expression(self.create_placeholder_expression()),
                }));
            }
            _ => {}
        }

        Ok(operations)
    }

    /// Navigate to a specific path in the AST (immutable)
    fn navigate_to_path(&self, ast: &CompilationUnit, path: &[usize]) -> Result<AstTarget, EditError> {
        if path.is_empty() {
            return Ok(AstTarget::CompilationUnit(ast));
        }

        let mut current = AstTarget::CompilationUnit(ast);
        for &index in path {
            current = match current {
                AstTarget::CompilationUnit(cu) => {
                    if index < cu.modules.len() {
                        AstTarget::Module(&cu.modules[index])
                    } else {
                        return Err(EditError::PathNotFound { path: path.to_vec() });
                    }
                }
                AstTarget::Module(module) => {
                    if index < module.items.len() {
                        AstTarget::Item(&module.items[index])
                    } else {
                        return Err(EditError::PathNotFound { path: path.to_vec() });
                    }
                }
                _ => return Err(EditError::PathNotFound { path: path.to_vec() }),
            };
        }

        Ok(current)
    }

    /// Navigate to a specific path in the AST (mutable)
    fn navigate_to_path_mut(&mut self, ast: &mut CompilationUnit, path: &[usize]) -> Result<AstTarget, EditError> {
        if path.is_empty() {
            return Ok(AstTarget::CompilationUnit(ast));
        }

        // For mutable access, we need to handle this differently
        if path.len() == 1 {
            let index = path[0];
            if index < ast.modules.len() {
                return Ok(AstTarget::ModuleItems(&mut ast.modules[index].items));
            }
        }

        // For now, return an error for complex paths
        Err(EditError::PathNotFound { path: path.to_vec() })
    }

    /// Find nodes by type
    fn find_nodes_by_type(
        &self,
        ast: &CompilationUnit,
        node_type: &str,
        results: &mut Vec<AstTarget>,
        path: &mut Vec<usize>,
    ) {
        // TODO: Implement node type searching
    }

    /// Find nodes by pattern
    fn find_nodes_by_pattern(
        &self,
        ast: &CompilationUnit,
        pattern: &crate::query::QueryPattern,
        results: &mut Vec<AstTarget>,
        path: &mut Vec<usize>,
    ) {
        // TODO: Implement pattern matching
    }

    /// Get children of a node
    fn get_children(&self, ast: &CompilationUnit, path: &[usize]) -> Result<Vec<AstTarget>, EditError> {
        let target = self.navigate_to_path(ast, path)?;
        let mut children = Vec::new();

        match target {
            AstTarget::CompilationUnit(cu) => {
                for (i, module) in cu.modules.iter().enumerate() {
                    children.push(AstTarget::Module(module));
                }
            }
            AstTarget::Module(module) => {
                for (i, item) in module.items.iter().enumerate() {
                    children.push(AstTarget::Item(item));
                }
            }
            _ => {}
        }

        Ok(children)
    }

    /// Validate an operation before applying
    fn validate_operation(
        &self,
        ast: &CompilationUnit,
        operation: &EditOperation,
    ) -> Result<(), EditError> {
        // TODO: Implement operation validation
        Ok(())
    }

    /// Generate a unique node ID
    fn generate_node_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Create a placeholder item for template operations
    fn create_placeholder_item(&self) -> Item {
        Item::Let {
            name: "placeholder".into(),
            value: Expression::Literal(Literal::Int(0)),
            type_annotation: None,
        }
    }

    /// Create a placeholder expression for template operations
    fn create_placeholder_expression(&self) -> Expression {
        Expression::Literal(Literal::Int(0))
    }
}

impl Default for AstEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Target for AST navigation
#[derive(Debug)]
enum AstTarget<'a> {
    CompilationUnit(&'a CompilationUnit),
    Module(&'a Module),
    Item(&'a Item),
    Expression(&'a Expression),
    ModuleItems(&'a mut Vec<Item>),
    Expressions(&'a mut Vec<Expression>),
}

/// Result of an edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditResult {
    Inserted {
        path: Vec<usize>,
        node_id: String,
    },
    Deleted {
        path: Vec<usize>,
        removed_node: AstNode,
    },
    Replaced {
        path: Vec<usize>,
        old_node: AstNode,
        new_node: AstNode,
    },
    Moved {
        source_path: Vec<usize>,
        dest_path: Vec<usize>,
    },
}

/// Edit operation errors
#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("Path not found: {path:?}")]
    PathNotFound { path: Vec<usize> },

    #[error("Invalid node type: expected {expected}, found {found}")]
    InvalidNodeType { expected: String, found: String },

    #[error("Invalid insert target at path: {path:?}")]
    InvalidInsertTarget { path: Vec<usize> },

    #[error("Invalid delete target at path: {path:?}")]
    InvalidDeleteTarget { path: Vec<usize> },

    #[error("Invalid replace target at path: {path:?}")]
    InvalidReplaceTarget { path: Vec<usize> },

    #[error("Invalid move source at path: {path:?}")]
    InvalidMoveSource { path: Vec<usize> },

    #[error("Invalid move destination at path: {path:?}")]
    InvalidMoveDestination { path: Vec<usize> },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: crate::session::SessionId },

    #[error("Parse error: {0}")]
    Parse(#[from] x_lang_parser::ParseError),

    #[error("Type check error: {message}")]
    TypeCheck { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lang_parser::{parse_source, FileId, SyntaxStyle};

    #[test]
    fn test_ast_editor_creation() {
        let editor = AstEditor::new();
        assert!(editor.change_history.is_empty());
    }

    #[test]
    fn test_insert_operation() {
        let mut editor = AstEditor::new();
        let source = "let x = 42";
        let mut ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();

        let operation = EditOperation::Insert(InsertOperation {
            path: vec![0],
            node: AstNode::Item(Item::Let {
                name: "y".into(),
                value: Expression::Literal(Literal::Bool(true)),
                type_annotation: None,
            }),
        });

        let result = editor.apply_operation(&mut ast, operation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_operations() {
        let editor = AstEditor::new();
        let source = "let x = 42\nlet y = true";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();

        let query = AstQuery::FindByType("Item".to_string());
        let result = editor.query(&ast, query);
        assert!(result.is_ok());
    }
}