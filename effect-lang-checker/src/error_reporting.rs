//! Enhanced error reporting for type checking
//! 
//! This module provides detailed error reporting with:
//! - Source location tracking
//! - Context-aware error messages
//! - Suggestions for fixes
//! - Pretty-printed error displays

use crate::core::{
    span::Span,
    symbol::Symbol,
};
use crate::analysis::types::*;
use crate::{Error, Result};
use std::fmt;
use std::collections::HashMap;

/// Enhanced type error with context and suggestions
#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
    pub context: ErrorContext,
    pub suggestions: Vec<Suggestion>,
}

/// Different kinds of type errors
#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    /// Type mismatch between expected and actual
    TypeMismatch {
        expected: Type,
        actual: Type,
    },
    
    /// Unbound variable or type
    UnboundVariable {
        name: Symbol,
        kind: VariableKind,
    },
    
    /// Occurs check failure in unification
    InfiniteType {
        var: TypeVar,
        typ: Type,
    },
    
    /// Arity mismatch in function application
    ArityMismatch {
        expected: usize,
        actual: usize,
        function_type: Type,
    },
    
    /// Missing effect handler
    UnhandledEffect {
        effect: Symbol,
        operation: Symbol,
        required_effects: EffectSet,
    },
    
    /// Effect escaping its handler scope
    EscapingEffect {
        effect: Symbol,
        operation: Symbol,
    },
    
    /// Pattern match is not exhaustive
    NonExhaustivePattern {
        missing_patterns: Vec<String>,
    },
    
    /// Invalid recursive type
    InvalidRecursiveType {
        var: TypeVar,
        issue: RecursiveTypeIssue,
    },
    
    /// Type class constraint cannot be satisfied
    UnsatisfiedConstraint {
        constraint: Constraint,
        available_instances: Vec<Instance>,
    },
}

/// Kind of unbound variable
#[derive(Debug, Clone)]
pub enum VariableKind {
    Value,
    Type,
    Effect,
    Module,
}

/// Issues with recursive types
#[derive(Debug, Clone)]
pub enum RecursiveTypeIssue {
    NotStrictlyPositive,
    UnboundVariable,
    InvalidNesting,
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub function_name: Option<Symbol>,
    pub expression_type: Option<ExpressionType>,
    pub expected_from: Option<ExpectationSource>,
    pub local_bindings: HashMap<Symbol, Type>,
}

/// What kind of expression caused the error
#[derive(Debug, Clone)]
pub enum ExpressionType {
    FunctionApplication,
    LetBinding,
    Lambda,
    MatchExpression,
    EffectOperation,
    TypeAnnotation,
}

/// Where the type expectation came from
#[derive(Debug, Clone)]
pub enum ExpectationSource {
    FunctionReturn,
    FunctionParameter(usize),
    MatchArm,
    LetBinding,
    TypeAnnotation,
    EffectSignature,
}

/// Suggestion for fixing the error
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub kind: SuggestionKind,
    pub message: String,
    pub span: Option<Span>,
    pub replacement: Option<String>,
}

/// Different kinds of suggestions
#[derive(Debug, Clone)]
pub enum SuggestionKind {
    AddTypeAnnotation,
    ChangeType,
    AddImport,
    AddEffectHandler,
    FixPattern,
    UseCorrectArity,
    BreakRecursion,
}

impl TypeError {
    pub fn new(kind: TypeErrorKind, span: Span) -> Self {
        TypeError {
            kind,
            span,
            context: ErrorContext::default(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }
    
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    
    /// Create a type mismatch error with helpful context
    pub fn type_mismatch(expected: Type, actual: Type, span: Span) -> Self {
        let mut error = TypeError::new(
            TypeErrorKind::TypeMismatch { expected: expected.clone(), actual: actual.clone() },
            span
        );
        
        // Add suggestions based on common patterns
        if let (Type::Fun { .. }, Type::Con(_)) = (&expected, &actual) {
            error = error.with_suggestion(Suggestion {
                kind: SuggestionKind::UseCorrectArity,
                message: "Did you forget function arguments?".to_string(),
                span: Some(span),
                replacement: None,
            });
        }
        
        error
    }
    
    /// Create an unbound variable error
    pub fn unbound_variable(name: Symbol, kind: VariableKind, span: Span) -> Self {
        TypeError::new(
            TypeErrorKind::UnboundVariable { name, kind },
            span
        )
    }
    
    /// Create an infinite type error (occurs check failure)
    pub fn infinite_type(var: TypeVar, typ: Type, span: Span) -> Self {
        TypeError::new(
            TypeErrorKind::InfiniteType { var, typ },
            span
        )
    }
    
    /// Create an arity mismatch error
    pub fn arity_mismatch(expected: usize, actual: usize, function_type: Type, span: Span) -> Self {
        TypeError::new(
            TypeErrorKind::ArityMismatch { expected, actual, function_type },
            span
        )
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        ErrorContext {
            function_name: None,
            expression_type: None,
            expected_from: None,
            local_bindings: HashMap::new(),
        }
    }
}

impl ErrorContext {
    pub fn in_function(name: Symbol) -> Self {
        ErrorContext {
            function_name: Some(name),
            ..Default::default()
        }
    }
    
    pub fn with_expression_type(mut self, expr_type: ExpressionType) -> Self {
        self.expression_type = Some(expr_type);
        self
    }
    
    pub fn with_expectation(mut self, source: ExpectationSource) -> Self {
        self.expected_from = Some(source);
        self
    }
    
    pub fn with_binding(mut self, name: Symbol, typ: Type) -> Self {
        self.local_bindings.insert(name, typ);
        self
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl TypeError {
    /// Format the error with context and suggestions
    pub fn format_error(&self) -> String {
        let mut output = String::new();
        
        // Main error message
        output.push_str(&self.format_main_message());
        
        // Context information
        if let Some(context_info) = self.format_context() {
            output.push_str("\n\n");
            output.push_str(&context_info);
        }
        
        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\n\nSuggestions:");
            for suggestion in &self.suggestions {
                output.push_str(&format!("\n  • {}", suggestion.message));
            }
        }
        
        output
    }
    
    fn format_main_message(&self) -> String {
        match &self.kind {
            TypeErrorKind::TypeMismatch { expected, actual } => {
                format!(
                    "Type mismatch:\n  Expected: {}\n  Actual:   {}",
                    expected, actual
                )
            }
            
            TypeErrorKind::UnboundVariable { name, kind } => {
                let kind_str = match kind {
                    VariableKind::Value => "variable",
                    VariableKind::Type => "type",
                    VariableKind::Effect => "effect",
                    VariableKind::Module => "module",
                };
                format!("Unbound {}: {}", kind_str, name)
            }
            
            TypeErrorKind::InfiniteType { var, typ } => {
                format!(
                    "Cannot construct infinite type: {} = {}",
                    Type::Var(*var), typ
                )
            }
            
            TypeErrorKind::ArityMismatch { expected, actual, function_type } => {
                format!(
                    "Function expects {} arguments but got {}\n  Function type: {}",
                    expected, actual, function_type
                )
            }
            
            TypeErrorKind::UnhandledEffect { effect, operation, .. } => {
                format!(
                    "Unhandled effect operation: {}.{}",
                    effect, operation
                )
            }
            
            TypeErrorKind::EscapingEffect { effect, operation } => {
                format!(
                    "Effect {}.{} escapes its handler scope",
                    effect, operation
                )
            }
            
            TypeErrorKind::NonExhaustivePattern { missing_patterns } => {
                format!(
                    "Non-exhaustive pattern match. Missing patterns:\n{}",
                    missing_patterns.iter()
                        .map(|p| format!("  {}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            
            TypeErrorKind::InvalidRecursiveType { var, issue } => {
                let issue_str = match issue {
                    RecursiveTypeIssue::NotStrictlyPositive => "not strictly positive",
                    RecursiveTypeIssue::UnboundVariable => "contains unbound variables",
                    RecursiveTypeIssue::InvalidNesting => "invalid nesting",
                };
                format!("Invalid recursive type {}: {}", Type::Var(*var), issue_str)
            }
            
            TypeErrorKind::UnsatisfiedConstraint { constraint, .. } => {
                format!("Cannot satisfy constraint: {:?}", constraint)
            }
        }
    }
    
    fn format_context(&self) -> Option<String> {
        let mut context_parts = Vec::new();
        
        if let Some(func_name) = self.context.function_name {
            context_parts.push(format!("In function '{}'", func_name));
        }
        
        if let Some(expr_type) = &self.context.expression_type {
            let expr_str = match expr_type {
                ExpressionType::FunctionApplication => "function application",
                ExpressionType::LetBinding => "let binding",
                ExpressionType::Lambda => "lambda expression",
                ExpressionType::MatchExpression => "match expression",
                ExpressionType::EffectOperation => "effect operation",
                ExpressionType::TypeAnnotation => "type annotation",
            };
            context_parts.push(format!("In {}", expr_str));
        }
        
        if let Some(expectation) = &self.context.expected_from {
            let expectation_str = match expectation {
                ExpectationSource::FunctionReturn => "function return type",
                ExpectationSource::FunctionParameter(n) => &format!("parameter {} type", n + 1),
                ExpectationSource::MatchArm => "match arm",
                ExpectationSource::LetBinding => "let binding",
                ExpectationSource::TypeAnnotation => "type annotation",
                ExpectationSource::EffectSignature => "effect signature",
            };
            context_parts.push(format!("Expected from {}", expectation_str));
        }
        
        if !context_parts.is_empty() {
            Some(context_parts.join("\n"))
        } else {
            None
        }
    }
}

/// Convert TypeError to the main Error type
impl From<TypeError> for Error {
    fn from(type_error: TypeError) -> Self {
        Error::Type {
            message: type_error.format_error(),
        }
    }
}

/// Error reporter that collects and formats multiple errors
#[derive(Debug, Default, Clone)]
pub struct ErrorReporter {
    errors: Vec<TypeError>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        ErrorReporter {
            errors: Vec::new(),
        }
    }
    
    pub fn report(&mut self, error: TypeError) {
        self.errors.push(error);
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }
    
    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.has_errors() {
            // Return the first error
            Err(self.errors.into_iter().next().unwrap().into())
        } else {
            Ok(value)
        }
    }
    
    /// Format all errors for display
    pub fn format_all_errors(&self) -> String {
        if self.errors.is_empty() {
            return "No errors".to_string();
        }
        
        let mut output = String::new();
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
                output.push_str(&"─".repeat(50));
                output.push_str("\n\n");
            }
            output.push_str(&format!("Error {}: {}", i + 1, error.format_error()));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::span::{FileId, ByteOffset};
    use crate::core::symbol::Symbol;
    
    fn test_span() -> Span {
        Span::new(
            FileId::new(0),
            ByteOffset::new(0),
            ByteOffset::new(10),
        )
    }
    
    #[test]
    fn test_type_mismatch_error() {
        let expected = Type::Con(Symbol::intern("Int"));
        let actual = Type::Con(Symbol::intern("String"));
        let error = TypeError::type_mismatch(expected, actual, test_span());
        
        let formatted = error.format_error();
        assert!(formatted.contains("Type mismatch"));
        assert!(formatted.contains("Int"));
        assert!(formatted.contains("String"));
    }
    
    #[test]
    fn test_unbound_variable_error() {
        let error = TypeError::unbound_variable(
            Symbol::intern("undefined_var"),
            VariableKind::Value,
            test_span(),
        );
        
        let formatted = error.format_error();
        assert!(formatted.contains("Unbound variable"));
        assert!(formatted.contains("undefined_var"));
    }
    
    #[test]
    fn test_error_with_context() {
        let error = TypeError::unbound_variable(
            Symbol::intern("x"),
            VariableKind::Value,
            test_span(),
        ).with_context(
            ErrorContext::in_function(Symbol::intern("main"))
                .with_expression_type(ExpressionType::FunctionApplication)
        );
        
        let formatted = error.format_error();
        assert!(formatted.contains("In function 'main'"));
        assert!(formatted.contains("function application"));
    }
    
    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();
        
        assert!(!reporter.has_errors());
        assert_eq!(reporter.error_count(), 0);
        
        let error = TypeError::unbound_variable(
            Symbol::intern("x"),
            VariableKind::Value,
            test_span(),
        );
        
        reporter.report(error);
        
        assert!(reporter.has_errors());
        assert_eq!(reporter.error_count(), 1);
        
        let formatted = reporter.format_all_errors();
        assert!(formatted.contains("Error 1"));
    }
}