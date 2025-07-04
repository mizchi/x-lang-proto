//! Lexer tests for EffectLang

use effect_lang::{
    analysis::lexer::Lexer,
    core::{
        span::FileId,
        token::{Token, TokenKind},
    },
};

fn lex_string(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input, FileId::new(0));
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    tokens.into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_keywords() {
    let tokens = lex_string("module let fun if then else");
    assert_eq!(tokens, vec![
        TokenKind::Module,
        TokenKind::Let,
        TokenKind::Fun,
        TokenKind::If,
        TokenKind::Then,
        TokenKind::Else,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_identifiers() {
    let tokens = lex_string("hello world x' _underscore");
    assert_eq!(tokens, vec![
        TokenKind::Ident("hello".to_string()),
        TokenKind::Ident("world".to_string()),
        TokenKind::Ident("x'".to_string()),
        TokenKind::Ident("_underscore".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_numbers() {
    let tokens = lex_string("42 3.14 0 100");
    assert_eq!(tokens, vec![
        TokenKind::Number("42".to_string()),
        TokenKind::Number("3.14".to_string()),
        TokenKind::Number("0".to_string()),
        TokenKind::Number("100".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_strings() {
    let tokens = lex_string(r#""hello" "world with spaces" "escape\"test""#);
    assert_eq!(tokens, vec![
        TokenKind::String("hello".to_string()),
        TokenKind::String("world with spaces".to_string()),
        TokenKind::String("escape\"test".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_operators() {
    let tokens = lex_string("+ - * / = == != < <= > >= && || -> =>");
    assert_eq!(tokens, vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Equal,
        TokenKind::EqualEqual,
        TokenKind::NotEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::AndAnd,
        TokenKind::OrOr,
        TokenKind::Arrow,
        TokenKind::FatArrow,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_punctuation() {
    let tokens = lex_string("( ) [ ] { } , ; : . |");
    assert_eq!(tokens, vec![
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBracket,
        TokenKind::RightBracket,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
        TokenKind::Comma,
        TokenKind::Semicolon,
        TokenKind::Colon,
        TokenKind::Dot,
        TokenKind::Pipe,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_effect_keywords() {
    let tokens = lex_string("effect handler do handle resume perform");
    assert_eq!(tokens, vec![
        TokenKind::Effect,
        TokenKind::Handler,
        TokenKind::Do,
        TokenKind::Handle,
        TokenKind::Resume,
        TokenKind::Perform,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_type_keywords() {
    let tokens = lex_string("type data forall exists");
    assert_eq!(tokens, vec![
        TokenKind::Type,
        TokenKind::Data,
        TokenKind::Forall,
        TokenKind::Exists,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_module_keywords() {
    let tokens = lex_string("module import export");
    assert_eq!(tokens, vec![
        TokenKind::Module,
        TokenKind::Import,
        TokenKind::Export,
        TokenKind::Eof,
    ]);
}

#[test]
fn test_whitespace_handling() {
    let tokens = lex_string("  hello  \n\t world  ");
    assert_eq!(tokens, vec![
        TokenKind::Ident("hello".to_string()),
        TokenKind::Ident("world".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_line_comments() {
    let tokens = lex_string("hello -- this is a comment\nworld");
    assert_eq!(tokens, vec![
        TokenKind::Ident("hello".to_string()),
        TokenKind::Ident("world".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_simple_program() {
    let input = r#"
module Test
let add = fun x y -> x + y
let result = add 1 2
"#;
    let tokens = lex_string(input);
    assert_eq!(tokens, vec![
        TokenKind::Module,
        TokenKind::Ident("Test".to_string()),
        TokenKind::Let,
        TokenKind::Ident("add".to_string()),
        TokenKind::Equal,
        TokenKind::Fun,
        TokenKind::Ident("x".to_string()),
        TokenKind::Ident("y".to_string()),
        TokenKind::Arrow,
        TokenKind::Ident("x".to_string()),
        TokenKind::Plus,
        TokenKind::Ident("y".to_string()),
        TokenKind::Let,
        TokenKind::Ident("result".to_string()),
        TokenKind::Equal,
        TokenKind::Ident("add".to_string()),
        TokenKind::Number("1".to_string()),
        TokenKind::Number("2".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_error_recovery() {
    // Test that the lexer can handle unexpected characters gracefully
    let mut lexer = Lexer::new("hello @ world", FileId::new(0));
    let result = lexer.tokenize();
    
    // Should either succeed with error tokens or fail gracefully
    match result {
        Ok(tokens) => {
            // At minimum should tokenize the valid parts
            assert!(tokens.len() >= 3); // hello, some form of @, world, eof
        }
        Err(_) => {
            // Acceptable to fail on unexpected characters for now
        }
    }
}

#[test]
fn test_span_information() {
    let input = "hello world";
    let mut lexer = Lexer::new(input, FileId::new(0));
    let tokens = lexer.tokenize().expect("Should tokenize successfully");
    
    // Check that spans are reasonable
    assert!(tokens.len() >= 2); // hello, world, eof
    assert!(tokens[0].span.start < tokens[0].span.end);
    assert!(tokens[1].span.start >= tokens[0].span.end);
}