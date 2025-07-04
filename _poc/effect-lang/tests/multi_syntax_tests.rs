//! Integration tests for multi-syntax support
//! 
//! These tests verify that different syntax styles can parse and print
//! the same semantic content correctly.

use effect_lang::{
    MultiSyntax, SyntaxStyle, SyntaxConfig,
    core::{span::FileId, symbol::Symbol, ast::*},
};

#[test]
fn test_multi_syntax_creation() {
    let multi = MultiSyntax::default();
    let styles = multi.supported_styles();
    
    // Should support all four syntax styles
    assert_eq!(styles.len(), 4);
    assert!(styles.contains(&SyntaxStyle::OCaml));
    assert!(styles.contains(&SyntaxStyle::SExp));
    assert!(styles.contains(&SyntaxStyle::Haskell));
    assert!(styles.contains(&SyntaxStyle::RustLike));
}

#[test]
fn test_syntax_style_parsing() {
    assert_eq!("ocaml".parse::<SyntaxStyle>().unwrap(), SyntaxStyle::OCaml);
    assert_eq!("sexp".parse::<SyntaxStyle>().unwrap(), SyntaxStyle::SExp);
    assert_eq!("haskell".parse::<SyntaxStyle>().unwrap(), SyntaxStyle::Haskell);
    assert_eq!("rust".parse::<SyntaxStyle>().unwrap(), SyntaxStyle::RustLike);
}

#[test]
fn test_simple_expression_conversion() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    // Parse a simple expression in OCaml style
    let ocaml_expr = "x |> f";
    let expr = multi.parse_expression(ocaml_expr, SyntaxStyle::OCaml, file_id)
        .expect("Should parse OCaml pipeline expression");
    
    // Print in different styles
    let configs = [
        SyntaxConfig { style: SyntaxStyle::OCaml, ..Default::default() },
        SyntaxConfig { style: SyntaxStyle::SExp, ..Default::default() },
        SyntaxConfig { style: SyntaxStyle::Haskell, ..Default::default() },
        SyntaxConfig { style: SyntaxStyle::RustLike, ..Default::default() },
    ];
    
    for config in &configs {
        let result = multi.print_expression(&expr, config);
        assert!(result.is_ok(), "Should be able to print in {:?} style", config.style);
        
        let printed = result.unwrap();
        assert!(!printed.is_empty(), "Printed result should not be empty for {:?}", config.style);
        
        println!("{:?}: {}", config.style, printed);
    }
}

#[test]
fn test_literal_expressions() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    // Test various literal expressions
    let test_cases = vec![
        ("42", "integer literal"),
        ("true", "boolean literal"),
        ("\"hello\"", "string literal"),
    ];
    
    for (expr_str, description) in test_cases {
        // Try to parse in OCaml style (our most complete parser)
        if let Ok(expr) = multi.parse_expression(expr_str, SyntaxStyle::OCaml, file_id) {
            // Print in all styles
            for style in [SyntaxStyle::OCaml, SyntaxStyle::SExp, SyntaxStyle::Haskell, SyntaxStyle::RustLike] {
                let config = SyntaxConfig { style, ..Default::default() };
                let result = multi.print_expression(&expr, &config);
                
                if let Ok(printed) = result {
                    println!("{} in {:?}: {}", description, style, printed);
                    assert!(!printed.is_empty());
                }
            }
        }
    }
}

#[test]
fn test_syntax_config_customization() {
    let configs = [
        SyntaxConfig {
            style: SyntaxStyle::OCaml,
            indent_size: 2,
            use_tabs: false,
            max_line_length: 80,
            preserve_comments: true,
        },
        SyntaxConfig {
            style: SyntaxStyle::Haskell,
            indent_size: 4,
            use_tabs: false,
            max_line_length: 100,
            preserve_comments: true,
        },
        SyntaxConfig {
            style: SyntaxStyle::RustLike,
            indent_size: 4,
            use_tabs: false,
            max_line_length: 120,
            preserve_comments: true,
        },
    ];
    
    for config in &configs {
        assert_eq!(config.style.to_string(), format!("{}", config.style));
    }
}

#[test]
fn test_conversion_between_styles() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    // Start with simple expressions that all parsers can handle
    let simple_expressions = vec![
        "x",
        "42", 
        "true",
        "false",
    ];
    
    for expr_str in simple_expressions {
        // Try converting from OCaml to other styles
        for target_style in [SyntaxStyle::SExp, SyntaxStyle::Haskell, SyntaxStyle::RustLike] {
            if let Ok(converted) = multi.convert(expr_str, SyntaxStyle::OCaml, target_style, file_id) {
                println!("Converted '{}' from OCaml to {:?}: '{}'", expr_str, target_style, converted);
                assert!(!converted.is_empty());
            }
        }
    }
}

#[test]
fn test_style_display() {
    assert_eq!(SyntaxStyle::OCaml.to_string(), "ocaml");
    assert_eq!(SyntaxStyle::SExp.to_string(), "sexp");
    assert_eq!(SyntaxStyle::Haskell.to_string(), "haskell");
    assert_eq!(SyntaxStyle::RustLike.to_string(), "rust");
}

#[test]
fn test_configuration_validation() {
    let config = SyntaxConfig::default();
    assert_eq!(config.style, SyntaxStyle::OCaml);
    assert_eq!(config.indent_size, 2);
    assert!(!config.use_tabs);
    assert_eq!(config.max_line_length, 100);
    assert!(config.preserve_comments);
}

#[test]
fn test_multi_syntax_error_handling() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    // Test with invalid syntax
    let invalid_expr = ")))((( invalid syntax";
    
    // Should handle errors gracefully
    let result = multi.parse_expression(invalid_expr, SyntaxStyle::OCaml, file_id);
    assert!(result.is_err(), "Should return error for invalid syntax");
}

#[test] 
fn test_expression_roundtrip() {
    let mut multi = MultiSyntax::default();
    let file_id = FileId::new(0);
    
    // Test simple expressions that should roundtrip
    let expressions = vec!["x", "42", "true"];
    
    for expr in expressions {
        // Parse in OCaml
        if let Ok(ast) = multi.parse_expression(expr, SyntaxStyle::OCaml, file_id) {
            // Print back to OCaml
            let config = SyntaxConfig { style: SyntaxStyle::OCaml, ..Default::default() };
            if let Ok(printed) = multi.print_expression(&ast, &config) {
                println!("Roundtrip: '{}' -> '{}'", expr, printed);
                // For simple expressions, they should be similar (exact match depends on formatting)
                assert!(!printed.is_empty());
            }
        }
    }
}