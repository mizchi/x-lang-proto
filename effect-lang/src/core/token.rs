//! Token definitions for lexical analysis

use crate::core::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Token types in the EffectLang language
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
    
    // Module keywords
    Module,
    Import,
    Export,
    
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
    Cons,          // ::
    
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
    
    // Special
    Newline,
    Whitespace,
    Comment(String),
    
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
    
    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self, 
            TokenKind::Let | TokenKind::Fun | TokenKind::In | TokenKind::If |
            TokenKind::Then | TokenKind::Else | TokenKind::Match | TokenKind::With |
            TokenKind::Data | TokenKind::Type | TokenKind::Effect | TokenKind::Handler |
            TokenKind::Handle | TokenKind::Do | TokenKind::Pure | TokenKind::Forall |
            TokenKind::Resume | TokenKind::Return
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
            TokenKind::Pipe | TokenKind::Cons
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
            TokenKind::Or => Some(1),
            TokenKind::And => Some(2),
            TokenKind::EqualEqual | TokenKind::NotEqual => Some(3),
            TokenKind::Less | TokenKind::LessEqual | 
            TokenKind::Greater | TokenKind::GreaterEqual => Some(4),
            TokenKind::Plus | TokenKind::Minus => Some(5),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(6),
            TokenKind::Not => Some(7),
            _ => None,
        }
    }
    
    /// Returns true if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        match self {
            TokenKind::Arrow => false, // Right-associative
            _ if self.is_operator() => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Bool(b) => write!(f, "{}", b),
            TokenKind::Number(s) => write!(f, "{}", s),
            TokenKind::Ident(name) => write!(f, "{}", name),
            
            // Keywords
            TokenKind::Let => write!(f, "let"),
            TokenKind::Fun => write!(f, "fun"),
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
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
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
            TokenKind::Cons => write!(f, "::"),
            
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
            
            // Special
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Whitespace => write!(f, " "),
            TokenKind::Comment(text) => write!(f, "--{}", text),
            TokenKind::Error(msg) => write!(f, "ERROR({})", msg),
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
        "module" => Some(TokenKind::Module),
        "import" => Some(TokenKind::Import),
        "export" => Some(TokenKind::Export),
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
    use crate::core::span::{FileId, ByteOffset};

    #[test]
    fn test_token_precedence() {
        assert_eq!(TokenKind::Plus.precedence(), Some(5));
        assert_eq!(TokenKind::Star.precedence(), Some(6));
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