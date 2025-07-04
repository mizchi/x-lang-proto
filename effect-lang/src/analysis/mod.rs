//! Analysis infrastructure using Salsa for incremental computation

pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod types;
pub mod inference;
pub mod unification;
pub mod effects;
pub mod error_reporting;
pub mod binary_type_checker;

pub use resolver::ModuleResolver;
pub use binary_type_checker::{BinaryTypeChecker, TypeCheckResult, ValidationMode};