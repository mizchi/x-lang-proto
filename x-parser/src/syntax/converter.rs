//! Syntax conversion utilities
//! 
//! This module provides utilities for converting between different syntax styles
//! and performing syntax transformations.

use super::{MultiSyntax, SyntaxStyle, SyntaxConfig};
use crate::{ast::*, span::FileId};
use crate::error::{ParseError as Error, Result};

/// Syntax converter that can transform code between different styles
pub struct SyntaxConverter {
    multi_syntax: MultiSyntax,
}

impl SyntaxConverter {
    pub fn new() -> Self {
        SyntaxConverter {
            multi_syntax: MultiSyntax::default(),
        }
    }
    
    /// Convert code from one syntax style to another
    pub fn convert(
        &mut self,
        input: &str,
        from: SyntaxStyle,
        to: SyntaxStyle,
        file_id: FileId,
    ) -> Result<String> {
        self.multi_syntax.convert(input, from, to, file_id)
    }
    
    /// Convert expression from one syntax style to another
    pub fn convert_expression(
        &mut self,
        input: &str,
        from: SyntaxStyle,
        to: SyntaxStyle,
        file_id: FileId,
    ) -> Result<String> {
        // Parse with source syntax
        let expr = self.multi_syntax.parse_expression(input, from, file_id)?;
        
        // Print with target syntax
        let config = SyntaxConfig {
            style: to,
            ..Default::default()
        };
        self.multi_syntax.print_expression(&expr, &config)
    }
    
    /// Batch convert multiple files
    pub fn batch_convert(
        &mut self,
        files: &[(String, String)], // (filename, content) pairs
        from: SyntaxStyle,
        to: SyntaxStyle,
    ) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        
        for (i, (filename, content)) in files.iter().enumerate() {
            let file_id = FileId::new(i as u32);
            let converted = self.convert(content, from, to, file_id)?;
            results.push((filename.clone(), converted));
        }
        
        Ok(results)
    }
    
    /// Validate that a conversion is roundtrip-safe
    pub fn validate_roundtrip(
        &mut self,
        input: &str,
        style1: SyntaxStyle,
        style2: SyntaxStyle,
        file_id: FileId,
    ) -> Result<bool> {
        // Convert from style1 to style2
        let intermediate = self.convert(input, style1, style2, file_id)?;
        
        // Convert back from style2 to style1
        let result = self.convert(&intermediate, style2, style1, file_id)?;
        
        // Parse both original and result to compare ASTs
        let original_ast = self.multi_syntax.parse(input, style1, file_id)?;
        let result_ast = self.multi_syntax.parse(&result, style1, file_id)?;
        
        // Compare ASTs (this would need a proper AST equality implementation)
        Ok(ast_equal(&original_ast, &result_ast))
    }
    
    /// Get conversion statistics
    pub fn conversion_stats(
        &mut self,
        input: &str,
        from: SyntaxStyle,
        to: SyntaxStyle,
        file_id: FileId,
    ) -> Result<ConversionStats> {
        let converted = self.convert(input, from, to, file_id)?;
        
        Ok(ConversionStats {
            original_lines: input.lines().count(),
            converted_lines: converted.lines().count(),
            original_chars: input.chars().count(),
            converted_chars: converted.chars().count(),
            from_style: from,
            to_style: to,
        })
    }
}

/// Statistics about a syntax conversion
#[derive(Debug, Clone)]
pub struct ConversionStats {
    pub original_lines: usize,
    pub converted_lines: usize,
    pub original_chars: usize,
    pub converted_chars: usize,
    pub from_style: SyntaxStyle,
    pub to_style: SyntaxStyle,
}

impl ConversionStats {
    pub fn line_change_ratio(&self) -> f64 {
        if self.original_lines == 0 {
            0.0
        } else {
            self.converted_lines as f64 / self.original_lines as f64
        }
    }
    
    pub fn char_change_ratio(&self) -> f64 {
        if self.original_chars == 0 {
            0.0
        } else {
            self.converted_chars as f64 / self.original_chars as f64
        }
    }
}

/// Compare two ASTs for structural equality (simplified implementation)
fn ast_equal(ast1: &CompilationUnit, ast2: &CompilationUnit) -> bool {
    // This is a simplified implementation
    // A full implementation would need to compare all fields recursively
    // while ignoring spans and other non-semantic differences
    
    module_equal(&ast1.module, &ast2.module)
}

fn module_equal(mod1: &Module, mod2: &Module) -> bool {
    mod1.name.to_string() == mod2.name.to_string() &&
    mod1.imports.len() == mod2.imports.len() &&
    mod1.items.len() == mod2.items.len()
    // Would need more detailed comparison in practice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let converter = SyntaxConverter::new();
        // Just test that it can be created
    }

    #[test]
    fn test_conversion_stats() {
        let stats = ConversionStats {
            original_lines: 10,
            converted_lines: 15,
            original_chars: 200,
            converted_chars: 250,
            from_style: SyntaxStyle::OCaml,
            to_style: SyntaxStyle::Haskell,
        };
        
        assert_eq!(stats.line_change_ratio(), 1.5);
        assert_eq!(stats.char_change_ratio(), 1.25);
    }
}