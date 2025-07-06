//! Language service functionality

use crate::validation::{ValidationResult, ValidationError};
use x_parser::{CompilationUnit, ParseError, SyntaxStyle, parse_source, FileId};
use x_checker::{type_check, CheckResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the language service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageServiceConfig {
    /// Default syntax style for parsing
    pub default_syntax: SyntaxStyle,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl Default for LanguageServiceConfig {
    fn default() -> Self {
        Self {
            default_syntax: SyntaxStyle::Haskell,
            enable_caching: true,
            cache_dir: None,
            max_cache_size: 1000,
        }
    }
}

/// Language service providing parsing, type checking, and validation
#[derive(Debug)]
pub struct LanguageService {
    config: LanguageServiceConfig,
}

impl LanguageService {
    /// Create a new language service
    pub fn new(config: LanguageServiceConfig) -> Self {
        Self { config }
    }

    /// Parse source code into an AST
    pub fn parse(&self, source: &str) -> Result<CompilationUnit, ParseError> {
        let file_id = FileId::new(0);
        parse_source(source, file_id, self.config.default_syntax)
    }

    /// Parse with specific syntax style
    pub fn parse_with_syntax(&self, source: &str, syntax: SyntaxStyle) -> Result<CompilationUnit, ParseError> {
        let file_id = FileId::new(0);
        parse_source(source, file_id, syntax)
    }

    /// Type check an AST
    pub fn type_check(&self, ast: &CompilationUnit) -> Result<CheckResult, crate::ast_editor::EditError> {
        Ok(type_check(ast))
    }

    /// Validate an AST
    pub fn validate(&self, ast: &CompilationUnit) -> Result<ValidationResult, crate::ast_editor::EditError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic validation checks
        if ast.module.items.is_empty() {
            warnings.push(ValidationError::EmptyCompilationUnit);
        }
        
        // Check module name
        if ast.module.name.to_string().is_empty() {
            errors.push(ValidationError::EmptyModuleName { module_index: 0 });
        }
        
        let is_valid = errors.is_empty();
        Ok(ValidationResult {
            errors,
            warnings,
            is_valid,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &LanguageServiceConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LanguageServiceConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_service_creation() {
        let config = LanguageServiceConfig::default();
        let service = LanguageService::new(config);
        assert_eq!(service.config.default_syntax, SyntaxStyle::OCaml);
    }

    #[test]
    fn test_parsing() {
        let config = LanguageServiceConfig::default();
        let service = LanguageService::new(config);
        
        let source = "let x = 42";
        let result = service.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation() {
        let config = LanguageServiceConfig::default();
        let service = LanguageService::new(config);
        
        let source = "let x = 42";
        let ast = service.parse(source).unwrap();
        let validation = service.validate(&ast).unwrap();
        
        assert!(validation.is_valid);
    }
}