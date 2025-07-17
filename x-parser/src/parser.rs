//! Parser for x Language
//! 
//! Recursive descent parser with error recovery and precedence climbing

use crate::{
    ast::*,
    span::{Span, FileId},
    token::{Token, TokenKind},
    symbol::Symbol,
    lexer::Lexer,
    error::{ParseError as Error, Result},
};

/// Parser state
#[allow(dead_code)]
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
    
    /// Parse a single expression (public for testing)
    pub fn parse_expression_public(&mut self) -> Result<Expr> {
        self.parse_expression()
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
            // Skip standalone doc comments at module level
            if matches!(self.current_token().kind, TokenKind::DocComment(_)) {
                self.advance();
                continue;
            }
            
            items.push(self.parse_item()?);
        }
        
        let end_span = self.current_span();
        
        Ok(Module {
            name: module_path,
            documentation: None, // TODO: Parse module doc comments
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
        
        // Parse optional version specification (e.g., @^1.0.0)
        let version_spec = if self.match_token(&TokenKind::At) {
            if let TokenKind::Ident(version) = self.current() {
                let version_str = version.clone();
                self.advance();
                Some(version_str)
            } else if let TokenKind::String(version) = self.current() {
                let version_str = version.clone();
                self.advance();
                Some(version_str)
            } else {
                return self.error("Expected version specification after '@'");
            }
        } else {
            None
        };
        
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
        
        let alias = if self.match_ident("as") {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        let end_span = self.current_span();
        
        Ok(Import {
            module_path,
            kind,
            alias,
            version_spec,
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
        
        // Parse optional version specification (e.g., @^1.0.0)
        let version_spec = if self.match_token(&TokenKind::At) {
            if let TokenKind::Ident(version) = self.current() {
                let version_str = version.clone();
                self.advance();
                Some(version_str)
            } else if let TokenKind::String(version) = self.current() {
                let version_str = version.clone();
                self.advance();
                Some(version_str)
            } else {
                return self.error("Expected version specification after '@'");
            }
        } else {
            None
        };
        
        let alias = if self.match_ident("as") {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        let end_span = self.current_span();
        
        Ok(ImportItem {
            kind,
            name,
            alias,
            version_spec,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse top-level item
    fn parse_item(&mut self) -> Result<Item> {
        // Parse visibility modifier first
        let visibility = self.parse_visibility()?;
        
        // Check for module shorthand syntax (e.g., Math.pi : Float)
        if visibility == Visibility::Private && self.check(&TokenKind::Ident(String::new())) {
            // Look ahead to see if this is a module shorthand
            let saved_pos = self.current;
            if let Ok(_ident) = self.parse_identifier() {
                if self.match_token(&TokenKind::Dot) {
                    // This is module shorthand syntax
                    self.current = saved_pos; // Reset position
                    return self.parse_module_shorthand_item(visibility).map(Item::ValueDef);
                } else {
                    // Not module shorthand, reset and continue
                    self.current = saved_pos;
                }
            } else {
                self.current = saved_pos;
            }
        }
        
        if self.check(&TokenKind::Test) {
            Ok(Item::TestDef(self.parse_test_def_with_visibility(visibility)?))
        } else if self.check(&TokenKind::Interface) {
            Ok(Item::InterfaceDef(self.parse_interface_def(visibility)?))
        } else if self.check(&TokenKind::Data) {
            Ok(Item::TypeDef(self.parse_data_type_with_visibility(visibility)?))
        } else if self.check(&TokenKind::Type) {
            Ok(Item::TypeDef(self.parse_type_alias_with_visibility(visibility)?))
        } else if self.check(&TokenKind::Effect) {
            Ok(Item::EffectDef(self.parse_effect_def_with_visibility(visibility)?))
        } else if self.check(&TokenKind::Handler) {
            Ok(Item::HandlerDef(self.parse_handler_def_with_visibility(visibility)?))
        } else if self.check(&TokenKind::Let) {
            Ok(Item::ValueDef(self.parse_value_def_with_visibility(visibility)?))
        } else {
            // Try to parse as module shorthand if no 'let' keyword
            self.parse_module_shorthand_item(visibility).map(Item::ValueDef)
        }
    }
    
    /// Parse visibility modifier
    fn parse_visibility(&mut self) -> Result<Visibility> {
        if !self.check(&TokenKind::Pub) {
            return Ok(Visibility::Private); // Default is private
        }
        
        self.advance(); // consume 'pub'
        
        // Check for pub(...)
        if self.match_token(&TokenKind::LeftParen) {
            let visibility = if self.match_token(&TokenKind::Crate) {
                Visibility::Crate
            } else if self.match_token(&TokenKind::Package) {
                Visibility::Package
            } else if self.match_token(&TokenKind::Super) {
                Visibility::Super
            } else if self.match_token(&TokenKind::Self_) {
                Visibility::SelfModule
            } else if self.check(&TokenKind::Ident(String::new())) {
                // pub(in path)
                self.expect(TokenKind::Ident("in".to_string()))?;
                let path = self.parse_module_path()?;
                Visibility::InPath(path)
            } else {
                return Err(Error::Parse {
                    message: "Expected crate, package, super, self, or 'in path' after pub(".to_string(),
                });
            };
            
            self.expect(TokenKind::RightParen)?;
            Ok(visibility)
        } else {
            Ok(Visibility::Public) // Just 'pub'
        }
    }
    
    /// Parse data type definition with visibility
    fn parse_data_type_with_visibility(&mut self, visibility: Visibility) -> Result<TypeDef> {
        let documentation = self.collect_doc_comments();
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
            documentation,
            type_params,
            kind: TypeDefKind::Data(constructors),
            visibility,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse data type definition (backward compatibility)
    #[allow(dead_code)]
    fn parse_data_type(&mut self) -> Result<TypeDef> {
        self.parse_data_type_with_visibility(Visibility::Private)
    }
    
    /// Parse type alias with visibility
    fn parse_type_alias_with_visibility(&mut self, visibility: Visibility) -> Result<TypeDef> {
        let documentation = self.collect_doc_comments();
        let start_span = self.current_span();
        self.expect(TokenKind::Type)?;
        
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        
        self.expect(TokenKind::Equal)?;
        let aliased_type = self.parse_type()?;
        
        let end_span = self.current_span();
        
        Ok(TypeDef {
            name,
            documentation,
            type_params,
            kind: TypeDefKind::Alias(aliased_type),
            visibility,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse type alias (backward compatibility)
    #[allow(dead_code)]
    fn parse_type_alias(&mut self) -> Result<TypeDef> {
        self.parse_type_alias_with_visibility(Visibility::Private)
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
    
    /// Parse effect definition with visibility
    fn parse_effect_def_with_visibility(&mut self, visibility: Visibility) -> Result<EffectDef> {
        let documentation = self.collect_doc_comments();
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
            documentation,
            type_params,
            operations,
            visibility,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse effect definition (backward compatibility)
    #[allow(dead_code)]
    fn parse_effect_def(&mut self) -> Result<EffectDef> {
        self.parse_effect_def_with_visibility(Visibility::Private)
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
    
    /// Parse handler definition with visibility
    fn parse_handler_def_with_visibility(&mut self, visibility: Visibility) -> Result<HandlerDef> {
        let start_span = self.current_span();
        self.expect(TokenKind::Handler)?;
        
        let name = self.parse_identifier()?;
        
        // Simplified handler parsing - just skip to end for now
        while !self.is_at_end() && !self.check(&TokenKind::Let) && !self.check(&TokenKind::Data) && !self.check(&TokenKind::Pub) {
            self.advance();
        }
        
        let end_span = self.current_span();
        
        Ok(HandlerDef {
            name,
            type_annotation: None,
            handled_effects: Vec::new(),
            handlers: Vec::new(),
            return_clause: None,
            visibility,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse handler definition (backward compatibility)
    #[allow(dead_code)]
    fn parse_handler_def(&mut self) -> Result<HandlerDef> {
        self.parse_handler_def_with_visibility(Visibility::Private)
    }
    
    /// Parse value definition with visibility
    fn parse_value_def_with_visibility(&mut self, visibility: Visibility) -> Result<ValueDef> {
        let documentation = self.collect_doc_comments();
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
            documentation,
            type_annotation,
            parameters: Vec::new(), // Simplified for now
            body,
            visibility,
            purity: Purity::Inferred,
            imports: Vec::new(),
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse value definition (backward compatibility)
    #[allow(dead_code)]
    fn parse_value_def(&mut self) -> Result<ValueDef> {
        self.parse_value_def_with_visibility(Visibility::Private)
    }
    
    /// Parse test definition with visibility
    fn parse_test_def_with_visibility(&mut self, visibility: Visibility) -> Result<TestDef> {
        let documentation = self.collect_doc_comments();
        let start_span = self.current_span();
        self.expect(TokenKind::Test)?;
        
        // Parse test name (string literal)
        let (name, description) = match &self.current_token().kind {
            TokenKind::String(s) => {
                let desc = s.clone();
                self.advance();
                // Use the string as the test name (convert to identifier)
                let name = Symbol::intern(&desc.replace(' ', "_"));
                (name, Some(desc))
            }
            TokenKind::Ident(s) => {
                // Alternative: test identifier { ... }
                let name = Symbol::intern(s);
                self.advance();
                (name, None)
            }
            _ => return Err(Error::Parse {
                message: "Expected test name (string or identifier)".to_string(),
            }),
        };
        
        // Parse optional tags and attributes
        let mut tags = Vec::new();
        let mut timeout = None;
        let mut expected_failure = false;
        
        // Parse 'with' clause for attributes
        if self.match_token(&TokenKind::With) {
            // Parse attributes like: with tags ["tag1", "tag2"], timeout = 5000
            loop {
                if self.match_ident("tags") {
                    self.expect(TokenKind::LeftBracket)?;
                    while !self.check(&TokenKind::RightBracket) {
                        if let TokenKind::String(tag) = &self.current_token().kind {
                            tags.push(tag.clone());
                            self.advance();
                        }
                        if !self.check(&TokenKind::RightBracket) {
                            self.expect(TokenKind::Comma)?;
                        }
                    }
                    self.expect(TokenKind::RightBracket)?;
                } else if self.match_ident("timeout") {
                    self.expect(TokenKind::Equal)?;
                    if let TokenKind::Integer(t) = self.current_token().kind {
                        timeout = Some(t as u64);
                        self.advance();
                    }
                } else if self.match_ident("expected_failure") {
                    self.expect(TokenKind::Equal)?;
                    if let TokenKind::Bool(b) = self.current_token().kind {
                        expected_failure = b;
                        self.advance();
                    }
                } else {
                    break;
                }
                
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        // Parse test body block
        self.expect(TokenKind::LeftBrace)?;
        
        let mut setup = None;
        let mut teardown = None;
        let mut body_expr = None;
        
        // Parse setup, body, teardown blocks
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.match_ident("setup") {
                self.expect(TokenKind::LeftBrace)?;
                let setup_expr = self.parse_expression()?;
                self.expect(TokenKind::RightBrace)?;
                setup = Some(Box::new(setup_expr));
            } else if self.match_ident("teardown") {
                self.expect(TokenKind::LeftBrace)?;
                let teardown_expr = self.parse_expression()?;
                self.expect(TokenKind::RightBrace)?;
                teardown = Some(Box::new(teardown_expr));
            } else if self.match_ident("body") {
                self.expect(TokenKind::LeftBrace)?;
                body_expr = Some(self.parse_expression()?);
                self.expect(TokenKind::RightBrace)?;
            } else {
                // If no explicit "body" block, treat the content as the body
                if body_expr.is_none() {
                    body_expr = Some(self.parse_expression()?);
                } else {
                    return Err(Error::Parse {
                        message: "Unexpected content in test definition".to_string(),
                    });
                }
            }
        }
        
        self.expect(TokenKind::RightBrace)?;
        
        let body = body_expr.ok_or_else(|| Error::Parse {
            message: "Test body is required".to_string(),
        })?;
        
        let end_span = self.current_span();
        
        Ok(TestDef {
            name,
            documentation,
            description,
            tags,
            setup,
            teardown,
            body,
            timeout,
            expected_failure,
            visibility,
            imports: Vec::new(),
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse component interface definition
    fn parse_interface_def(&mut self, _visibility: Visibility) -> Result<ComponentInterface> {
        let start_span = self.current_span();
        self.expect(TokenKind::Interface)?;
        
        // Parse interface name (e.g., "wasi:io/poll@0.2.0")
        let name = match &self.current_token().kind {
            TokenKind::String(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            _ => return Err(Error::Parse {
                message: "Expected interface name string".to_string(),
            }),
        };
        
        // Parse optional version
        let version = None; // TODO: implement version parsing
        
        // Parse interface items
        self.expect(TokenKind::LeftBrace)?;
        let mut items = Vec::new();
        
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            items.push(self.parse_interface_item()?);
        }
        
        self.expect(TokenKind::RightBrace)?;
        let end_span = self.current_span();
        
        Ok(ComponentInterface {
            name,
            version,
            items,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse interface item
    fn parse_interface_item(&mut self) -> Result<InterfaceItem> {
        let start_span = self.current_span();
        
        if self.check(&TokenKind::Func) {
            // Parse function declaration
            self.advance(); // consume 'func'
            let name = self.parse_identifier()?;
            let signature = self.parse_function_signature()?;
            let end_span = self.current_span();
            
            Ok(InterfaceItem::Func {
                name,
                signature,
                span: start_span.merge(end_span),
            })
        } else if self.check(&TokenKind::Type) {
            // Parse type declaration
            self.advance(); // consume 'type'
            let name = self.parse_identifier()?;
            
            // Optional type definition
            let definition = if self.match_token(&TokenKind::Equal) {
                Some(self.parse_type()?)
            } else {
                None
            };
            
            let end_span = self.current_span();
            Ok(InterfaceItem::Type {
                name,
                definition,
                span: start_span.merge(end_span),
            })
        } else if self.check(&TokenKind::Resource) {
            // Parse resource declaration
            self.advance(); // consume 'resource'
            let name = self.parse_identifier()?;
            
            self.expect(TokenKind::LeftBrace)?;
            let mut methods = Vec::new();
            
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                methods.push(self.parse_resource_method()?);
            }
            
            self.expect(TokenKind::RightBrace)?;
            let end_span = self.current_span();
            
            Ok(InterfaceItem::Resource {
                name,
                methods,
                span: start_span.merge(end_span),
            })
        } else {
            Err(Error::Parse {
                message: "Expected func, type, or resource in interface".to_string(),
            })
        }
    }
    
    /// Parse function signature for WebAssembly-style interfaces
    fn parse_function_signature(&mut self) -> Result<FunctionSignature> {
        let start_span = self.current_span();
        
        // Parse parameters: (param type type ...)
        let mut params = Vec::new();
        if self.match_token(&TokenKind::LeftParen) {
            if self.check(&TokenKind::Param) {
                self.advance(); // consume 'param'
                
                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    params.push(self.parse_wasm_type()?);
                }
            }
            self.expect(TokenKind::RightParen)?;
        }
        
        // Parse results: (result type type ...)
        let mut results = Vec::new();
        if self.match_token(&TokenKind::LeftParen) {
            if self.check(&TokenKind::Result) {
                self.advance(); // consume 'result'
                
                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    results.push(self.parse_wasm_type()?);
                }
            }
            self.expect(TokenKind::RightParen)?;
        }
        
        let end_span = self.current_span();
        Ok(FunctionSignature {
            params,
            results,
            span: start_span.merge(end_span),
        })
    }
    
    /// Parse WebAssembly type
    fn parse_wasm_type(&mut self) -> Result<WasmType> {
        match &self.current_token().kind {
            TokenKind::Ident(type_name) => {
                let wasm_type = match type_name.as_str() {
                    "i32" => WasmType::I32,
                    "i64" => WasmType::I64,
                    "f32" => WasmType::F32,
                    "f64" => WasmType::F64,
                    "v128" => WasmType::V128,
                    "funcref" => WasmType::FuncRef,
                    "externref" => WasmType::ExternRef,
                    _ => WasmType::Named(Symbol::intern(type_name)),
                };
                self.advance();
                Ok(wasm_type)
            }
            _ => Err(Error::Parse {
                message: "Expected WebAssembly type".to_string(),
            }),
        }
    }
    
    /// Parse resource method
    fn parse_resource_method(&mut self) -> Result<ResourceMethod> {
        let start_span = self.current_span();
        
        // Check for constructor or static modifiers
        let is_constructor = self.match_ident("constructor");
        let is_static = self.match_ident("static");
        
        let name = self.parse_identifier()?;
        let signature = self.parse_function_signature()?;
        
        let end_span = self.current_span();
        Ok(ResourceMethod {
            name,
            signature,
            is_constructor,
            is_static,
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
    
    /// Parse expression with precedence climbing
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_binary_expression(0)
    }
    
    /// Parse binary expressions with precedence climbing
    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expr> {
        let mut left = self.parse_application()?;
        
        while !self.is_at_end() {
            let token_kind = self.current_token().kind.clone();
            
            // Check if current token is a binary operator
            if let Some(precedence) = token_kind.precedence() {
                if precedence < min_precedence {
                    break;
                }
                
                let operator = token_kind.clone();
                self.advance(); // consume operator
                
                let right_precedence = if token_kind.is_left_associative() {
                    precedence + 1
                } else {
                    precedence
                };
                
                let right = self.parse_binary_expression(right_precedence)?;
                
                // Handle pipeline operator specifically
                if matches!(operator, TokenKind::PipeForward) {
                    // Transform x |> f into f(x)
                    let span = left.span().merge(right.span());
                    left = Expr::App(Box::new(right), vec![left], span);
                } else {
                    // For other operators, create function application
                    let span = left.span().merge(right.span());
                    let op_var = Expr::Var(self.operator_to_symbol(&operator), span);
                    left = Expr::App(Box::new(op_var), vec![left, right], span);
                }
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    /// Convert operator token to symbol
    fn operator_to_symbol(&self, operator: &TokenKind) -> Symbol {
        match operator {
            TokenKind::Plus => Symbol::intern("+"),
            TokenKind::Minus => Symbol::intern("-"),
            TokenKind::Star => Symbol::intern("*"),
            TokenKind::Slash => Symbol::intern("/"),
            TokenKind::EqualEqual => Symbol::intern("=="),
            TokenKind::NotEqual => Symbol::intern("!="),
            TokenKind::Less => Symbol::intern("<"),
            TokenKind::LessEqual => Symbol::intern("<="),
            TokenKind::Greater => Symbol::intern(">"),
            TokenKind::GreaterEqual => Symbol::intern(">="),
            TokenKind::AndAnd | TokenKind::And => Symbol::intern("&&"),
            TokenKind::OrOr | TokenKind::Or => Symbol::intern("||"),
            TokenKind::Cons => Symbol::intern("::"),
            TokenKind::Caret => Symbol::intern("^"),
            _ => Symbol::intern("unknown_op"),
        }
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
        } else if self.check(&TokenKind::Fun) || self.check(&TokenKind::Fn) {
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
            TokenKind::Number(_) | TokenKind::If | TokenKind::Fun | TokenKind::Fn |
            TokenKind::Match | TokenKind::Do | TokenKind::LeftBracket
            // Note: Removed Let - let expressions should be handled carefully to avoid confusion with top-level let definitions
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
            // Inside parentheses, we can have let expressions
            let expr = if self.check(&TokenKind::Let) {
                self.parse_let()
            } else {
                self.parse_expression()
            }?;
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
        // Accept either 'fun' or 'fn'
        if !self.match_token(&TokenKind::Fun) && !self.match_token(&TokenKind::Fn) {
            return Err(Error::Parse {
                message: "Expected 'fun' or 'fn'".to_string(),
            });
        }
        
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
            // Skip optional leading pipe
            self.match_token(&TokenKind::Pipe);
            
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
            TokenKind::LeftBracket => {
                self.advance();
                if self.match_token(&TokenKind::RightBracket) {
                    // Empty list pattern []
                    Ok(Pattern::Variable(Symbol::intern("[]"), start_span))
                } else {
                    Err(Error::Parse {
                        message: "List patterns not yet supported".to_string(),
                    })
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
            TokenKind::LeftParen | TokenKind::LeftBracket
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
            TokenKind::Number(s) => {
                let s = s.clone();
                self.advance();
                // Parse as integer or float
                if s.contains('.') {
                    if let Ok(f) = s.parse::<f64>() {
                        Ok(Expr::Literal(Literal::Float(f), start_span))
                    } else {
                        Err(Error::Parse {
                            message: format!("Invalid float literal: {s}"),
                        })
                    }
                } else if let Ok(i) = s.parse::<i64>() {
                    Ok(Expr::Literal(Literal::Integer(i), start_span))
                } else {
                    Err(Error::Parse {
                        message: format!("Invalid integer literal: {s}"),
                    })
                }
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
            TokenKind::LeftBracket => {
                self.advance();
                
                // Empty list []
                if self.match_token(&TokenKind::RightBracket) {
                    let end_span = self.current_span();
                    // Empty list is represented as a constructor
                    return Ok(Expr::Var(Symbol::intern("[]"), start_span.merge(end_span)));
                }
                
                // Parse first element
                let first = self.parse_expression()?;
                
                // Check if it's a list literal [1; 2; 3] or [1, 2, 3]
                let separator = if self.check(&TokenKind::Semicolon) {
                    TokenKind::Semicolon
                } else {
                    TokenKind::Comma
                };
                
                let mut elements = vec![first];
                
                while self.match_token(&separator) {
                    if self.check(&TokenKind::RightBracket) {
                        break; // Allow trailing separator
                    }
                    elements.push(self.parse_expression()?);
                }
                
                self.expect(TokenKind::RightBracket)?;
                let end_span = self.current_span();
                
                // Build list using :: and []
                let mut list_expr = Expr::Var(Symbol::intern("[]"), end_span);
                
                // Build from right to left: [1; 2; 3] becomes 1 :: 2 :: 3 :: []
                for elem in elements.into_iter().rev() {
                    let cons_span = elem.span().merge(list_expr.span());
                    let cons_op = Expr::Var(Symbol::intern("::"), cons_span);
                    list_expr = Expr::App(Box::new(cons_op), vec![elem, list_expr], cons_span);
                }
                
                Ok(list_expr)
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
                use crate::span::{FileId, ByteOffset, Span};
                Token::eof(Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0)))
            })
        }
    }
    
    fn current_span(&self) -> Span {
        self.current_token().span
    }
    
    fn current(&self) -> &TokenKind {
        &self.current_token().kind
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
    
    /// Match an identifier with a specific value
    fn match_ident(&mut self, name: &str) -> bool {
        if let TokenKind::Ident(current_name) = &self.current_token().kind {
            if current_name == name {
                self.advance();
                return true;
            }
        }
        false
    }
    
    /// Collect documentation comments before the current position
    fn collect_doc_comments(&mut self) -> Option<Documentation> {
        let mut doc_tokens = Vec::new();
        let mut current = self.current;
        
        // Look backwards for doc comments
        while current > 0 {
            current -= 1;
            match &self.tokens[current].kind {
                TokenKind::DocComment(content) => {
                    doc_tokens.push((content.clone(), self.tokens[current].span));
                }
                TokenKind::Whitespace | TokenKind::Newline => {
                    // Continue looking
                }
                _ => {
                    // Stop at first non-whitespace, non-doc token
                    break;
                }
            }
        }
        
        if doc_tokens.is_empty() {
            return None;
        }
        
        // Reverse to get chronological order
        doc_tokens.reverse();
        
        // Merge all doc comments
        let mut full_content = String::new();
        let first_span = doc_tokens[0].1;
        let last_span = doc_tokens[doc_tokens.len() - 1].1;
        
        for (content, _) in doc_tokens {
            if !full_content.is_empty() {
                full_content.push('\n');
            }
            full_content.push_str(&content);
        }
        
        // Parse the documentation
        let doc_comment = self.parse_doc_comment_content(&full_content, first_span.merge(last_span));
        
        Some(Documentation {
            doc_comment,
            inline_comments: Vec::new(),
            is_module_doc: false,
        })
    }
    
    /// Parse documentation comment content
    fn parse_doc_comment_content(&self, content: &str, span: Span) -> DocComment {
        use std::collections::HashMap;
        
        let mut attributes = HashMap::new();
        let mut code_blocks = Vec::new();
        let mut main_content = String::new();
        
        // Simple parsing - in real implementation, use a proper markdown parser
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        // Check for frontmatter
        if lines.first() == Some(&"---") {
            i = 1;
            while i < lines.len() && lines[i] != "---" {
                if let Some((key, value)) = lines[i].split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Handle JSDoc-style attributes (e.g., @param, @returns)
                    let key = if key.starts_with('@') {
                        key.trim_start_matches('@')
                    } else {
                        key
                    };
                    
                    // Parse attribute value
                    if let Ok(num) = value.parse::<f64>() {
                        attributes.insert(key.to_string(), DocAttributeValue::Number(num));
                    } else if value == "true" || value == "false" {
                        attributes.insert(key.to_string(), DocAttributeValue::Boolean(value == "true"));
                    } else if value.starts_with('[') && value.ends_with(']') {
                        // Simple list parsing
                        let items: Vec<String> = value[1..value.len()-1]
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .collect();
                        attributes.insert(key.to_string(), DocAttributeValue::List(items));
                    } else if value.starts_with('{') && value.contains('}') {
                        // Parse typed parameters like {a: Int} or {Type}
                        if let Some(end) = value.find('}') {
                            let type_part = &value[1..end];
                            let desc_part = value[end+1..].trim();
                            attributes.insert(key.to_string(), DocAttributeValue::TypedParam {
                                type_info: type_part.to_string(),
                                description: desc_part.to_string(),
                            });
                        } else {
                            attributes.insert(key.to_string(), DocAttributeValue::String(value.to_string()));
                        }
                    } else {
                        attributes.insert(key.to_string(), DocAttributeValue::String(value.to_string()));
                    }
                }
                i += 1;
            }
            i += 1; // Skip closing ---
        }
        
        // Parse remaining content
        while i < lines.len() {
            if lines[i].starts_with("```") {
                // Code block
                let lang = lines[i][3..].trim();
                let lang = if lang.is_empty() { None } else { Some(lang.to_string()) };
                let mut code_content = String::new();
                i += 1;
                
                while i < lines.len() && !lines[i].starts_with("```") {
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    code_content.push_str(lines[i]);
                    i += 1;
                }
                
                code_blocks.push(CodeBlock {
                    language: lang,
                    content: code_content,
                    metadata: None,
                    span,
                });
                i += 1; // Skip closing ```
            } else {
                if !main_content.is_empty() {
                    main_content.push('\n');
                }
                main_content.push_str(lines[i]);
                i += 1;
            }
        }
        
        DocComment {
            content: main_content.trim().to_string(),
            attributes,
            code_blocks,
            span,
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
    
    fn error<T>(&self, message: &str) -> Result<T> {
        Err(Error::Parse {
            message: message.to_string(),
        })
    }
    
    /// Parse module shorthand item (e.g., Math.pi : Float)
    fn parse_module_shorthand_item(&mut self, visibility: Visibility) -> Result<ValueDef> {
        let documentation = self.collect_doc_comments();
        let start_span = self.current_span();
        
        // Parse module path (e.g., Math.pi or Core.List.map)
        let mut path_parts = vec![self.parse_identifier()?];
        
        while self.match_token(&TokenKind::Dot) {
            path_parts.push(self.parse_identifier()?);
        }
        
        if path_parts.len() < 2 {
            return Err(Error::Parse {
                message: "Module shorthand requires at least Module.member".to_string(),
            });
        }
        
        // The last part is the member name
        let name = path_parts.pop().unwrap();
        // The rest is the module path
        let _module_path = path_parts; // Will be used for module resolution
        
        // Parse optional type annotation
        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse function parameters if any
        let mut parameters = Vec::new();
        while !self.check(&TokenKind::Equal) && !self.is_at_end() {
            // Check if the current token could be a parameter
            match &self.current_token().kind {
                TokenKind::Ident(_) => {
                    parameters.push(self.parse_pattern()?);
                }
                _ => break,
            }
        }
        
        // Now check if we have '=' for the body
        if !self.check(&TokenKind::Equal) {
            // This might be just a type declaration (e.g., Math.pi : Float)
            // In this case, we need a dummy body
            if type_annotation.is_some() && parameters.is_empty() {
                // Create a placeholder expression for now
                let body = Expr::Var(Symbol::intern("undefined"), self.current_span());
                let end_span = self.current_span();
                
                return Ok(ValueDef {
                    name,
                    documentation,
                    type_annotation,
                    parameters: Vec::new(),
                    body,
                    visibility,
                    purity: Purity::Inferred,
                    imports: Vec::new(),
                    span: start_span.merge(end_span),
                });
            } else {
                return Err(Error::Parse {
                    message: "Expected '=' after module member name or parameters".to_string(),
                });
            }
        }
        
        self.expect(TokenKind::Equal)?;
        let body = self.parse_expression()?;
        
        let end_span = self.current_span();
        
        // If we have parameters, wrap the body in a lambda
        let final_body = if !parameters.is_empty() {
            Expr::Lambda {
                parameters,
                body: Box::new(body),
                span: start_span.merge(end_span),
            }
        } else {
            body
        };
        
        Ok(ValueDef {
            name,
            documentation,
            type_annotation,
            parameters: Vec::new(), // Not used when we wrap in lambda
            body: final_body,
            visibility,
            purity: Purity::Inferred,
            imports: Vec::new(),
            span: start_span.merge(end_span),
        })
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
    use crate::span::FileId;

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
    fn test_parse_simple_lambda() {
        let input = r#"module Test

let identity = fun x -> x"#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
    }
    
    #[test]
    fn test_parse_two_lambdas() {
        let input = r#"module Test

let identity = fun x -> x
let add = fun x y -> y"#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 2);
    }
    
    #[test]
    fn test_parse_lambda_expression() {
        let input = r#"module Test

let identity = fun x -> x
let add = fun x y -> x + y"#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 2);
    }
    
    #[test]
    fn test_parse_fn_lambda() {
        let input = r#"module Test

let identity = fn x -> x
let add = fn x y -> x + y
let mixed1 = fun x -> fn y -> x + y
let mixed2 = fn x -> fun y -> x + y"#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 4);
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
            
            let res = f x y
        "#;
        
        let result = parse(input, FileId::new(0));
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
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
        match &result {
            Ok(_) => {},
            Err(e) => panic!("Parse failed: {e:?}"),
        }
        
        let cu = result.unwrap();
        assert_eq!(cu.module.items.len(), 1);
        
        if let Item::ValueDef(value_def) = &cu.module.items[0] {
            if let Expr::Match { arms, .. } = &value_def.body {
                assert_eq!(arms.len(), 2);
            }
        }
    }
}