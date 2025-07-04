//! Parser tests for EffectLang

use effect_lang::{
    analysis::parser::parse,
    core::{
        ast::*,
        span::FileId,
        symbol::Symbol,
    },
};

fn parse_string(input: &str) -> CompilationUnit {
    parse(input, FileId::new(0)).expect("Parsing should succeed")
}

#[test]
fn test_simple_module() {
    let input = r#"
module Test
let x = 42
"#;
    let cu = parse_string(input);
    assert_eq!(cu.module.name.segments[0], Symbol::intern("Test"));
    assert_eq!(cu.module.items.len(), 1);
    
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("x"));
            match &def.body {
                Expr::Literal(Literal::Integer(42), _) => {},
                _ => panic!("Expected integer literal 42"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_function_definition() {
    let input = r#"
module Test
let add = fun x y -> x + y
"#;
    let cu = parse_string(input);
    assert_eq!(cu.module.items.len(), 1);
    
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("add"));
            match &def.body {
                Expr::Lambda { params, body, .. } => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[0].name, Symbol::intern("x"));
                    assert_eq!(params[1].name, Symbol::intern("y"));
                    
                    // Check body is an application: x + y
                    match body.as_ref() {
                        Expr::App(_, args, _) => {
                            assert_eq!(args.len(), 2);
                        }
                        _ => panic!("Expected function application in lambda body"),
                    }
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_if_expression() {
    let input = r#"
module Test
let max = fun x y -> if x > y then x else y
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Lambda { body, .. } => {
                    match body.as_ref() {
                        Expr::If { condition, then_branch, else_branch, .. } => {
                            // Should have condition, then, and else parts
                            assert!(condition.as_ref() != then_branch.as_ref());
                            assert!(then_branch.as_ref() != else_branch.as_ref());
                        }
                        _ => panic!("Expected if expression in lambda body"),
                    }
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_multiple_definitions() {
    let input = r#"
module Test
let x = 42
let y = "hello"
let z = true
"#;
    let cu = parse_string(input);
    assert_eq!(cu.module.items.len(), 3);
    
    // Check all are value definitions
    for item in &cu.module.items {
        match item {
            Item::ValueDef(_) => {},
            _ => panic!("Expected all items to be value definitions"),
        }
    }
}

#[test]
fn test_type_annotation() {
    let input = r#"
module Test
let x : Int = 42
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert!(def.type_annotation.is_some());
            match def.type_annotation.as_ref().unwrap() {
                Type::Con(sym, _) => {
                    assert_eq!(*sym, Symbol::intern("Int"));
                }
                _ => panic!("Expected type constructor Int"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_module_imports() {
    let input = r#"
module Test
import Core.List
import Data.String as Str
let x = 42
"#;
    let cu = parse_string(input);
    assert_eq!(cu.module.imports.len(), 2);
    
    // Check first import
    match &cu.module.imports[0] {
        Import::Simple { module_path, .. } => {
            assert_eq!(module_path.segments.len(), 2);
            assert_eq!(module_path.segments[0], Symbol::intern("Core"));
            assert_eq!(module_path.segments[1], Symbol::intern("List"));
        }
        _ => panic!("Expected simple import"),
    }
    
    // Check second import with alias
    match &cu.module.imports[1] {
        Import::Qualified { module_path, alias, .. } => {
            assert_eq!(module_path.segments.len(), 2);
            assert_eq!(alias.as_ref().unwrap(), &Symbol::intern("Str"));
        }
        _ => panic!("Expected qualified import"),
    }
}

#[test]
fn test_let_expression() {
    let input = r#"
module Test
let result = let x = 10 in x * 2
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Let { bindings, body, .. } => {
                    assert_eq!(bindings.len(), 1);
                    assert_eq!(bindings[0].name, Symbol::intern("x"));
                    
                    // Body should be an application: x * 2
                    match body.as_ref() {
                        Expr::App(_, _, _) => {},
                        _ => panic!("Expected application in let body"),
                    }
                }
                _ => panic!("Expected let expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_pattern_matching() {
    let input = r#"
module Test
let head = fun xs -> match xs with
  | [] -> None
  | x :: _ -> Some x
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            match &def.body {
                Expr::Lambda { body, .. } => {
                    match body.as_ref() {
                        Expr::Match { expr: _, arms, .. } => {
                            assert_eq!(arms.len(), 2);
                            
                            // Check patterns
                            match &arms[0].pattern {
                                Pattern::Constructor { .. } => {}, // []
                                _ => panic!("Expected constructor pattern for empty list"),
                            }
                            
                            match &arms[1].pattern {
                                Pattern::Constructor { .. } => {}, // x :: _
                                _ => panic!("Expected constructor pattern for cons"),
                            }
                        }
                        _ => panic!("Expected match expression in lambda body"),
                    }
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}

#[test]
fn test_type_definition() {
    let input = r#"
module Test
type Maybe a = None | Some a
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::TypeDef(def) => {
            assert_eq!(def.name, Symbol::intern("Maybe"));
            assert_eq!(def.params.len(), 1);
            assert_eq!(def.params[0], Symbol::intern("a"));
            
            match &def.body {
                TypeDefBody::Variants(variants) => {
                    assert_eq!(variants.len(), 2);
                    assert_eq!(variants[0].name, Symbol::intern("None"));
                    assert_eq!(variants[1].name, Symbol::intern("Some"));
                }
                _ => panic!("Expected variant type definition"),
            }
        }
        _ => panic!("Expected type definition"),
    }
}

#[test]
fn test_error_recovery() {
    // Test that parser can handle some syntax errors gracefully
    let input = r#"
module Test
let x = 
let y = 42
"#;
    let result = parse(input, FileId::new(0));
    
    // Should either succeed with partial AST or fail gracefully
    match result {
        Ok(_) => {
            // Acceptable if parser can recover
        }
        Err(_) => {
            // Acceptable to fail on malformed input
        }
    }
}

#[test]
fn test_complex_expression() {
    let input = r#"
module Test
let factorial = fun n ->
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;
    let cu = parse_string(input);
    match &cu.module.items[0] {
        Item::ValueDef(def) => {
            assert_eq!(def.name, Symbol::intern("factorial"));
            match &def.body {
                Expr::Lambda { params, body, .. } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].name, Symbol::intern("n"));
                    
                    // Body should be an if expression
                    match body.as_ref() {
                        Expr::If { .. } => {},
                        _ => panic!("Expected if expression in factorial"),
                    }
                }
                _ => panic!("Expected lambda expression"),
            }
        }
        _ => panic!("Expected value definition"),
    }
}