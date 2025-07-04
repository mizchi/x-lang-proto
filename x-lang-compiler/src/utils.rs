//! Utility functions for code generation

use crate::core::symbol::Symbol;

/// Sanitize an identifier for target language compatibility
pub fn sanitize_identifier(symbol: Symbol, target: &str) -> String {
    let name = symbol.as_str();
    
    match target {
        "typescript" => sanitize_typescript_identifier(name),
        "wasm-gc" => sanitize_wasm_identifier(name),
        _ => name.to_string(),
    }
}

/// Sanitize identifier for TypeScript
fn sanitize_typescript_identifier(name: &str) -> String {
    // TypeScript reserved words
    let reserved = &[
        "any", "boolean", "break", "case", "catch", "class", "const", "continue",
        "debugger", "default", "delete", "do", "else", "enum", "export", "extends",
        "false", "finally", "for", "function", "if", "import", "in", "instanceof",
        "new", "null", "number", "return", "string", "super", "switch", "this",
        "throw", "true", "try", "typeof", "undefined", "var", "void", "while", "with",
    ];
    
    let mut result = name.to_string();
    
    // Replace invalid characters
    result = result.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '$' { c } else { '_' })
        .collect();
    
    // Ensure it starts with a letter or underscore
    if result.chars().next().map_or(false, |c| c.is_numeric()) {
        result = format!("_{}", result);
    }
    
    // Avoid reserved words
    if reserved.contains(&result.as_str()) {
        result = format!("_{}", result);
    }
    
    result
}

/// Sanitize identifier for WebAssembly
fn sanitize_wasm_identifier(name: &str) -> String {
    let mut result = name.to_string();
    
    // WebAssembly identifiers can contain alphanumeric, _, $, ., +, -, *, /, \, ^, ~, =, <, >, !, ?, @, #, |, &, %, `
    result = result.chars()
        .map(|c| if c.is_alphanumeric() || "_$.+-*/\\^~=<>!?@#|&%`".contains(c) { c } else { '_' })
        .collect();
    
    // Ensure it's not empty
    if result.is_empty() {
        result = "_".to_string();
    }
    
    result
}

/// Escape string for target language
pub fn escape_string(s: &str, target: &str) -> String {
    match target {
        "typescript" => escape_typescript_string(s),
        "wasm-gc" => escape_wasm_string(s),
        _ => s.to_string(),
    }
}

fn escape_typescript_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn escape_wasm_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::symbol::Symbol;
    
    #[test]
    fn test_sanitize_typescript_identifier() {
        assert_eq!(sanitize_identifier(Symbol::intern("test"), "typescript"), "test");
        assert_eq!(sanitize_identifier(Symbol::intern("class"), "typescript"), "_class");
        assert_eq!(sanitize_identifier(Symbol::intern("123invalid"), "typescript"), "_123invalid");
        assert_eq!(sanitize_identifier(Symbol::intern("my-var"), "typescript"), "my_var");
    }
    
    #[test]
    fn test_sanitize_wasm_identifier() {
        assert_eq!(sanitize_identifier(Symbol::intern("test"), "wasm-gc"), "test");
        assert_eq!(sanitize_identifier(Symbol::intern("my-var"), "wasm-gc"), "my-var");
        assert_eq!(sanitize_identifier(Symbol::intern("func$1"), "wasm-gc"), "func$1");
    }
}