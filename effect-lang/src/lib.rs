//! EffectLang - LSP-first effect system functional programming language
//! 
//! This crate provides a complete implementation of a statically-typed functional
//! programming language with algebraic effects and handlers, designed primarily
//! for excellent editor integration through LSP.

pub mod core;
pub mod analysis;

// Re-exports for convenience
pub use core::{ast, span, symbol};

/// Result type used throughout the codebase
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the language implementation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parse error: {message}")]
    Parse { message: String },
    
    #[error("Type error: {message}")]
    Type { message: String },
    
    #[error("Effect error: {message}")]
    Effect { message: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("LSP error: {0}")]
    Lsp(#[from] lsp_server::ProtocolError),
}

/// Language version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LANGUAGE_NAME: &str = "EffectLang";
pub const FILE_EXTENSIONS: &[&str] = &["eff", "effect"];

/// LSP server capabilities that we support
pub mod capabilities {
    use lsp_types::*;
    
    pub fn server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            ..Default::default()
        }
    }
}