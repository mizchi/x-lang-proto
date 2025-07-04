//! Abstract Syntax Tree definitions for EffectLang
//! 
//! This module defines the AST nodes for the EffectLang language,
//! including modules, types, effects, and expressions.

use crate::core::{span::{Span, HasSpan}, symbol::Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level compilation unit (usually a file)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationUnit {
    pub module: Module,
    pub span: Span,
}

/// Module definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: ModulePath,
    pub exports: Option<ExportList>,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
    pub span: Span,
}

/// Module path (e.g., Core.Types.User)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulePath {
    pub segments: Vec<Symbol>,
    pub span: Span,
}

impl ModulePath {
    pub fn new(segments: Vec<Symbol>, span: Span) -> Self {
        ModulePath { segments, span }
    }
    
    pub fn single(name: Symbol, span: Span) -> Self {
        ModulePath {
            segments: vec![name],
            span,
        }
    }
    
    pub fn push(&mut self, segment: Symbol) {
        self.segments.push(segment);
    }
    
    pub fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Export list specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportList {
    pub items: Vec<ExportItem>,
    pub span: Span,
}

/// Individual export item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportItem {
    pub kind: ExportKind,
    pub name: Symbol,
    pub alias: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportKind {
    Type,
    Value,
    Effect,
    Module,
}

/// Import declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub module_path: ModulePath,
    pub kind: ImportKind,
    pub alias: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    /// `import Module`
    Qualified,
    /// `import Module { item1, item2 }`
    Selective(Vec<ImportItem>),
    /// `import Module.*`
    Wildcard,
    /// `lazy import Module`
    Lazy,
    /// `import Module when condition`
    Conditional(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportItem {
    pub kind: ExportKind,
    pub name: Symbol,
    pub alias: Option<Symbol>,
    pub span: Span,
}

/// Top-level module item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    /// Type definition
    TypeDef(TypeDef),
    /// Value definition
    ValueDef(ValueDef),
    /// Effect definition
    EffectDef(EffectDef),
    /// Handler definition
    HandlerDef(HandlerDef),
    /// Module type definition
    ModuleTypeDef(ModuleTypeDef),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::TypeDef(def) => def.span,
            Item::ValueDef(def) => def.span,
            Item::EffectDef(def) => def.span,
            Item::HandlerDef(def) => def.span,
            Item::ModuleTypeDef(def) => def.span,
        }
    }
}

/// Type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: Symbol,
    pub type_params: Vec<TypeParam>,
    pub kind: TypeDefKind,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeDefKind {
    /// `data List[a] = Nil | Cons a (List[a])`
    Data(Vec<Constructor>),
    /// `type UserId = Int`
    Alias(Type),
    /// `type Reader[r, a] = r -> a`
    Abstract,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constructor {
    pub name: Symbol,
    pub fields: Vec<Type>,
    pub span: Span,
}

/// Value definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueDef {
    pub name: Symbol,
    pub type_annotation: Option<Type>,
    pub parameters: Vec<Pattern>,
    pub body: Expr,
    pub visibility: Visibility,
    pub purity: Purity,
    pub span: Span,
}

/// Effect definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectDef {
    pub name: Symbol,
    pub type_params: Vec<TypeParam>,
    pub operations: Vec<EffectOperation>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectOperation {
    pub name: Symbol,
    pub parameters: Vec<Type>,
    pub return_type: Type,
    pub span: Span,
}

/// Handler definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandlerDef {
    pub name: Symbol,
    pub type_annotation: Option<Type>,
    pub handled_effects: Vec<EffectRef>,
    pub handlers: Vec<EffectHandler>,
    pub return_clause: Option<ReturnClause>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectHandler {
    pub effect: EffectRef,
    pub operation: Symbol,
    pub parameters: Vec<Pattern>,
    pub continuation: Option<Symbol>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnClause {
    pub parameter: Pattern,
    pub body: Box<Expr>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Symbol,
    pub type_annotation: Option<Type>,
    pub span: Span,
}

/// Let binding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetBinding {
    pub name: Symbol,
    pub value: Expr,
    pub span: Span,
}

/// Handler for effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Handler {
    pub effect: Symbol,
    pub clauses: Vec<HandlerClause>,
    pub return_clause: Option<ReturnClause>,
    pub span: Span,
}

/// Handler clause for an operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandlerClause {
    pub operation: Symbol,
    pub params: Vec<Symbol>,
    pub body: Expr,
    pub span: Span,
}

/// Operation definition in effect
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationDef {
    pub name: Symbol,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub span: Span,
}

/// Match arm in pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
    pub span: Span,
}

/// Statement in do notation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DoStatement {
    /// let x = expr
    Let {
        pattern: Pattern,
        expr: Expr,
        span: Span,
    },
    /// let x <- expr (monadic bind)
    Bind {
        pattern: Pattern,
        expr: Expr,
        span: Span,
    },
    /// expr (expression statement)
    Expr(Expr),
}

/// Module type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleTypeDef {
    pub name: Symbol,
    pub signature: ModuleSignature,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSignature {
    pub items: Vec<SignatureItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignatureItem {
    TypeSig {
        name: Symbol,
        type_params: Vec<TypeParam>,
        kind: Option<Kind>,
        span: Span,
    },
    ValueSig {
        name: Symbol,
        type_annotation: Type,
        span: Span,
    },
    EffectSig {
        name: Symbol,
        operations: Vec<EffectOperation>,
        span: Span,
    },
}

/// Type expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Type variable: `a`
    Var(Symbol, Span),
    /// Type constructor: `Int`, `List`
    Con(Symbol, Span),
    /// Type application: `List[Int]`
    App(Box<Type>, Vec<Type>, Span),
    /// Function type: `a -> b <e>`
    Fun {
        params: Vec<Type>,
        return_type: Box<Type>,
        effects: EffectSet,
        span: Span,
    },
    /// Forall type: `forall a. a -> a`
    Forall {
        type_params: Vec<TypeParam>,
        body: Box<Type>,
        span: Span,
    },
    /// Effect type: `<IO, State[Int]>`
    Effects(EffectSet, Span),
    /// Exists type: `exists a. a`
    Exists {
        type_params: Vec<TypeParam>,
        body: Box<Type>,
        span: Span,
    },
    /// Record type: `{field1: Int, field2: String}`
    Record {
        fields: HashMap<Symbol, Type>,
        rest: Option<Box<Type>>,
        span: Span,
    },
    /// Variant type: `[Tag1 Int | Tag2 String]`
    Variant {
        variants: HashMap<Symbol, Type>,
        rest: Option<Box<Type>>,
        span: Span,
    },
    /// Tuple type: `(Int, String, Bool)`
    Tuple {
        types: Vec<Type>,
        span: Span,
    },
    /// Row type: `{field1: Int, field2: String | r}`
    Row {
        fields: HashMap<Symbol, Type>,
        rest: Option<Box<Type>>,
        span: Span,
    },
    /// Type hole: `?`
    Hole(Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Var(_, span) => *span,
            Type::Con(_, span) => *span,
            Type::App(_, _, span) => *span,
            Type::Fun { span, .. } => *span,
            Type::Forall { span, .. } => *span,
            Type::Effects(_, span) => *span,
            Type::Exists { span, .. } => *span,
            Type::Record { span, .. } => *span,
            Type::Variant { span, .. } => *span,
            Type::Tuple { span, .. } => *span,
            Type::Row { span, .. } => *span,
            Type::Hole(span) => *span,
        }
    }
}

/// Type parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: Symbol,
    pub kind: Option<Kind>,
    pub constraints: Vec<TypeConstraint>,
    pub span: Span,
}

/// Kind annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    /// `Type`
    Type,
    /// `Effect`
    Effect,
    /// `Row`
    Row,
    /// `k1 -> k2`
    Arrow(Box<Kind>, Box<Kind>),
}

/// Type constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeConstraint {
    pub class: Symbol,
    pub types: Vec<Type>,
    pub span: Span,
}

/// Effect reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectRef {
    pub name: Symbol,
    pub args: Vec<Type>,
    pub span: Span,
}

/// Effect set (collection of effects)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectSet {
    pub effects: Vec<EffectRef>,
    pub row_var: Option<Symbol>,
    pub span: Span,
}

impl EffectSet {
    pub fn empty(span: Span) -> Self {
        EffectSet {
            effects: Vec::new(),
            row_var: None,
            span,
        }
    }
    
    pub fn pure(span: Span) -> Self {
        Self::empty(span)
    }
}

/// Expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value: `42`, `"hello"`, `true`
    Literal(Literal, Span),
    /// Variable reference: `x`
    Var(Symbol, Span),
    /// Function application: `f x y`
    App(Box<Expr>, Vec<Expr>, Span),
    /// Lambda expression: `fun x -> x + 1`
    Lambda {
        parameters: Vec<Pattern>,
        body: Box<Expr>,
        span: Span,
    },
    /// Let binding: `let x = 1 in x + 2`
    Let {
        pattern: Pattern,
        type_annotation: Option<Type>,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    /// Conditional: `if cond then a else b`
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        span: Span,
    },
    /// Pattern matching: `match x with | pattern -> expr`
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// Do notation: `do { x <- action; return x }`
    Do {
        statements: Vec<DoStatement>,
        span: Span,
    },
    /// Handle expression: `handle expr { effect -> handler }`
    Handle {
        expr: Box<Expr>,
        handlers: Vec<EffectHandler>,
        return_clause: Option<Box<ReturnClause>>,
        span: Span,
    },
    /// Resume continuation: `resume value`
    Resume {
        value: Box<Expr>,
        span: Span,
    },
    /// Effect operation call: `effect.operation args`
    Perform {
        effect: Symbol,
        operation: Symbol,
        args: Vec<Expr>,
        span: Span,
    },
    /// Type annotation: `expr : Type`
    Ann {
        expr: Box<Expr>,
        type_annotation: Type,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Var(_, span) => *span,
            Expr::App(_, _, span) => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Let { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Do { span, .. } => *span,
            Expr::Handle { span, .. } => *span,
            Expr::Resume { span, .. } => *span,
            Expr::Perform { span, .. } => *span,
            Expr::Ann { span, .. } => *span,
        }
    }
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

/// Patterns for destructuring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: `_`
    Wildcard(Span),
    /// Variable binding: `x`
    Variable(Symbol, Span),
    /// Literal pattern: `42`, `"hello"`
    Literal(Literal, Span),
    /// Constructor pattern: `Some x`, `Cons h t`
    Constructor {
        name: Symbol,
        args: Vec<Pattern>,
        span: Span,
    },
    /// Record pattern: `{ x, y }`
    Record {
        fields: HashMap<Symbol, Pattern>,
        rest: Option<Box<Pattern>>,
        span: Span,
    },
    /// Tuple pattern: `(x, y, z)`
    Tuple {
        patterns: Vec<Pattern>,
        span: Span,
    },
    /// Or pattern: `Some x | None`
    Or {
        left: Box<Pattern>,
        right: Box<Pattern>,
        span: Span,
    },
    /// As pattern: `x@(Some y)`
    As {
        pattern: Box<Pattern>,
        name: Symbol,
        span: Span,
    },
    /// Type annotation: `x : Int`
    Ann {
        pattern: Box<Pattern>,
        type_annotation: Type,
        span: Span,
    },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(span) => *span,
            Pattern::Variable(_, span) => *span,
            Pattern::Literal(_, span) => *span,
            Pattern::Constructor { span, .. } => *span,
            Pattern::Record { span, .. } => *span,
            Pattern::Tuple { span, .. } => *span,
            Pattern::Or { span, .. } => *span,
            Pattern::As { span, .. } => *span,
            Pattern::Ann { span, .. } => *span,
        }
    }
}

/// Match case
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}


/// Visibility modifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal, // Visible within module
}

/// Purity annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Purity {
    Pure,
    Impure,
    Inferred,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::span::{FileId, ByteOffset};

    fn test_span() -> Span {
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
    }

    #[test]
    fn test_module_path() {
        let path = ModulePath::new(
            vec![
                Symbol::intern("Core"),
                Symbol::intern("Types"),
                Symbol::intern("User"),
            ],
            test_span(),
        );
        
        assert_eq!(path.to_string(), "Core.Types.User");
        assert_eq!(path.segments.len(), 3);
    }

    #[test]
    fn test_effect_set() {
        let empty_effects = EffectSet::empty(test_span());
        assert!(empty_effects.effects.is_empty());
        assert!(empty_effects.row_var.is_none());
    }

    #[test]
    fn test_ast_spans() {
        let literal = Expr::Literal(Literal::Integer(42), test_span());
        assert_eq!(literal.span(), test_span());
        
        let pattern = Pattern::Wildcard(test_span());
        assert_eq!(pattern.span(), test_span());
    }
}

// HasSpan trait implementations
impl HasSpan for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Var(_, span) => *span,
            Expr::App(_, _, span) => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Let { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Do { span, .. } => *span,
            Expr::Handle { span, .. } => *span,
            Expr::Resume { span, .. } => *span,
            Expr::Perform { span, .. } => *span,
            Expr::Ann { span, .. } => *span,
        }
    }
}

impl HasSpan for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(span) => *span,
            Pattern::Variable(_, span) => *span,
            Pattern::Literal(_, span) => *span,
            Pattern::Constructor { span, .. } => *span,
            Pattern::Tuple { span, .. } => *span,
            Pattern::Record { span, .. } => *span,
            Pattern::Or { span, .. } => *span,
            Pattern::As { span, .. } => *span,
            Pattern::Ann { span, .. } => *span,
        }
    }
}

impl HasSpan for Type {
    fn span(&self) -> Span {
        match self {
            Type::Var(_, span) => *span,
            Type::Con(_, span) => *span,
            Type::App(_, _, span) => *span,
            Type::Fun { span, .. } => *span,
            Type::Forall { span, .. } => *span,
            Type::Exists { span, .. } => *span,
            Type::Record { span, .. } => *span,
            Type::Variant { span, .. } => *span,
            Type::Tuple { span, .. } => *span,
            Type::Row { span, .. } => *span,
            Type::Effects(_, span) => *span,
            Type::Hole(span) => *span,
        }
    }
}