//! Edit operations for AST manipulation

use x_parser::{Item, Expr, Pattern, Type};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Edit operation that can be applied to an AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperation {
    Insert(InsertOperation),
    Delete(DeleteOperation),
    Replace(ReplaceOperation),
    Move(MoveOperation),
}

/// Insert a new node at a specific path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertOperation {
    /// Path to the parent node where the new node will be inserted
    pub path: Vec<usize>,
    /// The node to insert
    pub node: EditableNode,
}

/// Delete a node at a specific path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOperation {
    /// Path to the node to delete
    pub path: Vec<usize>,
}

/// Replace a node at a specific path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceOperation {
    /// Path to the node to replace
    pub path: Vec<usize>,
    /// The new node to replace it with
    pub new_node: EditableNode,
}

/// Move a node from one path to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOperation {
    /// Path to the source node
    pub source_path: Vec<usize>,
    /// Path to the destination
    pub dest_path: Vec<usize>,
}

/// Structural transformation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuralTransformation {
    /// Extract a subexpression into a variable
    ExtractVariable {
        path: Vec<usize>,
        variable_name: String,
    },
    /// Inline a variable
    InlineVariable {
        path: Vec<usize>,
    },
    /// Wrap expression in a function
    WrapInFunction {
        path: Vec<usize>,
        function_name: String,
        parameters: Vec<String>,
    },
    /// Unwrap a function call
    UnwrapFunction {
        path: Vec<usize>,
    },
    /// Refactor to use pattern matching
    RefactorToMatch {
        path: Vec<usize>,
        patterns: Vec<String>,
    },
}

/// Result of a structural transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationResult {
    pub transformation: StructuralTransformation,
    pub modified_paths: Vec<Vec<usize>>,
    pub new_nodes: Vec<EditableNode>,
}

/// Editable AST node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditableNode {
    Item(Item),
    Expr(Expr),
    Pattern(Pattern),
    Type(Type),
}

impl EditOperation {
    /// Create a new insert operation
    pub fn insert(path: Vec<usize>, node: EditableNode) -> Self {
        Self::Insert(InsertOperation { path, node })
    }

    /// Create a new delete operation
    pub fn delete(path: Vec<usize>) -> Self {
        Self::Delete(DeleteOperation { path })
    }

    /// Create a new replace operation
    pub fn replace(path: Vec<usize>, new_node: EditableNode) -> Self {
        Self::Replace(ReplaceOperation { path, new_node })
    }

    /// Create a new move operation
    pub fn move_node(source_path: Vec<usize>, dest_path: Vec<usize>) -> Self {
        Self::Move(MoveOperation { source_path, dest_path })
    }

    /// Get the primary path affected by this operation
    pub fn primary_path(&self) -> &[usize] {
        match self {
            EditOperation::Insert(op) => &op.path,
            EditOperation::Delete(op) => &op.path,
            EditOperation::Replace(op) => &op.path,
            EditOperation::Move(op) => &op.source_path,
        }
    }

    /// Get all paths affected by this operation
    pub fn affected_paths(&self) -> Vec<&[usize]> {
        match self {
            EditOperation::Insert(op) => vec![&op.path],
            EditOperation::Delete(op) => vec![&op.path],
            EditOperation::Replace(op) => vec![&op.path],
            EditOperation::Move(op) => vec![&op.source_path, &op.dest_path],
        }
    }

    /// Check if this operation conflicts with another operation
    pub fn conflicts_with(&self, other: &EditOperation) -> bool {
        let self_paths = self.affected_paths();
        let other_paths = other.affected_paths();
        
        for self_path in self_paths {
            for other_path in &other_paths {
                if paths_overlap(self_path, other_path) {
                    return true;
                }
            }
        }
        
        false
    }

    /// Generate a unique ID for this operation
    pub fn generate_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

/// Check if two paths overlap (one is a prefix of the other)
fn paths_overlap(path1: &[usize], path2: &[usize]) -> bool {
    let min_len = path1.len().min(path2.len());
    path1[..min_len] == path2[..min_len]
}

/// Builder for creating edit operations
#[derive(Debug, Default)]
pub struct EditOperationBuilder {
    operations: Vec<EditOperation>,
}

impl EditOperationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an insert operation
    pub fn insert(mut self, path: Vec<usize>, node: EditableNode) -> Self {
        self.operations.push(EditOperation::insert(path, node));
        self
    }

    /// Add a delete operation
    pub fn delete(mut self, path: Vec<usize>) -> Self {
        self.operations.push(EditOperation::delete(path));
        self
    }

    /// Add a replace operation
    pub fn replace(mut self, path: Vec<usize>, new_node: EditableNode) -> Self {
        self.operations.push(EditOperation::replace(path, new_node));
        self
    }

    /// Add a move operation
    pub fn move_node(mut self, source_path: Vec<usize>, dest_path: Vec<usize>) -> Self {
        self.operations.push(EditOperation::move_node(source_path, dest_path));
        self
    }

    /// Build the list of operations
    pub fn build(self) -> Vec<EditOperation> {
        self.operations
    }

    /// Check for conflicts between operations
    pub fn validate(&self) -> Result<(), String> {
        for (i, op1) in self.operations.iter().enumerate() {
            for (j, op2) in self.operations.iter().enumerate() {
                if i != j && op1.conflicts_with(op2) {
                    return Err(format!("Operation {} conflicts with operation {}", i, j));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{Literal, Expression};

    #[test]
    fn test_edit_operation_creation() {
        let node = AstNode::Expression(Expression::Literal(Literal::Int(42)));
        let op = EditOperation::insert(vec![0, 1], node);
        
        match op {
            EditOperation::Insert(insert_op) => {
                assert_eq!(insert_op.path, vec![0, 1]);
            }
            _ => panic!("Expected insert operation"),
        }
    }

    #[test]
    fn test_operation_conflicts() {
        let node1 = AstNode::Expression(Expression::Literal(Literal::Int(42)));
        let node2 = AstNode::Expression(Expression::Literal(Literal::Bool(true)));
        
        let op1 = EditOperation::insert(vec![0, 1], node1);
        let op2 = EditOperation::delete(vec![0, 1, 2]);
        
        assert!(op1.conflicts_with(&op2));
    }

    #[test]
    fn test_operation_builder() {
        let node = AstNode::Expression(Expression::Literal(Literal::Int(42)));
        
        let operations = EditOperationBuilder::new()
            .insert(vec![0], node)
            .delete(vec![1])
            .build();
        
        assert_eq!(operations.len(), 2);
    }

    #[test]
    fn test_paths_overlap() {
        assert!(paths_overlap(&[0, 1], &[0, 1, 2]));
        assert!(paths_overlap(&[0, 1, 2], &[0, 1]));
        assert!(!paths_overlap(&[0, 1], &[0, 2]));
        assert!(!paths_overlap(&[1], &[2]));
    }
}