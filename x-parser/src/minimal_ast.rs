//! Minimal AST for x Language (Unison-inspired)
//! 
//! This module defines a minimal AST with only three node types:
//! - Atom: literals and identifiers
//! - List: applications and compound expressions  
//! - Ann: type annotations

use std::fmt;

/// Minimal expression type with only three variants
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Atomic values: numbers, strings, identifiers
    Atom(Atom),
    
    /// Lists: function applications and compound expressions
    /// First element determines the kind of expression
    List(Vec<Expr>),
    
    /// Type annotations (can be added by inference or written by user)
    Ann(Box<Expr>, Type),
}

/// Atomic values
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// Integer literal (default: i32)
    Int(i32),
    
    /// Float literal  
    Float(f64),
    
    /// Text literal
    Text(String),
    
    /// Boolean literal
    Bool(bool),
    
    /// Unit/void
    Unit,
    
    /// Identifier/symbol
    Symbol(String),
    
    /// Operator (like +, -, |>, etc.)
    Operator(String),
}

/// Types in the minimal AST
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Type variable
    Var(String),
    
    /// Type constructor (Int, Text, etc.)
    Con(String),
    
    /// Type application (List Int, Maybe Int, etc.)
    App(Box<Type>, Vec<Type>),
    
    /// Function type (a -> b)
    Fun(Box<Type>, Box<Type>),
    
    /// Effect type (a ->{Effect} b)
    Effect(Box<Type>, Vec<String>, Box<Type>),
    
    /// Forall type (forall a. ...)
    Forall(Vec<String>, Box<Type>),
}

impl Expr {
    /// Check if this is a special form
    pub fn is_special_form(&self) -> Option<&str> {
        match self {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Atom(Atom::Symbol(sym)) => {
                        match sym.as_str() {
                            "def" | "let" | "if" | "match" | "handle" | "do" | "with" => Some(sym),
                            _ => None,
                        }
                    }
                    Expr::Atom(Atom::Operator(op)) => {
                        match op.as_str() {
                            "->" | "|>" | ":" => Some(op),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    /// Check if this is a lambda expression
    pub fn is_lambda(&self) -> bool {
        match self {
            Expr::List(exprs) if exprs.len() >= 3 => {
                matches!(&exprs[1], Expr::Atom(Atom::Operator(op)) if op == "->")
            }
            _ => false,
        }
    }
    
    /// Check if this is a pipeline expression
    pub fn is_pipeline(&self) -> bool {
        match self {
            Expr::List(exprs) if exprs.len() >= 3 => {
                matches!(&exprs[1], Expr::Atom(Atom::Operator(op)) if op == "|>")
            }
            _ => false,
        }
    }
    
    /// Strip all type annotations
    pub fn strip_annotations(&self) -> Expr {
        match self {
            Expr::Atom(atom) => Expr::Atom(atom.clone()),
            Expr::List(exprs) => {
                Expr::List(exprs.iter().map(|e| e.strip_annotations()).collect())
            }
            Expr::Ann(expr, _) => expr.strip_annotations(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Atom(atom) => write!(f, "{}", atom),
            Expr::List(exprs) => {
                // Special formatting for common patterns
                if self.is_lambda() && exprs.len() >= 3 {
                    // Lambda: x -> body
                    write!(f, "{} -> {}", exprs[0], exprs[2])
                } else if self.is_pipeline() && exprs.len() >= 3 {
                    // Pipeline: expr |> func
                    write!(f, "{} |> {}", exprs[0], exprs[2])
                } else {
                    // General list
                    write!(f, "(")?;
                    for (i, expr) in exprs.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", expr)?;
                    }
                    write!(f, ")")
                }
            }
            Expr::Ann(expr, typ) => write!(f, "{} : {}", expr, typ),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Int(n) => write!(f, "{}", n),
            Atom::Float(fl) => write!(f, "{}", fl),
            Atom::Text(s) => write!(f, "\"{}\"", s),
            Atom::Bool(b) => write!(f, "{}", b),
            Atom::Unit => write!(f, "()"),
            Atom::Symbol(sym) => write!(f, "{}", sym),
            Atom::Operator(op) => write!(f, "{}", op),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Var(v) => write!(f, "{}", v),
            Type::Con(c) => write!(f, "{}", c),
            Type::App(typ, args) => {
                write!(f, "{}", typ)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                Ok(())
            }
            Type::Fun(from, to) => write!(f, "{} -> {}", from, to),
            Type::Effect(from, effects, to) => {
                write!(f, "{} ->{{", from)?;
                for (i, effect) in effects.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", effect)?;
                }
                write!(f, "}} {}", to)
            }
            Type::Forall(vars, body) => {
                write!(f, "forall")?;
                for var in vars {
                    write!(f, " {}", var)?;
                }
                write!(f, ". {}", body)
            }
        }
    }
}

/// Module representation in minimal AST
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// Module name
    pub name: String,
    
    /// Module items (definitions)
    pub items: Vec<Item>,
}

/// Top-level items
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Value definition with optional type signature
    ValueDef {
        name: String,
        type_sig: Option<Type>,
        body: Expr,
    },
    
    /// Type definition
    TypeDef {
        name: String,
        params: Vec<String>,
        definition: TypeDefKind,
    },
}

/// Kind of type definition
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    /// Type alias
    Alias(Type),
    
    /// Data type with constructors
    Data(Vec<Constructor>),
    
    /// Effect (like algebraic effects)
    Effect(Vec<EffectOperation>),
}

/// Data constructor
#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<Type>,
}

/// Effect operation
#[derive(Debug, Clone, PartialEq)]
pub struct EffectOperation {
    pub name: String,
    pub signature: Type,
}

/// Parser helper functions
impl Expr {
    /// Create a lambda expression: arg -> body
    pub fn lambda(arg: Expr, body: Expr) -> Expr {
        Expr::List(vec![
            arg,
            Expr::Atom(Atom::Operator("->".to_string())),
            body,
        ])
    }
    
    /// Create a pipeline expression: expr |> func
    pub fn pipeline(expr: Expr, func: Expr) -> Expr {
        Expr::List(vec![
            expr,
            Expr::Atom(Atom::Operator("|>".to_string())),
            func,
        ])
    }
    
    /// Create a function application: func arg1 arg2 ...
    pub fn app(func: Expr, args: Vec<Expr>) -> Expr {
        let mut exprs = vec![func];
        exprs.extend(args);
        Expr::List(exprs)
    }
    
    /// Create a type annotation: expr : type
    pub fn annotate(expr: Expr, typ: Type) -> Expr {
        Expr::Ann(Box::new(expr), typ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atom_display() {
        assert_eq!(format!("{}", Atom::Int(42)), "42");
        assert_eq!(format!("{}", Atom::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Atom::Text("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Atom::Bool(true)), "true");
        assert_eq!(format!("{}", Atom::Unit), "()");
        assert_eq!(format!("{}", Atom::Symbol("foo".to_string())), "foo");
        assert_eq!(format!("{}", Atom::Operator("|>".to_string())), "|>");
    }
    
    #[test]
    fn test_lambda_expression() {
        let lambda = Expr::lambda(
            Expr::Atom(Atom::Symbol("x".to_string())),
            Expr::Atom(Atom::Symbol("x".to_string())),
        );
        assert_eq!(format!("{}", lambda), "x -> x");
        assert!(lambda.is_lambda());
    }
    
    #[test]
    fn test_pipeline_expression() {
        let pipeline = Expr::pipeline(
            Expr::Atom(Atom::Symbol("list".to_string())),
            Expr::Atom(Atom::Symbol("reverse".to_string())),
        );
        assert_eq!(format!("{}", pipeline), "list |> reverse");
        assert!(pipeline.is_pipeline());
    }
    
    #[test]
    fn test_type_display() {
        let func_type = Type::Fun(
            Box::new(Type::Con("Int".to_string())),
            Box::new(Type::Con("Int".to_string())),
        );
        assert_eq!(format!("{}", func_type), "Int -> Int");
        
        let effect_type = Type::Effect(
            Box::new(Type::Con("String".to_string())),
            vec!["IO".to_string()],
            Box::new(Type::Con("Unit".to_string())),
        );
        assert_eq!(format!("{}", effect_type), "String ->{IO} Unit");
    }
}