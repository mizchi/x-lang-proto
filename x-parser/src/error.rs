//! Parser error types and utilities

use crate::span::Span;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("Lexer error: {message}")]
    Lexer { message: String },

    #[error("Syntax error at {span:?}: {message}")]
    Syntax { message: String, span: Span },

    #[error("Unexpected token at {span:?}: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Unexpected end of file: expected {expected}")]
    UnexpectedEof { expected: String },

    #[error("Invalid syntax style: {style}")]
    InvalidSyntaxStyle { style: String },

    #[error("Binary format error: {message}")]
    BinaryFormat { message: String },

    #[error("I/O error: {message}")]
    Io { message: String },
}

impl ParseError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    pub fn lexer(message: impl Into<String>) -> Self {
        Self::Lexer {
            message: message.into(),
        }
    }

    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self::Syntax {
            message: message.into(),
            span,
        }
    }

    pub fn unexpected_token(expected: impl Into<String>, found: impl Into<String>, span: Span) -> Self {
        Self::UnexpectedToken {
            expected: expected.into(),
            found: found.into(),
            span,
        }
    }

    pub fn unexpected_eof(expected: impl Into<String>) -> Self {
        Self::UnexpectedEof {
            expected: expected.into(),
        }
    }

    pub fn invalid_syntax_style(style: impl Into<String>) -> Self {
        Self::InvalidSyntaxStyle {
            style: style.into(),
        }
    }

    pub fn binary_format(message: impl Into<String>) -> Self {
        Self::BinaryFormat {
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Get the span associated with this error, if any
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Syntax { span, .. } | Self::UnexpectedToken { span, .. } => Some(*span),
            _ => None,
        }
    }

    /// Check if this is a fatal error that should stop parsing
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::UnexpectedEof { .. } | Self::BinaryFormat { .. } | Self::Io { .. }
        )
    }
}

/// Error recovery strategies for parsing
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Skip to the next statement or declaration
    SkipToNext,
    /// Insert a missing token
    InsertToken(String),
    /// Replace the current token
    ReplaceToken(String),
    /// Abort parsing
    Abort,
}

/// Error reporter for collecting and formatting parse errors
#[derive(Debug, Default)]
pub struct ErrorReporter {
    errors: Vec<ParseError>,
    warnings: Vec<ParseError>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn report_warning(&mut self, warning: ParseError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn warnings(&self) -> &[ParseError] {
        &self.warnings
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    /// Format all errors and warnings as a string
    pub fn format_diagnostics(&self, source: &str) -> String {
        let mut output = String::new();

        for error in &self.errors {
            output.push_str(&format!("Error: {}\n", error));
            if let Some(span) = error.span() {
                output.push_str(&self.format_span_context(source, span));
            }
        }

        for warning in &self.warnings {
            output.push_str(&format!("Warning: {}\n", warning));
            if let Some(span) = warning.span() {
                output.push_str(&self.format_span_context(source, span));
            }
        }

        output
    }

    fn byte_to_line(&self, source: &str, byte_offset: u32) -> usize {
        source.chars().take(byte_offset as usize).filter(|&c| c == '\n').count() + 1
    }

    fn format_span_context(&self, source: &str, span: Span) -> String {
        let lines: Vec<&str> = source.lines().collect();
        // Convert byte offsets to line numbers
        let start_line = self.byte_to_line(source, span.start.as_u32()).saturating_sub(1);
        let _end_line = self.byte_to_line(source, span.end.as_u32()).min(lines.len());

        let mut output = String::new();
        
        if start_line < lines.len() {
            output.push_str(&format!("  {}: {}\n", start_line + 1, lines[start_line]));
            
            // Add pointer to error location
            let pointer_offset = 0; // TODO: Calculate column from byte offset
            let pointer = " ".repeat(pointer_offset + 4) + "^";
            output.push_str(&format!("  {}\n", pointer));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, ByteOffset};

    #[test]
    fn test_error_creation() {
        let span = Span::new(
            FileId::new(0),
            ByteOffset::new(0),
            ByteOffset::new(5),
        );

        let error = ParseError::syntax("test error", span);
        assert_eq!(error.span(), Some(span));
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();
        
        let span = Span::new(
            FileId::new(0),
            ByteOffset::new(0),
            ByteOffset::new(5),
        );

        reporter.report_error(ParseError::syntax("test error", span));
        reporter.report_warning(ParseError::parse("test warning"));

        assert!(reporter.has_errors());
        assert!(reporter.has_warnings());
        assert_eq!(reporter.errors().len(), 1);
        assert_eq!(reporter.warnings().len(), 1);
    }

    #[test]
    fn test_format_diagnostics() {
        let mut reporter = ErrorReporter::new();
        let source = "let x = 42\nlet y = invalid";
        
        let span = Span::new(
            FileId::new(0),
            ByteOffset::new(8),
            ByteOffset::new(15),
        );

        reporter.report_error(ParseError::syntax("invalid syntax", span));
        
        let formatted = reporter.format_diagnostics(source);
        println!("Formatted diagnostics:\n{}", formatted);
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("invalid syntax"));
    }
}