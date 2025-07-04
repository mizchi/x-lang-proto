//! Error types for the S-expression library

use thiserror::Error;

/// Main error type for S-expression operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SExpError {
    #[error("Parse error at position {pos}: {message}")]
    ParseError { pos: usize, message: String },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid character: {char} at position {pos}")]
    InvalidCharacter { char: char, pos: usize },
    
    #[error("Unterminated string literal at position {pos}")]
    UnterminatedString { pos: usize },
    
    #[error("Invalid number format: {value}")]
    InvalidNumber { value: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },
    
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for SExpError {
    fn from(error: std::io::Error) -> Self {
        SExpError::IoError(error.to_string())
    }
}

impl From<std::num::ParseFloatError> for SExpError {
    fn from(error: std::num::ParseFloatError) -> Self {
        SExpError::InvalidNumber {
            value: error.to_string(),
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, SExpError>;