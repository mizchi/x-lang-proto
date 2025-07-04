//! Tests for pipeline syntax

use effect_lang::{
    core::{
        ast::*,
        span::{Span, FileId, ByteOffset},
        symbol::Symbol,
        token::{Token, TokenKind},
    },
    analysis::{
        lexer::Lexer,
        parser::Parser,
    },
};

fn create_test_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
}

fn parse_expression(input: &str) -> Result<Expr, effect_lang::Error> {
    let mut parser = Parser::new(input, FileId::new(0))?;
    parser.parse_expression_public()
}

#[test]
fn test_lexer_pipeline_operator() {
    let mut lexer = Lexer::new("x |> f", FileId::new(0));
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let token_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    
    assert_eq!(token_kinds, vec![
        TokenKind::Ident("x".to_string()),
        TokenKind::PipeForward,
        TokenKind::Ident("f".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_lexer_multiple_pipeline_operators() {
    let mut lexer = Lexer::new("x |> f |> g", FileId::new(0));
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let token_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    
    assert_eq!(token_kinds, vec![
        TokenKind::Ident("x".to_string()),
        TokenKind::PipeForward,
        TokenKind::Ident("f".to_string()),
        TokenKind::PipeForward,
        TokenKind::Ident("g".to_string()),
        TokenKind::Eof,
    ]);
}

#[test]
fn test_parser_simple_pipeline() {
    let expr = parse_expression("x |> f").expect("Parsing should succeed");
    
    // Pipeline x |> f should be parsed as f(x)
    match expr {
        Expr::App(func, args, _) => {
            // Function should be 'f'
            match func.as_ref() {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("f")),
                _ => panic!("Expected variable 'f'"),
            }
            
            // Arguments should be [x]
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("x")),
                _ => panic!("Expected variable 'x'"),
            }
        }
        _ => panic!("Expected function application, got: {:?}", expr),
    }
}

#[test]
fn test_parser_chained_pipeline() {
    let expr = parse_expression("x |> f |> g").expect("Parsing should succeed");
    
    // Pipeline x |> f |> g should be parsed as g(f(x))
    match expr {
        Expr::App(func, args, _) => {
            // Outer function should be 'g'
            match func.as_ref() {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("g")),
                _ => panic!("Expected variable 'g'"),
            }
            
            // Arguments should be [f(x)]
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::App(inner_func, inner_args, _) => {
                    // Inner function should be 'f'
                    match inner_func.as_ref() {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("f")),
                        _ => panic!("Expected variable 'f'"),
                    }
                    
                    // Inner arguments should be [x]
                    assert_eq!(inner_args.len(), 1);
                    match &inner_args[0] {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("x")),
                        _ => panic!("Expected variable 'x'"),
                    }
                }
                _ => panic!("Expected inner function application"),
            }
        }
        _ => panic!("Expected function application, got: {:?}", expr),
    }
}

#[test]
fn test_parser_pipeline_with_function_application() {
    let expr = parse_expression("x |> f y").expect("Parsing should succeed");
    
    // Pipeline x |> f y should be parsed as (f y)(x)
    match expr {
        Expr::App(func, args, _) => {
            // Function should be (f y)
            match func.as_ref() {
                Expr::App(inner_func, inner_args, _) => {
                    match inner_func.as_ref() {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("f")),
                        _ => panic!("Expected variable 'f'"),
                    }
                    assert_eq!(inner_args.len(), 1);
                    match &inner_args[0] {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("y")),
                        _ => panic!("Expected variable 'y'"),
                    }
                }
                _ => panic!("Expected function application for 'f y'"),
            }
            
            // Arguments should be [x]
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("x")),
                _ => panic!("Expected variable 'x'"),
            }
        }
        _ => panic!("Expected function application, got: {:?}", expr),
    }
}

#[test]
fn test_parser_pipeline_precedence() {
    let expr = parse_expression("x + y |> f").expect("Parsing should succeed");
    
    // Pipeline has lower precedence than +, so this should be (x + y) |> f = f(x + y)
    match expr {
        Expr::App(func, args, _) => {
            // Function should be 'f'
            match func.as_ref() {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("f")),
                _ => panic!("Expected variable 'f'"),
            }
            
            // Arguments should be [x + y]
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::App(op_func, op_args, _) => {
                    // Should be + operator
                    match op_func.as_ref() {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("+")),
                        _ => panic!("Expected + operator"),
                    }
                    assert_eq!(op_args.len(), 2);
                }
                _ => panic!("Expected addition expression"),
            }
        }
        _ => panic!("Expected function application, got: {:?}", expr),
    }
}

#[test]
fn test_parser_pipeline_with_arithmetic() {
    let expr = parse_expression("1 + 2 |> double |> print").expect("Parsing should succeed");
    
    // Should parse as print(double(1 + 2))
    match expr {
        Expr::App(func, args, _) => {
            // Outer function should be 'print'
            match func.as_ref() {
                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("print")),
                _ => panic!("Expected variable 'print'"),
            }
            
            // Argument should be double(1 + 2)
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::App(double_func, double_args, _) => {
                    match double_func.as_ref() {
                        Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("double")),
                        _ => panic!("Expected variable 'double'"),
                    }
                    assert_eq!(double_args.len(), 1);
                    // The inner should be 1 + 2
                    match &double_args[0] {
                        Expr::App(plus_func, plus_args, _) => {
                            match plus_func.as_ref() {
                                Expr::Var(name, _) => assert_eq!(*name, Symbol::intern("+")),
                                _ => panic!("Expected + operator"),
                            }
                            assert_eq!(plus_args.len(), 2);
                        }
                        _ => panic!("Expected addition"),
                    }
                }
                _ => panic!("Expected double function application"),
            }
        }
        _ => panic!("Expected function application"),
    }
}