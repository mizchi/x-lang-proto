//! Code refinement and interactive improvement
//! 
//! This module handles code refinement based on validation results and user feedback.

use anyhow::{Result, bail};
use std::collections::HashMap;
use x_parser::ast::*;
use x_parser::{Symbol, Span, FileId, span::ByteOffset};
use crate::{
    GeneratedCode, ValidationResult,
    intent::{RefinementIntent, RefinementAction},
    context::CodeGenContext,
    validator::{ValidationError, ErrorKind},
};

/// Code refiner
pub struct CodeRefiner {
    file_id: FileId,
    transformation_rules: TransformationRules,
}

/// Transformation rules for code improvement
struct TransformationRules {
    error_fixes: Vec<ErrorFix>,
}

/// Error fix rule
struct ErrorFix {
    error_kind: ErrorKind,
    fix: fn(FileId, &ValidationError, &CompilationUnit) -> Result<CompilationUnit>,
}

impl CodeRefiner {
    pub fn new() -> Self {
        Self {
            file_id: FileId::new(0),
            transformation_rules: TransformationRules::new(),
        }
    }
    
    fn span(&self) -> Span {
        Span::new(self.file_id, ByteOffset::new(0), ByteOffset::new(1))
    }
    
    /// Refine code based on validation results
    pub fn refine(
        &self,
        mut code: GeneratedCode,
        validation: &ValidationResult,
    ) -> Result<GeneratedCode> {
        // Apply error fixes
        for error in &validation.errors {
            if let Some(fixed_ast) = self.fix_error(error, &code.ast)? {
                code.ast = fixed_ast;
            }
        }
        
        // Update metadata
        code.metadata.explanation.push_str("\n\nCode has been refined based on validation results.");
        
        Ok(code)
    }
    
    /// Apply feedback from user
    pub fn apply_feedback(
        &self,
        mut code: GeneratedCode,
        intent: &RefinementIntent,
        context: &CodeGenContext,
    ) -> Result<GeneratedCode> {
        match &intent.action {
            RefinementAction::ChangeType => {
                code.ast = self.change_type(&code.ast, &intent.target, &intent.details)?;
            }
            RefinementAction::AddParameter => {
                code.ast = self.add_parameter(&code.ast, &intent.target, &intent.details)?;
            }
            RefinementAction::RemoveParameter => {
                code.ast = self.remove_parameter(&code.ast, &intent.target, &intent.details)?;
            }
            RefinementAction::RenameItem => {
                code.ast = self.rename_item(&code.ast, &intent.target, &intent.details)?;
            }
            RefinementAction::AddCase => {
                code.ast = self.add_match_case(&code.ast, &intent.target, &intent.details)?;
            }
            RefinementAction::FixError => {
                code.ast = self.fix_specific_error(&code.ast, &intent.details)?;
            }
            RefinementAction::ImprovePerformance => {
                code.ast = self.improve_performance(&code.ast, &intent.details)?;
            }
            RefinementAction::Clarify => {
                code.ast = self.clarify_code(&code.ast, &intent.details)?;
            }
        }
        
        Ok(code)
    }
    
    /// Fix a specific error
    fn fix_error(
        &self,
        error: &ValidationError,
        ast: &CompilationUnit,
    ) -> Result<Option<CompilationUnit>> {
        for fix_rule in &self.transformation_rules.error_fixes {
            if fix_rule.error_kind == error.kind {
                return (fix_rule.fix)(self.file_id, error, ast).map(Some);
            }
        }
        
        Ok(None)
    }
    
    /// Change type annotation
    fn change_type(
        &self,
        ast: &CompilationUnit,
        target: &Option<String>,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        if let Some(target_name) = target {
            for item in &mut module.items {
                if let Item::ValueDef(def) = item {
                    if def.name.as_str() == target_name {
                        // Parse the new type from details
                        def.type_annotation = Some(self.parse_type_annotation(details)?);
                    }
                }
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Add parameter to function
    fn add_parameter(
        &self,
        ast: &CompilationUnit,
        target: &Option<String>,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        if let Some(func_name) = target {
            for item in &mut module.items {
                if let Item::ValueDef(def) = item {
                    if def.name.as_str() == func_name {
                        let param_name = self.extract_parameter_name(details)?;
                        let param_pattern = Pattern::Variable(
                            Symbol::intern(&param_name),
                            self.span(),
                        );
                        
                        // Update body to add parameter
                        match &mut def.body {
                            Expr::Lambda { parameters, .. } => {
                                parameters.push(param_pattern);
                            }
                            _ => {
                                // Wrap in lambda
                                def.body = Expr::Lambda {
                                    parameters: vec![param_pattern],
                                    body: Box::new(def.body.clone()),
                                    span: self.span(),
                                };
                            }
                        }
                    }
                }
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Remove parameter from function
    fn remove_parameter(
        &self,
        ast: &CompilationUnit,
        target: &Option<String>,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        if let Some(func_name) = target {
            let param_to_remove = self.extract_parameter_name(details)?;
            
            for item in &mut module.items {
                if let Item::ValueDef(def) = item {
                    if def.name.as_str() == func_name {
                        if let Expr::Lambda { parameters, body, span } = &mut def.body {
                            parameters.retain(|p| {
                                if let Pattern::Variable(name, _) = p {
                                    name.as_str() != param_to_remove
                                } else {
                                    true
                                }
                            });
                            
                            if parameters.is_empty() {
                                def.body = *body.clone();
                            }
                        }
                    }
                }
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Rename an item
    fn rename_item(
        &self,
        ast: &CompilationUnit,
        target: &Option<String>,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        if let Some(old_name) = target {
            let new_name = self.extract_new_name(details)?;
            let new_symbol = Symbol::intern(&new_name);
            
            // Rename in definitions
            for item in &mut module.items {
                match item {
                    Item::ValueDef(def) if def.name.as_str() == old_name => {
                        def.name = new_symbol;
                    }
                    Item::TypeDef(def) if def.name.as_str() == old_name => {
                        def.name = new_symbol;
                    }
                    _ => {}
                }
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Add a case to pattern match
    fn add_match_case(
        &self,
        ast: &CompilationUnit,
        target: &Option<String>,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        // Add default case to all match expressions
        for item in &mut module.items {
            if let Item::ValueDef(def) = item {
                def.body = self.add_case_to_expr(&def.body, details)?;
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Fix specific error based on description
    fn fix_specific_error(
        &self,
        ast: &CompilationUnit,
        details: &str,
    ) -> Result<CompilationUnit> {
        // Apply heuristics based on error description
        if details.contains("undefined") {
            self.fix_undefined_symbols(ast, details)
        } else if details.contains("type") {
            self.add_type_annotations(ast)
        } else {
            Ok(ast.clone())
        }
    }
    
    /// Improve performance
    fn improve_performance(
        &self,
        ast: &CompilationUnit,
        details: &str,
    ) -> Result<CompilationUnit> {
        // Currently no performance optimizations implemented
        Ok(ast.clone())
    }
    
    /// Clarify code
    fn clarify_code(
        &self,
        ast: &CompilationUnit,
        details: &str,
    ) -> Result<CompilationUnit> {
        // Currently no clarification implemented
        Ok(ast.clone())
    }
    
    /// Helper: Add case to match expressions
    fn add_case_to_expr(&self, expr: &Expr, case_details: &str) -> Result<Expr> {
        Ok(match expr {
            Expr::Match { scrutinee, arms, span } => {
                let mut new_arms = arms.clone();
                
                // Add a default case if not present
                let has_wildcard = arms.iter().any(|arm| {
                    matches!(arm.pattern, Pattern::Wildcard(_))
                });
                
                if !has_wildcard {
                    new_arms.push(MatchArm {
                        pattern: Pattern::Wildcard(self.span()),
                        guard: None,
                        body: Expr::Literal(Literal::String("unmatched case".to_string()), self.span()),
                        span: self.span(),
                    });
                }
                
                Expr::Match {
                    scrutinee: scrutinee.clone(),
                    arms: new_arms,
                    span: *span,
                }
            }
            Expr::Lambda { parameters, body, span } => {
                Expr::Lambda {
                    parameters: parameters.clone(),
                    body: Box::new(self.add_case_to_expr(body, case_details)?),
                    span: *span,
                }
            }
            Expr::Let { pattern, type_annotation, value, body, span } => {
                Expr::Let {
                    pattern: pattern.clone(),
                    type_annotation: type_annotation.clone(),
                    value: value.clone(),
                    body: Box::new(self.add_case_to_expr(body, case_details)?),
                    span: *span,
                }
            }
            _ => expr.clone(),
        })
    }
    
    /// Helper: Fix undefined symbols
    fn fix_undefined_symbols(
        &self,
        ast: &CompilationUnit,
        details: &str,
    ) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        // Extract undefined symbol name
        let undefined_symbol = details
            .split_whitespace()
            .find(|word| word.starts_with('\'') && word.ends_with('\''))
            .and_then(|s| s.strip_prefix('\'')?.strip_suffix('\''))
            .unwrap_or("undefined");
        
        // Add a stub definition
        let stub = Item::ValueDef(ValueDef {
            name: Symbol::intern(undefined_symbol),
            type_annotation: None,
            parameters: Vec::new(),
            body: Expr::Literal(
                Literal::String(format!("TODO: implement {}", undefined_symbol)),
                self.span()
            ),
            visibility: Visibility::Private,
            purity: Purity::Inferred,
            span: self.span(),
        });
        
        module.items.push(stub);
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Helper: Add type annotations
    fn add_type_annotations(&self, ast: &CompilationUnit) -> Result<CompilationUnit> {
        let mut module = ast.module.clone();
        
        for item in &mut module.items {
            if let Item::ValueDef(def) = item {
                if def.type_annotation.is_none() {
                    // Infer simple types
                    def.type_annotation = self.infer_type_annotation(&def.body);
                }
            }
        }
        
        Ok(CompilationUnit {
            module,
            span: ast.span,
        })
    }
    
    /// Helper: Parse type annotation from string
    fn parse_type_annotation(&self, type_str: &str) -> Result<Type> {
        // Simplified type parsing
        let type_str = type_str.trim();
        
        if type_str.contains("->") {
            // Function type
            let parts: Vec<&str> = type_str.split("->").collect();
            if parts.len() == 2 {
                let param_type = self.parse_type_annotation(parts[0])?;
                let return_type = self.parse_type_annotation(parts[1])?;
                
                return Ok(Type::Fun {
                    params: vec![param_type],
                    return_type: Box::new(return_type),
                    effects: EffectSet::empty(self.span()),
                    span: self.span(),
                });
            }
        }
        
        // Simple type constructor
        Ok(Type::Con(Symbol::intern(type_str), self.span()))
    }
    
    /// Helper: Infer simple type annotation
    fn infer_type_annotation(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Literal(lit, span) => match lit {
                Literal::Integer(_) => Some(Type::Con(Symbol::intern("Int"), *span)),
                Literal::Float(_) => Some(Type::Con(Symbol::intern("Float"), *span)),
                Literal::String(_) => Some(Type::Con(Symbol::intern("String"), *span)),
                Literal::Bool(_) => Some(Type::Con(Symbol::intern("Bool"), *span)),
                Literal::Unit => Some(Type::Con(Symbol::intern("Unit"), *span)),
            },
            _ => None,
        }
    }
    
    /// Helper: Extract parameter name from details
    fn extract_parameter_name(&self, details: &str) -> Result<String> {
        // Look for quoted name or word after "parameter"
        if let Some(start) = details.find('\'') {
            if let Some(end) = details[start + 1..].find('\'') {
                return Ok(details[start + 1..start + 1 + end].to_string());
            }
        }
        
        if let Some(pos) = details.find("parameter") {
            let after = &details[pos + 9..];
            if let Some(word) = after.split_whitespace().next() {
                return Ok(word.trim_matches(|c: char| !c.is_alphanumeric()).to_string());
            }
        }
        
        Ok("new_param".to_string())
    }
    
    /// Helper: Extract new name from rename details
    fn extract_new_name(&self, details: &str) -> Result<String> {
        // Look for "to <name>" pattern
        if let Some(pos) = details.find(" to ") {
            let after = &details[pos + 4..];
            if let Some(word) = after.split_whitespace().next() {
                return Ok(word.trim_matches(|c: char| !c.is_alphanumeric()).to_string());
            }
        }
        
        bail!("Could not extract new name from: {}", details)
    }
}

impl TransformationRules {
    fn new() -> Self {
        Self {
            error_fixes: Self::init_error_fixes(),
        }
    }
    
    fn init_error_fixes() -> Vec<ErrorFix> {
        vec![
            ErrorFix {
                error_kind: ErrorKind::UndefinedSymbol,
                fix: |file_id, error, ast| {
                    let span = Span::new(file_id, ByteOffset::new(0), ByteOffset::new(1));
                    let mut module = ast.module.clone();
                    
                    // Add import or definition for undefined symbol
                    let undefined_name = &error.location.item;
                    
                    // Check if it's a common function
                    let common_imports = [
                        ("map", "List"),
                        ("filter", "List"),
                        ("fold_left", "List"),
                        ("fold_right", "List"),
                    ];
                    
                    for (func, module_name) in &common_imports {
                        if undefined_name == func {
                            // Add import
                            module.imports.push(Import {
                                module_path: ModulePath::single(
                                    Symbol::intern(module_name),
                                    span,
                                ),
                                kind: ImportKind::Wildcard,
                                alias: None,
                                span,
                            });
                            
                            return Ok(CompilationUnit {
                                module,
                                span: ast.span,
                            });
                        }
                    }
                    
                    // Otherwise add a stub
                    module.items.push(Item::ValueDef(ValueDef {
                        name: Symbol::intern(undefined_name),
                        type_annotation: None,
                        parameters: Vec::new(),
                        body: Expr::Literal(Literal::Unit, span),
                        visibility: Visibility::Private,
                        purity: Purity::Inferred,
                        span,
                    }));
                    
                    Ok(CompilationUnit {
                        module,
                        span: ast.span,
                    })
                },
            },
            ErrorFix {
                error_kind: ErrorKind::RecursionWithoutBase,
                fix: |file_id, error, ast| {
                    let span = Span::new(file_id, ByteOffset::new(0), ByteOffset::new(1));
                    let mut module = ast.module.clone();
                    
                    // Find the recursive function and add a base case
                    for item in &mut module.items {
                        if let Item::ValueDef(def) = item {
                            if def.name.as_str() == error.location.item {
                                // Wrap body in a conditional with base case
                                def.body = Expr::If {
                                    condition: Box::new(Expr::Var(Symbol::intern("true"), span)),
                                    then_branch: Box::new(Expr::Literal(Literal::Unit, span)),
                                    else_branch: Box::new(def.body.clone()),
                                    span,
                                };
                            }
                        }
                    }
                    
                    Ok(CompilationUnit {
                        module,
                        span: ast.span,
                    })
                },
            },
        ]
    }
}