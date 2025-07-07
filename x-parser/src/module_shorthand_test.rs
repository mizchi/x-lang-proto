#[cfg(test)]
mod module_shorthand_tests {
    use crate::parser::parse;
    use crate::span::FileId;

    #[test]
    fn test_simple_module_member_definition() {
        // Start with simple value definitions
        let input = r#"
            module Test
            
            Math.pi = 3.14159
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_nested_module_definition() {
        let input = r#"
            module Test
            
            Core.List.map : (a -> b) -> List a -> List b
            Core.List.map f list = match list {
                Nil -> Nil
                Cons x xs -> Cons (f x) (Core.List.map f xs)
            }
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_module_member_with_visibility() {
        let input = r#"
            module Test
            
            pub String.length : String -> Int
            pub String.length s = lengthImpl s
            
            private String.validate : String -> Bool
            private String.validate s = true
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_pattern_matching_shorthand() {
        let input = r#"
            module Test
            
            List.head : List a -> Maybe a
            List.head Nil = None
            List.head (Cons x _) = Some x
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_mixed_syntax() {
        let input = r#"
            module Test
            
            # Traditional block syntax
            module Utils {
                export { helper }
                
                helper : Int -> Int
                helper x = x * 2
            }
            
            # Shorthand syntax
            Math.double : Int -> Int
            Math.double x = x * 2
            
            Math.triple : Int -> Int
            Math.triple x = x * 3
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_type_definition_shorthand() {
        let input = r#"
            module Test
            
            Http.Request = {
                method : Method
                path : String
                headers : Map String String
            }
            
            Http.Response = {
                status : Int
                body : String
            }
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_effect_definition_shorthand() {
        let input = r#"
            module Test
            
            Http.effect IO {
                send : Request -> Response
                receive : () -> Request
            }
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_function_call_with_module_path() {
        let input = r#"
            module Test
            
            let result = Math.add 2 3
            let text = String.concat "Hello" " World"
            let items = List.map double [1, 2, 3]
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }
}