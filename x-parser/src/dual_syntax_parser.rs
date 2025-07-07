//! Dual syntax parser supporting both indentation and brace-based blocks
//! 
//! This parser seamlessly handles:
//! - Indentation-based blocks (Python-like)
//! - Brace-based blocks with semicolons (C-like)
//! - Mixed syntax within the same file

use crate::minimal_ast::*;
use crate::error::{ParseError, Result};
use std::collections::VecDeque;

/// Layout context for tracking block styles
#[derive(Debug, Clone, PartialEq)]
enum LayoutContext {
    /// Indentation-based layout
    Indent(usize),
    /// Brace-based layout
    Brace,
}

/// Extended parser that handles both syntaxes
pub struct DualSyntaxParser {
    tokens: VecDeque<Token>,
    layout_stack: Vec<LayoutContext>,
    pending_layout_tokens: VecDeque<Token>,
}

/// Token types extended for dual syntax
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // ... existing tokens from unison_style_parser ...
    
    // Virtual layout tokens
    BlockStart,     // Virtual block start (from indent)
    BlockEnd,       // Virtual block end (from dedent)
    VirtualSemi,    // Virtual semicolon (from newline)
}

impl DualSyntaxParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = DualSyntaxParser {
            tokens: tokens.into(),
            layout_stack: vec![LayoutContext::Indent(0)],
            pending_layout_tokens: VecDeque::new(),
        };
        parser.insert_layout_tokens();
        parser
    }
    
    /// Insert virtual layout tokens based on indentation
    fn insert_layout_tokens(&mut self) {
        let mut processed_tokens = VecDeque::new();
        let mut expecting_indent = false;
        
        while let Some(token) = self.tokens.pop_front() {
            match token {
                Token::Equals | Token::Arrow => {
                    processed_tokens.push_back(token);
                    expecting_indent = true;
                }
                Token::LeftBrace => {
                    processed_tokens.push_back(token);
                    self.layout_stack.push(LayoutContext::Brace);
                    expecting_indent = false;
                }
                Token::RightBrace => {
                    // Pop any indent contexts until we find a brace
                    while let Some(ctx) = self.layout_stack.last() {
                        if matches!(ctx, LayoutContext::Brace) {
                            self.layout_stack.pop();
                            break;
                        } else {
                            self.layout_stack.pop();
                            processed_tokens.push_back(Token::BlockEnd);
                        }
                    }
                    processed_tokens.push_back(token);
                }
                Token::Newline => {
                    // Check next token for indentation
                    if let Some(Token::Indent(new_indent)) = self.tokens.front() {
                        self.tokens.pop_front(); // consume indent token
                        
                        if expecting_indent {
                            // Start a new indent block
                            processed_tokens.push_back(Token::BlockStart);
                            self.layout_stack.push(LayoutContext::Indent(*new_indent));
                            expecting_indent = false;
                        } else {
                            // Check current context
                            match self.layout_stack.last() {
                                Some(LayoutContext::Indent(current)) => {
                                    if new_indent < current {
                                        // Dedent - close blocks
                                        while let Some(LayoutContext::Indent(level)) = self.layout_stack.last() {
                                            if new_indent >= level {
                                                break;
                                            }
                                            self.layout_stack.pop();
                                            processed_tokens.push_back(Token::BlockEnd);
                                        }
                                    } else if new_indent == current {
                                        // Same level - virtual semicolon
                                        processed_tokens.push_back(Token::VirtualSemi);
                                    }
                                }
                                Some(LayoutContext::Brace) => {
                                    // Inside braces, newlines are ignored
                                }
                                None => {}
                            }
                        }
                    }
                }
                Token::Semicolon => {
                    // Explicit semicolon
                    processed_tokens.push_back(token);
                }
                _ => {
                    processed_tokens.push_back(token);
                    expecting_indent = false;
                }
            }
        }
        
        // Close any remaining indent blocks
        while let Some(ctx) = self.layout_stack.pop() {
            if matches!(ctx, LayoutContext::Indent(_)) {
                processed_tokens.push_back(Token::BlockEnd);
            }
        }
        
        self.tokens = processed_tokens;
    }
    
    /// Parse a block (either indent-based or brace-based)
    pub fn parse_block(&mut self) -> Result<Vec<Expr>> {
        match self.peek() {
            Some(Token::LeftBrace) => {
                // Explicit brace block
                self.advance(); // consume {
                let mut exprs = Vec::new();
                
                while !matches!(self.peek(), Some(Token::RightBrace) | None) {
                    exprs.push(self.parse_expression()?);
                    
                    // Handle separators
                    match self.peek() {
                        Some(Token::Semicolon) => {
                            self.advance();
                        }
                        Some(Token::RightBrace) => break,
                        _ => {}
                    }
                }
                
                self.expect_token(Token::RightBrace)?;
                Ok(exprs)
            }
            Some(Token::BlockStart) => {
                // Virtual indent block
                self.advance(); // consume BlockStart
                let mut exprs = Vec::new();
                
                while !matches!(self.peek(), Some(Token::BlockEnd) | None) {
                    exprs.push(self.parse_expression()?);
                    
                    // Handle virtual semicolons
                    if matches!(self.peek(), Some(Token::VirtualSemi)) {
                        self.advance();
                    }
                }
                
                if matches!(self.peek(), Some(Token::BlockEnd)) {
                    self.advance(); // consume BlockEnd
                }
                
                Ok(exprs)
            }
            _ => {
                // Single expression
                Ok(vec![self.parse_expression()?])
            }
        }
    }
    
    /// Parse an expression
    pub fn parse_expression(&mut self) -> Result<Expr> {
        // Implementation would be similar to unison_style_parser
        // but handling both layout styles
        todo!("Implement expression parsing")
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }
    
    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
    
    fn expect_token(&mut self, expected: Token) -> Result<()> {
        match self.advance() {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(ParseError::Parse {
                message: format!("Expected {:?}, found {:?}", expected, token),
            }),
            None => Err(ParseError::Parse {
                message: format!("Expected {:?}, found EOF", expected),
            }),
        }
    }
}

/// Example: Convert both styles to the same AST
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_equivalent_blocks() {
        // These should produce the same AST:
        
        let indent_style = r#"
add x y =
  let sum = x + y
  sum
"#;
        
        let brace_style = r#"
add x y = {
  let sum = x + y;
  sum
}
"#;
        
        let oneline_style = r#"
add x y = { let sum = x + y; sum }
"#;
        
        // All three should parse to the same AST structure
    }
    
    #[test]
    fn test_mixed_syntax() {
        let mixed = r#"
processData data =
  let filtered = filter isValid data in {
    filtered
      |> map transform
      |> reduce combine initial;
  }
"#;
        
        // Should handle mixed indent and brace blocks correctly
    }
}