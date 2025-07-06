//! Code validation and error detection
//! 
//! This module validates generated code and suggests improvements.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use x_parser::ast::*;
use x_parser::Symbol;
use x_checker::types::{Type as CheckerType};
use crate::{
    GeneratedCode, Suggestion, SuggestionKind, CodeLocation,
    context::CodeGenContext,
};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<Suggestion>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: CodeLocation,
    pub severity: Severity,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub kind: WarningKind,
    pub message: String,
    pub location: CodeLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UndefinedSymbol,
    TypeMismatch,
    PatternMismatch,
    MissingCase,
    RecursionWithoutBase,
    EffectNotHandled,
    SyntaxError,
}

#[derive(Debug, Clone)]
pub enum WarningKind {
    UnusedVariable,
    UnusedImport,
    ShadowedVariable,
    MissingTypeAnnotation,
    NonExhaustivePattern,
    ComplexityWarning,
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Code validator
pub struct CodeValidator {
    type_checker: TypeChecker,
    pattern_analyzer: PatternAnalyzer,
    effect_analyzer: EffectAnalyzer,
    complexity_analyzer: ComplexityAnalyzer,
}

/// Simple type checker for validation
struct TypeChecker {
    type_env: HashMap<Symbol, CheckerType>,
}

/// Pattern exhaustiveness analyzer
struct PatternAnalyzer;

/// Effect usage analyzer
struct EffectAnalyzer;

/// Complexity analyzer
struct ComplexityAnalyzer;

impl CodeValidator {
    pub fn new() -> Self {
        Self {
            type_checker: TypeChecker::new(),
            pattern_analyzer: PatternAnalyzer,
            effect_analyzer: EffectAnalyzer,
            complexity_analyzer: ComplexityAnalyzer,
        }
    }
    
    /// Validate generated code
    pub fn validate(&self, code: &GeneratedCode, context: &CodeGenContext) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        
        // Validate AST structure
        self.validate_ast(&code.ast, context, &mut errors, &mut warnings)?;
        
        // Check for undefined symbols
        self.check_undefined_symbols(&code.ast, context, &mut errors)?;
        
        // Analyze patterns
        self.analyze_patterns(&code.ast, &mut warnings)?;
        
        // Check effects
        self.check_effects(&code.ast, context, &mut errors)?;
        
        // Analyze complexity
        self.analyze_complexity(&code.ast, &mut warnings, &mut suggestions)?;
        
        // Generate improvement suggestions
        self.generate_suggestions(&code.ast, context, &mut suggestions)?;
        
        Ok(ValidationResult {
            errors,
            warnings,
            suggestions,
        })
    }
    
    /// Validate AST structure
    fn validate_ast(
        &self,
        ast: &CompilationUnit,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        // Walk through all items
        for item in &ast.module.items {
            self.validate_item(item, context, errors, warnings)?;
        }
        
        Ok(())
    }
    
    /// Validate a single item
    fn validate_item(
        &self,
        item: &Item,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        match item {
            Item::ValueDef(def) => {
                self.validate_value_def(def, context, errors, warnings)?;
            }
            Item::TypeDef(def) => {
                self.validate_type_def(def, errors)?;
            }
            Item::EffectDef(def) => {
                self.validate_effect_def(def, errors)?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Validate value definition
    fn validate_value_def(
        &self,
        def: &ValueDef,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        // Check for missing type annotation
        if def.type_annotation.is_none() && !def.parameters.is_empty() {
            warnings.push(ValidationWarning {
                kind: WarningKind::MissingTypeAnnotation,
                message: format!(
                    "Function '{}' lacks type annotation",
                    def.name.as_str()
                ),
                location: CodeLocation {
                    module: context.current_module
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_else(|| "Generated".to_string()),
                    item: def.name.as_str().to_string(),
                    line: None,
                },
            });
        }
        
        // Validate expression
        self.validate_expr(&def.body, context, errors, warnings)?;
        
        // Check for recursive functions without base case
        if self.is_recursive_without_base(&def.name, &def.body) {
            errors.push(ValidationError {
                kind: ErrorKind::RecursionWithoutBase,
                message: format!(
                    "Recursive function '{}' may lack a base case",
                    def.name.as_str()
                ),
                location: CodeLocation {
                    module: context.current_module
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_else(|| "Generated".to_string()),
                    item: def.name.as_str().to_string(),
                    line: None,
                },
                severity: Severity::Warning,
            });
        }
        
        Ok(())
    }
    
    /// Validate expression
    fn validate_expr(
        &self,
        expr: &Expr,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        match expr {
            Expr::Var(name, _) => {
                if !context.is_in_scope(*name) {
                    // Check if it's a built-in
                    let builtins = ["print", "print_endline", "+", "-", "*", "/", "==", "!="];
                    if !builtins.contains(&name.as_str()) {
                        errors.push(ValidationError {
                            kind: ErrorKind::UndefinedSymbol,
                            message: format!("Undefined variable '{}'", name.as_str()),
                            location: CodeLocation {
                                module: context.current_module
                                    .map(|s| s.as_str().to_string())
                                    .unwrap_or_else(|| "Generated".to_string()),
                                item: name.as_str().to_string(),
                                line: None,
                            },
                            severity: Severity::Error,
                        });
                    }
                }
            }
            Expr::Lambda { parameters, body, .. } => {
                // Check for unused parameters
                for param in parameters {
                    if let Pattern::Variable(name, _) = param {
                        if !self.is_used_in_expr(name, body) {
                            warnings.push(ValidationWarning {
                                kind: WarningKind::UnusedVariable,
                                message: format!("Unused parameter '{}'", name.as_str()),
                                location: CodeLocation {
                                    module: context.current_module
                                        .map(|s| s.as_str().to_string())
                                        .unwrap_or_else(|| "Generated".to_string()),
                                    item: "lambda".to_string(),
                                    line: None,
                                },
                            });
                        }
                    }
                }
                
                self.validate_expr(body, context, errors, warnings)?;
            }
            Expr::App(func, args, _) => {
                self.validate_expr(func, context, errors, warnings)?;
                for arg in args {
                    self.validate_expr(arg, context, errors, warnings)?;
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.validate_expr(scrutinee, context, errors, warnings)?;
                for arm in arms {
                    self.validate_expr(&arm.body, context, errors, warnings)?;
                }
            }
            Expr::Let { pattern, value, body, .. } => {
                self.validate_expr(value, context, errors, warnings)?;
                self.validate_expr(body, context, errors, warnings)?;
            }
            Expr::Let { pattern, value, body, .. } => {
                // Handle recursive let as nested lets
                self.validate_expr(value, context, errors, warnings)?;
                self.validate_expr(body, context, errors, warnings)?;
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.validate_expr(condition, context, errors, warnings)?;
                self.validate_expr(then_branch, context, errors, warnings)?;
                self.validate_expr(else_branch, context, errors, warnings)?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Validate type definition
    fn validate_type_def(
        &self,
        def: &TypeDef,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        match &def.kind {
            TypeDefKind::Data(constructors) => {
                // Check for duplicate constructor names
                let mut seen = HashSet::new();
                for ctor in constructors {
                    if !seen.insert(ctor.name) {
                        errors.push(ValidationError {
                            kind: ErrorKind::SyntaxError,
                            message: format!(
                                "Duplicate constructor '{}' in type '{}'",
                                ctor.name.as_str(),
                                def.name.as_str()
                            ),
                            location: CodeLocation {
                                module: "Generated".to_string(),
                                item: def.name.as_str().to_string(),
                                line: None,
                            },
                            severity: Severity::Error,
                        });
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Validate effect definition
    fn validate_effect_def(
        &self,
        def: &EffectDef,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        // Check for duplicate operation names
        let mut seen = HashSet::new();
        for op in &def.operations {
            if !seen.insert(op.name) {
                errors.push(ValidationError {
                    kind: ErrorKind::SyntaxError,
                    message: format!(
                        "Duplicate operation '{}' in effect '{}'",
                        op.name.as_str(),
                        def.name.as_str()
                    ),
                    location: CodeLocation {
                        module: "Generated".to_string(),
                        item: def.name.as_str().to_string(),
                        line: None,
                    },
                    severity: Severity::Error,
                });
            }
        }
        
        Ok(())
    }
    
    /// Check for undefined symbols
    fn check_undefined_symbols(
        &self,
        ast: &CompilationUnit,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        let mut defined_symbols = HashSet::new();
        
        // Collect all defined symbols
        for item in &ast.module.items {
            match item {
                Item::ValueDef(def) => {
                    defined_symbols.insert(def.name);
                }
                Item::TypeDef(def) => {
                    defined_symbols.insert(def.name);
                    if let TypeDefKind::Data(ctors) = &def.kind {
                        for ctor in ctors {
                            defined_symbols.insert(ctor.name);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Check for undefined references
        // (This would require a more sophisticated AST walker)
        
        Ok(())
    }
    
    /// Analyze pattern exhaustiveness
    fn analyze_patterns(
        &self,
        ast: &CompilationUnit,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        // Walk through all match expressions
        for item in &ast.module.items {
            if let Item::ValueDef(def) = item {
                self.analyze_patterns_in_expr(&def.body, warnings)?;
            }
        }
        
        Ok(())
    }
    
    /// Analyze patterns in expression
    fn analyze_patterns_in_expr(
        &self,
        expr: &Expr,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<()> {
        match expr {
            Expr::Match { arms, .. } => {
                // Simple exhaustiveness check
                let has_wildcard = arms.iter().any(|arm| {
                    matches!(arm.pattern, Pattern::Wildcard(_))
                });
                
                if !has_wildcard && arms.len() < 3 {
                    warnings.push(ValidationWarning {
                        kind: WarningKind::NonExhaustivePattern,
                        message: "Pattern match may not be exhaustive".to_string(),
                        location: CodeLocation {
                            module: "Generated".to_string(),
                            item: "match".to_string(),
                            line: None,
                        },
                    });
                }
                
                for arm in arms {
                    self.analyze_patterns_in_expr(&arm.body, warnings)?;
                }
            }
            Expr::Lambda { body, .. } => {
                self.analyze_patterns_in_expr(body, warnings)?;
            }
            Expr::App(func, args, _) => {
                self.analyze_patterns_in_expr(func, warnings)?;
                for arg in args {
                    self.analyze_patterns_in_expr(arg, warnings)?;
                }
            }
            Expr::Let { value, body, .. } => {
                self.analyze_patterns_in_expr(value, warnings)?;
                self.analyze_patterns_in_expr(body, warnings)?;
            }
            // LetRec handled in Let case above
            _ => {}
        }
        
        Ok(())
    }
    
    /// Check for unhandled effects
    fn check_effects(
        &self,
        ast: &CompilationUnit,
        context: &CodeGenContext,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        // This would require effect tracking through the AST
        Ok(())
    }
    
    /// Analyze complexity
    fn analyze_complexity(
        &self,
        ast: &CompilationUnit,
        warnings: &mut Vec<ValidationWarning>,
        suggestions: &mut Vec<Suggestion>,
    ) -> Result<()> {
        for item in &ast.module.items {
            if let Item::ValueDef(def) = item {
                let complexity = self.estimate_complexity(&def.body);
                
                if complexity > 10 {
                    warnings.push(ValidationWarning {
                        kind: WarningKind::ComplexityWarning,
                        message: format!(
                            "Function '{}' has high cyclomatic complexity ({})",
                            def.name.as_str(),
                            complexity
                        ),
                        location: CodeLocation {
                            module: "Generated".to_string(),
                            item: def.name.as_str().to_string(),
                            line: None,
                        },
                    });
                    
                    suggestions.push(Suggestion {
                        kind: SuggestionKind::Style,
                        description: "Consider breaking this function into smaller parts".to_string(),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate improvement suggestions
    fn generate_suggestions(
        &self,
        ast: &CompilationUnit,
        context: &CodeGenContext,
        suggestions: &mut Vec<Suggestion>,
    ) -> Result<()> {
        for item in &ast.module.items {
            match item {
                Item::ValueDef(def) => {
                    // Suggest type annotations
                    if def.type_annotation.is_none() && !def.parameters.is_empty() {
                        suggestions.push(Suggestion {
                            kind: SuggestionKind::TypeAnnotation,
                            description: format!(
                                "Add type annotation to function '{}'",
                                def.name.as_str()
                            ),
                        });
                    }
                    
                    // Suggest documentation
                    suggestions.push(Suggestion {
                        kind: SuggestionKind::Documentation,
                        description: format!(
                            "Add documentation comment to '{}'",
                            def.name.as_str()
                        ),
                    });
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Suggest fixes for validation errors
    pub fn suggest_fixes(
        &self,
        code: &GeneratedCode,
        validation: &ValidationResult,
    ) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();
        
        for error in &validation.errors {
            match error.kind {
                ErrorKind::UndefinedSymbol => {
                    suggestions.push(Suggestion {
                        kind: SuggestionKind::ErrorHandling,
                        description: format!(
                            "Import or define symbol '{}'",
                            error.location.item
                        ),
                    });
                }
                ErrorKind::RecursionWithoutBase => {
                    suggestions.push(Suggestion {
                        kind: SuggestionKind::ErrorHandling,
                        description: "Add base case to recursive function".to_string(),
                    });
                }
                _ => {}
            }
        }
        
        Ok(suggestions)
    }
    
    /// Check if expression is recursive without base case
    fn is_recursive_without_base(&self, name: &Symbol, expr: &Expr) -> bool {
        let has_recursive_call = self.contains_call_to(name, expr);
        let has_conditional = self.has_conditional(expr);
        
        has_recursive_call && !has_conditional
    }
    
    /// Check if expression contains a call to the given function
    fn contains_call_to(&self, name: &Symbol, expr: &Expr) -> bool {
        match expr {
            Expr::Var(n, _) if n == name => true,
            Expr::App(func, args, _) => {
                self.contains_call_to(name, func) ||
                args.iter().any(|arg| self.contains_call_to(name, arg))
            }
            Expr::Lambda { body, .. } => self.contains_call_to(name, body),
            Expr::Let { value, body, .. } => {
                self.contains_call_to(name, value) || self.contains_call_to(name, body)
            }
            // LetRec handled in Let case above
            Expr::Match { scrutinee, arms, .. } => {
                self.contains_call_to(name, scrutinee) ||
                arms.iter().any(|arm| self.contains_call_to(name, &arm.body))
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.contains_call_to(name, condition) ||
                self.contains_call_to(name, then_branch) ||
                self.contains_call_to(name, else_branch)
            }
            _ => false,
        }
    }
    
    /// Check if expression has a conditional
    fn has_conditional(&self, expr: &Expr) -> bool {
        match expr {
            Expr::If { .. } | Expr::Match { .. } => true,
            Expr::Lambda { body, .. } => self.has_conditional(body),
            Expr::Let { body, .. } => self.has_conditional(body),
            // LetRec handled in Let case above
            _ => false,
        }
    }
    
    /// Check if variable is used in expression
    fn is_used_in_expr(&self, name: &Symbol, expr: &Expr) -> bool {
        match expr {
            Expr::Var(n, _) => n == name,
            Expr::App(func, args, _) => {
                self.is_used_in_expr(name, func) ||
                args.iter().any(|arg| self.is_used_in_expr(name, arg))
            }
            Expr::Lambda { body, .. } => self.is_used_in_expr(name, body),
            Expr::Let { value, body, .. } => {
                self.is_used_in_expr(name, value) || self.is_used_in_expr(name, body)
            }
            // LetRec handled in Let case above
            Expr::Match { scrutinee, arms, .. } => {
                self.is_used_in_expr(name, scrutinee) ||
                arms.iter().any(|arm| self.is_used_in_expr(name, &arm.body))
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.is_used_in_expr(name, condition) ||
                self.is_used_in_expr(name, then_branch) ||
                self.is_used_in_expr(name, else_branch)
            }
            _ => false,
        }
    }
    
    /// Estimate cyclomatic complexity
    fn estimate_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::If { condition, then_branch, else_branch, .. } => {
                1 + self.estimate_complexity(condition) +
                self.estimate_complexity(then_branch) +
                self.estimate_complexity(else_branch)
            }
            Expr::Match { scrutinee, arms, .. } => {
                arms.len() + self.estimate_complexity(scrutinee) +
                arms.iter().map(|arm| self.estimate_complexity(&arm.body)).sum::<usize>()
            }
            Expr::Lambda { body, .. } => self.estimate_complexity(body),
            Expr::Let { value, body, .. } => {
                self.estimate_complexity(value) + self.estimate_complexity(body)
            }
            // LetRec handled in Let case above
            Expr::App(func, args, _) => {
                self.estimate_complexity(func) +
                args.iter().map(|arg| self.estimate_complexity(arg)).sum::<usize>()
            }
            _ => 0,
        }
    }
}

impl ValidationResult {
    /// Check if there are any issues
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
    
    /// Check if there are critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.errors.iter().any(|e| matches!(e.severity, Severity::Error))
    }
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            type_env: HashMap::new(),
        }
    }
}