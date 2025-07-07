//! AST validation functionality

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of AST validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationError>,
    /// Whether the AST is valid
    pub is_valid: bool,
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Empty compilation unit
    EmptyCompilationUnit,
    /// Empty module
    EmptyModule { module_index: usize },
    /// Empty module name
    EmptyModuleName { module_index: usize },
    /// Duplicate module name
    DuplicateModuleName { module_index: usize, name: String },
    /// Invalid identifier
    InvalidIdentifier { name: String },
    /// Undefined reference
    UndefinedReference { name: String },
    /// Type mismatch
    TypeMismatch { expected: String, found: String },
    /// Missing import
    MissingImport { module: String },
    /// Circular dependency
    CircularDependency { modules: Vec<String> },
    /// Unreachable code
    UnreachableCode,
    /// Unused variable
    UnusedVariable { name: String },
    /// Missing return
    MissingReturn,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyCompilationUnit => {
                write!(f, "Compilation unit is empty")
            }
            ValidationError::EmptyModule { module_index } => {
                write!(f, "Module {module_index} is empty")
            }
            ValidationError::EmptyModuleName { module_index } => {
                write!(f, "Module {module_index} has empty name")
            }
            ValidationError::DuplicateModuleName { module_index, name } => {
                write!(f, "Module {module_index} has duplicate name: {name}")
            }
            ValidationError::InvalidIdentifier { name } => {
                write!(f, "Invalid identifier: {name}")
            }
            ValidationError::UndefinedReference { name } => {
                write!(f, "Undefined reference: {name}")
            }
            ValidationError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {expected}, found {found}")
            }
            ValidationError::MissingImport { module } => {
                write!(f, "Missing import: {module}")
            }
            ValidationError::CircularDependency { modules } => {
                write!(f, "Circular dependency: {}", modules.join(" -> "))
            }
            ValidationError::UnreachableCode => {
                write!(f, "Unreachable code detected")
            }
            ValidationError::UnusedVariable { name } => {
                write!(f, "Unused variable: {name}")
            }
            ValidationError::MissingReturn => {
                write!(f, "Missing return statement")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(errors: Vec<ValidationError>, warnings: Vec<ValidationError>) -> Self {
        let is_valid = errors.is_empty();
        Self {
            errors,
            warnings,
            is_valid,
        }
    }

    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    /// Create a failed validation result
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            errors,
            warnings: Vec::new(),
            is_valid: false,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationError) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get total number of issues
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Merge with another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.errors.is_empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec![ValidationError::EmptyCompilationUnit];
        let result = ValidationResult::failure(errors);
        assert!(!result.is_valid);
        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::success();
        let mut result2 = ValidationResult::success();
        result2.add_error(ValidationError::EmptyCompilationUnit);
        
        result1.merge(result2);
        assert!(!result1.is_valid);
        assert!(result1.has_errors());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::InvalidIdentifier {
            name: "test".to_string(),
        };
        
        let display = format!("{}", error);
        assert!(display.contains("Invalid identifier"));
        assert!(display.contains("test"));
    }
}