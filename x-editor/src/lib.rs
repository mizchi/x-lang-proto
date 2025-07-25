//! x Language Language Service and AST Editor
//!
//! This crate provides a language service for x Language that supports direct AST manipulation.
//! It's designed specifically for AI-driven code editing without requiring text representation.

pub mod ast_editor;
pub mod language_service;
pub mod operations;
pub mod query;
pub mod session;
pub mod incremental;
pub mod validation;
pub mod index_system;
pub mod content_addressing;
pub mod tree_similarity;
pub mod annotated_ast;
pub mod namespace;
pub mod namespace_storage;
pub mod namespace_resolver;

// Re-export main types
pub use ast_editor::{AstEditor, EditResult, EditError};
pub use language_service::{LanguageService, LanguageServiceConfig};
pub use operations::{
    EditOperation, InsertOperation, DeleteOperation, ReplaceOperation, MoveOperation,
    StructuralTransformation, TransformationResult,
};
pub use query::{AstQuery, QueryResult, QueryPattern, NodeSelector};
pub use session::{EditSession, SessionId, SessionState};
pub use incremental::{IncrementalAnalyzer, AnalysisResult};
pub use validation::{ValidationResult, ValidationError};

use x_parser::CompilationUnit;
use x_checker::CheckResult;
use std::collections::HashMap;

/// Main entry point for the language service
#[derive(Debug)]
pub struct XLanguageEditor {
    language_service: LanguageService,
    ast_editor: AstEditor,
    sessions: HashMap<SessionId, EditSession>,
}

impl XLanguageEditor {
    /// Create a new editor instance
    pub fn new(config: LanguageServiceConfig) -> Self {
        Self {
            language_service: LanguageService::new(config),
            ast_editor: AstEditor::new(),
            sessions: HashMap::new(),
        }
    }

    /// Start a new editing session
    pub fn start_session(&mut self, source: &str) -> Result<SessionId, EditError> {
        let session_id = SessionId::new();
        let ast = self.language_service.parse(source)?;
        let session = EditSession::new(session_id, ast);
        self.sessions.insert(session_id, session);
        Ok(session_id)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: SessionId) -> Option<&EditSession> {
        self.sessions.get(&session_id)
    }

    /// Get mutable session by ID
    pub fn get_session_mut(&mut self, session_id: SessionId) -> Option<&mut EditSession> {
        self.sessions.get_mut(&session_id)
    }

    /// Apply an edit operation to a session
    pub fn apply_operation(
        &mut self,
        session_id: SessionId,
        operation: EditOperation,
    ) -> Result<EditResult, EditError> {
        let session = self.sessions.get_mut(&session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        self.ast_editor.apply_operation(&mut session.ast, operation)
    }

    /// Query AST in a session
    pub fn query_ast(
        &self,
        session_id: SessionId,
        query: AstQuery,
    ) -> Result<QueryResult, EditError> {
        let session = self.get_session(session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        self.ast_editor.query(&session.ast, query)
    }

    /// Type check a session
    pub fn type_check_session(
        &self,
        session_id: SessionId,
    ) -> Result<CheckResult, EditError> {
        let session = self.get_session(session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        self.language_service.type_check(&session.ast)
    }

    /// Validate a session
    pub fn validate_session(
        &self,
        session_id: SessionId,
    ) -> Result<ValidationResult, EditError> {
        let session = self.get_session(session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        self.language_service.validate(&session.ast)
    }

    /// Get available operations for a node
    pub fn get_available_operations(
        &self,
        session_id: SessionId,
        node_path: &[usize],
    ) -> Result<Vec<EditOperation>, EditError> {
        let session = self.get_session(session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        self.ast_editor.get_available_operations(&session.ast, node_path)
    }

    /// Close a session
    pub fn close_session(&mut self, session_id: SessionId) -> Result<(), EditError> {
        self.sessions.remove(&session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        Ok(())
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<SessionId> {
        self.sessions.keys().cloned().collect()
    }

    /// Get session statistics
    pub fn session_stats(&self, session_id: SessionId) -> Result<SessionStats, EditError> {
        let session = self.get_session(session_id)
            .ok_or(EditError::SessionNotFound { session_id })?;
        
        Ok(SessionStats {
            session_id,
            operations_count: session.operations.len(),
            nodes_count: self.count_nodes(&session.ast),
            last_modified: session.last_modified,
        })
    }

    /// Count nodes in AST
    fn count_nodes(&self, ast: &CompilationUnit) -> usize {
        // Simple node counting
        let mut count = 1; // CompilationUnit itself
        
        // Count module
        count += 1; // Module
        count += ast.module.items.len(); // Items
        count += ast.module.imports.len();
        count += ast.module.exports.as_ref().map(|e| e.items.len()).unwrap_or(0);
        
        count
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub session_id: SessionId,
    pub operations_count: usize,
    pub nodes_count: usize,
    pub last_modified: std::time::SystemTime,
}

/// Default configuration
impl Default for XLanguageEditor {
    fn default() -> Self {
        Self::new(LanguageServiceConfig::default())
    }
}

/// Convenience functions for common operations
pub mod convenience {
    use super::*;
    
    /// Quick AST editing without sessions
    pub fn edit_ast_direct(
        ast: &mut CompilationUnit,
        operation: EditOperation,
    ) -> Result<EditResult, EditError> {
        let mut editor = AstEditor::new();
        editor.apply_operation(ast, operation)
    }
    
    /// Quick AST querying
    pub fn query_ast_direct(
        ast: &CompilationUnit,
        query: AstQuery,
    ) -> Result<QueryResult, EditError> {
        let editor = AstEditor::new();
        editor.query(ast, query)
    }
    
    /// Parse and start editing in one step
    pub fn parse_and_edit(
        source: &str,
        operation: EditOperation,
    ) -> Result<(CompilationUnit, EditResult), EditError> {
        let config = LanguageServiceConfig::default();
        let service = LanguageService::new(config);
        let mut ast = service.parse(source)?;
        
        let mut editor = AstEditor::new();
        let result = editor.apply_operation(&mut ast, operation)?;
        
        Ok((ast, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::SyntaxStyle;
    
    #[test]
    fn test_editor_creation() {
        let config = LanguageServiceConfig::default();
        let editor = XLanguageEditor::new(config);
        
        assert!(editor.active_sessions().is_empty());
    }
    
    #[test]
    fn test_session_lifecycle() {
        let config = LanguageServiceConfig::default();
        let mut editor = XLanguageEditor::new(config);
        
        let source = "let x = 42";
        let session_id = editor.start_session(source).unwrap();
        
        assert_eq!(editor.active_sessions().len(), 1);
        assert!(editor.get_session(session_id).is_some());
        
        editor.close_session(session_id).unwrap();
        assert!(editor.active_sessions().is_empty());
    }
    
    #[test]
    fn test_convenience_functions() {
        let source = "let x = 42";
        let operation = EditOperation::Insert(InsertOperation {
            path: vec![0],
            node: crate::operations::EditableNode::Expr(x_parser::Expr::Literal(x_parser::Literal::Integer(100), x_parser::Span::single(x_parser::FileId::new(0), x_parser::span::ByteOffset::new(0)))),
        });
        
        let result = convenience::parse_and_edit(source, operation);
        assert!(result.is_ok());
    }
}