#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use crate::span::FileId;

    #[test]
    fn test_module_function_definition() {
        let input = r#"
            module Test
            
            Math.add x y = x + y
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_module_function_with_type() {
        let input = r#"
            module Test
            
            Math.multiply : Int -> Int -> Int
            Math.multiply x y = x * y
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }
}