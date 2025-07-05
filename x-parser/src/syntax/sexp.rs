//! S-expression (Lisp-like) syntax parser and printer
//! 
//! This provides a Lisp-like syntax for x Language that can be useful for
//! meta-programming, code generation, and data exchange.

use super::{SyntaxParser, SyntaxPrinter, SyntaxStyle, SyntaxConfig};
use crate::{ast::*, span::{FileId, Span, ByteOffset}, symbol::Symbol};
use crate::error::{ParseError as Error, Result};

/// S-expression parser
pub struct SExpParser;

impl SExpParser {
    pub fn new() -> Self {
        SExpParser
    }
    
    fn parse_sexp(&mut self, input: &str, file_id: FileId) -> Result<SExp> {
        let mut lexer = SExpLexer::new(input, file_id);
        let tokens = lexer.tokenize()?;
        let mut parser = SExpTokenParser::new(tokens, file_id);
        parser.parse_sexp()
    }
}

impl SyntaxParser for SExpParser {
    fn parse(&mut self, input: &str, file_id: FileId) -> Result<CompilationUnit> {
        let sexp = self.parse_sexp(input, file_id)?;
        sexp_to_ast(&sexp)
    }
    
    fn parse_expression(&mut self, input: &str, file_id: FileId) -> Result<Expr> {
        let sexp = self.parse_sexp(input, file_id)?;
        sexp_to_expr(&sexp)
    }
    
    fn syntax_style(&self) -> SyntaxStyle {
        SyntaxStyle::SExp
    }
}

/// S-expression printer
pub struct SExpPrinter;

impl SExpPrinter {
    pub fn new() -> Self {
        SExpPrinter
    }
    
    fn indent(&self, level: usize, config: &SyntaxConfig) -> String {
        if config.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * config.indent_size)
        }
    }
}

impl SyntaxPrinter for SExpPrinter {
    fn print(&self, ast: &CompilationUnit, config: &SyntaxConfig) -> Result<String> {
        let sexp = ast_to_sexp(ast);
        Ok(self.print_sexp(&sexp, config, 0))
    }
    
    fn print_expression(&self, expr: &Expr, config: &SyntaxConfig) -> Result<String> {
        let sexp = expr_to_sexp(expr);
        Ok(self.print_sexp(&sexp, config, 0))
    }
    
    fn print_type(&self, typ: &Type, config: &SyntaxConfig) -> Result<String> {
        let sexp = type_to_sexp(typ);
        Ok(self.print_sexp(&sexp, config, 0))
    }
    
    fn syntax_style(&self) -> SyntaxStyle {
        SyntaxStyle::SExp
    }
}

impl SExpPrinter {
    fn print_sexp(&self, sexp: &SExp, config: &SyntaxConfig, level: usize) -> String {
        match sexp {
            SExp::Atom(atom) => atom.clone(),
            SExp::List(list) => {
                if list.is_empty() {
                    "()".to_string()
                } else if list.len() == 1 {
                    format!("({})", self.print_sexp(&list[0], config, level))
                } else {
                    let _indent = self.indent(level, config);
                    let mut output = String::new();
                    output.push('(');
                    
                    // First element (usually the operator/function)
                    output.push_str(&self.print_sexp(&list[0], config, level));
                    
                    // Remaining elements
                    for item in &list[1..] {
                        if self.should_break_line(&list[0], item, config) {
                            output.push('\n');
                            output.push_str(&self.indent(level + 1, config));
                        } else {
                            output.push(' ');
                        }
                        output.push_str(&self.print_sexp(item, config, level + 1));
                    }
                    
                    output.push(')');
                    output
                }
            }
        }
    }
    
    fn should_break_line(&self, first: &SExp, item: &SExp, _config: &SyntaxConfig) -> bool {
        // Break lines for certain constructs to improve readability
        if let SExp::Atom(atom) = first {
            match atom.as_str() {
                "module" | "let" | "type" | "effect" | "handler" | "match" | "if" | "lambda" => {
                    matches!(item, SExp::List(_))
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

/// S-expression representation
#[derive(Debug, Clone, PartialEq)]
enum SExp {
    Atom(String),
    List(Vec<SExp>),
}

/// S-expression token
#[derive(Debug, Clone, PartialEq)]
enum SExpToken {
    LeftParen,
    RightParen,
    Atom(String),
    Eof,
}

/// S-expression lexer
#[allow(dead_code)]
struct SExpLexer {
    input: String,
    chars: Vec<char>,
    position: usize,
    file_id: FileId,
}

impl SExpLexer {
    fn new(input: &str, file_id: FileId) -> Self {
        SExpLexer {
            input: input.to_string(),
            chars: input.chars().collect(),
            position: 0,
            file_id,
        }
    }
    
    fn tokenize(&mut self) -> Result<Vec<SExpToken>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            
            if self.is_at_end() {
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(SExpToken::Eof);
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<SExpToken> {
        match self.current_char() {
            Some('(') => {
                self.advance();
                Ok(SExpToken::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(SExpToken::RightParen)
            }
            Some('"') => {
                let string = self.read_string()?;
                Ok(SExpToken::Atom(format!("\"{}\"", string)))
            }
            Some(ch) if ch.is_alphanumeric() || "+-*/<>=!_?:".contains(ch) => {
                let atom = self.read_atom();
                Ok(SExpToken::Atom(atom))
            }
            Some(ch) => Err(Error::Parse {
                message: format!("Unexpected character in S-expression: '{}'", ch),
            }),
            None => Ok(SExpToken::Eof),
        }
    }
    
    fn read_string(&mut self) -> Result<String> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(value);
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
            message: "Unterminated string literal".to_string(),
        })
    }
    
    fn read_atom(&mut self) -> String {
        let mut atom = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ';' {
                break;
            }
            atom.push(ch);
            self.advance();
        }
        
        atom
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == ';' {
                // Skip comment until end of line
                while let Some(ch) = self.current_char() {
                    self.advance();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
    
    fn current_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
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

/// S-expression token parser
#[allow(dead_code)]
struct SExpTokenParser {
    tokens: Vec<SExpToken>,
    current: usize,
    file_id: FileId,
}

impl SExpTokenParser {
    fn new(tokens: Vec<SExpToken>, file_id: FileId) -> Self {
        SExpTokenParser {
            tokens,
            current: 0,
            file_id,
        }
    }
    
    fn parse_sexp(&mut self) -> Result<SExp> {
        match &self.current_token() {
            SExpToken::LeftParen => self.parse_list(),
            SExpToken::Atom(atom) => {
                let atom = atom.clone();
                self.advance();
                Ok(SExp::Atom(atom))
            }
            SExpToken::RightParen => Err(Error::Parse {
                message: "Unexpected closing parenthesis".to_string(),
            }),
            SExpToken::Eof => Err(Error::Parse {
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
    
    fn parse_list(&mut self) -> Result<SExp> {
        self.advance(); // Skip opening paren
        let mut list = Vec::new();
        
        while !matches!(self.current_token(), SExpToken::RightParen | SExpToken::Eof) {
            list.push(self.parse_sexp()?);
        }
        
        if matches!(self.current_token(), SExpToken::RightParen) {
            self.advance(); // Skip closing paren
            Ok(SExp::List(list))
        } else {
            Err(Error::Parse {
                message: "Expected closing parenthesis".to_string(),
            })
        }
    }
    
    fn current_token(&self) -> &SExpToken {
        self.tokens.get(self.current).unwrap_or(&SExpToken::Eof)
    }
    
    fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }
}

// AST to S-expression conversion functions

fn ast_to_sexp(ast: &CompilationUnit) -> SExp {
    SExp::List(vec![
        SExp::Atom("compilation-unit".to_string()),
        module_to_sexp(&ast.module),
    ])
}

fn module_to_sexp(module: &Module) -> SExp {
    let mut elements = vec![
        SExp::Atom("module".to_string()),
        SExp::Atom(module.name.to_string()),
    ];
    
    // Exports
    if let Some(exports) = &module.exports {
        let export_list = SExp::List(
            exports.items.iter()
                .map(|item| SExp::Atom(item.name.as_str().to_string()))
                .collect()
        );
        elements.push(SExp::List(vec![SExp::Atom("export".to_string()), export_list]));
    }
    
    // Imports
    for import in &module.imports {
        elements.push(import_to_sexp(import));
    }
    
    // Items
    for item in &module.items {
        elements.push(item_to_sexp(item));
    }
    
    SExp::List(elements)
}

fn import_to_sexp(import: &Import) -> SExp {
    let mut elements = vec![
        SExp::Atom("import".to_string()),
        SExp::Atom(import.module_path.to_string()),
    ];
    
    match &import.kind {
        ImportKind::Qualified => {}
        ImportKind::Selective(items) => {
            let item_list = SExp::List(
                items.iter()
                    .map(|item| SExp::Atom(item.name.as_str().to_string()))
                    .collect()
            );
            elements.push(SExp::List(vec![SExp::Atom("select".to_string()), item_list]));
        }
        ImportKind::Wildcard => {
            elements.push(SExp::Atom("*".to_string()));
        }
        ImportKind::Lazy => {
            elements.push(SExp::Atom("lazy".to_string()));
        }
        ImportKind::Conditional(_) => {
            elements.push(SExp::List(vec![SExp::Atom("when".to_string()), SExp::Atom("condition".to_string())]));
        }
        ImportKind::Interface { interface, items } => {
            elements.push(SExp::List(vec![SExp::Atom("interface".to_string()), SExp::Atom(interface.clone())]));
            if !items.is_empty() {
                let item_list = SExp::List(
                    items.iter()
                        .map(|item| SExp::Atom(item.name.as_str().to_string()))
                        .collect()
                );
                elements.push(item_list);
            }
        }
        ImportKind::Core { module, items } => {
            elements.push(SExp::List(vec![SExp::Atom("core".to_string()), SExp::Atom(module.clone())]));
            if !items.is_empty() {
                let item_list = SExp::List(
                    items.iter()
                        .map(|item| SExp::Atom(item.name.as_str().to_string()))
                        .collect()
                );
                elements.push(item_list);
            }
        }
        ImportKind::Func { module, name, signature: _ } => {
            elements.push(SExp::List(vec![
                SExp::Atom("func".to_string()),
                SExp::Atom(module.clone()),
                SExp::Atom(name.clone()),
            ]));
        }
    }
    
    if let Some(alias) = &import.alias {
        elements.push(SExp::List(vec![SExp::Atom("as".to_string()), SExp::Atom(alias.as_str().to_string())]));
    }
    
    SExp::List(elements)
}

fn item_to_sexp(item: &Item) -> SExp {
    match item {
        Item::ValueDef(def) => value_def_to_sexp(def),
        Item::TypeDef(def) => type_def_to_sexp(def),
        Item::EffectDef(def) => effect_def_to_sexp(def),
        Item::HandlerDef(def) => handler_def_to_sexp(def),
        Item::ModuleTypeDef(def) => module_type_def_to_sexp(def),
        Item::InterfaceDef(def) => interface_def_to_sexp(def),
    }
}

fn value_def_to_sexp(def: &ValueDef) -> SExp {
    let mut elements = vec![
        SExp::Atom("let".to_string()),
        SExp::Atom(def.name.as_str().to_string()),
    ];
    
    // Parameters
    for param in &def.parameters {
        elements.push(pattern_to_sexp(param));
    }
    
    // Type annotation
    if let Some(typ) = &def.type_annotation {
        elements.push(SExp::List(vec![SExp::Atom("type".to_string()), type_to_sexp(typ)]));
    }
    
    // Body
    elements.push(expr_to_sexp(&def.body));
    
    SExp::List(elements)
}

fn type_def_to_sexp(def: &TypeDef) -> SExp {
    let mut elements = vec![
        SExp::Atom("type".to_string()),
        SExp::Atom(def.name.as_str().to_string()),
    ];
    
    // Type parameters
    if !def.type_params.is_empty() {
        let params = SExp::List(
            def.type_params.iter()
                .map(|param| SExp::Atom(param.name.as_str().to_string()))
                .collect()
        );
        elements.push(params);
    }
    
    // Definition
    match &def.kind {
        TypeDefKind::Alias(typ) => {
            elements.push(type_to_sexp(typ));
        }
        TypeDefKind::Data(constructors) => {
            let variants = SExp::List(
                constructors.iter()
                    .map(|ctor| {
                        let mut ctor_elements = vec![SExp::Atom(ctor.name.as_str().to_string())];
                        for field in &ctor.fields {
                            ctor_elements.push(type_to_sexp(field));
                        }
                        SExp::List(ctor_elements)
                    })
                    .collect()
            );
            elements.push(SExp::List(vec![SExp::Atom("variants".to_string()), variants]));
        }
        TypeDefKind::Abstract => {
            elements.push(SExp::Atom("abstract".to_string()));
        }
    }
    
    SExp::List(elements)
}

fn effect_def_to_sexp(def: &EffectDef) -> SExp {
    let mut elements = vec![
        SExp::Atom("effect".to_string()),
        SExp::Atom(def.name.as_str().to_string()),
    ];
    
    for operation in &def.operations {
        let mut op_elements = vec![SExp::Atom(operation.name.as_str().to_string())];
        for param in &operation.parameters {
            op_elements.push(type_to_sexp(param));
        }
        op_elements.push(type_to_sexp(&operation.return_type));
        elements.push(SExp::List(op_elements));
    }
    
    SExp::List(elements)
}

fn handler_def_to_sexp(def: &HandlerDef) -> SExp {
    let mut elements = vec![
        SExp::Atom("handler".to_string()),
        SExp::Atom(def.name.as_str().to_string()),
    ];
    
    for handler in &def.handlers {
        let mut handler_elements = vec![
            SExp::Atom(format!("{}.{}", handler.effect.name.as_str(), handler.operation.as_str())),
        ];
        for param in &handler.parameters {
            handler_elements.push(pattern_to_sexp(param));
        }
        handler_elements.push(expr_to_sexp(&handler.body));
        elements.push(SExp::List(handler_elements));
    }
    
    if let Some(return_clause) = &def.return_clause {
        elements.push(SExp::List(vec![
            SExp::Atom("return".to_string()),
            pattern_to_sexp(&return_clause.parameter),
            expr_to_sexp(&return_clause.body),
        ]));
    }
    
    SExp::List(elements)
}

fn module_type_def_to_sexp(def: &ModuleTypeDef) -> SExp {
    SExp::List(vec![
        SExp::Atom("module-type".to_string()),
        SExp::Atom(def.name.as_str().to_string()),
        SExp::Atom("signature".to_string()), // Simplified
    ])
}

fn interface_def_to_sexp(def: &ComponentInterface) -> SExp {
    let mut elements = vec![
        SExp::Atom("interface".to_string()),
        SExp::Atom(def.name.clone()),
    ];
    
    for item in &def.items {
        elements.push(interface_item_to_sexp(item));
    }
    
    SExp::List(elements)
}

fn interface_item_to_sexp(item: &InterfaceItem) -> SExp {
    match item {
        InterfaceItem::Func { name, signature, .. } => {
            SExp::List(vec![
                SExp::Atom("func".to_string()),
                SExp::Atom(name.as_str().to_string()),
                function_signature_to_sexp(signature),
            ])
        }
        InterfaceItem::Type { name, definition, .. } => {
            let mut elements = vec![
                SExp::Atom("type".to_string()),
                SExp::Atom(name.as_str().to_string()),
            ];
            if let Some(typ) = definition {
                elements.push(type_to_sexp(typ));
            }
            SExp::List(elements)
        }
        InterfaceItem::Resource { name, methods, .. } => {
            let mut elements = vec![
                SExp::Atom("resource".to_string()),
                SExp::Atom(name.as_str().to_string()),
            ];
            for method in methods {
                elements.push(resource_method_to_sexp(method));
            }
            SExp::List(elements)
        }
    }
}

fn function_signature_to_sexp(signature: &FunctionSignature) -> SExp {
    let mut elements = vec![SExp::Atom("sig".to_string())];
    
    if !signature.params.is_empty() {
        let mut param_elements = vec![SExp::Atom("param".to_string())];
        for param in &signature.params {
            param_elements.push(wasm_type_to_sexp(param));
        }
        elements.push(SExp::List(param_elements));
    }
    
    if !signature.results.is_empty() {
        let mut result_elements = vec![SExp::Atom("result".to_string())];
        for result in &signature.results {
            result_elements.push(wasm_type_to_sexp(result));
        }
        elements.push(SExp::List(result_elements));
    }
    
    SExp::List(elements)
}

fn resource_method_to_sexp(method: &ResourceMethod) -> SExp {
    let mut elements = vec![SExp::Atom("method".to_string())];
    
    if method.is_constructor {
        elements.push(SExp::Atom("constructor".to_string()));
    }
    if method.is_static {
        elements.push(SExp::Atom("static".to_string()));
    }
    
    elements.push(SExp::Atom(method.name.as_str().to_string()));
    elements.push(function_signature_to_sexp(&method.signature));
    
    SExp::List(elements)
}

fn wasm_type_to_sexp(wasm_type: &WasmType) -> SExp {
    let type_name = match wasm_type {
        WasmType::I32 => "i32",
        WasmType::I64 => "i64",
        WasmType::F32 => "f32",
        WasmType::F64 => "f64",
        WasmType::V128 => "v128",
        WasmType::FuncRef => "funcref",
        WasmType::ExternRef => "externref",
        WasmType::Named(name) => name.as_str(),
    };
    SExp::Atom(type_name.to_string())
}

fn expr_to_sexp(expr: &Expr) -> SExp {
    match expr {
        Expr::Literal(lit, _) => literal_to_sexp(lit),
        Expr::Var(name, _) => SExp::Atom(name.as_str().to_string()),
        Expr::App(func, args, _) => {
            let mut elements = vec![expr_to_sexp(func)];
            for arg in args {
                elements.push(expr_to_sexp(arg));
            }
            SExp::List(elements)
        }
        Expr::Lambda { parameters, body, span: _ } => {
            let mut elements = vec![SExp::Atom("lambda".to_string())];
            let params = SExp::List(
                parameters.iter().map(|p| pattern_to_sexp(p)).collect()
            );
            elements.push(params);
            elements.push(expr_to_sexp(body));
            SExp::List(elements)
        }
        Expr::Let { pattern, type_annotation, value, body, span: _ } => {
            let mut elements = vec![
                SExp::Atom("let".to_string()),
                pattern_to_sexp(pattern),
            ];
            if let Some(typ) = type_annotation {
                elements.push(SExp::List(vec![SExp::Atom("type".to_string()), type_to_sexp(typ)]));
            }
            elements.push(expr_to_sexp(value));
            elements.push(expr_to_sexp(body));
            SExp::List(elements)
        }
        Expr::If { condition, then_branch, else_branch, span: _ } => {
            SExp::List(vec![
                SExp::Atom("if".to_string()),
                expr_to_sexp(condition),
                expr_to_sexp(then_branch),
                expr_to_sexp(else_branch),
            ])
        }
        Expr::Match { scrutinee, arms, span: _ } => {
            let mut elements = vec![
                SExp::Atom("match".to_string()),
                expr_to_sexp(scrutinee),
            ];
            for arm in arms {
                let mut arm_elements = vec![pattern_to_sexp(&arm.pattern)];
                if let Some(guard) = &arm.guard {
                    arm_elements.push(SExp::List(vec![SExp::Atom("when".to_string()), expr_to_sexp(guard)]));
                }
                arm_elements.push(expr_to_sexp(&arm.body));
                elements.push(SExp::List(arm_elements));
            }
            SExp::List(elements)
        }
        Expr::Do { statements, span: _ } => {
            let mut elements = vec![SExp::Atom("do".to_string())];
            for statement in statements {
                elements.push(do_statement_to_sexp(statement));
            }
            SExp::List(elements)
        }
        Expr::Handle { expr, handlers, return_clause, span: _ } => {
            let mut elements = vec![
                SExp::Atom("handle".to_string()),
                expr_to_sexp(expr),
            ];
            for handler in handlers {
                elements.push(SExp::List(vec![
                    SExp::Atom(format!("{}.{}", handler.effect.name.as_str(), handler.operation.as_str())),
                    expr_to_sexp(&handler.body),
                ]));
            }
            if let Some(return_clause) = return_clause {
                elements.push(SExp::List(vec![
                    SExp::Atom("return".to_string()),
                    pattern_to_sexp(&return_clause.parameter),
                    expr_to_sexp(&return_clause.body),
                ]));
            }
            SExp::List(elements)
        }
        Expr::Resume { value, span: _ } => {
            SExp::List(vec![SExp::Atom("resume".to_string()), expr_to_sexp(value)])
        }
        Expr::Perform { effect, operation, args, span: _ } => {
            let mut elements = vec![
                SExp::Atom("perform".to_string()),
                SExp::Atom(format!("{}.{}", effect.as_str(), operation.as_str())),
            ];
            for arg in args {
                elements.push(expr_to_sexp(arg));
            }
            SExp::List(elements)
        }
        Expr::Ann { expr, type_annotation, span: _ } => {
            SExp::List(vec![
                SExp::Atom("ann".to_string()),
                expr_to_sexp(expr),
                type_to_sexp(type_annotation),
            ])
        }
    }
}

fn do_statement_to_sexp(statement: &DoStatement) -> SExp {
    match statement {
        DoStatement::Let { pattern, expr, span: _ } => {
            SExp::List(vec![
                SExp::Atom("let".to_string()),
                pattern_to_sexp(pattern),
                expr_to_sexp(expr),
            ])
        }
        DoStatement::Bind { pattern, expr, span: _ } => {
            SExp::List(vec![
                SExp::Atom("bind".to_string()),
                pattern_to_sexp(pattern),
                expr_to_sexp(expr),
            ])
        }
        DoStatement::Expr(expr) => expr_to_sexp(expr),
    }
}

fn pattern_to_sexp(pattern: &Pattern) -> SExp {
    match pattern {
        Pattern::Wildcard(_) => SExp::Atom("_".to_string()),
        Pattern::Variable(name, _) => SExp::Atom(name.as_str().to_string()),
        Pattern::Literal(lit, _) => literal_to_sexp(lit),
        Pattern::Constructor { name, args, span: _ } => {
            let mut elements = vec![SExp::Atom(name.as_str().to_string())];
            for arg in args {
                elements.push(pattern_to_sexp(arg));
            }
            SExp::List(elements)
        }
        Pattern::Record { fields, rest, span: _ } => {
            let mut elements = vec![SExp::Atom("record".to_string())];
            for (name, pattern) in fields {
                elements.push(SExp::List(vec![
                    SExp::Atom(name.as_str().to_string()),
                    pattern_to_sexp(pattern),
                ]));
            }
            if let Some(rest_pattern) = rest {
                elements.push(SExp::List(vec![SExp::Atom("rest".to_string()), pattern_to_sexp(rest_pattern)]));
            }
            SExp::List(elements)
        }
        Pattern::Tuple { patterns, span: _ } => {
            let mut elements = vec![SExp::Atom("tuple".to_string())];
            for pattern in patterns {
                elements.push(pattern_to_sexp(pattern));
            }
            SExp::List(elements)
        }
        Pattern::Or { left, right, span: _ } => {
            SExp::List(vec![
                SExp::Atom("or".to_string()),
                pattern_to_sexp(left),
                pattern_to_sexp(right),
            ])
        }
        Pattern::As { pattern, name, span: _ } => {
            SExp::List(vec![
                SExp::Atom("as".to_string()),
                pattern_to_sexp(pattern),
                SExp::Atom(name.as_str().to_string()),
            ])
        }
        Pattern::Ann { pattern, type_annotation, span: _ } => {
            SExp::List(vec![
                SExp::Atom("ann".to_string()),
                pattern_to_sexp(pattern),
                type_to_sexp(type_annotation),
            ])
        }
    }
}

fn type_to_sexp(typ: &Type) -> SExp {
    match typ {
        Type::Var(name, _) => SExp::Atom(name.as_str().to_string()),
        Type::Con(name, _) => SExp::Atom(name.as_str().to_string()),
        Type::App(typ, args, _) => {
            let mut elements = vec![type_to_sexp(typ)];
            for arg in args {
                elements.push(type_to_sexp(arg));
            }
            SExp::List(elements)
        }
        Type::Fun { params, return_type, effects, span: _ } => {
            let mut elements = vec![SExp::Atom("fun".to_string())];
            for param in params {
                elements.push(type_to_sexp(param));
            }
            elements.push(type_to_sexp(return_type));
            if !effects.effects.is_empty() {
                elements.push(SExp::List(vec![SExp::Atom("effects".to_string())])); // Simplified
            }
            SExp::List(elements)
        }
        Type::Forall { type_params, body, span: _ } => {
            let params = SExp::List(
                type_params.iter()
                    .map(|param| SExp::Atom(param.name.as_str().to_string()))
                    .collect()
            );
            SExp::List(vec![
                SExp::Atom("forall".to_string()),
                params,
                type_to_sexp(body),
            ])
        }
        Type::Effects(_effects, _) => {
            SExp::List(vec![SExp::Atom("effects".to_string())]) // Simplified
        }
        Type::Exists { type_params, body, span: _ } => {
            let params = SExp::List(
                type_params.iter()
                    .map(|param| SExp::Atom(param.name.as_str().to_string()))
                    .collect()
            );
            SExp::List(vec![
                SExp::Atom("exists".to_string()),
                params,
                type_to_sexp(body),
            ])
        }
        Type::Record { fields, rest, span: _ } => {
            let mut elements = vec![SExp::Atom("record".to_string())];
            for (name, typ) in fields {
                elements.push(SExp::List(vec![
                    SExp::Atom(name.as_str().to_string()),
                    type_to_sexp(typ),
                ]));
            }
            if let Some(rest_type) = rest {
                elements.push(SExp::List(vec![SExp::Atom("rest".to_string()), type_to_sexp(rest_type)]));
            }
            SExp::List(elements)
        }
        Type::Variant { variants, rest, span: _ } => {
            let mut elements = vec![SExp::Atom("variant".to_string())];
            for (name, typ) in variants {
                elements.push(SExp::List(vec![
                    SExp::Atom(name.as_str().to_string()),
                    type_to_sexp(typ),
                ]));
            }
            if let Some(rest_type) = rest {
                elements.push(SExp::List(vec![SExp::Atom("rest".to_string()), type_to_sexp(rest_type)]));
            }
            SExp::List(elements)
        }
        Type::Tuple { types, span: _ } => {
            let mut elements = vec![SExp::Atom("tuple".to_string())];
            for typ in types {
                elements.push(type_to_sexp(typ));
            }
            SExp::List(elements)
        }
        Type::Row { fields, rest, span: _ } => {
            let mut elements = vec![SExp::Atom("row".to_string())];
            for (name, typ) in fields {
                elements.push(SExp::List(vec![
                    SExp::Atom(name.as_str().to_string()),
                    type_to_sexp(typ),
                ]));
            }
            if let Some(rest_type) = rest {
                elements.push(SExp::List(vec![SExp::Atom("rest".to_string()), type_to_sexp(rest_type)]));
            }
            SExp::List(elements)
        }
        Type::Hole(_) => SExp::Atom("?".to_string()),
    }
}

fn literal_to_sexp(literal: &Literal) -> SExp {
    match literal {
        Literal::Integer(n) => SExp::Atom(n.to_string()),
        Literal::Float(f) => SExp::Atom(f.to_string()),
        Literal::String(s) => SExp::Atom(format!("\"{}\"", s)),
        Literal::Bool(b) => SExp::Atom(b.to_string()),
        Literal::Unit => SExp::Atom("()".to_string()),
    }
}

// S-expression to AST conversion functions (simplified stubs)

fn sexp_to_ast(sexp: &SExp) -> Result<CompilationUnit> {
    match sexp {
        SExp::List(list) if list.len() >= 2 => {
            if let (SExp::Atom(tag), module_sexp) = (&list[0], &list[1]) {
                if tag == "compilation-unit" {
                    let module = sexp_to_module(module_sexp)?;
                    return Ok(CompilationUnit {
                        module,
                        span: dummy_span(),
                    });
                }
            }
        }
        _ => {}
    }
    
    Err(Error::Parse {
        message: "Invalid S-expression for compilation unit".to_string(),
    })
}

fn sexp_to_module(sexp: &SExp) -> Result<Module> {
    match sexp {
        SExp::List(list) if list.len() >= 2 => {
            if let (SExp::Atom(tag), SExp::Atom(name)) = (&list[0], &list[1]) {
                if tag == "module" {
                    let module_path = ModulePath::single(Symbol::intern(name), dummy_span());
                    return Ok(Module {
                        name: module_path,
                        exports: None,
                        imports: Vec::new(),
                        items: Vec::new(),
                        span: dummy_span(),
                    });
                }
            }
        }
        _ => {}
    }
    
    Err(Error::Parse {
        message: "Invalid S-expression for module".to_string(),
    })
}

fn sexp_to_expr(sexp: &SExp) -> Result<Expr> {
    match sexp {
        SExp::Atom(atom) => {
            // Try to parse as number
            if let Ok(n) = atom.parse::<i64>() {
                return Ok(Expr::Literal(Literal::Integer(n), dummy_span()));
            }
            if let Ok(f) = atom.parse::<f64>() {
                return Ok(Expr::Literal(Literal::Float(f), dummy_span()));
            }
            if atom == "true" {
                return Ok(Expr::Literal(Literal::Bool(true), dummy_span()));
            }
            if atom == "false" {
                return Ok(Expr::Literal(Literal::Bool(false), dummy_span()));
            }
            if atom.starts_with('"') && atom.ends_with('"') {
                let string_val = atom[1..atom.len()-1].to_string();
                return Ok(Expr::Literal(Literal::String(string_val), dummy_span()));
            }
            
            // Variable
            Ok(Expr::Var(Symbol::intern(atom), dummy_span()))
        }
        SExp::List(list) if !list.is_empty() => {
            match &list[0] {
                SExp::Atom(tag) => {
                    match tag.as_str() {
                        "if" if list.len() == 4 => {
                            Ok(Expr::If {
                                condition: Box::new(sexp_to_expr(&list[1])?),
                                then_branch: Box::new(sexp_to_expr(&list[2])?),
                                else_branch: Box::new(sexp_to_expr(&list[3])?),
                                span: dummy_span(),
                            })
                        }
                        "lambda" if list.len() >= 3 => {
                            Ok(Expr::Lambda {
                                parameters: vec![Pattern::Wildcard(dummy_span())], // Simplified
                                body: Box::new(sexp_to_expr(&list[2])?),
                                span: dummy_span(),
                            })
                        }
                        _ => {
                            // Function application
                            let func = Box::new(sexp_to_expr(&list[0])?);
                            let mut args = Vec::new();
                            for arg_sexp in &list[1..] {
                                args.push(sexp_to_expr(arg_sexp)?);
                            }
                            Ok(Expr::App(func, args, dummy_span()))
                        }
                    }
                }
                _ => {
                    // Function application
                    let func = Box::new(sexp_to_expr(&list[0])?);
                    let mut args = Vec::new();
                    for arg_sexp in &list[1..] {
                        args.push(sexp_to_expr(arg_sexp)?);
                    }
                    Ok(Expr::App(func, args, dummy_span()))
                }
            }
        }
        SExp::List(_) => {
            Ok(Expr::Literal(Literal::Unit, dummy_span()))
        }
    }
}

fn dummy_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sexp_parser_creation() {
        let parser = SExpParser::new();
        assert_eq!(parser.syntax_style(), SyntaxStyle::SExp);
    }

    #[test]
    fn test_sexp_printer_creation() {
        let printer = SExpPrinter::new();
        assert_eq!(printer.syntax_style(), SyntaxStyle::SExp);
    }

    #[test]
    fn test_simple_atom_parsing() {
        let mut lexer = SExpLexer::new("hello", FileId::new(0));
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![SExpToken::Atom("hello".to_string()), SExpToken::Eof]);
    }

    #[test]
    fn test_simple_list_parsing() {
        let mut lexer = SExpLexer::new("(hello world)", FileId::new(0));
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![
            SExpToken::LeftParen,
            SExpToken::Atom("hello".to_string()),
            SExpToken::Atom("world".to_string()),
            SExpToken::RightParen,
            SExpToken::Eof
        ]);
    }

    #[test]
    fn test_expression_to_sexp() {
        let expr = Expr::Literal(Literal::Integer(42), dummy_span());
        let sexp = expr_to_sexp(&expr);
        assert_eq!(sexp, SExp::Atom("42".to_string()));
    }

    #[test]
    fn test_function_application_to_sexp() {
        let func = Expr::Var(Symbol::intern("f"), dummy_span());
        let arg = Expr::Literal(Literal::Integer(42), dummy_span());
        let app = Expr::App(Box::new(func), vec![arg], dummy_span());
        
        let sexp = expr_to_sexp(&app);
        assert_eq!(sexp, SExp::List(vec![
            SExp::Atom("f".to_string()),
            SExp::Atom("42".to_string()),
        ]));
    }

    #[test]
    fn test_sexp_printing() {
        let printer = SExpPrinter::new();
        let config = SyntaxConfig::default();
        
        let sexp = SExp::List(vec![
            SExp::Atom("f".to_string()),
            SExp::Atom("42".to_string()),
        ]);
        
        let result = printer.print_sexp(&sexp, &config, 0);
        assert_eq!(result, "(f 42)");
    }
}