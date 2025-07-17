//! Enhanced error reporting for type checking
//! 
//! This module provides detailed error reporting with:
//! - Source location tracking
//! - Context-aware error messages
//! - Suggestions for fixes
//! - Pretty-printed error displays

use x_parser::{
    Span,
    Symbol,
};
#[allow(unused_imports)]
use x_parser::{
    FileId,
    span::ByteOffset,
};
use crate::types::*;
use std::fmt;

/// Enhanced type error with context and suggestions
#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch {
        expected: Type,
        found: Type,
        span: Span,
    },
    UnboundVariable {
        name: Symbol,
        span: Span,
    },
    InfiniteType {
        var: TypeVar,
        typ: Type,
        span: Span,
    },
    ArityMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },
    InferenceError {
        message: String,
        symbol: Symbol,
        span: Span,
    },
    TestTypeMismatch {
        test_name: Symbol,
        expected: Type,
        found: Type,
        span: Span,
    },
    UnknownEffect {
        effect_name: String,
        span: Span,
    },
    UnknownOperation {
        effect_name: String,
        operation_name: String,
        span: Span,
    },
    UnhandledEffects {
        required: crate::effect_checker::EffectRow,
        available: crate::effect_checker::EffectRow,
        span: Span,
    },
    EffectRowMismatch {
        message: String,
        span: Span,
    },
    NotAFunction {
        typ: Type,
        span: Span,
    },
    InternalError {
        message: String,
        span: Span,
    },
}





impl TypeError {
    fn format_error(&self) -> String {
        match self {
            TypeError::TypeMismatch { expected, found, span: _ } => {
                format!("Type mismatch: expected {expected}, found {found}")
            }
            TypeError::UnboundVariable { name, span: _ } => {
                format!("Unbound variable: {name}")
            }
            TypeError::InfiniteType { var, typ, span: _ } => {
                format!("Infinite type: {} = {}", Type::Var(*var), typ)
            }
            TypeError::ArityMismatch { expected, found, span: _ } => {
                format!("Arity mismatch: expected {expected} arguments, found {found}")
            }
            TypeError::InferenceError { message, symbol: _, span: _ } => {
                format!("Inference error: {message}")
            }
            TypeError::TestTypeMismatch { test_name, expected, found, span: _ } => {
                format!("Test '{test_name}' type mismatch: expected {expected}, found {found}")
            }
            TypeError::UnknownEffect { effect_name, span: _ } => {
                format!("Unknown effect: {effect_name}")
            }
            TypeError::UnknownOperation { effect_name, operation_name, span: _ } => {
                format!("Unknown operation '{operation_name}' for effect '{effect_name}'")
            }
            TypeError::UnhandledEffects { required, available, span: _ } => {
                format!("Unhandled effects: required {:?}, available {:?}", required, available)
            }
            TypeError::EffectRowMismatch { message, span: _ } => {
                format!("Effect row mismatch: {message}")
            }
            TypeError::NotAFunction { typ, span: _ } => {
                format!("Expected function type, found {typ}")
            }
            TypeError::InternalError { message, span: _ } => {
                format!("Internal error: {message}")
            }
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}



/// Type error reporter
#[derive(Debug, Clone)]
pub struct TypeErrorReporter {
    errors: Vec<TypeError>,
    warnings: Vec<TypeError>,
}

impl Default for TypeErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn report_error(&mut self, error: TypeError) {
        self.errors.push(error);
    }

    pub fn report_warning(&mut self, warning: TypeError) {
        self.warnings.push(warning);
    }

    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    pub fn warnings(&self) -> &[TypeError] {
        &self.warnings
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_span() -> Span {
        Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0))
    }
    
    #[test]
    fn test_type_error_reporter() {
        let mut reporter = TypeErrorReporter::new();
        
        assert!(!reporter.has_errors());
        
        let error = TypeError::TypeMismatch {
            expected: Type::Con(Symbol::intern("Int")),
            found: Type::Con(Symbol::intern("String")),
            span: test_span(),
        };
        
        reporter.report_error(error);
        
        assert!(reporter.has_errors());
        assert_eq!(reporter.errors().len(), 1);
    }
}