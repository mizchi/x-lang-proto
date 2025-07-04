//! S-expression AST definition and core types

use serde::{Deserialize, Serialize};
use std::fmt;

/// S-expression abstract syntax tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SExp {
    /// Atomic value (string, number, boolean)
    Atom(Atom),
    /// Symbol identifier
    Symbol(String),
    /// List of S-expressions
    List(Vec<SExp>),
}

/// Atomic value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Atom {
    /// String literal
    String(String),
    /// Integer number
    Integer(i64),
    /// Floating point number
    Float(f64),
    /// Boolean value
    Boolean(bool),
}

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExp::Atom(atom) => write!(f, "{}", atom),
            SExp::Symbol(symbol) => write!(f, "{}", symbol),
            SExp::List(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::String(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
            Atom::Integer(i) => write!(f, "{}", i),
            Atom::Float(fl) => write!(f, "{}", fl),
            Atom::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
        }
    }
}

impl SExp {
    /// Returns true if this S-expression is an atom
    pub fn is_atom(&self) -> bool {
        matches!(self, SExp::Atom(_))
    }

    /// Returns true if this S-expression is a symbol
    pub fn is_symbol(&self) -> bool {
        matches!(self, SExp::Symbol(_))
    }

    /// Returns true if this S-expression is a list
    pub fn is_list(&self) -> bool {
        matches!(self, SExp::List(_))
    }

    /// Returns the length of a list, or 1 for atoms/symbols
    pub fn len(&self) -> usize {
        match self {
            SExp::List(elements) => elements.len(),
            _ => 1,
        }
    }

    /// Returns true if this is an empty list
    pub fn is_empty(&self) -> bool {
        match self {
            SExp::List(elements) => elements.is_empty(),
            _ => false,
        }
    }

    /// Get the nth element of a list, if this is a list
    pub fn get(&self, index: usize) -> Option<&SExp> {
        match self {
            SExp::List(elements) => elements.get(index),
            _ => None,
        }
    }

    /// Convert to a pretty-printed string with indentation
    pub fn to_pretty_string(&self, indent: usize) -> String {
        match self {
            SExp::Atom(atom) => atom.to_string(),
            SExp::Symbol(symbol) => symbol.clone(),
            SExp::List(elements) => {
                if elements.is_empty() {
                    "()".to_string()
                } else if elements.len() == 1 {
                    format!("({})", elements[0].to_pretty_string(indent))
                } else {
                    let mut result = String::from("(");
                    for (i, element) in elements.iter().enumerate() {
                        if i > 0 {
                            result.push('\n');
                            result.push_str(&" ".repeat(indent + 1));
                        }
                        result.push_str(&element.to_pretty_string(indent + 1));
                    }
                    result.push(')');
                    result
                }
            }
        }
    }
}

impl Atom {
    /// Returns true if this atom is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Atom::String(_))
    }

    /// Returns true if this atom is a number (integer or float)
    pub fn is_number(&self) -> bool {
        matches!(self, Atom::Integer(_) | Atom::Float(_))
    }

    /// Returns true if this atom is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, Atom::Boolean(_))
    }

    /// Convert number to f64 if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Atom::Integer(i) => Some(*i as f64),
            Atom::Float(f) => Some(*f),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sexp_display() {
        let atom = SExp::Atom(Atom::Integer(42));
        assert_eq!(atom.to_string(), "42");

        let symbol = SExp::Symbol("hello".to_string());
        assert_eq!(symbol.to_string(), "hello");

        let list = SExp::List(vec![
            SExp::Symbol("+".to_string()),
            SExp::Atom(Atom::Integer(1)),
            SExp::Atom(Atom::Integer(2)),
        ]);
        assert_eq!(list.to_string(), "(+ 1 2)");
    }

    #[test]
    fn test_sexp_properties() {
        let list = SExp::List(vec![SExp::Atom(Atom::Integer(1))]);
        assert!(list.is_list());
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);

        let empty_list = SExp::List(vec![]);
        assert!(empty_list.is_empty());
        assert_eq!(empty_list.len(), 0);
    }

    #[test]
    fn test_atom_properties() {
        let string_atom = Atom::String("hello".to_string());
        assert!(string_atom.is_string());
        assert!(!string_atom.is_number());

        let int_atom = Atom::Integer(42);
        assert!(int_atom.is_number());
        assert_eq!(int_atom.as_number(), Some(42.0));

        let float_atom = Atom::Float(3.14);
        assert!(float_atom.is_number());
        assert_eq!(float_atom.as_number(), Some(3.14));
    }
}