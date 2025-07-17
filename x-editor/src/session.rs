//! Edit session management

use crate::operations::{EditOperation, InsertOperation, EditableNode};
use x_parser::{CompilationUnit, Expr, Literal, Span, FileId, span::ByteOffset};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Unique identifier for an edit session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    /// Create a new session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State of an edit session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and can be edited
    Active,
    /// Session is paused
    Paused,
    /// Session is read-only
    ReadOnly,
    /// Session is closed
    Closed,
}

/// Edit session containing AST and operation history
#[derive(Debug)]
pub struct EditSession {
    /// Unique session identifier
    pub id: SessionId,
    /// Current AST state
    pub ast: CompilationUnit,
    /// History of applied operations
    pub operations: Vec<EditOperation>,
    /// Current session state
    pub state: SessionState,
    /// When the session was created
    pub created_at: SystemTime,
    /// When the session was last modified
    pub last_modified: SystemTime,
    /// Undo/redo position in operation history
    pub history_position: usize,
}

impl EditSession {
    /// Create a new edit session
    pub fn new(id: SessionId, ast: CompilationUnit) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            ast,
            operations: Vec::new(),
            state: SessionState::Active,
            created_at: now,
            last_modified: now,
            history_position: 0,
        }
    }

    /// Add an operation to the session history
    pub fn add_operation(&mut self, operation: EditOperation) {
        // Truncate history if we're not at the end (for redo)
        self.operations.truncate(self.history_position);
        
        // Add the new operation
        self.operations.push(operation);
        self.history_position = self.operations.len();
        self.last_modified = SystemTime::now();
    }

    /// Check if undo is possible
    pub fn can_undo(&self) -> bool {
        self.history_position > 0
    }

    /// Check if redo is possible
    pub fn can_redo(&self) -> bool {
        self.history_position < self.operations.len()
    }

    /// Get the operation that would be undone
    pub fn peek_undo(&self) -> Option<&EditOperation> {
        if self.can_undo() {
            self.operations.get(self.history_position - 1)
        } else {
            None
        }
    }

    /// Get the operation that would be redone
    pub fn peek_redo(&self) -> Option<&EditOperation> {
        if self.can_redo() {
            self.operations.get(self.history_position)
        } else {
            None
        }
    }

    /// Move undo position back
    pub fn undo(&mut self) -> Option<&EditOperation> {
        if self.can_undo() {
            self.history_position -= 1;
            self.last_modified = SystemTime::now();
            self.operations.get(self.history_position)
        } else {
            None
        }
    }

    /// Move redo position forward
    pub fn redo(&mut self) -> Option<&EditOperation> {
        if self.can_redo() {
            let operation = self.operations.get(self.history_position);
            self.history_position += 1;
            self.last_modified = SystemTime::now();
            operation
        } else {
            None
        }
    }

    /// Get session age
    pub fn age(&self) -> std::time::Duration {
        self.last_modified.duration_since(self.created_at)
            .unwrap_or_default()
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Check if session is read-only
    pub fn is_read_only(&self) -> bool {
        matches!(self.state, SessionState::ReadOnly)
    }

    /// Set session state
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
        self.last_modified = SystemTime::now();
    }

    /// Clear operation history
    pub fn clear_history(&mut self) {
        self.operations.clear();
        self.history_position = 0;
        self.last_modified = SystemTime::now();
    }

    /// Get operation count
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Get all operations
    pub fn operations(&self) -> &[EditOperation] {
        &self.operations
    }

    /// Get operations applied in current state
    pub fn applied_operations(&self) -> &[EditOperation] {
        &self.operations[..self.history_position]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{parse_source, FileId, SyntaxStyle};
    use crate::operations::{EditOperation, InsertOperation};
    use x_parser::{Expr, Literal};

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_session_creation() {
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::SExpression).unwrap();
        let id = SessionId::new();
        let session = EditSession::new(id, ast);
        
        assert_eq!(session.id, id);
        assert!(session.is_active());
        assert_eq!(session.operation_count(), 0);
    }

    #[test]
    fn test_operation_history() {
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::SExpression).unwrap();
        let id = SessionId::new();
        let mut session = EditSession::new(id, ast);
        
        let operation = EditOperation::Insert(InsertOperation {
            path: vec![0],
            node: EditableNode::Expr(Expr::Literal(Literal::Integer(100), Span::new(FileId::new(0), ByteOffset(0), ByteOffset(3)))),
        });
        
        session.add_operation(operation);
        assert_eq!(session.operation_count(), 1);
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn test_undo_redo() {
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::SExpression).unwrap();
        let id = SessionId::new();
        let mut session = EditSession::new(id, ast);
        
        let operation = EditOperation::Insert(InsertOperation {
            path: vec![0],
            node: EditableNode::Expr(Expr::Literal(Literal::Integer(100), Span::new(FileId::new(0), ByteOffset(0), ByteOffset(3)))),
        });
        
        session.add_operation(operation);
        
        // Test undo
        assert!(session.can_undo());
        let undone = session.undo();
        assert!(undone.is_some());
        assert!(!session.can_undo());
        assert!(session.can_redo());
        
        // Test redo
        let redone = session.redo();
        assert!(redone.is_some());
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn test_session_state() {
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::SExpression).unwrap();
        let id = SessionId::new();
        let mut session = EditSession::new(id, ast);
        
        assert!(session.is_active());
        assert!(!session.is_read_only());
        
        session.set_state(SessionState::ReadOnly);
        assert!(!session.is_active());
        assert!(session.is_read_only());
    }
}