//! Token definitions for lexical analysis

use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Token types in the x Language language
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Number(String),  // Raw number string for parsing
    
    // Identifiers
    Ident(String),
    
    // Keywords
    Let,
    Fun,
    Fn,  // Alternative to Fun
    In,
    If,
    Then,
    Else,
    Match,
    With,
    Data,
    Type,
    Effect,
    Handler,
    Handle,
    Do,
    Pure,
    Forall,
    Test,
    
    // Module keywords
    Module,
    Import,
    Export,
    Pub,
    Crate,
    Package,
    Super,
    Self_,
    
    // WebAssembly Component Model keywords
    Interface,
    Component,
    Core,
    Func,
    Param,
    Result,
    Resource,
    
    // Effect keywords
    Resume,
    Return,
    Perform,
    
    // Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Equal,         // =
    EqualEqual,    // ==
    NotEqual,      // !=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    And,           // &&
    AndAnd,        // && (alias)
    Or,            // ||
    OrOr,          // || (alias)
    Not,           // !
    Bang,          // ! (alias)
    Ampersand,     // &
    
    // Type operators
    Arrow,         // ->
    LeftArrow,     // <-
    FatArrow,      // =>
    Pipe,          // |
    PipeForward,   // |>
    Cons,          // ::
    Caret,         // ^
    
    // Delimiters
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    LeftAngle,     // <
    RightAngle,    // >
    
    // Punctuation
    Comma,         // ,
    Semicolon,     // ;
    Colon,         // :
    DoubleColon,   // ::
    ColonColon,    // :: (alias)
    Dot,           // .
    Underscore,    // _
    Question,      // ?
    At,            // @
    
    // Special
    Newline,
    Whitespace,
    Comment(String),
    DocComment(String),  // Structured documentation comment
    
    // Error recovery
    Error(String),
    
    // End of file
    Eof,
}

impl TokenKind {
    /// Returns true if this token should be skipped during parsing
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::Comment(_) | TokenKind::Newline)
    }
    
    /// Returns true if this is a documentation comment
    pub fn is_doc_comment(&self) -> bool {
        matches!(self, TokenKind::DocComment(_))
    }
    
    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self, 
            TokenKind::Let | TokenKind::Fun | TokenKind::Fn | TokenKind::In | TokenKind::If |
            TokenKind::Then | TokenKind::Else | TokenKind::Match | TokenKind::With |
            TokenKind::Data | TokenKind::Type | TokenKind::Effect | TokenKind::Handler |
            TokenKind::Handle | TokenKind::Do | TokenKind::Pure | TokenKind::Forall |
            TokenKind::Test | TokenKind::Module | TokenKind::Import | TokenKind::Export | 
            TokenKind::Pub | TokenKind::Crate | TokenKind::Package | TokenKind::Super | 
            TokenKind::Self_ | TokenKind::Interface | TokenKind::Component | TokenKind::Core | 
            TokenKind::Func | TokenKind::Param | TokenKind::Result | TokenKind::Resource |
            TokenKind::Resume | TokenKind::Return | TokenKind::Perform
        )
    }
    
    /// Returns true if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(self,
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash |
            TokenKind::Percent | TokenKind::Equal | TokenKind::EqualEqual |
            TokenKind::NotEqual | TokenKind::Less | TokenKind::LessEqual |
            TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::And |
            TokenKind::Or | TokenKind::Not | TokenKind::Arrow | TokenKind::FatArrow |
            TokenKind::Pipe | TokenKind::PipeForward | TokenKind::Cons | TokenKind::Caret
        )
    }
    
    /// Returns true if this token is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self,
            TokenKind::Integer(_) | TokenKind::Float(_) | 
            TokenKind::String(_) | TokenKind::Bool(_)
        )
    }
    
    /// Get the precedence of this operator token (higher number = higher precedence)
    pub fn precedence(&self) -> Option<u8> {
        match self {
            TokenKind::PipeForward => Some(0), // Lowest precedence, right-associative
            TokenKind::OrOr | TokenKind::Or => Some(1),
            TokenKind::AndAnd | TokenKind::And => Some(2),
            TokenKind::EqualEqual | TokenKind::NotEqual => Some(3),
            TokenKind::Less | TokenKind::LessEqual | 
            TokenKind::Greater | TokenKind::GreaterEqual => Some(4),
            TokenKind::Cons => Some(5), // Right-associative list construction
            TokenKind::Caret => Some(6), // String concatenation
            TokenKind::Plus | TokenKind::Minus => Some(7),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(8),
            TokenKind::Not => Some(9),
            _ => None,
        }
    }
    
    /// Returns true if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        match self {
            TokenKind::Arrow => false, // Right-associative
            TokenKind::Cons => false, // Right-associative
            TokenKind::PipeForward => true, // Left-associative
            _ if self.is_operator() => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "{n}"),
            TokenKind::Float(n) => write!(f, "{n}"),
            TokenKind::String(s) => write!(f, "\"{s}\""),
            TokenKind::Bool(b) => write!(f, "{b}"),
            TokenKind::Number(s) => write!(f, "{s}"),
            TokenKind::Ident(name) => write!(f, "{name}"),
            
            // Keywords
            TokenKind::Let => write!(f, "let"),
            TokenKind::Fun => write!(f, "fun"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::In => write!(f, "in"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::With => write!(f, "with"),
            TokenKind::Data => write!(f, "data"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Effect => write!(f, "effect"),
            TokenKind::Handler => write!(f, "handler"),
            TokenKind::Handle => write!(f, "handle"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Pure => write!(f, "pure"),
            TokenKind::Forall => write!(f, "forall"),
            TokenKind::Test => write!(f, "test"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Crate => write!(f, "crate"),
            TokenKind::Package => write!(f, "package"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::Self_ => write!(f, "self"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Component => write!(f, "component"),
            TokenKind::Core => write!(f, "core"),
            TokenKind::Func => write!(f, "func"),
            TokenKind::Param => write!(f, "param"),
            TokenKind::Result => write!(f, "result"),
            TokenKind::Resource => write!(f, "resource"),
            TokenKind::Resume => write!(f, "resume"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Perform => write!(f, "perform"),
            
            // Operators
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::LeftArrow => write!(f, "<-"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::PipeForward => write!(f, "|>"),
            TokenKind::Cons => write!(f, "::"),
            TokenKind::Caret => write!(f, "^"),
            
            // Delimiters
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftAngle => write!(f, "<"),
            TokenKind::RightAngle => write!(f, ">"),
            
            // Punctuation
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::DoubleColon => write!(f, "::"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::At => write!(f, "@"),
            
            // Special
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Whitespace => write!(f, " "),
            TokenKind::Comment(text) => write!(f, "--{text}"),
            TokenKind::DocComment(text) => write!(f, "```{text}```"),
            TokenKind::Error(msg) => write!(f, "ERROR({msg})"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its source span
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
    
    pub fn eof(span: Span) -> Self {
        Token::new(TokenKind::Eof, span)
    }
    
    pub fn error(message: String, span: Span) -> Self {
        Token::new(TokenKind::Error(message), span)
    }
    
    /// Returns true if this token should be skipped during parsing
    pub fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.kind, self.span)
    }
}

/// Convert string keywords to token kinds
pub fn keyword_to_token(s: &str) -> Option<TokenKind> {
    match s {
        "let" => Some(TokenKind::Let),
        "fun" => Some(TokenKind::Fun),
        "fn" => Some(TokenKind::Fn),
        "in" => Some(TokenKind::In),
        "if" => Some(TokenKind::If),
        "then" => Some(TokenKind::Then),
        "else" => Some(TokenKind::Else),
        "match" => Some(TokenKind::Match),
        "with" => Some(TokenKind::With),
        "data" => Some(TokenKind::Data),
        "type" => Some(TokenKind::Type),
        "effect" => Some(TokenKind::Effect),
        "handler" => Some(TokenKind::Handler),
        "handle" => Some(TokenKind::Handle),
        "do" => Some(TokenKind::Do),
        "pure" => Some(TokenKind::Pure),
        "forall" => Some(TokenKind::Forall),
        "test" => Some(TokenKind::Test),
        "module" => Some(TokenKind::Module),
        "import" => Some(TokenKind::Import),
        "export" => Some(TokenKind::Export),
        "pub" => Some(TokenKind::Pub),
        "crate" => Some(TokenKind::Crate),
        "package" => Some(TokenKind::Package),
        "super" => Some(TokenKind::Super),
        "self" => Some(TokenKind::Self_),
        "interface" => Some(TokenKind::Interface),
        "component" => Some(TokenKind::Component),
        "core" => Some(TokenKind::Core),
        "func" => Some(TokenKind::Func),
        "param" => Some(TokenKind::Param),
        "result" => Some(TokenKind::Result),
        "resource" => Some(TokenKind::Resource),
        "resume" => Some(TokenKind::Resume),
        "return" => Some(TokenKind::Return),
        "perform" => Some(TokenKind::Perform),
        "true" => Some(TokenKind::Bool(true)),
        "false" => Some(TokenKind::Bool(false)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_token_precedence() {
        assert_eq!(TokenKind::Plus.precedence(), Some(7));
        assert_eq!(TokenKind::Star.precedence(), Some(8));
        assert_eq!(TokenKind::And.precedence(), Some(2));
        assert_eq!(TokenKind::EqualEqual.precedence(), Some(3));
        assert!(TokenKind::Star.precedence() > TokenKind::Plus.precedence());
    }

    #[test]
    fn test_keyword_recognition() {
        assert_eq!(keyword_to_token("let"), Some(TokenKind::Let));
        assert_eq!(keyword_to_token("effect"), Some(TokenKind::Effect));
        assert_eq!(keyword_to_token("true"), Some(TokenKind::Bool(true)));
        assert_eq!(keyword_to_token("false"), Some(TokenKind::Bool(false)));
        assert_eq!(keyword_to_token("unknown"), None);
    }

    #[test]
    fn test_token_properties() {
        assert!(TokenKind::Let.is_keyword());
        assert!(TokenKind::Plus.is_operator());
        assert!(TokenKind::Integer(42).is_literal());
        assert!(TokenKind::Whitespace.is_trivia());
        
        assert!(!TokenKind::Ident("x".to_string()).is_keyword());
        assert!(!TokenKind::Let.is_operator());
    }

    #[test]
    fn test_associativity() {
        assert!(TokenKind::Plus.is_left_associative());
        assert!(!TokenKind::Arrow.is_left_associative()); // Right-associative
    }
}