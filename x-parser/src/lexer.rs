//! Lexer for x Language
//! 
//! Tokenizes source code into a stream of tokens for parsing

use crate::{
    span::{FileId, ByteOffset, Span},
    token::{Token, TokenKind, keyword_to_token},
    error::{ParseError as Error, Result},
};

/// Lexical analyzer
#[allow(dead_code)]
pub struct Lexer {
    input: String,
    chars: Vec<char>,
    position: usize,
    file_id: FileId,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: &str, file_id: FileId) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            input: input.to_string(),
            chars,
            position: 0,
            file_id,
        }
    }
    
    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// Get current character
    fn current_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }
    
    /// Peek at next character
    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }
    
    /// Advance to next character
    fn advance(&mut self) {
        if self.position < self.chars.len() {
            self.position += 1;
        }
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace_and_comments();
        
        let start_pos = self.position;
        
        match self.current_char() {
            None => Ok(Token::new(TokenKind::Eof, self.make_span(start_pos, self.position))),
            
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftParen, self.make_span(start_pos, self.position)))
            }
            Some(')') => {
                self.advance();
                Ok(Token::new(TokenKind::RightParen, self.make_span(start_pos, self.position)))
            }
            Some('[') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftBracket, self.make_span(start_pos, self.position)))
            }
            Some(']') => {
                self.advance();
                Ok(Token::new(TokenKind::RightBracket, self.make_span(start_pos, self.position)))
            }
            Some('{') => {
                self.advance();
                Ok(Token::new(TokenKind::LeftBrace, self.make_span(start_pos, self.position)))
            }
            Some('}') => {
                self.advance();
                Ok(Token::new(TokenKind::RightBrace, self.make_span(start_pos, self.position)))
            }
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenKind::Comma, self.make_span(start_pos, self.position)))
            }
            Some(';') => {
                self.advance();
                Ok(Token::new(TokenKind::Semicolon, self.make_span(start_pos, self.position)))
            }
            Some(':') => {
                self.advance();
                if self.current_char() == Some(':') {
                    self.advance();
                    Ok(Token::new(TokenKind::Cons, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Colon, self.make_span(start_pos, self.position)))
                }
            }
            Some('.') => {
                self.advance();
                Ok(Token::new(TokenKind::Dot, self.make_span(start_pos, self.position)))
            }
            Some('|') => {
                self.advance();
                if self.current_char() == Some('|') {
                    self.advance();
                    Ok(Token::new(TokenKind::OrOr, self.make_span(start_pos, self.position)))
                } else if self.current_char() == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenKind::PipeForward, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Pipe, self.make_span(start_pos, self.position)))
                }
            }
            Some('?') => {
                self.advance();
                Ok(Token::new(TokenKind::Question, self.make_span(start_pos, self.position)))
            }
            
            // String literals
            Some('"') => self.read_string(),
            
            // Numbers
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            
            // Identifiers and keywords
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            
            // Operators
            Some('+') => {
                self.advance();
                Ok(Token::new(TokenKind::Plus, self.make_span(start_pos, self.position)))
            }
            Some('-') => {
                self.advance();
                if self.current_char() == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenKind::Arrow, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Minus, self.make_span(start_pos, self.position)))
                }
            }
            Some('*') => {
                self.advance();
                Ok(Token::new(TokenKind::Star, self.make_span(start_pos, self.position)))
            }
            Some('/') => {
                self.advance();
                Ok(Token::new(TokenKind::Slash, self.make_span(start_pos, self.position)))
            }
            Some('%') => {
                self.advance();
                Ok(Token::new(TokenKind::Percent, self.make_span(start_pos, self.position)))
            }
            Some('=') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenKind::EqualEqual, self.make_span(start_pos, self.position)))
                } else if self.current_char() == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenKind::FatArrow, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Equal, self.make_span(start_pos, self.position)))
                }
            }
            Some('!') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenKind::NotEqual, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Bang, self.make_span(start_pos, self.position)))
                }
            }
            Some('<') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenKind::LessEqual, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Less, self.make_span(start_pos, self.position)))
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenKind::GreaterEqual, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Greater, self.make_span(start_pos, self.position)))
                }
            }
            Some('&') => {
                self.advance();
                if self.current_char() == Some('&') {
                    self.advance();
                    Ok(Token::new(TokenKind::AndAnd, self.make_span(start_pos, self.position)))
                } else {
                    Ok(Token::new(TokenKind::Ampersand, self.make_span(start_pos, self.position)))
                }
            }
            Some('^') => {
                self.advance();
                Ok(Token::new(TokenKind::Caret, self.make_span(start_pos, self.position)))
            }
            
            // Backtick for doc comments
            Some('`') => {
                if self.peek_ahead(1) == Some('`') && self.peek_ahead(2) == Some('`') {
                    // Triple backtick at start of line = doc comment
                    if self.position == 0 || self.chars.get(self.position.saturating_sub(1)) == Some(&'\n') {
                        self.read_doc_comment()
                    } else {
                        Err(Error::Parse { 
                            message: "Triple backticks only allowed at start of line for doc comments".to_string() 
                        })
                    }
                } else {
                    Err(Error::Parse { 
                        message: "Unexpected character: '`'".to_string() 
                    })
                }
            }
            
            Some(ch) => {
                Err(Error::Parse { 
                    message: format!("Unexpected character: '{}'", ch) 
                })
            }
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current_char() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('-') if self.peek_char() == Some('-') => {
                    self.skip_line_comment();
                }
                _ => break,
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        // Skip '--'
        self.advance();
        self.advance();
        
        // Skip until end of line
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn read_string(&mut self) -> Result<Token> {
        let start_pos = self.position;
        self.advance(); // Skip opening quote
        
        let mut value = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(Token::new(
                    TokenKind::String(value),
                    self.make_span(start_pos, self.position)
                ));
            } else if ch == '\\' {
                self.advance();
                match self.current_char() {
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
        
        Err(Error::Parse { 
            message: "Unterminated string literal".to_string() 
        })
    }
    
    fn read_number(&mut self) -> Result<Token> {
        let start_pos = self.position;
        let mut value = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(Token::new(
            TokenKind::Number(value),
            self.make_span(start_pos, self.position)
        ))
    }
    
    fn read_identifier(&mut self) -> Result<Token> {
        let start_pos = self.position;
        let mut value = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '\'' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let span = self.make_span(start_pos, self.position);
        
        // Check if it's a keyword
        if let Some(keyword_token) = keyword_to_token(&value) {
            Ok(Token::new(keyword_token, span))
        } else {
            Ok(Token::new(TokenKind::Ident(value), span))
        }
    }
    
    fn make_span(&self, start: usize, end: usize) -> Span {
        Span::new(
            self.file_id,
            ByteOffset::new(start as u32),
            ByteOffset::new(end as u32),
        )
    }
    
    /// Peek ahead n characters without advancing
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.position + n).copied()
    }
    
    /// Read a documentation comment
    fn read_doc_comment(&mut self) -> Result<Token> {
        let start_pos = self.position;
        
        // Skip the opening ```
        self.advance(); // `
        self.advance(); // `
        self.advance(); // `
        
        let mut content = String::new();
        
        // Read until closing ```
        while self.position + 2 < self.chars.len() {
            if self.current_char() == Some('`') 
                && self.peek_char() == Some('`') 
                && self.peek_ahead(2) == Some('`') {
                // Found closing ```
                self.advance(); // `
                self.advance(); // `
                self.advance(); // `
                
                return Ok(Token::new(
                    TokenKind::DocComment(content.trim().to_string()),
                    self.make_span(start_pos, self.position)
                ));
            }
            
            if let Some(ch) = self.current_char() {
                content.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Err(Error::Parse {
            message: "Unterminated documentation comment".to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn lex_string(input: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(input, FileId::new(0));
        let tokens = lexer.tokenize().expect("Lexing should succeed");
        tokens.into_iter().map(|t| t.kind).collect()
    }
    
    #[test]
    fn test_simple_tokens() {
        let tokens = lex_string("( ) + -");
        assert_eq!(tokens, vec![
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Eof,
        ]);
    }
    
    #[test]
    fn test_identifier() {
        let tokens = lex_string("hello world");
        assert_eq!(tokens, vec![
            TokenKind::Ident("hello".to_string()),
            TokenKind::Ident("world".to_string()),
            TokenKind::Eof,
        ]);
    }
    
    #[test]
    fn test_keyword() {
        let tokens = lex_string("let module");
        assert_eq!(tokens, vec![
            TokenKind::Let,
            TokenKind::Module,
            TokenKind::Eof,
        ]);
    }
    
    #[test]
    fn test_pipeline_operator() {
        let tokens = lex_string("x |> f |> g");
        assert_eq!(tokens, vec![
            TokenKind::Ident("x".to_string()),
            TokenKind::PipeForward,
            TokenKind::Ident("f".to_string()),
            TokenKind::PipeForward,
            TokenKind::Ident("g".to_string()),
            TokenKind::Eof,
        ]);
    }
    
    #[test]
    fn test_pipe_operators() {
        let tokens = lex_string("| || |>");
        assert_eq!(tokens, vec![
            TokenKind::Pipe,
            TokenKind::OrOr,
            TokenKind::PipeForward,
            TokenKind::Eof,
        ]);
    }
}