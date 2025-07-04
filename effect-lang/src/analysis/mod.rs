//! Analysis infrastructure using Salsa for incremental computation

pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod types;
pub mod inference;
pub mod unification;
pub mod effects;
pub mod error_reporting;

pub use resolver::ModuleResolver;