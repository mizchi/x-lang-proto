//! High-performance S-expression parser and structural diff tool
//! 
//! This library provides fast, memory-efficient parsing and diffing of S-expressions
//! with support for binary serialization and content-addressed storage.

pub mod sexp;
pub mod parser;
pub mod serializer;
pub mod diff;
pub mod hash;
pub mod error;

pub use sexp::{SExp, Atom};
pub use parser::Parser;
pub use serializer::{BinarySerializer, BinaryDeserializer};
pub use diff::{StructuralDiff, DiffOp, DiffResult};
pub use hash::ContentHash;
pub use error::{SExpError, Result};