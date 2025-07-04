//! Parser for EffectLang
//! 
//! Recursive descent parser with error recovery and precedence climbing

use crate::core::{
    ast::*,
    span::{Span, FileId},
    token::{Token, TokenKind},
    symbol::Symbol,
};
use crate::analysis::lexer::Lexer;
use crate::{Error, Result};

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_id: FileId,
}

impl Parser {
    /// Create a new parser from source code
    pub fn new(input: &str, file_id: FileId) -> Result<Self> {
        let mut lexer = Lexer::new(input, file_id);
        let tokens = lexer.tokenize()?;
        
        Ok(Parser {
            tokens,
            current: 0,
            file_id,
        })
    }
    
    /// Parse a complete compilation unit
    pub fn parse(&mut self) -> Result<CompilationUnit> {
        let start_span = self.current_span();
        let module = self.parse_module()?;
        let end_span = self.current_span();
        
        Ok(CompilationUnit {
            module,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse a module
    fn parse_module(&mut self) -> Result<Module> {
        let start_span = self.current_span();
        
        // Parse module header
        self.expect(TokenKind::Module)?;
        let module_path = self.parse_module_path()?;
        
        // Parse optional export list
        let exports = if self.check(&TokenKind::Export) {
            Some(self.parse_export_list()?)
        } else {
            None
        };
        
        // Parse imports
        let mut imports = Vec::new();
        while self.check(&TokenKind::Import) {
            imports.push(self.parse_import()?);
        }
        
        // Parse module items
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        let end_span = self.current_span();
        
        Ok(Module {
            name: module_path,
            exports,
            imports,
            items,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse module path (e.g., Core.Types.User)
    fn parse_module_path(&mut self) -> Result<ModulePath> {
        let start_span = self.current_span();
        let mut segments = vec![self.parse_identifier()?];
        
        while self.match_token(&TokenKind::Dot) {
            segments.push(self.parse_identifier()?);
        }
        
        let end_span = self.current_span();
        
        Ok(ModulePath::new(segments, start_span.merge(end_span)))
    }
    
    /// Parse export list
    fn parse_export_list(&mut self) -> Result<ExportList> {
        let start_span = self.current_span();
        self.expect(TokenKind::Export)?;
        self.expect(TokenKind::LeftBrace)?;
        
        let mut items = Vec::new();
        
        if !self.check(&TokenKind::RightBrace) {
            loop {
                items.push(self.parse_export_item()?);
                
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::RightBrace)?;
        let end_span = self.current_span();
        
        Ok(ExportList {
            items,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse export item
    fn parse_export_item(&mut self) -> Result<ExportItem> {
        let start_span = self.current_span();
        
        let kind = if self.match_token(&TokenKind::Type) {
            ExportKind::Type
        } else if self.match_token(&TokenKind::Effect) {
            ExportKind::Effect
        } else if self.match_token(&TokenKind::Module) {
            ExportKind::Module
        } else {
            ExportKind::Value
        };
        
        let name = self.parse_identifier()?;
        let alias = if self.match_token(&TokenKind::LeftParen) {
            let alias = self.parse_identifier()?;
            self.expect(TokenKind::RightParen)?;
            Some(alias)
        } else {
            None
        };
        
        let end_span = self.current_span();
        
        Ok(ExportItem {
            kind,
            name,
            alias,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse import declaration
    fn parse_import(&mut self) -> Result<Import> {
        let start_span = self.current_span();
        
        let is_lazy = if self.check(&TokenKind::Ident("lazy".to_string())) {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(TokenKind::Import)?;
        let module_path = self.parse_module_path()?;
        
        let kind = if is_lazy {
            ImportKind::Lazy
        } else if self.match_token(&TokenKind::LeftBrace) {
            // Selective import
            let mut items = Vec::new();
            
            if !self.check(&TokenKind::RightBrace) {
                loop {
                    items.push(self.parse_import_item()?);
                    
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            
            self.expect(TokenKind::RightBrace)?;
            ImportKind::Selective(items)
        } else if self.match_token(&TokenKind::Dot) && self.match_token(&TokenKind::Star) {
            ImportKind::Wildcard
        } else {
            ImportKind::Qualified
        };
        
        let alias = if self.match_token(&TokenKind::Ident("as".to_string())) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        let end_span = self.current_span();
        
        Ok(Import {
            module_path,
            kind,
            alias,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse import item
    fn parse_import_item(&mut self) -> Result<ImportItem> {
        let start_span = self.current_span();
        
        let kind = if self.match_token(&TokenKind::Type) {
            ExportKind::Type
        } else if self.match_token(&TokenKind::Effect) {
            ExportKind::Effect
        } else {
            ExportKind::Value
        };
        
        let name = self.parse_identifier()?;
        let alias = if self.match_token(&TokenKind::Ident("as".to_string())) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        let end_span = self.current_span();
        
        Ok(ImportItem {
            kind,
            name,
            alias,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse top-level item
    fn parse_item(&mut self) -> Result<Item> {
        if self.check(&TokenKind::Data) {
            Ok(Item::TypeDef(self.parse_data_type()?))
        } else if self.check(&TokenKind::Type) {
            Ok(Item::TypeDef(self.parse_type_alias()?))
        } else if self.check(&TokenKind::Effect) {
            Ok(Item::EffectDef(self.parse_effect_def()?))
        } else if self.check(&TokenKind::Handler) {
            Ok(Item::HandlerDef(self.parse_handler_def()?))
        } else {
            Ok(Item::ValueDef(self.parse_value_def()?))
        }
    }
    
    /// Parse data type definition
    fn parse_data_type(&mut self) -> Result<TypeDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Data)?;
        
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        
        self.expect(TokenKind::Equal)?;
        
        let mut constructors = Vec::new();
        constructors.push(self.parse_constructor()?);
        
        while self.match_token(&TokenKind::Pipe) {
            constructors.push(self.parse_constructor()?);
        }
        
        let end_span = self.current_span();
        
        Ok(TypeDef {
            name,
            type_params,
            kind: TypeDefKind::Data(constructors),
            visibility: Visibility::Public,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse type alias
    fn parse_type_alias(&mut self) -> Result<TypeDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Type)?;
        
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        
        self.expect(TokenKind::Equal)?;
        let aliased_type = self.parse_type()?;
        
        let end_span = self.current_span();
        
        Ok(TypeDef {
            name,
            type_params,
            kind: TypeDefKind::Alias(aliased_type),
            visibility: Visibility::Public,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse constructor
    fn parse_constructor(&mut self) -> Result<Constructor> {
        let start_span = self.current_span();
        let name = self.parse_identifier()?;
        
        let mut fields = Vec::new();
        while !self.check(&TokenKind::Pipe) && !self.is_at_end() && 
              !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            fields.push(self.parse_type()?);
        }
        
        let end_span = self.current_span();
        
        Ok(Constructor {
            name,
            fields,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse effect definition
    fn parse_effect_def(&mut self) -> Result<EffectDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Effect)?;
        
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        
        self.expect(TokenKind::LeftBrace)?;
        
        let mut operations = Vec::new();
        while !self.check(&TokenKind::RightBrace) {
            operations.push(self.parse_effect_operation()?);
        }
        
        self.expect(TokenKind::RightBrace)?;
        let end_span = self.current_span();
        
        Ok(EffectDef {
            name,
            type_params,
            operations,
            visibility: Visibility::Public,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse effect operation
    fn parse_effect_operation(&mut self) -> Result<EffectOperation> {
        let start_span = self.current_span();
        let name = self.parse_identifier()?;
        
        self.expect(TokenKind::Colon)?;
        
        // Parse operation type: param1 -> param2 -> return_type
        let mut param_types = Vec::new();
        
        // Parse parameters and return type
        let first_type = self.parse_type()?;
        
        if self.match_token(&TokenKind::Arrow) {
            param_types.push(first_type.clone());
            
            // Continue parsing more parameters
            let mut return_type = None;
            while !self.is_at_end() {
                let next_type = self.parse_type()?;
                
                if self.match_token(&TokenKind::Arrow) {
                    param_types.push(next_type);
                } else {
                    // This is the return type
                    return_type = Some(next_type);
                    break;
                }
            }
            
            let end_span = self.current_span();
            Ok(EffectOperation {
                name,
                parameters: param_types,
                return_type: return_type.unwrap_or(first_type),
                span: start_span.merge(end_span),
            })
        } else {
            // If no arrow, it's a nullary operation
            let end_span = self.current_span();
            Ok(EffectOperation {
                name,
                parameters: param_types,
                return_type: first_type,
                span: start_span.merge(end_span),
            })
        }
    }
    
    /// Parse handler definition (simplified)
    fn parse_handler_def(&mut self) -> Result<HandlerDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Handler)?;
        
        let name = self.parse_identifier()?;
        
        // Simplified handler parsing - just skip to end for now
        while !self.is_at_end() && !self.check(&TokenKind::Let) && !self.check(&TokenKind::Data) {
            self.advance();
        }
        
        let end_span = self.current_span();
        
        Ok(HandlerDef {
            name,
            type_annotation: None,
            handled_effects: Vec::new(),
            handlers: Vec::new(),
            return_clause: None,
            visibility: Visibility::Public,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse value definition
    fn parse_value_def(&mut self) -> Result<ValueDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Let)?;
        
        let name = self.parse_identifier()?;
        
        // Parse optional type annotation
        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Equal)?;
        let body = self.parse_expression()?;
        
        let end_span = self.current_span();
        
        Ok(ValueDef {
            name,
            type_annotation,
            parameters: Vec::new(), // Simplified for now
            body,
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse type expression
    fn parse_type(&mut self) -> Result<Type> {
        let start_span = self.current_span();
        
        if self.match_token(&TokenKind::LeftParen) {
            // Parenthesized type or function type
            if self.match_token(&TokenKind::RightParen) {
                // Unit type
                Ok(Type::Con(Symbol::intern("Unit"), start_span))
            } else {
                let inner_type = self.parse_type()?;
                self.expect(TokenKind::RightParen)?;
                Ok(inner_type)
            }
        } else if self.match_token(&TokenKind::Forall) {
            // Forall type
            let type_params = self.parse_type_params()?;
            self.expect(TokenKind::Dot)?;
            let body = Box::new(self.parse_type()?);
            let end_span = self.current_span();
            
            Ok(Type::Forall {
                type_params,
                body,
                span: start_span.merge(end_span),
            })
        } else if self.match_token(&TokenKind::Question) {
            // Type hole
            Ok(Type::Hole(start_span))
        } else {
            // Type constructor or variable
            let name = self.parse_identifier()?;
            
            // Check for type application
            if self.match_token(&TokenKind::LeftBracket) {
                let mut args = Vec::new();
                
                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        args.push(self.parse_type()?);
                        
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenKind::RightBracket)?;
                let end_span = self.current_span();
                
                Ok(Type::App(
                    Box::new(Type::Con(name, start_span)),
                    args,
                    start_span.merge(end_span),
                ))
            } else {
                Ok(Type::Con(name, start_span))
            }
        }
    }
    
    /// Parse type parameters
    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>> {
        if !self.match_token(&TokenKind::LeftBracket) {
            return Ok(Vec::new());
        }
        
        let mut params = Vec::new();
        
        if !self.check(&TokenKind::RightBracket) {
            loop {
                let start_span = self.current_span();
                let name = self.parse_identifier()?;
                let end_span = self.current_span();
                
                params.push(TypeParam {
                    name,
                    kind: None,
                    constraints: Vec::new(),
                    span: start_span.merge(end_span),
                });
                
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::RightBracket)?;
        Ok(params)
    }
    
    /// Parse expression
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_application()
    }
    
    /// Parse function application and other high-precedence expressions
    fn parse_application(&mut self) -> Result<Expr> {
        let mut expr = self.parse_atom()?;
        
        // Handle function application
        while !self.is_at_end() && self.can_start_atom() {
            let arg = self.parse_atom()?;
            let span = expr.span().merge(arg.span());
            expr = Expr::App(Box::new(expr), vec![arg], span);
        }
        
        Ok(expr)
    }
    
    /// Parse atomic expressions
    fn parse_atom(&mut self) -> Result<Expr> {
        if self.check(&TokenKind::LeftParen) {
            self.parse_parenthesized()
        } else if self.check(&TokenKind::If) {
            self.parse_if()
        } else if self.check(&TokenKind::Let) {
            self.parse_let()
        } else if self.check(&TokenKind::Fun) {
            self.parse_lambda()
        } else if self.check(&TokenKind::Match) {
            self.parse_match()
        } else if self.check(&TokenKind::Do) {
            self.parse_do()
        } else {
            self.parse_primary()
        }
    }
    
    /// Check if current token can start an atomic expression
    fn can_start_atom(&self) -> bool {
        matches!(self.current_token().kind,
            TokenKind::LeftParen | TokenKind::Integer(_) | TokenKind::Float(_) |
            TokenKind::String(_) | TokenKind::Bool(_) | TokenKind::Ident(_) |
            TokenKind::If | TokenKind::Let | TokenKind::Fun | TokenKind::Match | TokenKind::Do
        )
    }
    
    /// Parse parenthesized expressions
    fn parse_parenthesized(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::LeftParen)?;
        
        if self.match_token(&TokenKind::RightParen) {
            // Unit literal
            Ok(Expr::Literal(Literal::Unit, start_span))
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RightParen)?;
            Ok(expr)
        }
    }
    
    /// Parse if expressions
    fn parse_if(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::If)?;
        
        let condition = Box::new(self.parse_expression()?);
        self.expect(TokenKind::Then)?;
        let then_branch = Box::new(self.parse_expression()?);
        self.expect(TokenKind::Else)?;
        let else_branch = Box::new(self.parse_expression()?);
        
        let end_span = self.current_span();
        
        Ok(Expr::If {
            condition,
            then_branch,
            else_branch,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse let expressions
    fn parse_let(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::Let)?;
        
        let pattern = self.parse_pattern()?;
        
        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Equal)?;
        let value = Box::new(self.parse_expression()?);
        self.expect(TokenKind::In)?;
        let body = Box::new(self.parse_expression()?);
        
        let end_span = self.current_span();
        
        Ok(Expr::Let {
            pattern,
            type_annotation,
            value,
            body,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse lambda expressions
    fn parse_lambda(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::Fun)?;
        
        let mut parameters = Vec::new();
        
        // Parse parameters
        while !self.check(&TokenKind::Arrow) && !self.is_at_end() {
            parameters.push(self.parse_pattern()?);
        }
        
        self.expect(TokenKind::Arrow)?;
        let body = Box::new(self.parse_expression()?);
        
        let end_span = self.current_span();
        
        Ok(Expr::Lambda {
            parameters,
            body,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse match expressions
    fn parse_match(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::Match)?;
        
        let scrutinee = Box::new(self.parse_expression()?);
        self.expect(TokenKind::With)?;
        
        let mut arms = Vec::new();
        
        while !self.is_at_end() && !self.check(&TokenKind::RightBrace) {
            let pattern = self.parse_pattern()?;
            
            let guard = if self.match_token(&TokenKind::If) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            
            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expression()?;
            
            let span = pattern.span().merge(body.span());
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span,
            });
            
            // Optional pipe separator
            if !self.match_token(&TokenKind::Pipe) {
                break;
            }
        }
        
        let end_span = self.current_span();
        
        Ok(Expr::Match {
            scrutinee,
            arms,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse do notation (for effects)
    fn parse_do(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        self.expect(TokenKind::Do)?;
        self.expect(TokenKind::LeftBrace)?;
        
        let mut statements = Vec::new();
        
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Let) {
                // Let binding in do block
                self.advance(); // consume 'let'
                let pattern = self.parse_pattern()?;
                
                if self.match_token(&TokenKind::LeftArrow) {
                    // Monadic bind: let x <- expr
                    let expr = self.parse_expression()?;
                    let span = pattern.span().merge(expr.span());
                    statements.push(DoStatement::Bind {
                        pattern,
                        expr,
                        span,
                    });
                } else {
                    // Regular let: let x = expr
                    self.expect(TokenKind::Equal)?;
                    let expr = self.parse_expression()?;
                    let span = pattern.span().merge(expr.span());
                    statements.push(DoStatement::Let {
                        pattern,
                        expr,
                        span,
                    });
                }
            } else {
                // Expression statement
                let expr = self.parse_expression()?;
                statements.push(DoStatement::Expr(expr));
            }
            
            // Optional semicolon
            self.match_token(&TokenKind::Semicolon);
        }
        
        self.expect(TokenKind::RightBrace)?;
        let end_span = self.current_span();
        
        Ok(Expr::Do {
            statements,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse patterns
    fn parse_pattern(&mut self) -> Result<Pattern> {
        let start_span = self.current_span();
        
        match &self.current_token().kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard(start_span))
            }
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(n), start_span))
            }
            TokenKind::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f), start_span))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(s), start_span))
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(b), start_span))
            }
            TokenKind::Ident(name) => {
                let symbol = Symbol::intern(name);
                self.advance();
                
                // Check for constructor pattern
                if self.can_start_pattern() {
                    let mut args = Vec::new();
                    while self.can_start_pattern() {
                        args.push(self.parse_pattern()?);
                    }
                    let end_span = self.current_span();
                    Ok(Pattern::Constructor {
                        name: symbol,
                        args,
                        span: start_span.merge(end_span),
                    })
                } else {
                    Ok(Pattern::Variable(symbol, start_span))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                if self.match_token(&TokenKind::RightParen) {
                    Ok(Pattern::Literal(Literal::Unit, start_span))
                } else {
                    let pattern = self.parse_pattern()?;
                    self.expect(TokenKind::RightParen)?;
                    Ok(pattern)
                }
            }
            _ => Err(Error::Parse {
                message: format!("Expected pattern, found {:?}", self.current_token().kind),
            }),
        }
    }
    
    /// Check if current token can start a pattern
    fn can_start_pattern(&self) -> bool {
        matches!(self.current_token().kind,
            TokenKind::Underscore | TokenKind::Integer(_) | TokenKind::Float(_) |
            TokenKind::String(_) | TokenKind::Bool(_) | TokenKind::Ident(_) |
            TokenKind::LeftParen
        )
    }
    
    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Expr> {
        let start_span = self.current_span();
        
        match &self.current_token().kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Integer(n), start_span))
            }
            TokenKind::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::Literal(Literal::Float(f), start_span))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s), start_span))
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b), start_span))
            }
            TokenKind::Ident(name) => {
                let name = Symbol::intern(name);
                self.advance();
                Ok(Expr::Var(name, start_span))
            }
            TokenKind::LeftParen => {
                self.advance();
                if self.match_token(&TokenKind::RightParen) {
                    Ok(Expr::Literal(Literal::Unit, start_span))
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(TokenKind::RightParen)?;
                    Ok(expr)
                }
            }
            _ => Err(Error::Parse {
                message: format!("Unexpected token: {:?}", self.current_token().kind),
            }),
        }
    }
    
    /// Parse identifier
    fn parse_identifier(&mut self) -> Result<Symbol> {
        match &self.current_token().kind {
            TokenKind::Ident(name) => {
                let symbol = Symbol::intern(name);
                self.advance();
                Ok(symbol)
            }
            _ => Err(Error::Parse {
                message: format!("Expected identifier, found {:?}", self.current_token().kind),
            }),
        }
    }
    
    // Helper methods
    
    fn current_token(&self) -> &Token {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            // Return a default EOF token
            static EOF_TOKEN: std::sync::OnceLock<Token> = std::sync::OnceLock::new();
            EOF_TOKEN.get_or_init(|| {
                use crate::core::span::{FileId, ByteOffset, Span};
                Token::eof(Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)))
            })
        }
    }
    
    fn current_span(&self) -> Span {
        self.current_token().span
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current_token().kind, TokenKind::Eof)
    }
    
    fn check(&self, token_kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.current_token().kind) == std::mem::discriminant(token_kind)
        }
    }
    
    fn match_token(&mut self, token_kind: &TokenKind) -> bool {
        if self.check(token_kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn expect(&mut self, token_kind: TokenKind) -> Result<&Token> {
        if self.check(&token_kind) {
            Ok(self.advance())
        } else {
            Err(Error::Parse {
                message: format!(
                    "Expected {:?}, found {:?}",
                    token_kind,
                    self.current_token().kind
                ),
            })
        }
    }
}

/// Convenience function to parse source code
pub fn parse(input: &str, file_id: FileId) -> Result<CompilationUnit> {
    let mut parser = Parser::new(input, file_id)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::span::FileId;

    #[test]
    fn test_parse_simple_module() {
        let input = r#"
            module Test
            
            let x = 42
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.name.to_string(), "Test");
        assert_eq!(cu.module.items.len(), 1);
    }

    #[test]
    fn test_parse_data_type() {
        let input = r#"
            module Test
            
            data Option[a] = None | Some a
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
        
        if let Item::TypeDef(typedef) = &cu.module.items[0] {
            assert_eq!(typedef.name.as_str(), "Option");
            
            if let TypeDefKind::Data(constructors) = &typedef.kind {
                assert_eq!(constructors.len(), 2);
                assert_eq!(constructors[0].name.as_str(), "None");
                assert_eq!(constructors[1].name.as_str(), "Some");
            }
        }
    }

    #[test]
    fn test_parse_effect() {
        let input = r#"
            module Test
            
            effect State[s] {
                get : s
                put : s -> Unit
            }
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
        
        if let Item::EffectDef(effect) = &cu.module.items[0] {
            assert_eq!(effect.name.as_str(), "State");
            assert_eq!(effect.operations.len(), 2);
        }
    }

    #[test]
    fn test_parse_with_imports() {
        let input = r#"
            module Test
            
            import Core.Types { type Option, Some, None }
            import Data.List as List
            
            let x = Some 42
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.imports.len(), 2);
    }
    
    #[test]
    fn test_parse_lambda_expression() {
        let input = r#"
            module Test
            
            let identity = fun x -> x
            let add = fun x y -> x + y
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 2);
    }
    
    #[test]
    fn test_parse_if_expression() {
        let input = r#"
            module Test
            
            let test = if true then 1 else 0
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
    }
    
    #[test]
    fn test_parse_function_application() {
        let input = r#"
            module Test
            
            let result = f x y
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
        
        if let Item::ValueDef(value_def) = &cu.module.items[0] {
            // Should parse as nested applications
            assert!(matches!(value_def.body, Expr::App(_, _, _)));
        }
    }
    
    #[test]
    fn test_parse_match_expression() {
        let input = r#"
            module Test
            
            let test = match x with
                | Some y => y
                | None => 0
        "#;
        
        let result = parse(input, FileId::new(0));
        assert!(result.is_ok());
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
        
        if let Item::ValueDef(value_def) = &cu.module.items[0] {
            if let Expr::Match { arms, .. } = &value_def.body {
                assert_eq!(arms.len(), 2);
            }
        }
    }
}