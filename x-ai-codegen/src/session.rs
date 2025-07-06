//! Code generation session management
//! 
//! This module manages the state and history of code generation sessions.

use anyhow::{Result, Context as _};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use x_parser::{Symbol, FileId};
use x_parser::ast::*;
use crate::{
    GeneratedCode, CodeIntent, RefinementIntent, IntentTarget,
    context::{CodeGenContext, ContextBuilder, GeneratedItem, GeneratedItemKind},
};

/// Code generation session
#[derive(Debug, Clone)]
pub struct CodeGenSession {
    /// Session ID
    pub id: String,
    
    /// Session start time
    pub started_at: DateTime<Utc>,
    
    /// Current context
    pub context: CodeGenContext,
    
    /// Generation history
    pub history: GenerationHistory,
    
    /// Session metadata
    pub metadata: SessionMetadata,
    
    /// Active file being worked on
    pub active_file: Option<FileId>,
}

/// Generation history
#[derive(Debug, Clone)]
pub struct GenerationHistory {
    /// Past generations (newest first)
    generations: VecDeque<HistoryEntry>,
    
    /// Maximum history size
    max_size: usize,
    
    /// Undo stack
    undo_stack: Vec<HistoryEntry>,
    
    /// Redo stack
    redo_stack: Vec<HistoryEntry>,
}

/// History entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Generation ID
    pub id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Generated code
    pub code: GeneratedCode,
    
    /// Intent that created this entry
    pub intent: GenerationIntent,
    
    /// Parent entry ID (for tracking lineage)
    pub parent_id: Option<String>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Intent that led to generation
#[derive(Debug, Clone)]
pub enum GenerationIntent {
    Initial(CodeIntent),
    Refinement(RefinementIntent),
    Completion(String),
    Manual(String),
}

/// Session metadata
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// User preferences
    pub preferences: HashMap<String, String>,
    
    /// Session notes
    pub notes: Vec<SessionNote>,
    
    /// Statistics
    pub stats: SessionStats,
}

/// Session note
#[derive(Debug, Clone)]
pub struct SessionNote {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub tags: Vec<String>,
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_generations: usize,
    pub successful_generations: usize,
    pub failed_generations: usize,
    pub refinements: usize,
    pub undos: usize,
    pub redos: usize,
}

impl CodeGenSession {
    /// Create a new session
    pub fn new() -> Self {
        Self {
            id: Self::generate_id(),
            started_at: Utc::now(),
            context: CodeGenContext::new(),
            history: GenerationHistory::new(),
            metadata: SessionMetadata::new(),
            active_file: None,
        }
    }
    
    /// Build context for an intent
    pub fn build_context(&self, intent: &CodeIntent) -> Result<CodeGenContext> {
        let mut builder = ContextBuilder::new();
        
        // Start with current context
        let mut context = self.context.clone();
        
        // Add all previously generated items
        for entry in self.history.recent_entries(10) {
            for item in &entry.code.ast.module.items {
                let generated_item = self.item_to_generated(&entry.code, item);
                context.add_generated_item(generated_item);
            }
        }
        
        // Build specific context for intent
        let intent_context = builder.build_for_intent(intent)?;
        
        // Merge contexts
        context.imports.extend(intent_context.imports);
        context.preferences = intent_context.preferences;
        
        Ok(context)
    }
    
    /// Build context for refinement
    pub fn build_refinement_context(
        &self,
        code: &GeneratedCode,
        intent: &RefinementIntent,
    ) -> Result<CodeGenContext> {
        let mut context = self.context.clone();
        
        // Add the code being refined to context
        for item in &code.ast.module.items {
            let generated_item = self.item_to_generated(code, item);
            context.add_generated_item(generated_item);
        }
        
        // Set up for refinement
        if let Some(target) = &intent.target {
            // Focus on the target item
            context.current_module = Some(Symbol::intern(&code.ast.module.name.to_string()));
        }
        
        Ok(context)
    }
    
    /// Get current context
    pub fn current_context(&self) -> CodeGenContext {
        self.context.clone()
    }
    
    /// Add generated code to session
    pub fn add_generated_code(&mut self, code: &GeneratedCode) {
        let entry = HistoryEntry {
            id: Self::generate_id(),
            timestamp: Utc::now(),
            code: code.clone(),
            intent: GenerationIntent::Initial(code.metadata.intent.clone()),
            parent_id: self.history.current_id(),
            tags: self.extract_tags(code),
        };
        
        self.history.add_entry(entry.clone());
        
        // Update context with new items
        for item in &code.ast.module.items {
            let generated_item = self.item_to_generated(code, item);
            self.context.add_generated_item(generated_item);
        }
        
        // Update stats
        self.metadata.stats.total_generations += 1;
        self.metadata.stats.successful_generations += 1;
    }
    
    /// Undo last generation
    pub fn undo(&mut self) -> Option<GeneratedCode> {
        if let Some(entry) = self.history.undo() {
            self.metadata.stats.undos += 1;
            
            // Rebuild context without the undone entry
            self.rebuild_context();
            
            Some(entry.code)
        } else {
            None
        }
    }
    
    /// Redo previously undone generation
    pub fn redo(&mut self) -> Option<GeneratedCode> {
        if let Some(entry) = self.history.redo() {
            self.metadata.stats.redos += 1;
            
            // Add back to context
            for item in &entry.code.ast.module.items {
                let generated_item = self.item_to_generated(&entry.code, item);
                self.context.add_generated_item(generated_item);
            }
            
            Some(entry.code)
        } else {
            None
        }
    }
    
    /// Get generation history
    pub fn history(&self) -> Vec<HistoryEntry> {
        self.history.all_entries()
    }
    
    /// Find generations by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&HistoryEntry> {
        self.history.find_by_tag(tag)
    }
    
    /// Add a note to the session
    pub fn add_note(&mut self, content: String, tags: Vec<String>) {
        self.metadata.notes.push(SessionNote {
            timestamp: Utc::now(),
            content,
            tags,
        });
    }
    
    /// Export session to JSON
    pub fn export(&self) -> Result<serde_json::Value> {
        use serde_json::json;
        
        Ok(json!({
            "id": self.id,
            "started_at": self.started_at.to_rfc3339(),
            "history": self.history.generations.iter().map(|entry| {
                json!({
                    "id": entry.id,
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "intent": format!("{:?}", entry.intent),
                    "parent_id": entry.parent_id,
                    "tags": entry.tags,
                })
            }).collect::<Vec<_>>(),
            "metadata": {
                "preferences": self.metadata.preferences,
                "notes": self.metadata.notes.iter().map(|note| {
                    json!({
                        "timestamp": note.timestamp.to_rfc3339(),
                        "content": note.content,
                        "tags": note.tags,
                    })
                }).collect::<Vec<_>>(),
                "stats": {
                    "total_generations": self.metadata.stats.total_generations,
                    "successful_generations": self.metadata.stats.successful_generations,
                    "failed_generations": self.metadata.stats.failed_generations,
                    "refinements": self.metadata.stats.refinements,
                    "undos": self.metadata.stats.undos,
                    "redos": self.metadata.stats.redos,
                },
            },
        }))
    }
    
    /// Helper: Generate unique ID
    fn generate_id() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }
    
    /// Helper: Convert AST item to GeneratedItem
    fn item_to_generated(&self, code: &GeneratedCode, item: &Item) -> GeneratedItem {
        let (name, kind) = match item {
            Item::ValueDef(def) => (def.name, GeneratedItemKind::Value),
            Item::TypeDef(def) => (def.name, GeneratedItemKind::Type),
            Item::EffectDef(def) => (def.name, GeneratedItemKind::Effect),
            _ => (Symbol::intern("unknown"), GeneratedItemKind::Value),
        };
        
        GeneratedItem {
            name,
            kind,
            ast: item.clone(),
            metadata: HashMap::new(),
        }
    }
    
    /// Helper: Extract tags from generated code
    fn extract_tags(&self, code: &GeneratedCode) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Tag by intent type
        match &code.metadata.intent.target {
            IntentTarget::Function { .. } => tags.push("function".to_string()),
            IntentTarget::DataType { .. } => tags.push("type".to_string()),
            IntentTarget::Module { .. } => tags.push("module".to_string()),
            IntentTarget::Algorithm { .. } => tags.push("algorithm".to_string()),
            IntentTarget::Interface { .. } => tags.push("interface".to_string()),
            IntentTarget::Effect { .. } => tags.push("effect".to_string()),
        }
        
        // Tag by constraints
        for constraint in &code.metadata.intent.constraints {
            use crate::intent::{Constraint, PerformanceConstraint, StyleConstraint};
            
            match constraint {
                Constraint::Performance(perf) => match perf {
                    PerformanceConstraint::Tailrecursive => tags.push("tail-recursive".to_string()),
                    PerformanceConstraint::TimeComplexity(c) => tags.push(format!("complexity-{}", c)),
                    _ => {}
                },
                Constraint::Style(style) => match style {
                    StyleConstraint::Functional => tags.push("functional".to_string()),
                    StyleConstraint::Imperative => tags.push("imperative".to_string()),
                    _ => {}
                },
                _ => {}
            }
        }
        
        tags
    }
    
    /// Helper: Rebuild context from history
    fn rebuild_context(&mut self) {
        self.context = CodeGenContext::new();
        
        for entry in self.history.all_entries() {
            for item in &entry.code.ast.module.items {
                let generated_item = self.item_to_generated(&entry.code, item);
                self.context.add_generated_item(generated_item);
            }
        }
    }
}

impl GenerationHistory {
    fn new() -> Self {
        Self {
            generations: VecDeque::new(),
            max_size: 100,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
    
    fn add_entry(&mut self, entry: HistoryEntry) {
        // Clear redo stack when new entry is added
        self.redo_stack.clear();
        
        // Add to history
        self.generations.push_front(entry);
        
        // Maintain max size
        if self.generations.len() > self.max_size {
            self.generations.pop_back();
        }
    }
    
    fn current_id(&self) -> Option<String> {
        self.generations.front().map(|e| e.id.clone())
    }
    
    fn recent_entries(&self, count: usize) -> Vec<&HistoryEntry> {
        self.generations.iter().take(count).collect()
    }
    
    fn all_entries(&self) -> Vec<HistoryEntry> {
        self.generations.iter().cloned().collect()
    }
    
    fn find_by_tag(&self, tag: &str) -> Vec<&HistoryEntry> {
        self.generations.iter()
            .filter(|entry| entry.tags.contains(&tag.to_string()))
            .collect()
    }
    
    fn undo(&mut self) -> Option<HistoryEntry> {
        if let Some(entry) = self.generations.pop_front() {
            self.undo_stack.push(entry.clone());
            Some(entry)
        } else {
            None
        }
    }
    
    fn redo(&mut self) -> Option<HistoryEntry> {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(entry.clone());
            self.generations.push_front(entry.clone());
            Some(entry)
        } else {
            None
        }
    }
}

impl SessionMetadata {
    fn new() -> Self {
        Self {
            preferences: HashMap::new(),
            notes: Vec::new(),
            stats: SessionStats::new(),
        }
    }
}

impl SessionStats {
    fn new() -> Self {
        Self {
            total_generations: 0,
            successful_generations: 0,
            failed_generations: 0,
            refinements: 0,
            undos: 0,
            redos: 0,
        }
    }
}

impl Default for CodeGenSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for multiple concurrent sessions
pub struct SessionManager {
    sessions: HashMap<String, CodeGenSession>,
    active_session: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
        }
    }
    
    /// Create a new session
    pub fn create_session(&mut self) -> String {
        let session = CodeGenSession::new();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.active_session = Some(id.clone());
        id
    }
    
    /// Get active session
    pub fn active_session(&self) -> Option<&CodeGenSession> {
        self.active_session.as_ref()
            .and_then(|id| self.sessions.get(id))
    }
    
    /// Get active session mutably
    pub fn active_session_mut(&mut self) -> Option<&mut CodeGenSession> {
        if let Some(id) = self.active_session.clone() {
            self.sessions.get_mut(&id)
        } else {
            None
        }
    }
    
    /// Switch to a different session
    pub fn switch_session(&mut self, id: &str) -> Result<()> {
        if self.sessions.contains_key(id) {
            self.active_session = Some(id.to_string());
            Ok(())
        } else {
            anyhow::bail!("Session {} not found", id)
        }
    }
    
    /// List all sessions
    pub fn list_sessions(&self) -> Vec<(&String, &CodeGenSession)> {
        self.sessions.iter().collect()
    }
    
    /// Close a session
    pub fn close_session(&mut self, id: &str) -> Result<()> {
        if self.active_session.as_ref() == Some(&id.to_string()) {
            self.active_session = None;
        }
        self.sessions.remove(id)
            .ok_or_else(|| anyhow::anyhow!("Session {} not found", id))?;
        Ok(())
    }
}