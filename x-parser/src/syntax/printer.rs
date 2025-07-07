//! Common printer utilities and formatting helpers
//! 
//! This module provides shared functionality for all syntax printers.

use super::SyntaxConfig;

/// Pretty-printing utilities
pub struct PrinterUtils;

impl PrinterUtils {
    /// Calculate appropriate line breaks based on content length
    pub fn should_break_line(content: &str, max_line_length: usize) -> bool {
        content.len() > max_line_length
    }
    
    /// Format a list of items with appropriate separators
    pub fn format_list<T, F>(
        items: &[T], 
        formatter: F, 
        separator: &str, 
        config: &SyntaxConfig
    ) -> String 
    where
        F: Fn(&T) -> String,
    {
        let formatted_items: Vec<String> = items.iter().map(formatter).collect();
        
        let total_length: usize = formatted_items.iter().map(|s| s.len()).sum::<usize>() 
            + separator.len() * (items.len().saturating_sub(1));
        
        if total_length > config.max_line_length {
            // Multi-line format
            formatted_items.join(&format!("{separator}\n"))
        } else {
            // Single-line format
            formatted_items.join(separator)
        }
    }
    
    /// Indent all lines in a multi-line string
    pub fn indent_lines(text: &str, indent: &str) -> String {
        text.lines()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("{indent}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}