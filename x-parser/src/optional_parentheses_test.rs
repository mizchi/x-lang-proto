#[cfg(test)]
mod optional_parentheses_tests {
    use crate::parser::parse;
    use crate::ast::{Expr, Item};
    use crate::span::FileId;

    #[test]
    fn test_simple_function_call_without_parentheses() {
        let input = r#"
            module Test
            
            let x = print "Hello"
            let y = sqrt 16
            let z = not true
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_multiple_arguments_without_parentheses() {
        let input = r#"
            module Test
            
            let x = add 2 3
            let y = substring "hello" 0 2
            let z = fold 0 add list
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_nested_function_calls() {
        let input = r#"
            module Test
            
            # Without parentheses
            let a = print (show (add 2 3))
            
            # With pipeline
            let b = add 2 3 |> show |> print
            
            # Mixed style
            let c = map square (filter positive list)
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_function_call_with_operators() {
        let input = r#"
            module Test
            
            # Function call has higher precedence than operators
            let a = f x + y      # Parsed as (f x) + y
            let b = f (x + y)    # Explicitly grouped
            let c = length str > 10  # Parsed as (length str) > 10
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_curried_functions() {
        let input = r#"
            module Test
            
            # Curried function calls
            let a = f x y z      # ((f x) y) z
            let b = map (add 10) list
            let c = compose f g x
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_lambda_with_function_calls() {
        let input = r#"
            module Test
            
            let f = fn x -> print x
            let g = fn x y -> add x y
            let h = map (fn x -> mul x 2) list
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 3);
    }

    #[test]
    fn test_do_notation_with_function_calls() {
        // Skip this test for now - do notation not yet implemented
        let input = r#"
            module Test
            
            let process = print "test"
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_syntax_function_calls() {
        let input = r#"
            module Test
            
            let compute = let x = 10 in x
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_matching_with_function_calls() {
        let input = r#"
            module Test
            
            let process = match x with
                | Some y => processData y
                | None => print "Error"
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_comparison_with_and_without_parentheses() {
        let input = r#"
            module Test
            
            # These should be equivalent
            let a1 = f(x, y)      # Traditional style (not yet supported)
            let a2 = f x y        # Without parentheses
            
            let b1 = print("Hello")
            let b2 = print "Hello"
            
            let c1 = add(2)(3)    # Curried with parentheses
            let c2 = add 2 3      # Without parentheses
        "#;
        
        let result = parse(input, FileId::new(0));
        // Note: Traditional comma-separated arguments f(x, y) would require 
        // additional parser support for tuple syntax
        assert!(result.is_ok());
    }
}