//! Unison-style parser for minimal AST
//! 
//! This parser implements Unison-inspired syntax:
//! - Indentation-based blocks
//! - Pipeline operator |>
//! - Lambda expressions with ->
//! - Type signatures with :
//! - Effect annotations with {}

use crate::minimal_ast::*;
use crate::error::{ParseError, Result};
use std::collections::VecDeque;

/// Token types for Unison-style syntax
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    Int(i64),
    Nat(u64),
    Float(f64),
    Text(String),
    Bool(bool),
    
    // Identifiers and operators
    Symbol(String),
    Operator(String),
    
    // Special tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Equals,
    Semicolon,
    Pipe,
    Arrow,
    Dot,
    Comma,
    
    // Layout tokens
    Indent(usize),
    Newline,
    Eof,
}

/// Lexer for Unison-style syntax
pub struct Lexer {
    input: String,
    chars: Vec<char>,
    position: usize,
    current_indent: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.to_string(),
            chars: input.chars().collect(),
            position: 0,
            current_indent: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut at_line_start = true;
        
        while !self.is_at_end() {
            if at_line_start {
                let indent = self.count_indent();
                if indent != self.current_indent {
                    tokens.push(Token::Indent(indent));
                    self.current_indent = indent;
                }
                at_line_start = false;
            }
            
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }
            
            if self.peek() == Some('\n') {
                self.advance();
                tokens.push(Token::Newline);
                at_line_start = true;
                continue;
            }
            
            if self.peek() == Some('-') && self.peek_next() == Some('-') {
                self.skip_comment();
                continue;
            }
            
            tokens.push(self.next_token()?);
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token> {
        match self.peek() {
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some('.') => {
                self.advance();
                Ok(Token::Dot)
            }
            Some('|') if self.peek_next() == Some('>') => {
                self.advance();
                self.advance();
                Ok(Token::Pipe)
            }
            Some('-') if self.peek_next() == Some('>') => {
                self.advance();
                self.advance();
                Ok(Token::Arrow)
            }
            Some('"') => self.read_string(),
            Some(c) if c.is_ascii_digit() => self.read_number(),
            Some(c) if c.is_alphabetic() || c == '_' => self.read_identifier(),
            Some(c) if is_operator_char(c) => self.read_operator(),
            Some(c) => Err(ParseError::Parse {
                message: format!("Unexpected character: '{}'", c),
            }),
            None => Ok(Token::Eof),
        }
    }
    
    fn read_string(&mut self) -> Result<Token> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token::Text(value));
            } else if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some(c) => value.push(c),
                    None => break,
                }
                self.advance();
            } else {
                value.push(ch);
                self.advance();
            }
        }
        
        Err(ParseError::Parse {
            message: "Unterminated string literal".to_string(),
        })
    }
    
    fn read_number(&mut self) -> Result<Token> {
        let mut num_str = String::new();
        let mut is_float = false;
        
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !is_float && self.peek_next().map_or(false, |ch| ch.is_ascii_digit()) {
                is_float = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            Ok(Token::Float(num_str.parse().map_err(|_| ParseError::Parse {
                message: format!("Invalid float: {}", num_str),
            })?))
        } else {
            // Check for Nat suffix or default to Int
            Ok(Token::Int(num_str.parse().map_err(|_| ParseError::Parse {
                message: format!("Invalid integer: {}", num_str),
            })?))
        }
    }
    
    fn read_identifier(&mut self) -> Result<Token> {
        let mut ident = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '\'' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        match ident.as_str() {
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
            _ => Ok(Token::Symbol(ident)),
        }
    }
    
    fn read_operator(&mut self) -> Result<Token> {
        let mut op = String::new();
        
        while let Some(c) = self.peek() {
            if is_operator_char(c) {
                op.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(Token::Operator(op))
    }
    
    fn count_indent(&mut self) -> usize {
        let mut count = 0;
        while let Some(c) = self.peek() {
            match c {
                ' ' => count += 1,
                '\t' => count += 2, // Treat tab as 2 spaces
                _ => break,
            }
            self.advance();
        }
        count
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            self.advance();
            if c == '\n' {
                break;
            }
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }
    
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }
    
    fn advance(&mut self) {
        if self.position < self.chars.len() {
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.chars.len()
    }
}

fn is_operator_char(c: char) -> bool {
    "+-*/<>=!&|^%~?".contains(c)
}

/// Parser for Unison-style syntax
pub struct Parser {
    tokens: VecDeque<Token>,
    indent_stack: Vec<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into(),
            indent_stack: vec![0],
        }
    }
    
    pub fn parse_module(&mut self) -> Result<Module> {
        let mut items = Vec::new();
        
        // Skip initial newlines
        while matches!(self.peek(), Some(Token::Newline)) {
            self.advance();
        }
        
        while !matches!(self.peek(), Some(Token::Eof) | None) {
            items.push(self.parse_item()?);
            
            // Skip trailing newlines
            while matches!(self.peek(), Some(Token::Newline)) {
                self.advance();
            }
        }
        
        Ok(Module {
            name: "Main".to_string(), // Default module name
            items,
        })
    }
    
    fn parse_item(&mut self) -> Result<Item> {
        // Look for type signature (name : type)
        if self.is_type_signature() {
            let (name, type_sig) = self.parse_type_signature()?;
            
            // Expect the value definition to follow
            self.expect_newline()?;
            let body = self.parse_value_def_body(&name)?;
            
            Ok(Item::ValueDef {
                name,
                type_sig: Some(type_sig),
                body,
            })
        } else {
            // Parse value definition without type signature
            let name = self.expect_symbol()?;
            let body = self.parse_value_def_body(&name)?;
            
            Ok(Item::ValueDef {
                name,
                type_sig: None,
                body,
            })
        }
    }
    
    fn parse_value_def_body(&mut self, _name: &str) -> Result<Expr> {
        // Parse parameters (if any)
        let mut params = Vec::new();
        while let Some(Token::Symbol(_)) = self.peek() {
            params.push(self.parse_pattern()?);
        }
        
        self.expect_token(Token::Equals)?;
        
        // Parse body with increased indentation
        let body = if matches!(self.peek(), Some(Token::Newline)) {
            self.advance();
            self.parse_indented_block()?
        } else {
            self.parse_expression()?
        };
        
        // If there are parameters, wrap in lambda
        if params.is_empty() {
            Ok(body)
        } else {
            Ok(params.into_iter().rev().fold(body, |acc, param| {
                Expr::lambda(param, acc)
            }))
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_pipeline()
    }
    
    fn parse_pipeline(&mut self) -> Result<Expr> {
        let mut expr = self.parse_lambda()?;
        
        while matches!(self.peek(), Some(Token::Pipe)) {
            self.advance();
            let right = self.parse_lambda()?;
            expr = Expr::pipeline(expr, right);
        }
        
        Ok(expr)
    }
    
    fn parse_lambda(&mut self) -> Result<Expr> {
        let expr = self.parse_application()?;
        
        if matches!(self.peek(), Some(Token::Arrow)) {
            self.advance();
            let body = self.parse_expression()?;
            Ok(Expr::lambda(expr, body))
        } else {
            Ok(expr)
        }
    }
    
    fn parse_application(&mut self) -> Result<Expr> {
        let func = self.parse_primary()?;
        
        let mut args = Vec::new();
        while self.is_start_of_primary() {
            args.push(self.parse_primary()?);
        }
        
        if args.is_empty() {
            Ok(func)
        } else {
            Ok(Expr::app(func, args))
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expr> {
        match self.advance() {
            Some(Token::Int(n)) => Ok(Expr::Atom(Atom::Int(n as i32))),
            Some(Token::Nat(n)) => Ok(Expr::Atom(Atom::Int(n as i32))),
            Some(Token::Float(f)) => Ok(Expr::Atom(Atom::Float(f))),
            Some(Token::Text(s)) => Ok(Expr::Atom(Atom::Text(s))),
            Some(Token::Bool(b)) => Ok(Expr::Atom(Atom::Bool(b))),
            Some(Token::Symbol(s)) => Ok(Expr::Atom(Atom::Symbol(s))),
            Some(Token::Operator(op)) => Ok(Expr::Atom(Atom::Operator(op))),
            Some(Token::LeftParen) => {
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::Parse {
                message: "Expected expression".to_string(),
            }),
        }
    }
    
    fn parse_pattern(&mut self) -> Result<Expr> {
        // For now, patterns are just identifiers
        match self.advance() {
            Some(Token::Symbol(s)) => Ok(Expr::Atom(Atom::Symbol(s))),
            _ => Err(ParseError::Parse {
                message: "Expected pattern".to_string(),
            }),
        }
    }
    
    fn parse_type_signature(&mut self) -> Result<(String, Type)> {
        let name = self.expect_symbol()?;
        self.expect_token(Token::Colon)?;
        let typ = self.parse_type()?;
        Ok((name, typ))
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        self.parse_function_type()
    }
    
    fn parse_function_type(&mut self) -> Result<Type> {
        let mut typ = self.parse_effect_type()?;
        
        if matches!(self.peek(), Some(Token::Arrow)) {
            self.advance();
            let ret = self.parse_function_type()?;
            typ = Type::Fun(Box::new(typ), Box::new(ret));
        }
        
        Ok(typ)
    }
    
    fn parse_effect_type(&mut self) -> Result<Type> {
        let typ = self.parse_primary_type()?;
        
        // Check for effect annotation ->{Effect}
        if matches!(self.peek(), Some(Token::Arrow)) {
            if matches!(self.peek_next(), Some(Token::LeftBrace)) {
                self.advance(); // Skip ->
                self.advance(); // Skip {
                
                let mut effects = Vec::new();
                loop {
                    effects.push(self.expect_symbol()?);
                    if matches!(self.peek(), Some(Token::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                self.expect_token(Token::RightBrace)?;
                let ret = self.parse_function_type()?;
                
                return Ok(Type::Effect(Box::new(typ), effects, Box::new(ret)));
            }
        }
        
        Ok(typ)
    }
    
    fn parse_primary_type(&mut self) -> Result<Type> {
        match self.advance() {
            Some(Token::Symbol(s)) => {
                // Check for type application
                let mut args = Vec::new();
                while self.is_start_of_primary_type() {
                    args.push(self.parse_primary_type()?);
                }
                
                if args.is_empty() {
                    // Check if it's a type variable (lowercase) or constructor (uppercase)
                    if s.chars().next().map_or(false, |c| c.is_lowercase()) {
                        Ok(Type::Var(s))
                    } else {
                        Ok(Type::Con(s))
                    }
                } else {
                    Ok(Type::App(Box::new(Type::Con(s)), args))
                }
            }
            Some(Token::LeftParen) => {
                let typ = self.parse_type()?;
                self.expect_token(Token::RightParen)?;
                Ok(typ)
            }
            _ => Err(ParseError::Parse {
                message: "Expected type".to_string(),
            }),
        }
    }
    
    fn parse_indented_block(&mut self) -> Result<Expr> {
        // For now, just parse a single expression
        // TODO: Handle proper indentation-based blocks
        self.parse_expression()
    }
    
    fn is_type_signature(&self) -> bool {
        // Check if we have: symbol : ...
        matches!(
            (self.peek(), self.peek_nth(1)),
            (Some(Token::Symbol(_)), Some(Token::Colon))
        )
    }
    
    fn is_start_of_primary(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Int(_)) | Some(Token::Nat(_)) | Some(Token::Float(_)) |
            Some(Token::Text(_)) | Some(Token::Bool(_)) | Some(Token::Symbol(_)) |
            Some(Token::Operator(_)) | Some(Token::LeftParen)
        )
    }
    
    fn is_start_of_primary_type(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Symbol(_)) | Some(Token::LeftParen)
        )
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }
    
    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(1)
    }
    
    fn peek_nth(&self, n: usize) -> Option<&Token> {
        self.tokens.get(n)
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
    
    fn expect_symbol(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::Symbol(s)) => Ok(s),
            Some(token) => Err(ParseError::Parse {
                message: format!("Expected symbol, found {:?}", token),
            }),
            None => Err(ParseError::Parse {
                message: "Expected symbol, found EOF".to_string(),
            }),
        }
    }
    
    fn expect_newline(&mut self) -> Result<()> {
        match self.peek() {
            Some(Token::Newline) => {
                self.advance();
                Ok(())
            }
            Some(Token::Eof) | None => Ok(()),
            _ => Err(ParseError::Parse {
                message: "Expected newline".to_string(),
            }),
        }
    }
}

/// Parse Unison-style syntax
pub fn parse(input: &str) -> Result<Module> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_value() {
        let input = "x = 42";
        let module = parse(input).unwrap();
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_function_with_type() {
        let input = r#"
add : Int -> Int -> Int
add x y = x + y
"#;
        let module = parse(input).unwrap();
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_pipeline() {
        let input = r#"
result = list |> reverse |> head
"#;
        let module = parse(input).unwrap();
        assert_eq!(module.items.len(), 1);
    }
    
    #[test]
    fn test_lambda() {
        let input = r#"
inc = x -> x + 1
"#;
        let module = parse(input).unwrap();
        assert_eq!(module.items.len(), 1);
    }
}