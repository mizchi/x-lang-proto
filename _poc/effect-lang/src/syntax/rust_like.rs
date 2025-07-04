//! Rust-like syntax parser and printer
//! 
//! This provides a Rust-like syntax with explicit types, pattern matching,
//! and familiar syntax for Rust developers.

use super::{SyntaxParser, SyntaxPrinter, SyntaxStyle, SyntaxConfig};
use crate::core::{ast::*, span::{FileId, Span, ByteOffset}, symbol::Symbol};
use crate::{Error, Result};

/// Rust-like parser
pub struct RustLikeParser;

impl RustLikeParser {
    pub fn new() -> Self {
        RustLikeParser
    }
}

impl SyntaxParser for RustLikeParser {
    fn parse(&mut self, input: &str, file_id: FileId) -> Result<CompilationUnit> {
        // Stub implementation - full parser would be needed
        let module = Module {
            name: ModulePath::single(Symbol::intern("main"), dummy_span()),
            exports: None,
            imports: Vec::new(),
            items: Vec::new(),
            span: dummy_span(),
        };
        
        Ok(CompilationUnit {
            module,
            span: dummy_span(),
        })
    }
    
    fn parse_expression(&mut self, input: &str, file_id: FileId) -> Result<Expr> {
        // Stub implementation
        Ok(Expr::Literal(Literal::Unit, dummy_span()))
    }
    
    fn syntax_style(&self) -> SyntaxStyle {
        SyntaxStyle::RustLike
    }
}

/// Rust-like printer
pub struct RustLikePrinter;

impl RustLikePrinter {
    pub fn new() -> Self {
        RustLikePrinter
    }
    
    fn indent(&self, level: usize, config: &SyntaxConfig) -> String {
        if config.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * config.indent_size)
        }
    }
}

impl SyntaxPrinter for RustLikePrinter {
    fn print(&self, ast: &CompilationUnit, config: &SyntaxConfig) -> Result<String> {
        let mut output = String::new();
        output.push_str(&self.print_module(&ast.module, config, 0)?);
        Ok(output)
    }
    
    fn print_expression(&self, expr: &Expr, config: &SyntaxConfig) -> Result<String> {
        self.print_expr(expr, config, 0)
    }
    
    fn print_type(&self, typ: &Type, config: &SyntaxConfig) -> Result<String> {
        self.print_type_impl(typ, config)
    }
    
    fn syntax_style(&self) -> SyntaxStyle {
        SyntaxStyle::RustLike
    }
}

impl RustLikePrinter {
    fn print_module(&self, module: &Module, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        
        // In Rust style, modules are more implicit
        // We'll start with use statements (imports)
        for import in &module.imports {
            output.push_str(&self.print_import(import, config)?);
            output.push('\n');
        }
        
        if !module.imports.is_empty() {
            output.push('\n');
        }
        
        // Items
        for (i, item) in module.items.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&self.print_item(item, config, level)?);
            output.push('\n');
        }
        
        Ok(output)
    }
    
    fn print_import(&self, import: &Import, config: &SyntaxConfig) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("use ");
        
        let module_path = import.module_path.to_string().replace(".", "::");
        output.push_str(&module_path);
        
        match &import.kind {
            ImportKind::Qualified => {
                // Nothing extra
            }
            ImportKind::Selective(items) => {
                output.push_str("::{");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&item.name.as_str());
                }
                output.push_str("}");
            }
            ImportKind::Wildcard => {
                output.push_str("::*");
            }
            ImportKind::Lazy => {
                // Rust doesn't have lazy imports
            }
            ImportKind::Conditional(_) => {
                // Rust doesn't have conditional imports (cfg aside)
            }
            ImportKind::Interface { interface, items } => {
                // Map to use statement with comment
                output.push_str(&format!(" /* interface: {} */", interface));
                if !items.is_empty() {
                    output.push_str(" /* items */");
                }
            }
            ImportKind::Core { module, items } => {
                // Map to use statement with comment
                output.push_str(&format!(" /* core: {} */", module));
                if !items.is_empty() {
                    output.push_str(" /* items */");
                }
            }
            ImportKind::Func { module, name, signature: _ } => {
                // Map to extern declaration
                output.push_str(&format!(" /* extern func from {}: {} */", module, name));
            }
        }
        
        if let Some(alias) = &import.alias {
            output.push_str(&format!(" as {}", alias.as_str()));
        }
        
        output.push(';');
        
        Ok(output)
    }
    
    fn print_item(&self, item: &Item, config: &SyntaxConfig, level: usize) -> Result<String> {
        match item {
            Item::ValueDef(def) => self.print_value_def(def, config, level),
            Item::TypeDef(def) => self.print_type_def(def, config, level),
            Item::EffectDef(def) => self.print_effect_def(def, config, level),
            Item::HandlerDef(def) => self.print_handler_def(def, config, level),
            Item::ModuleTypeDef(_) => Ok("// Module type definitions not supported in Rust syntax".to_string()),
            Item::InterfaceDef(def) => Ok(format!("// Interface '{}' not supported in Rust syntax", def.name)),
        }
    }
    
    fn print_value_def(&self, def: &ValueDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        
        // Visibility
        match &def.visibility {
            Visibility::Public => output.push_str("pub "),
            Visibility::Private => {}, // Default in Rust
            Visibility::Crate => output.push_str("pub(crate) "),
            Visibility::Package => output.push_str("pub(crate) "), // Map to crate for Rust
            Visibility::Super => output.push_str("pub(super) "),
            Visibility::SelfModule => output.push_str("pub(self) "),
            Visibility::InPath(path) => output.push_str(&format!("pub(in {}) ", path.to_string())),
            Visibility::Component { .. } => output.push_str("pub "), // Map to pub for Rust
        }
        
        output.push_str("fn ");
        output.push_str(&def.name.as_str());
        output.push('(');
        
        // Parameters
        for (i, param) in def.parameters.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&self.print_pattern(param, config)?);
            output.push_str(": ");
            // In a real implementation, we'd need parameter types
            output.push_str("_"); // Placeholder
        }
        
        output.push(')');
        
        // Return type
        if let Some(typ) = &def.type_annotation {
            output.push_str(" -> ");
            output.push_str(&self.print_type_impl(typ, config)?);
        }
        
        output.push_str(" {\n");
        output.push_str(&self.indent(level + 1, config));
        output.push_str(&self.print_expr(&def.body, config, level + 1)?);
        output.push('\n');
        output.push_str(&self.indent(level, config));
        output.push('}');
        
        Ok(output)
    }
    
    fn print_type_def(&self, def: &TypeDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        
        // Visibility
        match &def.visibility {
            Visibility::Public => output.push_str("pub "),
            Visibility::Private => {},
            Visibility::Crate => output.push_str("pub(crate) "),
            Visibility::Package => output.push_str("pub(crate) "), // Map to crate for Rust
            Visibility::Super => output.push_str("pub(super) "),
            Visibility::SelfModule => output.push_str("pub(self) "),
            Visibility::InPath(path) => output.push_str(&format!("pub(in {}) ", path.to_string())),
            Visibility::Component { .. } => output.push_str("pub "), // Map to pub for Rust
        }
        
        match &def.kind {
            TypeDefKind::Alias(typ) => {
                output.push_str("type ");
                output.push_str(&def.name.as_str());
                
                // Type parameters
                if !def.type_params.is_empty() {
                    output.push('<');
                    for (i, param) in def.type_params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&param.name.as_str());
                    }
                    output.push('>');
                }
                
                output.push_str(" = ");
                output.push_str(&self.print_type_impl(typ, config)?);
                output.push(';');
            }
            TypeDefKind::Data(constructors) => {
                output.push_str("enum ");
                output.push_str(&def.name.as_str());
                
                // Type parameters
                if !def.type_params.is_empty() {
                    output.push('<');
                    for (i, param) in def.type_params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&param.name.as_str());
                    }
                    output.push('>');
                }
                
                output.push_str(" {\n");
                
                for constructor in constructors {
                    output.push_str(&self.indent(level + 1, config));
                    output.push_str(&constructor.name.as_str());
                    
                    if !constructor.fields.is_empty() {
                        if constructor.fields.len() == 1 {
                            // Tuple-like variant
                            output.push('(');
                            output.push_str(&self.print_type_impl(&constructor.fields[0], config)?);
                            output.push(')');
                        } else {
                            // Tuple with multiple fields
                            output.push('(');
                            for (i, field) in constructor.fields.iter().enumerate() {
                                if i > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&self.print_type_impl(field, config)?);
                            }
                            output.push(')');
                        }
                    }
                    
                    output.push_str(",\n");
                }
                
                output.push_str(&self.indent(level, config));
                output.push('}');
            }
            TypeDefKind::Abstract => {
                // Rust doesn't have abstract types in the same way
                output.push_str("// Abstract type ");
                output.push_str(&def.name.as_str());
            }
        }
        
        Ok(output)
    }
    
    fn print_effect_def(&self, def: &EffectDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        
        // In Rust, effects might be represented as traits
        output.push_str("trait ");
        output.push_str(&def.name.as_str());
        output.push_str(" {\n");
        
        for operation in &def.operations {
            output.push_str(&self.indent(level + 1, config));
            output.push_str("fn ");
            output.push_str(&operation.name.as_str());
            output.push('(');
            
            // Parameters
            output.push_str("&self");
            for (i, param) in operation.parameters.iter().enumerate() {
                output.push_str(", ");
                output.push_str(&format!("arg{}: ", i));
                output.push_str(&self.print_type_impl(param, config)?);
            }
            
            output.push(')');
            
            // Return type
            output.push_str(" -> ");
            output.push_str(&self.print_type_impl(&operation.return_type, config)?);
            output.push_str(";\n");
        }
        
        output.push_str(&self.indent(level, config));
        output.push('}');
        
        Ok(output)
    }
    
    fn print_handler_def(&self, def: &HandlerDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        
        // In Rust, handlers might be trait implementations
        output.push_str("impl ");
        if !def.handled_effects.is_empty() {
            output.push_str(&def.handled_effects[0].name.as_str());
            output.push_str(" for ");
        }
        output.push_str(&def.name.as_str());
        output.push_str(" {\n");
        
        for handler in &def.handlers {
            output.push_str(&self.indent(level + 1, config));
            output.push_str("fn ");
            output.push_str(&handler.operation.as_str());
            output.push_str("(&self");
            
            for (i, param) in handler.parameters.iter().enumerate() {
                output.push_str(", ");
                output.push_str(&self.print_pattern(param, config)?);
                output.push_str(": _"); // Type would be inferred from trait
            }
            
            output.push_str(") -> _ {\n");
            output.push_str(&self.indent(level + 2, config));
            output.push_str(&self.print_expr(&handler.body, config, level + 2)?);
            output.push('\n');
            output.push_str(&self.indent(level + 1, config));
            output.push_str("}\n");
        }
        
        output.push_str(&self.indent(level, config));
        output.push('}');
        
        Ok(output)
    }
    
    fn print_expr(&self, expr: &Expr, config: &SyntaxConfig, level: usize) -> Result<String> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.print_literal(lit)),
            Expr::Var(name, _) => Ok(name.as_str().to_string()),
            Expr::App(func, args, _) => {
                let mut output = String::new();
                
                // Check if this is an infix operator
                if let Expr::Var(op_name, _) = func.as_ref() {
                    let op_str = op_name.as_str();
                    if args.len() == 2 && self.is_infix_operator(op_str) {
                        // Print as infix
                        let left_str = self.print_expr(&args[0], config, level)?;
                        let right_str = self.print_expr(&args[1], config, level)?;
                        
                        if self.needs_parens_in_infix(&args[0]) {
                            output.push_str(&format!("({})", left_str));
                        } else {
                            output.push_str(&left_str);
                        }
                        
                        output.push_str(&format!(" {} ", op_str));
                        
                        if self.needs_parens_in_infix(&args[1]) {
                            output.push_str(&format!("({})", right_str));
                        } else {
                            output.push_str(&right_str);
                        }
                        
                        return Ok(output);
                    }
                }
                
                // Print as function call
                output.push_str(&self.print_expr(func, config, level)?);
                output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.print_expr(arg, config, level)?);
                }
                output.push(')');
                Ok(output)
            }
            Expr::Lambda { parameters, body, span: _ } => {
                let mut output = String::new();
                output.push_str("|");
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.print_pattern(param, config)?);
                }
                output.push_str("| ");
                output.push_str(&self.print_expr(body, config, level)?);
                Ok(output)
            }
            Expr::Let { pattern, type_annotation, value, body, span: _ } => {
                let mut output = String::new();
                output.push_str("{\n");
                output.push_str(&self.indent(level + 1, config));
                output.push_str("let ");
                output.push_str(&self.print_pattern(pattern, config)?);
                
                if let Some(typ) = type_annotation {
                    output.push_str(": ");
                    output.push_str(&self.print_type_impl(typ, config)?);
                }
                
                output.push_str(" = ");
                output.push_str(&self.print_expr(value, config, level + 1)?);
                output.push_str(";\n");
                output.push_str(&self.indent(level + 1, config));
                output.push_str(&self.print_expr(body, config, level + 1)?);
                output.push('\n');
                output.push_str(&self.indent(level, config));
                output.push('}');
                Ok(output)
            }
            Expr::If { condition, then_branch, else_branch, span: _ } => {
                let mut output = String::new();
                output.push_str("if ");
                output.push_str(&self.print_expr(condition, config, level)?);
                output.push_str(" {\n");
                output.push_str(&self.indent(level + 1, config));
                output.push_str(&self.print_expr(then_branch, config, level + 1)?);
                output.push('\n');
                output.push_str(&self.indent(level, config));
                output.push_str("} else {\n");
                output.push_str(&self.indent(level + 1, config));
                output.push_str(&self.print_expr(else_branch, config, level + 1)?);
                output.push('\n');
                output.push_str(&self.indent(level, config));
                output.push('}');
                Ok(output)
            }
            Expr::Match { scrutinee, arms, span: _ } => {
                let mut output = String::new();
                output.push_str("match ");
                output.push_str(&self.print_expr(scrutinee, config, level)?);
                output.push_str(" {\n");
                
                for arm in arms {
                    output.push_str(&self.indent(level + 1, config));
                    output.push_str(&self.print_pattern(&arm.pattern, config)?);
                    
                    if let Some(guard) = &arm.guard {
                        output.push_str(" if ");
                        output.push_str(&self.print_expr(guard, config, level + 1)?);
                    }
                    
                    output.push_str(" => {\n");
                    output.push_str(&self.indent(level + 2, config));
                    output.push_str(&self.print_expr(&arm.body, config, level + 2)?);
                    output.push('\n');
                    output.push_str(&self.indent(level + 1, config));
                    output.push_str("},\n");
                }
                
                output.push_str(&self.indent(level, config));
                output.push('}');
                Ok(output)
            }
            Expr::Do { statements, span: _ } => {
                let mut output = String::new();
                output.push_str("{\n");
                
                for (i, statement) in statements.iter().enumerate() {
                    output.push_str(&self.indent(level + 1, config));
                    match statement {
                        DoStatement::Let { pattern, expr, span: _ } => {
                            output.push_str("let ");
                            output.push_str(&self.print_pattern(pattern, config)?);
                            output.push_str(" = ");
                            output.push_str(&self.print_expr(expr, config, level + 1)?);
                            output.push(';');
                        }
                        DoStatement::Bind { pattern, expr, span: _ } => {
                            output.push_str("let ");
                            output.push_str(&self.print_pattern(pattern, config)?);
                            output.push_str(" = ");
                            output.push_str(&self.print_expr(expr, config, level + 1)?);
                            output.push_str(".await?;"); // Rust-like async
                        }
                        DoStatement::Expr(expr) => {
                            let expr_str = self.print_expr(expr, config, level + 1)?;
                            output.push_str(&expr_str);
                            if i == statements.len() - 1 {
                                // Last expression doesn't need semicolon
                            } else {
                                output.push(';');
                            }
                        }
                    }
                    output.push('\n');
                }
                
                output.push_str(&self.indent(level, config));
                output.push('}');
                Ok(output)
            }
            Expr::Handle { expr, handlers, return_clause, span: _ } => {
                // Rust doesn't have algebraic effects, so we use a comment
                let mut output = String::new();
                output.push_str("/* handle ");
                output.push_str(&self.print_expr(expr, config, level)?);
                output.push_str(" with handlers */");
                output.push_str(&self.print_expr(expr, config, level)?);
                Ok(output)
            }
            Expr::Resume { value, span: _ } => {
                Ok(format!("resume({})", self.print_expr(value, config, level)?))
            }
            Expr::Perform { effect, operation, args, span: _ } => {
                let mut output = format!("{}::{}", effect.as_str(), operation.as_str());
                output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.print_expr(arg, config, level)?);
                }
                output.push(')');
                Ok(output)
            }
            Expr::Ann { expr, type_annotation, span: _ } => {
                Ok(format!("({}: {})", 
                    self.print_expr(expr, config, level)?,
                    self.print_type_impl(type_annotation, config)?
                ))
            }
        }
    }
    
    fn print_pattern(&self, pattern: &Pattern, config: &SyntaxConfig) -> Result<String> {
        match pattern {
            Pattern::Wildcard(_) => Ok("_".to_string()),
            Pattern::Variable(name, _) => Ok(name.as_str().to_string()),
            Pattern::Literal(lit, _) => Ok(self.print_literal(lit)),
            Pattern::Constructor { name, args, span: _ } => {
                let mut output = format!("{}::{}", "EnumName", name.as_str()); // Simplified
                if !args.is_empty() {
                    output.push('(');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&self.print_pattern(arg, config)?);
                    }
                    output.push(')');
                }
                Ok(output)
            }
            Pattern::Record { fields, rest, span: _ } => {
                let mut output = String::new();
                output.push_str("StructName { "); // Simplified
                
                let mut first = true;
                for (name, pattern) in fields {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(&format!("{}: {}", name.as_str(), self.print_pattern(pattern, config)?));
                }
                
                if let Some(_) = rest {
                    if !first {
                        output.push_str(", ");
                    }
                    output.push_str("..");
                }
                
                output.push_str(" }");
                Ok(output)
            }
            Pattern::Tuple { patterns, span: _ } => {
                let mut output = String::new();
                output.push('(');
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.print_pattern(pattern, config)?);
                }
                output.push(')');
                Ok(output)
            }
            Pattern::Or { left, right, span: _ } => {
                Ok(format!("{} | {}", 
                    self.print_pattern(left, config)?,
                    self.print_pattern(right, config)?
                ))
            }
            Pattern::As { pattern, name, span: _ } => {
                Ok(format!("{} @ {}", 
                    self.print_pattern(pattern, config)?,
                    name.as_str()
                ))
            }
            Pattern::Ann { pattern, type_annotation, span: _ } => {
                Ok(format!("{}: {}",
                    self.print_pattern(pattern, config)?,
                    self.print_type_impl(type_annotation, config)?
                ))
            }
        }
    }
    
    fn print_type_impl(&self, typ: &Type, config: &SyntaxConfig) -> Result<String> {
        match typ {
            Type::Var(name, _) => Ok(name.as_str().to_string()),
            Type::Con(name, _) => {
                // Map common types to Rust equivalents
                match name.as_str() {
                    "Int" => Ok("i32".to_string()),
                    "Float" => Ok("f64".to_string()),
                    "String" => Ok("String".to_string()),
                    "Bool" => Ok("bool".to_string()),
                    "Unit" => Ok("()".to_string()),
                    _ => Ok(name.as_str().to_string()),
                }
            }
            Type::App(typ, args, _) => {
                let base_str = self.print_type_impl(typ, config)?;
                
                // Handle special cases
                if let Type::Con(name, _) = typ.as_ref() {
                    match name.as_str() {
                        "List" if args.len() == 1 => {
                            return Ok(format!("Vec<{}>", self.print_type_impl(&args[0], config)?));
                        }
                        "Option" if args.len() == 1 => {
                            return Ok(format!("Option<{}>", self.print_type_impl(&args[0], config)?));
                        }
                        "Result" if args.len() == 2 => {
                            return Ok(format!("Result<{}, {}>", 
                                self.print_type_impl(&args[0], config)?,
                                self.print_type_impl(&args[1], config)?
                            ));
                        }
                        _ => {}
                    }
                }
                
                // Generic type application
                let mut output = base_str;
                if !args.is_empty() {
                    output.push('<');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&self.print_type_impl(arg, config)?);
                    }
                    output.push('>');
                }
                Ok(output)
            }
            Type::Fun { params, return_type, effects: _, span: _ } => {
                let mut output = String::new();
                
                // Rust function types are more complex, using Fn traits
                if params.len() == 1 {
                    output.push_str("fn(");
                    output.push_str(&self.print_type_impl(&params[0], config)?);
                    output.push_str(") -> ");
                    output.push_str(&self.print_type_impl(return_type, config)?);
                } else {
                    output.push_str("fn(");
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&self.print_type_impl(param, config)?);
                    }
                    output.push_str(") -> ");
                    output.push_str(&self.print_type_impl(return_type, config)?);
                }
                
                Ok(output)
            }
            Type::Forall { type_params: _, body, span: _ } => {
                // Rust doesn't have explicit forall types
                self.print_type_impl(body, config)
            }
            Type::Effects(_, _) => {
                // Effects in Rust might be represented differently
                Ok("()".to_string()) // Simplified
            }
            Type::Exists { type_params: _, body, span: _ } => {
                // Rust doesn't have existential types in the same way
                self.print_type_impl(body, config)
            }
            Type::Record { fields, rest: _, span: _ } => {
                // Rust structs
                let mut output = String::new();
                output.push_str("struct { ");
                
                let mut first = true;
                for (name, typ) in fields {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(&format!("{}: {}", name.as_str(), self.print_type_impl(typ, config)?));
                }
                
                output.push_str(" }");
                Ok(output)
            }
            Type::Variant { variants: _, rest: _, span: _ } => {
                Ok("enum { /* variants */ }".to_string()) // Simplified
            }
            Type::Tuple { types, span: _ } => {
                let mut output = String::new();
                output.push('(');
                for (i, typ) in types.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.print_type_impl(typ, config)?);
                }
                if types.len() == 1 {
                    output.push(','); // Single element tuple in Rust
                }
                output.push(')');
                Ok(output)
            }
            Type::Row { fields: _, rest: _, span: _ } => {
                Ok("/* row type */".to_string()) // Simplified
            }
            Type::Hole(_) => Ok("_".to_string()),
        }
    }
    
    fn print_literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::Integer(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Bool(b) => b.to_string(),
            Literal::Unit => "()".to_string(),
        }
    }
    
    fn is_infix_operator(&self, op: &str) -> bool {
        matches!(op, "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||")
    }
    
    fn needs_parens_in_infix(&self, expr: &Expr) -> bool {
        matches!(expr, 
            Expr::Lambda { .. } | 
            Expr::Let { .. } | 
            Expr::If { .. } | 
            Expr::Match { .. }
        )
    }
}

fn dummy_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_like_parser_creation() {
        let parser = RustLikeParser::new();
        assert_eq!(parser.syntax_style(), SyntaxStyle::RustLike);
    }

    #[test]
    fn test_rust_like_printer_creation() {
        let printer = RustLikePrinter::new();
        assert_eq!(printer.syntax_style(), SyntaxStyle::RustLike);
    }

    #[test]
    fn test_rust_type_mapping() {
        let printer = RustLikePrinter::new();
        let config = SyntaxConfig::default();
        
        let int_type = Type::Con(Symbol::intern("Int"), dummy_span());
        let result = printer.print_type_impl(&int_type, &config).unwrap();
        assert_eq!(result, "i32");
        
        let string_type = Type::Con(Symbol::intern("String"), dummy_span());
        let result = printer.print_type_impl(&string_type, &config).unwrap();
        assert_eq!(result, "String");
        
        let bool_type = Type::Con(Symbol::intern("Bool"), dummy_span());
        let result = printer.print_type_impl(&bool_type, &config).unwrap();
        assert_eq!(result, "bool");
    }

    #[test]
    fn test_rust_list_type() {
        let printer = RustLikePrinter::new();
        let config = SyntaxConfig::default();
        
        let list_type = Type::App(
            Box::new(Type::Con(Symbol::intern("List"), dummy_span())),
            vec![Type::Con(Symbol::intern("Int"), dummy_span())],
            dummy_span()
        );
        
        let result = printer.print_type_impl(&list_type, &config).unwrap();
        assert_eq!(result, "Vec<i32>");
    }

    #[test]
    fn test_rust_function_type() {
        let printer = RustLikePrinter::new();
        let config = SyntaxConfig::default();
        
        let fun_type = Type::Fun {
            params: vec![Type::Con(Symbol::intern("Int"), dummy_span())],
            return_type: Box::new(Type::Con(Symbol::intern("String"), dummy_span())),
            effects: EffectSet::empty(dummy_span()),
            span: dummy_span(),
        };
        
        let result = printer.print_type_impl(&fun_type, &config).unwrap();
        assert_eq!(result, "fn(i32) -> String");
    }

    #[test]
    fn test_rust_lambda_expression() {
        let printer = RustLikePrinter::new();
        let config = SyntaxConfig::default();
        
        let lambda = Expr::Lambda {
            parameters: vec![Pattern::Variable(Symbol::intern("x"), dummy_span())],
            body: Box::new(Expr::Var(Symbol::intern("x"), dummy_span())),
            span: dummy_span(),
        };
        
        let result = printer.print_expression(&lambda, &config).unwrap();
        assert_eq!(result, "|x| x");
    }

    #[test]
    fn test_rust_match_expression() {
        let printer = RustLikePrinter::new();
        let config = SyntaxConfig::default();
        
        let match_expr = Expr::Match {
            scrutinee: Box::new(Expr::Var(Symbol::intern("x"), dummy_span())),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard(dummy_span()),
                guard: None,
                body: Expr::Literal(Literal::Integer(42), dummy_span()),
                span: dummy_span(),
            }],
            span: dummy_span(),
        };
        
        let result = printer.print_expression(&match_expr, &config).unwrap();
        assert!(result.contains("match x"));
        assert!(result.contains("_ => {"));
        assert!(result.contains("42"));
    }
}