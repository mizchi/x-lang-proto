//! OCaml-style syntax parser and printer
//! 
//! This wraps the existing parser to conform to the SyntaxParser trait.

use super::{SyntaxParser, SyntaxPrinter, SyntaxStyle, SyntaxConfig};
use crate::{ast::*, span::FileId};
use crate::parser::Parser;
use crate::error::{ParseError as Error, Result};

/// OCaml-style parser wrapper
pub struct OCamlParser;

impl OCamlParser {
    pub fn new() -> Self {
        OCamlParser
    }
}

impl SyntaxParser for OCamlParser {
    fn parse(&mut self, input: &str, file_id: FileId) -> Result<CompilationUnit> {
        let mut parser = Parser::new(input, file_id)?;
        parser.parse()
    }
    
    fn parse_expression(&mut self, input: &str, file_id: FileId) -> Result<Expr> {
        let mut parser = Parser::new(input, file_id)?;
        parser.parse_expression_public()
    }
    
    fn syntax_style(&self) -> SyntaxStyle {
        SyntaxStyle::OCaml
    }
}

/// OCaml-style printer
pub struct OCamlPrinter;

impl OCamlPrinter {
    pub fn new() -> Self {
        OCamlPrinter
    }
    
    fn indent(&self, level: usize, config: &SyntaxConfig) -> String {
        if config.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * config.indent_size)
        }
    }
}

impl SyntaxPrinter for OCamlPrinter {
    fn print(&self, ast: &CompilationUnit, config: &SyntaxConfig) -> Result<String> {
        let mut output = String::new();
        
        // Print module header
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
        SyntaxStyle::OCaml
    }
}

impl OCamlPrinter {
    fn print_module(&self, module: &Module, config: &SyntaxConfig, level: usize) -> Result<String> {
        let mut output = String::new();
        let indent = self.indent(level, config);
        
        // Module declaration
        output.push_str(&format!("{}module {}", indent, module.name.to_string()));
        
        // Exports
        if let Some(exports) = &module.exports {
            output.push_str(" export (");
            for (i, export) in exports.items.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&export.name.as_str());
            }
            output.push_str(")");
        }
        
        output.push_str(" =\n");
        
        // Imports
        for import in &module.imports {
            output.push_str(&self.print_import(import, config, level + 1)?);
            output.push('\n');
        }
        
        if !module.imports.is_empty() && !module.items.is_empty() {
            output.push('\n');
        }
        
        // Items
        for (i, item) in module.items.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&self.print_item(item, config, level + 1)?);
        }
        
        Ok(output)
    }
    
    fn print_import(&self, import: &Import, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        output.push_str(&format!("{}import {}", indent, import.module_path.to_string()));
        
        match &import.kind {
            ImportKind::Qualified => {
                // Nothing extra
            }
            ImportKind::Selective(items) => {
                output.push_str(" (");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&item.name.as_str());
                }
                output.push_str(")");
            }
            ImportKind::Wildcard => {
                output.push_str(".*");
            }
            ImportKind::Lazy => {
                output.push_str(" lazy");
            }
            ImportKind::Conditional(_) => {
                output.push_str(" when <condition>");
            }
            ImportKind::Interface { .. } | ImportKind::Core { .. } | ImportKind::Func { .. } => {
                return Err(Error::parse("WebAssembly Component Model imports not supported in OCaml syntax"));
            }
        }
        
        if let Some(alias) = &import.alias {
            output.push_str(&format!(" as {}", alias.as_str()));
        }
        
        Ok(output)
    }
    
    fn print_item(&self, item: &Item, config: &SyntaxConfig, level: usize) -> Result<String> {
        match item {
            Item::ValueDef(def) => self.print_value_def(def, config, level),
            Item::TypeDef(def) => self.print_type_def(def, config, level),
            Item::EffectDef(def) => self.print_effect_def(def, config, level),
            Item::HandlerDef(def) => self.print_handler_def(def, config, level),
            Item::ModuleTypeDef(def) => self.print_module_type_def(def, config, level),
            Item::InterfaceDef(def) => self.print_interface_def(def, config, level),
            Item::TestDef(def) => Ok(format!("(* Test '{}' - tests not yet supported in pretty printer *)", def.name.as_str())),
        }
    }
    
    fn print_value_def(&self, def: &ValueDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        // Visibility
        output.push_str(&self.print_visibility(&def.visibility, &indent)?);
        
        output.push_str("let ");
        output.push_str(&def.name.as_str());
        
        // Parameters
        for param in &def.parameters {
            output.push(' ');
            output.push_str(&self.print_pattern(param, config)?);
        }
        
        // Type annotation
        if let Some(typ) = &def.type_annotation {
            output.push_str(" : ");
            output.push_str(&self.print_type_impl(typ, config)?);
        }
        
        output.push_str(" =\n");
        output.push_str(&self.print_expr(&def.body, config, level + 1)?);
        
        Ok(output)
    }
    
    fn print_type_def(&self, def: &TypeDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        // Visibility
        output.push_str(&self.print_visibility(&def.visibility, &indent)?);
        
        output.push_str("type ");
        output.push_str(&def.name.as_str());
        
        // Type parameters
        if !def.type_params.is_empty() {
            output.push('[');
            for (i, param) in def.type_params.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&param.name.as_str());
            }
            output.push(']');
        }
        
        match &def.kind {
            TypeDefKind::Alias(typ) => {
                output.push_str(" = ");
                output.push_str(&self.print_type_impl(typ, config)?);
            }
            TypeDefKind::Data(constructors) => {
                output.push_str(" =");
                for (i, constructor) in constructors.iter().enumerate() {
                    if i == 0 {
                        output.push_str("\n");
                        output.push_str(&self.indent(level + 1, config));
                        output.push_str("| ");
                    } else {
                        output.push_str("\n");
                        output.push_str(&self.indent(level + 1, config));
                        output.push_str("| ");
                    }
                    output.push_str(&constructor.name.as_str());
                    for field in &constructor.fields {
                        output.push(' ');
                        output.push_str(&self.print_type_impl(field, config)?);
                    }
                }
            }
            TypeDefKind::Abstract => {
                // Abstract types have no definition
            }
        }
        
        Ok(output)
    }
    
    fn print_effect_def(&self, def: &EffectDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        output.push_str(&format!("{}effect {}", indent, def.name.as_str()));
        
        if !def.operations.is_empty() {
            output.push_str(" =\n");
            for operation in &def.operations {
                output.push_str(&self.indent(level + 1, config));
                output.push_str(&format!("{} : ", operation.name.as_str()));
                
                for (i, param) in operation.parameters.iter().enumerate() {
                    if i > 0 {
                        output.push_str(" -> ");
                    }
                    output.push_str(&self.print_type_impl(param, config)?);
                }
                
                if !operation.parameters.is_empty() {
                    output.push_str(" -> ");
                }
                output.push_str(&self.print_type_impl(&operation.return_type, config)?);
                output.push('\n');
            }
        }
        
        Ok(output)
    }
    
    fn print_handler_def(&self, def: &HandlerDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        output.push_str(&format!("{}handler {}", indent, def.name.as_str()));
        
        if let Some(typ) = &def.type_annotation {
            output.push_str(" : ");
            output.push_str(&self.print_type_impl(typ, config)?);
        }
        
        output.push_str(" = {\n");
        
        for handler in &def.handlers {
            output.push_str(&self.indent(level + 1, config));
            output.push_str(&format!("{}.{}", handler.effect.name.as_str(), handler.operation.as_str()));
            
            for param in &handler.parameters {
                output.push(' ');
                output.push_str(&self.print_pattern(param, config)?);
            }
            
            if let Some(cont) = &handler.continuation {
                output.push_str(&format!(" k:{}", cont.as_str()));
            }
            
            output.push_str(" -> ");
            output.push_str(&self.print_expr(&handler.body, config, level + 2)?);
            output.push('\n');
        }
        
        if let Some(return_clause) = &def.return_clause {
            output.push_str(&self.indent(level + 1, config));
            output.push_str("return ");
            output.push_str(&self.print_pattern(&return_clause.parameter, config)?);
            output.push_str(" -> ");
            output.push_str(&self.print_expr(&return_clause.body, config, level + 2)?);
            output.push('\n');
        }
        
        output.push_str(&self.indent(level, config));
        output.push('}');
        
        Ok(output)
    }
    
    fn print_module_type_def(&self, def: &ModuleTypeDef, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        output.push_str(&format!("{}module type {} = sig\n", indent, def.name.as_str()));
        
        for item in &def.signature.items {
            output.push_str(&self.print_signature_item(item, config, level + 1)?);
            output.push('\n');
        }
        
        output.push_str(&self.indent(level, config));
        output.push_str("end");
        
        Ok(output)
    }
    
    fn print_signature_item(&self, item: &SignatureItem, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        
        match item {
            SignatureItem::TypeSig { name, type_params, kind: _, span: _ } => {
                let mut output = format!("{}type {}", indent, name.as_str());
                if !type_params.is_empty() {
                    output.push('[');
                    for (i, param) in type_params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&param.name.as_str());
                    }
                    output.push(']');
                }
                Ok(output)
            }
            SignatureItem::ValueSig { name, type_annotation, span: _ } => {
                Ok(format!("{}val {} : {}", indent, name.as_str(), self.print_type_impl(type_annotation, config)?))
            }
            SignatureItem::EffectSig { name, operations, span: _ } => {
                let mut output = format!("{}effect {}", indent, name.as_str());
                if !operations.is_empty() {
                    output.push_str(" with");
                    for operation in operations {
                        output.push_str(&format!("\n{}  {} : ", indent, operation.name.as_str()));
                        for (i, param) in operation.parameters.iter().enumerate() {
                            if i > 0 {
                                output.push_str(" -> ");
                            }
                            output.push_str(&self.print_type_impl(param, config)?);
                        }
                        if !operation.parameters.is_empty() {
                            output.push_str(" -> ");
                        }
                        output.push_str(&self.print_type_impl(&operation.return_type, config)?);
                    }
                }
                Ok(output)
            }
        }
    }
    
    fn print_expr(&self, expr: &Expr, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        
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
                        output.push_str(&self.print_expr(&args[0], config, 0)?);
                        output.push_str(&format!(" {} ", op_str));
                        output.push_str(&self.print_expr(&args[1], config, 0)?);
                        return Ok(output);
                    }
                }
                
                // Print as regular application
                output.push_str(&self.print_expr(func, config, 0)?);
                for arg in args {
                    output.push(' ');
                    let arg_str = self.print_expr(arg, config, 0)?;
                    if self.needs_parens(arg) {
                        output.push_str(&format!("({})", arg_str));
                    } else {
                        output.push_str(&arg_str);
                    }
                }
                Ok(output)
            }
            Expr::Lambda { parameters, body, span: _ } => {
                let mut output = String::new();
                output.push_str("fun ");
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    output.push_str(&self.print_pattern(param, config)?);
                }
                output.push_str(" -> ");
                output.push_str(&self.print_expr(body, config, 0)?);
                Ok(output)
            }
            Expr::Let { pattern, type_annotation, value, body, span: _ } => {
                let mut output = String::new();
                output.push_str(&format!("{}let ", indent));
                output.push_str(&self.print_pattern(pattern, config)?);
                
                if let Some(typ) = type_annotation {
                    output.push_str(" : ");
                    output.push_str(&self.print_type_impl(typ, config)?);
                }
                
                output.push_str(" = ");
                output.push_str(&self.print_expr(value, config, 0)?);
                output.push_str(" in\n");
                output.push_str(&self.print_expr(body, config, level)?);
                Ok(output)
            }
            Expr::If { condition, then_branch, else_branch, span: _ } => {
                let mut output = String::new();
                output.push_str("if ");
                output.push_str(&self.print_expr(condition, config, 0)?);
                output.push_str(" then\n");
                output.push_str(&self.print_expr(then_branch, config, level + 1)?);
                output.push_str(&format!("\n{}else\n", indent));
                output.push_str(&self.print_expr(else_branch, config, level + 1)?);
                Ok(output)
            }
            Expr::Match { scrutinee, arms, span: _ } => {
                let mut output = String::new();
                output.push_str("match ");
                output.push_str(&self.print_expr(scrutinee, config, 0)?);
                output.push_str(" with");
                
                for arm in arms {
                    output.push_str(&format!("\n{}| ", self.indent(level + 1, config)));
                    output.push_str(&self.print_pattern(&arm.pattern, config)?);
                    
                    if let Some(guard) = &arm.guard {
                        output.push_str(" when ");
                        output.push_str(&self.print_expr(guard, config, 0)?);
                    }
                    
                    output.push_str(" -> ");
                    output.push_str(&self.print_expr(&arm.body, config, level + 2)?);
                }
                
                Ok(output)
            }
            Expr::Do { statements, span: _ } => {
                let mut output = String::new();
                output.push_str("do {\n");
                
                for statement in statements {
                    match statement {
                        DoStatement::Let { pattern, expr, span: _ } => {
                            output.push_str(&format!("{}let ", self.indent(level + 1, config)));
                            output.push_str(&self.print_pattern(pattern, config)?);
                            output.push_str(" = ");
                            output.push_str(&self.print_expr(expr, config, 0)?);
                        }
                        DoStatement::Bind { pattern, expr, span: _ } => {
                            output.push_str(&format!("{}let ", self.indent(level + 1, config)));
                            output.push_str(&self.print_pattern(pattern, config)?);
                            output.push_str(" <- ");
                            output.push_str(&self.print_expr(expr, config, 0)?);
                        }
                        DoStatement::Expr(expr) => {
                            output.push_str(&self.indent(level + 1, config));
                            output.push_str(&self.print_expr(expr, config, 0)?);
                        }
                    }
                    output.push_str(";\n");
                }
                
                output.push_str(&format!("{}}}", indent));
                Ok(output)
            }
            Expr::Handle { expr, handlers, return_clause, span: _ } => {
                let mut output = String::new();
                output.push_str("handle ");
                output.push_str(&self.print_expr(expr, config, 0)?);
                output.push_str(" with {\n");
                
                for handler in handlers {
                    output.push_str(&format!("{}{}.{}", 
                        self.indent(level + 1, config),
                        handler.effect.name.as_str(),
                        handler.operation.as_str()
                    ));
                    
                    for param in &handler.parameters {
                        output.push(' ');
                        output.push_str(&self.print_pattern(param, config)?);
                    }
                    
                    if let Some(cont) = &handler.continuation {
                        output.push_str(&format!(" k:{}", cont.as_str()));
                    }
                    
                    output.push_str(" -> ");
                    output.push_str(&self.print_expr(&handler.body, config, level + 2)?);
                    output.push('\n');
                }
                
                if let Some(return_clause) = return_clause {
                    output.push_str(&format!("{}return ", self.indent(level + 1, config)));
                    output.push_str(&self.print_pattern(&return_clause.parameter, config)?);
                    output.push_str(" -> ");
                    output.push_str(&self.print_expr(&return_clause.body, config, level + 2)?);
                    output.push('\n');
                }
                
                output.push_str(&format!("{}}}", indent));
                Ok(output)
            }
            Expr::Resume { value, span: _ } => {
                Ok(format!("resume {}", self.print_expr(value, config, 0)?))
            }
            Expr::Perform { effect, operation, args, span: _ } => {
                let mut output = format!("{}.{}", effect.as_str(), operation.as_str());
                for arg in args {
                    output.push(' ');
                    let arg_str = self.print_expr(arg, config, 0)?;
                    if self.needs_parens(arg) {
                        output.push_str(&format!("({})", arg_str));
                    } else {
                        output.push_str(&arg_str);
                    }
                }
                Ok(output)
            }
            Expr::Ann { expr, type_annotation, span: _ } => {
                Ok(format!("({} : {})", 
                    self.print_expr(expr, config, 0)?,
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
                let mut output = name.as_str().to_string();
                for arg in args {
                    output.push(' ');
                    let arg_str = self.print_pattern(arg, config)?;
                    if self.pattern_needs_parens(arg) {
                        output.push_str(&format!("({})", arg_str));
                    } else {
                        output.push_str(&arg_str);
                    }
                }
                Ok(output)
            }
            Pattern::Record { fields, rest, span: _ } => {
                let mut output = String::new();
                output.push('{');
                
                let mut first = true;
                for (name, pattern) in fields {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(&format!("{} = {}", name.as_str(), self.print_pattern(pattern, config)?));
                }
                
                if let Some(rest_pattern) = rest {
                    if !first {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("..{}", self.print_pattern(rest_pattern, config)?));
                }
                
                output.push('}');
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
                Ok(format!("{} as {}", 
                    self.print_pattern(pattern, config)?,
                    name.as_str()
                ))
            }
            Pattern::Ann { pattern, type_annotation, span: _ } => {
                Ok(format!("({} : {})",
                    self.print_pattern(pattern, config)?,
                    self.print_type_impl(type_annotation, config)?
                ))
            }
        }
    }
    
    fn print_type_impl(&self, typ: &Type, config: &SyntaxConfig) -> Result<String> {
        match typ {
            Type::Var(name, _) => Ok(name.as_str().to_string()),
            Type::Con(name, _) => Ok(name.as_str().to_string()),
            Type::App(typ, args, _) => {
                let mut output = self.print_type_impl(typ, config)?;
                if !args.is_empty() {
                    output.push('[');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&self.print_type_impl(arg, config)?);
                    }
                    output.push(']');
                }
                Ok(output)
            }
            Type::Fun { params, return_type, effects, span: _ } => {
                let mut output = String::new();
                
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        output.push_str(" -> ");
                    }
                    output.push_str(&self.print_type_impl(param, config)?);
                }
                
                if !params.is_empty() {
                    output.push_str(" -> ");
                }
                
                output.push_str(&self.print_type_impl(return_type, config)?);
                
                if !effects.effects.is_empty() {
                    output.push_str(" <");
                    for (i, effect) in effects.effects.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&effect.name.as_str());
                        if !effect.args.is_empty() {
                            output.push('[');
                            for (j, arg) in effect.args.iter().enumerate() {
                                if j > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&self.print_type_impl(arg, config)?);
                            }
                            output.push(']');
                        }
                    }
                    if let Some(row_var) = &effects.row_var {
                        if !effects.effects.is_empty() {
                            output.push_str(" | ");
                        }
                        output.push_str(&row_var.as_str());
                    }
                    output.push('>');
                }
                
                Ok(output)
            }
            Type::Forall { type_params, body, span: _ } => {
                let mut output = String::new();
                output.push_str("forall ");
                for (i, param) in type_params.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    output.push_str(&param.name.as_str());
                }
                output.push_str(". ");
                output.push_str(&self.print_type_impl(body, config)?);
                Ok(output)
            }
            Type::Effects(effects, _) => {
                let mut output = String::new();
                output.push('<');
                for (i, effect) in effects.effects.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&effect.name.as_str());
                }
                if let Some(row_var) = &effects.row_var {
                    if !effects.effects.is_empty() {
                        output.push_str(" | ");
                    }
                    output.push_str(&row_var.as_str());
                }
                output.push('>');
                Ok(output)
            }
            Type::Exists { type_params, body, span: _ } => {
                let mut output = String::new();
                output.push_str("exists ");
                for (i, param) in type_params.iter().enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    output.push_str(&param.name.as_str());
                }
                output.push_str(". ");
                output.push_str(&self.print_type_impl(body, config)?);
                Ok(output)
            }
            Type::Record { fields, rest, span: _ } => {
                let mut output = String::new();
                output.push('{');
                
                let mut first = true;
                for (name, typ) in fields {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(&format!("{} : {}", name.as_str(), self.print_type_impl(typ, config)?));
                }
                
                if let Some(rest_type) = rest {
                    if !first {
                        output.push_str(" | ");
                    }
                    output.push_str(&self.print_type_impl(rest_type, config)?);
                }
                
                output.push('}');
                Ok(output)
            }
            Type::Variant { variants, rest, span: _ } => {
                let mut output = String::new();
                output.push('[');
                
                let mut first = true;
                for (name, typ) in variants {
                    if !first {
                        output.push_str(" | ");
                    }
                    first = false;
                    output.push_str(&format!("{} {}", name.as_str(), self.print_type_impl(typ, config)?));
                }
                
                if let Some(rest_type) = rest {
                    if !first {
                        output.push_str(" | ");
                    }
                    output.push_str(&self.print_type_impl(rest_type, config)?);
                }
                
                output.push(']');
                Ok(output)
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
                output.push(')');
                Ok(output)
            }
            Type::Row { fields, rest, span: _ } => {
                let mut output = String::new();
                output.push('{');
                
                let mut first = true;
                for (name, typ) in fields {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(&format!("{} : {}", name.as_str(), self.print_type_impl(typ, config)?));
                }
                
                if let Some(rest_type) = rest {
                    if !first {
                        output.push_str(" | ");
                    }
                    output.push_str(&self.print_type_impl(rest_type, config)?);
                }
                
                output.push_str(" | _}");
                Ok(output)
            }
            Type::Hole(_) => Ok("?".to_string()),
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
    
    fn needs_parens(&self, expr: &Expr) -> bool {
        matches!(expr, 
            Expr::App(_, _, _) | 
            Expr::Lambda { .. } | 
            Expr::Let { .. } | 
            Expr::If { .. } | 
            Expr::Match { .. }
        )
    }
    
    fn pattern_needs_parens(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Constructor { args, .. } if !args.is_empty() => true,
            Pattern::Or { .. } => true,
            Pattern::As { .. } => true,
            _ => false,
        }
    }
    
    /// Print visibility modifier with proper formatting
    fn print_visibility(&self, visibility: &Visibility, indent: &str) -> Result<String> {
        match visibility {
            Visibility::Private => Ok(indent.to_string()),
            Visibility::Public => Ok(format!("{}pub ", indent)),
            Visibility::Crate => Ok(format!("{}pub(crate) ", indent)),
            Visibility::Package => Ok(format!("{}pub(package) ", indent)),
            Visibility::Super => Ok(format!("{}pub(super) ", indent)),
            Visibility::InPath(path) => Ok(format!("{}pub(in {}) ", indent, path.to_string())),
            Visibility::SelfModule => Ok(format!("{}pub(self) ", indent)),
            Visibility::Component { export, import, interface } => {
                let mut output = indent.to_string();
                if *export {
                    output.push_str("export ");
                }
                if *import {
                    output.push_str("import ");
                }
                if let Some(interface_name) = interface {
                    output.push_str(&format!("interface({}) ", interface_name.as_str()));
                }
                Ok(output)
            }
        }
    }
    
    /// Print interface definition
    fn print_interface_def(&self, def: &ComponentInterface, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        let mut output = String::new();
        
        output.push_str(&format!("{}interface \"{}\" {{\n", indent, def.name));
        
        for item in &def.items {
            output.push_str(&self.print_interface_item(item, config, level + 1)?);
            output.push('\n');
        }
        
        output.push_str(&format!("{}}}", indent));
        Ok(output)
    }
    
    /// Print interface item
    fn print_interface_item(&self, item: &InterfaceItem, config: &SyntaxConfig, level: usize) -> Result<String> {
        let indent = self.indent(level, config);
        
        match item {
            InterfaceItem::Func { name, signature, .. } => {
                Ok(format!("{}func {} {}", indent, name.as_str(), self.print_function_signature(signature)?))
            }
            InterfaceItem::Type { name, definition, .. } => {
                let mut output = format!("{}type {}", indent, name.as_str());
                if let Some(typ) = definition {
                    output.push_str(" = ");
                    output.push_str(&self.print_type_impl(typ, config)?);
                }
                Ok(output)
            }
            InterfaceItem::Resource { name, methods, .. } => {
                let mut output = format!("{}resource {} {{\n", indent, name.as_str());
                for method in methods {
                    output.push_str(&self.indent(level + 1, config));
                    if method.is_constructor {
                        output.push_str("constructor ");
                    }
                    if method.is_static {
                        output.push_str("static ");
                    }
                    output.push_str(&format!("{} {}\n", method.name.as_str(), self.print_function_signature(&method.signature)?));
                }
                output.push_str(&format!("{}}}", indent));
                Ok(output)
            }
        }
    }
    
    /// Print function signature
    fn print_function_signature(&self, signature: &FunctionSignature) -> Result<String> {
        let mut output = String::new();
        
        if !signature.params.is_empty() {
            output.push_str("(param");
            for param in &signature.params {
                output.push(' ');
                output.push_str(&self.print_wasm_type(param));
            }
            output.push(')');
        }
        
        if !signature.results.is_empty() {
            if !signature.params.is_empty() {
                output.push(' ');
            }
            output.push_str("(result");
            for result in &signature.results {
                output.push(' ');
                output.push_str(&self.print_wasm_type(result));
            }
            output.push(')');
        }
        
        Ok(output)
    }
    
    /// Print WebAssembly type
    fn print_wasm_type(&self, wasm_type: &WasmType) -> String {
        match wasm_type {
            WasmType::I32 => "i32".to_string(),
            WasmType::I64 => "i64".to_string(),
            WasmType::F32 => "f32".to_string(),
            WasmType::F64 => "f64".to_string(),
            WasmType::V128 => "v128".to_string(),
            WasmType::FuncRef => "funcref".to_string(),
            WasmType::ExternRef => "externref".to_string(),
            WasmType::Named(name) => name.as_str().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, Span, ByteOffset};
    use crate::symbol::Symbol;

    #[test]
    fn test_ocaml_parser_creation() {
        let parser = OCamlParser::new();
        assert_eq!(parser.syntax_style(), SyntaxStyle::OCaml);
    }

    #[test]
    fn test_ocaml_printer_creation() {
        let printer = OCamlPrinter::new();
        assert_eq!(printer.syntax_style(), SyntaxStyle::OCaml);
    }

    #[test]
    fn test_simple_expression_printing() {
        let printer = OCamlPrinter::new();
        let config = SyntaxConfig::default();
        
        let expr = Expr::Literal(Literal::Integer(42), test_span());
        let result = printer.print_expression(&expr, &config).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_function_application_printing() {
        let printer = OCamlPrinter::new();
        let config = SyntaxConfig::default();
        
        let func = Expr::Var(Symbol::intern("f"), test_span());
        let arg = Expr::Literal(Literal::Integer(42), test_span());
        let app = Expr::App(Box::new(func), vec![arg], test_span());
        
        let result = printer.print_expression(&app, &config).unwrap();
        assert_eq!(result, "f 42");
    }

    #[test]
    fn test_infix_operator_printing() {
        let printer = OCamlPrinter::new();
        let config = SyntaxConfig::default();
        
        let left = Expr::Literal(Literal::Integer(1), test_span());
        let right = Expr::Literal(Literal::Integer(2), test_span());
        let op = Expr::Var(Symbol::intern("+"), test_span());
        let app = Expr::App(Box::new(op), vec![left, right], test_span());
        
        let result = printer.print_expression(&app, &config).unwrap();
        assert_eq!(result, "1 + 2");
    }

    fn test_span() -> Span {
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
    }
}