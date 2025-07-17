//! Semantic AST for x Language
//! 
//! This module defines a more semantic AST that's easier to analyze
//! while still being relatively minimal.

use crate::span::Span;

/// Expression with semantic information
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literals
    Literal(Literal, Span),
    
    /// Variable reference
    Var(String, Span),
    
    /// Function application
    App {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    
    /// Lambda expression
    Lambda {
        params: Vec<Pattern>,
        body: Box<Expr>,
        span: Span,
    },
    
    /// Let binding
    Let {
        pattern: Pattern,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    
    /// Conditional
    If {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
    
    /// Pattern matching
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    
    /// Effect handler (with before handle)
    With {
        handlers: Vec<Handler>,
        body: Box<Expr>,
        span: Span,
    },
    
    /// Do notation
    Do {
        stmts: Vec<DoStatement>,
        span: Span,
    },
    
    /// Pipeline
    Pipeline {
        expr: Box<Expr>,
        func: Box<Expr>,
        span: Span,
    },
    
    /// Type annotation
    Ann {
        expr: Box<Expr>,
        typ: Type,
        span: Span,
    },
    
    /// Effect operation
    Perform {
        effect: String,
        operation: String,
        args: Vec<Expr>,
        span: Span,
    },
    
    /// Resume continuation
    Resume {
        value: Box<Expr>,
        span: Span,
    },
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i32),
    Float(f64),
    Text(String),
    Bool(bool),
    Unit,
}

/// Patterns for matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Var(String),
    Literal(Literal),
    Constructor(String, Vec<Pattern>),
    Wildcard,
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

/// Do statement
#[derive(Debug, Clone, PartialEq)]
pub enum DoStatement {
    /// Expression statement
    Expr(Box<Expr>),
    
    /// Bind statement: pattern <- expr
    Bind {
        pattern: Pattern,
        expr: Box<Expr>,
    },
    
    /// Let statement: let pattern = expr
    Let {
        pattern: Pattern,
        expr: Box<Expr>,
    },
}

/// Effect handler
#[derive(Debug, Clone, PartialEq)]
pub struct Handler {
    /// Handler name or inline definition
    pub kind: HandlerKind,
    
    /// Handler operations
    pub operations: Vec<Operation>,
    
    /// Return clause
    pub return_clause: Option<ReturnClause>,
}

/// Handler kind
#[derive(Debug, Clone, PartialEq)]
pub enum HandlerKind {
    /// Named handler with arguments
    Named(String, Vec<Expr>),
    
    /// Inline handler
    Inline,
    
    /// Extensible effect handler
    Extension(String),
}

/// Effect operation handler
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub name: String,
    pub params: Vec<Pattern>,
    pub body: Box<Expr>,
}

/// Return clause for handlers
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub param: Pattern,
    pub body: Box<Expr>,
}

/// Types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Type variable
    Var(String),
    
    /// Type constructor
    Con(String),
    
    /// Type application
    App(Box<Type>, Vec<Type>),
    
    /// Function type
    Fun(Box<Type>, Box<Type>),
    
    /// Effect type
    Effect {
        from: Box<Type>,
        effects: Vec<String>,
        to: Box<Type>,
    },
    
    /// Forall type
    Forall(Vec<String>, Box<Type>),
    
    /// Row type (for extensible effects)
    Row(Vec<(String, Type)>, Option<String>), // fields and row variable
}

/// Top-level definitions
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    /// Value definition
    Value {
        name: String,
        params: Vec<Pattern>,
        body: Box<Expr>,
        type_sig: Option<Type>,
    },
    
    /// Type definition
    Type {
        name: String,
        params: Vec<String>,
        body: TypeBody,
    },
    
    /// Effect definition
    Effect {
        name: String,
        params: Vec<String>,
        operations: Vec<EffectOp>,
    },
    
    /// Handler definition
    Handler {
        name: String,
        params: Vec<Pattern>,
        handler: Handler,
    },
}

/// Type definition body
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// Type alias
    Alias(Type),
    
    /// Algebraic data type
    Data(Vec<Constructor>),
}

/// Data constructor
#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<Type>,
}

/// Effect operation signature
#[derive(Debug, Clone, PartialEq)]
pub struct EffectOp {
    pub name: String,
    pub signature: Type,
}

/// Module
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
}

/// Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module: String,
    pub items: ImportItems,
}

/// Import items
#[derive(Debug, Clone, PartialEq)]
pub enum ImportItems {
    All,
    Selected(Vec<String>),
}

/// Helper functions for building semantic AST
impl Expr {
    /// Create a with expression
    pub fn with_handler(handler: Handler, body: Expr) -> Expr {
        Expr::With {
            handlers: vec![handler],
            body: Box::new(body),
            span: Span::dummy(),
        }
    }
    
    /// Create a with expression with multiple handlers
    pub fn with_handlers(handlers: Vec<Handler>, body: Expr) -> Expr {
        Expr::With {
            handlers,
            body: Box::new(body),
            span: Span::dummy(),
        }
    }
    
    /// Create a pipeline expression
    pub fn pipe(expr: Expr, func: Expr) -> Expr {
        Expr::Pipeline {
            expr: Box::new(expr),
            func: Box::new(func),
            span: Span::dummy(),
        }
    }
}

impl Span {
    /// Dummy span for examples
    pub fn dummy() -> Self {
        // This would be implemented based on your span type
        unimplemented!("Span::dummy")
    }
}

/// Extensible effect example
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extensible_effects() {
        // effect State s {
        //   get : () -> s
        //   put : s -> ()
        // }
        let state_effect = Definition::Effect {
            name: "State".to_string(),
            params: vec!["s".to_string()],
            operations: vec![
                EffectOp {
                    name: "get".to_string(),
                    signature: Type::Fun(
                        Box::new(Type::Con("Unit".to_string())),
                        Box::new(Type::Var("s".to_string())),
                    ),
                },
                EffectOp {
                    name: "put".to_string(),
                    signature: Type::Fun(
                        Box::new(Type::Var("s".to_string())),
                        Box::new(Type::Con("Unit".to_string())),
                    ),
                },
            ],
        };
        
        // with state(42) { ... }
        let with_expr = Expr::With {
            handlers: vec![Handler {
                kind: HandlerKind::Named("state".to_string(), vec![
                    Expr::Literal(Literal::Int(42), Span::dummy())
                ]),
                operations: vec![],
                return_clause: None,
            }],
            body: Box::new(Expr::Perform {
                effect: "State".to_string(),
                operation: "get".to_string(),
                args: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        
        // Extensible handler
        let extensible = Handler {
            kind: HandlerKind::Extension("MyExtension".to_string()),
            operations: vec![
                Operation {
                    name: "log".to_string(),
                    params: vec![Pattern::Var("msg".to_string())],
                    body: Box::new(Expr::Resume {
                        value: Box::new(Expr::Literal(Literal::Unit, Span::dummy())),
                        span: Span::dummy(),
                    }),
                },
            ],
            return_clause: None,
        };
    }
}