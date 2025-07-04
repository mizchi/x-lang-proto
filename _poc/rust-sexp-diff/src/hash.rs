//! Content-addressed hashing for S-expressions

use crate::sexp::SExp;
use crate::serializer::serialize;
use sha2::{Sha256, Digest};

/// Content hash for S-expressions
pub struct ContentHash;

impl ContentHash {
    /// Calculate the content hash of an S-expression
    pub fn hash(sexp: &SExp) -> String {
        let binary = serialize(sexp).expect("Serialization should not fail");
        let mut hasher = Sha256::new();
        hasher.update(&binary);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Calculate short hash (first 8 characters)
    pub fn short_hash(sexp: &SExp) -> String {
        let full_hash = Self::hash(sexp);
        full_hash[..8].to_string()
    }

    /// Calculate hash from binary data
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Calculate short hash from binary data
    pub fn short_hash_bytes(data: &[u8]) -> String {
        let full_hash = Self::hash_bytes(data);
        full_hash[..8].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::{SExp, Atom};

    #[test]
    fn test_content_hash_consistency() {
        let sexp = SExp::List(vec![
            SExp::Symbol("defun".to_string()),
            SExp::Symbol("factorial".to_string()),
            SExp::List(vec![SExp::Symbol("n".to_string())]),
            SExp::List(vec![
                SExp::Symbol("if".to_string()),
                SExp::List(vec![
                    SExp::Symbol("=".to_string()),
                    SExp::Symbol("n".to_string()),
                    SExp::Atom(Atom::Integer(0)),
                ]),
                SExp::Atom(Atom::Integer(1)),
                SExp::List(vec![
                    SExp::Symbol("*".to_string()),
                    SExp::Symbol("n".to_string()),
                    SExp::List(vec![
                        SExp::Symbol("factorial".to_string()),
                        SExp::List(vec![
                            SExp::Symbol("-".to_string()),
                            SExp::Symbol("n".to_string()),
                            SExp::Atom(Atom::Integer(1)),
                        ]),
                    ]),
                ]),
            ]),
        ]);

        let hash1 = ContentHash::hash(&sexp);
        let hash2 = ContentHash::hash(&sexp);
        assert_eq!(hash1, hash2);

        let short_hash1 = ContentHash::short_hash(&sexp);
        let short_hash2 = ContentHash::short_hash(&sexp);
        assert_eq!(short_hash1, short_hash2);
        assert_eq!(short_hash1.len(), 8);
    }

    #[test]
    fn test_different_expressions_different_hashes() {
        let sexp1 = SExp::Atom(Atom::Integer(42));
        let sexp2 = SExp::Atom(Atom::Integer(43));

        let hash1 = ContentHash::hash(&sexp1);
        let hash2 = ContentHash::hash(&sexp2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_structural_equality_same_hash() {
        let sexp1 = SExp::List(vec![
            SExp::Symbol("+".to_string()),
            SExp::Atom(Atom::Integer(1)),
            SExp::Atom(Atom::Integer(2)),
        ]);

        let sexp2 = SExp::List(vec![
            SExp::Symbol("+".to_string()),
            SExp::Atom(Atom::Integer(1)),
            SExp::Atom(Atom::Integer(2)),
        ]);

        assert_eq!(ContentHash::hash(&sexp1), ContentHash::hash(&sexp2));
    }
}