//! High-performance S-expression parser
//! 
//! Provides zero-copy parsing where possible and efficient error reporting

use crate::sexp::{SExp, Atom};
use crate::error::{SExpError, Result};

/// High-performance S-expression parser
pub struct Parser<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input
    pub fn new(input: &'a str) -> Self {
        let mut parser = Parser {
            input,
            position: 0,
            current_char: None,
        };
        parser.advance();
        parser
    }

    /// Parse a complete S-expression from the input
    pub fn parse(&mut self) -> Result<SExp> {
        self.skip_whitespace();
        let result = self.parse_sexp()?;
        self.skip_whitespace();
        
        if self.current_char.is_some() {
            return Err(SExpError::ParseError {
                pos: self.position,
                message: "Unexpected characters after S-expression".to_string(),
            });
        }
        
        Ok(result)
    }

    /// Parse multiple S-expressions from input
    pub fn parse_multiple(&mut self) -> Result<Vec<SExp>> {
        let mut expressions = Vec::new();
        
        while self.current_char.is_some() {
            self.skip_whitespace();
            if self.current_char.is_none() {
                break;
            }
            expressions.push(self.parse_sexp()?);
        }
        
        Ok(expressions)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.current_char = self.input.chars().nth(self.position);
            if let Some(ch) = self.current_char {
                self.position += ch.len_utf8();
            }
        } else {
            self.current_char = None;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == ';' {
                // Skip line comments
                self.skip_line_comment();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current_char {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn parse_sexp(&mut self) -> Result<SExp> {
        self.skip_whitespace();
        
        match self.current_char {
            Some('(') => self.parse_list(),
            Some('"') => self.parse_string(),
            Some('#') => self.parse_boolean(),
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '+' => {
                self.parse_number_or_symbol()
            }
            Some(_) => self.parse_symbol(),
            None => Err(SExpError::UnexpectedEof),
        }
    }

    fn parse_list(&mut self) -> Result<SExp> {
        // Consume opening parenthesis
        self.advance();
        let mut elements = Vec::new();

        loop {
            self.skip_whitespace();
            
            match self.current_char {
                Some(')') => {
                    self.advance();
                    break;
                }
                Some(_) => {
                    elements.push(self.parse_sexp()?);
                }
                None => {
                    return Err(SExpError::ParseError {
                        pos: self.position,
                        message: "Unterminated list".to_string(),
                    });
                }
            }
        }

        Ok(SExp::List(elements))
    }

    fn parse_string(&mut self) -> Result<SExp> {
        // Consume opening quote
        self.advance();
        let start_pos = self.position;
        let mut string_value = String::new();
        let mut escaped = false;

        while let Some(ch) = self.current_char {
            if escaped {
                match ch {
                    '"' => string_value.push('"'),
                    '\\' => string_value.push('\\'),
                    'n' => string_value.push('\n'),
                    'r' => string_value.push('\r'),
                    't' => string_value.push('\t'),
                    _ => {
                        string_value.push('\\');
                        string_value.push(ch);
                    }
                }
                escaped = false;
                self.advance();
            } else if ch == '\\' {
                escaped = true;
                self.advance();
            } else if ch == '"' {
                self.advance();
                return Ok(SExp::Atom(Atom::String(string_value)));
            } else {
                string_value.push(ch);
                self.advance();
            }
        }

        Err(SExpError::UnterminatedString { pos: start_pos })
    }

    fn parse_boolean(&mut self) -> Result<SExp> {
        // Consume '#'
        self.advance();
        
        match self.current_char {
            Some('t') => {
                self.advance();
                Ok(SExp::Atom(Atom::Boolean(true)))
            }
            Some('f') => {
                self.advance();
                Ok(SExp::Atom(Atom::Boolean(false)))
            }
            Some(ch) => Err(SExpError::InvalidCharacter {
                char: ch,
                pos: self.position,
            }),
            None => Err(SExpError::UnexpectedEof),
        }
    }

    fn parse_number_or_symbol(&mut self) -> Result<SExp> {
        let start_pos = self.position;
        let mut token = String::new();

        // Collect the token
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' {
                break;
            }
            token.push(ch);
            self.advance();
        }

        // Try to parse as number first
        if let Ok(int_val) = token.parse::<i64>() {
            return Ok(SExp::Atom(Atom::Integer(int_val)));
        }

        if let Ok(float_val) = token.parse::<f64>() {
            return Ok(SExp::Atom(Atom::Float(float_val)));
        }

        // If not a number, treat as symbol
        if token.is_empty() {
            return Err(SExpError::ParseError {
                pos: start_pos,
                message: "Empty token".to_string(),
            });
        }

        Ok(SExp::Symbol(token))
    }

    fn parse_symbol(&mut self) -> Result<SExp> {
        let start_pos = self.position;
        let mut symbol = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' {
                break;
            }
            symbol.push(ch);
            self.advance();
        }

        if symbol.is_empty() {
            return Err(SExpError::ParseError {
                pos: start_pos,
                message: "Empty symbol".to_string(),
            });
        }

        Ok(SExp::Symbol(symbol))
    }
}

/// Convenience function for parsing a single S-expression from a string
pub fn parse(input: &str) -> Result<SExp> {
    Parser::new(input).parse()
}

/// Convenience function for parsing multiple S-expressions from a string
pub fn parse_multiple(input: &str) -> Result<Vec<SExp>> {
    Parser::new(input).parse_multiple()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        assert_eq!(
            parse("42").unwrap(),
            SExp::Atom(Atom::Integer(42))
        );
        
        assert_eq!(
            parse("3.14").unwrap(),
            SExp::Atom(Atom::Float(3.14))
        );
        
        assert_eq!(
            parse("\"hello\"").unwrap(),
            SExp::Atom(Atom::String("hello".to_string()))
        );
        
        assert_eq!(
            parse("#t").unwrap(),
            SExp::Atom(Atom::Boolean(true))
        );
        
        assert_eq!(
            parse("#f").unwrap(),
            SExp::Atom(Atom::Boolean(false))
        );
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            parse("hello").unwrap(),
            SExp::Symbol("hello".to_string())
        );
        
        assert_eq!(
            parse("+").unwrap(),
            SExp::Symbol("+".to_string())
        );
    }

    #[test]
    fn test_parse_list() {
        let result = parse("(+ 1 2)").unwrap();
        if let SExp::List(elements) = result {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], SExp::Symbol("+".to_string()));
            assert_eq!(elements[1], SExp::Atom(Atom::Integer(1)));
            assert_eq!(elements[2], SExp::Atom(Atom::Integer(2)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let result = parse("(if (= x 0) 1 (* x (factorial (- x 1))))").unwrap();
        assert!(matches!(result, SExp::List(_)));
    }

    #[test]
    fn test_parse_empty_list() {
        assert_eq!(
            parse("()").unwrap(),
            SExp::List(vec![])
        );
    }

    #[test]
    fn test_parse_with_comments() {
        let result = parse("; This is a comment\n(+ 1 2)").unwrap();
        if let SExp::List(elements) = result {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_error() {
        assert!(parse("(+ 1 2").is_err()); // Unterminated list
        assert!(parse("\"unterminated").is_err()); // Unterminated string
    }

    #[test]
    fn test_parse_multiple() {
        let result = parse_multiple("42 \"hello\" (+ 1 2)").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], SExp::Atom(Atom::Integer(42)));
        assert_eq!(result[1], SExp::Atom(Atom::String("hello".to_string())));
        assert!(matches!(result[2], SExp::List(_)));
    }
}