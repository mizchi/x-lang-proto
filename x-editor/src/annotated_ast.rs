//! Type-annotated AST for preserving inferred types
//! 
//! This module provides AST nodes annotated with type information
//! from the type checker, allowing preservation of inferred types.

use x_parser::ast::*;
use x_parser::{Symbol, Span};
use x_checker::types::{Type as InferredType, TypeScheme};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Type-annotated compilation unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCompilationUnit {
    pub module: AnnotatedModule,
    pub type_environment: TypeEnvironment,
    pub span: Span,
}

/// Type-annotated module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedModule {
    pub name: ModulePath,
    pub imports: Vec<Import>,
    pub items: Vec<AnnotatedItem>,
    pub exports: Option<ExportList>,
    pub span: Span,
}

/// Type-annotated item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotatedItem {
    ValueDef(AnnotatedValueDef),
    TypeDef(TypeDef),
    EffectDef(EffectDef),
    LetRec(Vec<AnnotatedValueDef>, Span),
    Open(ModulePath, Span),
}

/// Type-annotated value definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedValueDef {
    pub name: Symbol,
    pub type_annotation: Option<Type>,
    pub inferred_type: Option<TypeScheme>,
    pub parameters: Vec<AnnotatedPattern>,
    pub body: AnnotatedExpr,
    pub visibility: Visibility,
    pub purity: Purity,
    pub span: Span,
}

/// Type-annotated expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedExpr {
    pub expr: Expr,
    pub inferred_type: Option<InferredType>,
    pub span: Span,
}

/// Type-annotated pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedPattern {
    pub pattern: Pattern,
    pub inferred_type: Option<InferredType>,
    pub span: Span,
}

/// Type environment storing all type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEnvironment {
    /// Global type schemes for top-level definitions
    pub globals: HashMap<Symbol, TypeScheme>,
    
    /// Local type bindings
    pub locals: HashMap<Symbol, InferredType>,
    
    /// Type aliases
    pub aliases: HashMap<Symbol, InferredType>,
    
    /// Constructor types
    pub constructors: HashMap<Symbol, TypeScheme>,
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: HashMap::new(),
            aliases: HashMap::new(),
            constructors: HashMap::new(),
        }
    }
    
    /// Merge type information from type checker
    pub fn merge_from_checker(&mut self, _checker_result: &x_checker::CheckResult) {
        // This would extract type information from the checker's result
        // Implementation depends on x_checker's internal structure
    }
}

/// Annotator that adds type information to AST
pub struct TypeAnnotator {
    type_env: TypeEnvironment,
}

impl Default for TypeAnnotator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeAnnotator {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnvironment::new(),
        }
    }
    
    /// Annotate AST with type information
    pub fn annotate(
        &mut self,
        ast: &CompilationUnit,
        type_check_result: &x_checker::CheckResult,
    ) -> AnnotatedCompilationUnit {
        // Extract type information from check result
        self.extract_types(type_check_result);
        
        // Annotate the AST
        AnnotatedCompilationUnit {
            module: self.annotate_module(&ast.module),
            type_environment: self.type_env.clone(),
            span: ast.span,
        }
    }
    
    /// Extract types from check result
    fn extract_types(&mut self, _result: &x_checker::CheckResult) {
        // This would extract type information from the type checker's result
        // The actual implementation depends on CheckResult's structure
        
        // For now, we'll just note that this is where the extraction happens
        // In practice, this would iterate through the result and populate type_env
    }
    
    /// Annotate module
    fn annotate_module(&self, module: &Module) -> AnnotatedModule {
        AnnotatedModule {
            name: module.name.clone(),
            imports: module.imports.clone(),
            items: module.items.iter().map(|item| self.annotate_item(item)).collect(),
            exports: module.exports.clone(),
            span: module.span,
        }
    }
    
    /// Annotate item
    fn annotate_item(&self, item: &Item) -> AnnotatedItem {
        match item {
            Item::ValueDef(def) => AnnotatedItem::ValueDef(self.annotate_value_def(def)),
            Item::TypeDef(def) => AnnotatedItem::TypeDef(def.clone()),
            Item::EffectDef(def) => AnnotatedItem::EffectDef(def.clone()),
            Item::HandlerDef(_) => panic!("Handler definitions not yet supported in annotated AST"),
            Item::ModuleTypeDef(_) => panic!("Module type definitions not yet supported in annotated AST"),
            Item::InterfaceDef(_) => panic!("Interface definitions not yet supported in annotated AST"),
            Item::TestDef(_) => panic!("Test definitions not yet supported in annotated AST"),
        }
    }
    
    /// Annotate value definition
    fn annotate_value_def(&self, def: &ValueDef) -> AnnotatedValueDef {
        // Look up inferred type for this definition
        let inferred_type = self.type_env.globals.get(&def.name).cloned();
        
        AnnotatedValueDef {
            name: def.name,
            type_annotation: def.type_annotation.clone(),
            inferred_type,
            parameters: def.parameters.iter()
                .map(|p| self.annotate_pattern(p))
                .collect(),
            body: self.annotate_expr(&def.body),
            visibility: def.visibility.clone(),
            purity: def.purity.clone(),
            span: def.span,
        }
    }
    
    /// Annotate expression
    fn annotate_expr(&self, expr: &Expr) -> AnnotatedExpr {
        // In a real implementation, we would look up the inferred type
        // for this specific expression node
        let inferred_type = self.infer_expr_type(expr);
        
        AnnotatedExpr {
            expr: expr.clone(),
            inferred_type,
            span: expr.span(),
        }
    }
    
    /// Annotate pattern
    fn annotate_pattern(&self, pattern: &Pattern) -> AnnotatedPattern {
        let inferred_type = self.infer_pattern_type(pattern);
        
        AnnotatedPattern {
            pattern: pattern.clone(),
            inferred_type,
            span: pattern.span(),
        }
    }
    
    /// Infer type for expression (simplified)
    fn infer_expr_type(&self, expr: &Expr) -> Option<InferredType> {
        match expr {
            Expr::Literal(lit, _) => Some(self.literal_type(lit)),
            Expr::Var(name, _) => {
                // Look up in locals first, then globals
                self.type_env.locals.get(name).cloned()
                    .or_else(|| self.type_env.globals.get(name).map(|ts| {
                        // Instantiate type scheme
                        self.instantiate_type_scheme(ts)
                    }))
            }
            _ => None, // Would handle other cases in full implementation
        }
    }
    
    /// Infer type for pattern
    fn infer_pattern_type(&self, pattern: &Pattern) -> Option<InferredType> {
        match pattern {
            Pattern::Variable(name, _) => self.type_env.locals.get(name).cloned(),
            Pattern::Literal(lit, _) => Some(self.literal_type(lit)),
            _ => None,
        }
    }
    
    /// Get type for literal
    fn literal_type(&self, lit: &Literal) -> InferredType {
        match lit {
            Literal::Integer(_) => InferredType::Con(Symbol::intern("Int")),
            Literal::Float(_) => InferredType::Con(Symbol::intern("Float")),
            Literal::String(_) => InferredType::Con(Symbol::intern("String")),
            Literal::Bool(_) => InferredType::Con(Symbol::intern("Bool")),
            Literal::Unit => InferredType::Con(Symbol::intern("Unit")),
        }
    }
    
    /// Instantiate a type scheme (simplified)
    fn instantiate_type_scheme(&self, scheme: &TypeScheme) -> InferredType {
        // In a real implementation, this would properly instantiate
        // the type scheme with fresh type variables
        scheme.body.clone()
    }
}

/// Serialization format for annotated AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAnnotatedAST {
    /// The annotated AST
    pub ast: AnnotatedCompilationUnit,
    
    /// Metadata about the annotation
    pub metadata: AnnotationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationMetadata {
    /// Version of the type checker used
    pub type_checker_version: String,
    
    /// Timestamp of annotation
    pub annotated_at: chrono::DateTime<chrono::Utc>,
    
    /// Whether all types were successfully inferred
    pub fully_typed: bool,
    
    /// Number of type errors (if any)
    pub type_errors: usize,
    
    /// Additional flags
    pub flags: HashMap<String, bool>,
}

/// Convert annotated AST back to regular AST
impl AnnotatedCompilationUnit {
    pub fn to_ast(&self) -> CompilationUnit {
        CompilationUnit {
            module: self.module.to_ast(),
            span: self.span,
        }
    }
    
    /// Get all inferred types as a map
    pub fn get_type_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        // Add global types
        for (name, scheme) in &self.type_environment.globals {
            map.insert(
                name.as_str().to_string(),
                format!("{scheme:?}"), // Would use proper pretty-printing
            );
        }
        
        // Add from annotated items
        for item in &self.module.items {
            if let AnnotatedItem::ValueDef(def) = item {
                if let Some(ref inferred) = def.inferred_type {
                    map.insert(
                        def.name.as_str().to_string(),
                        format!("{inferred:?}"),
                    );
                }
            }
        }
        
        map
    }
}

impl AnnotatedModule {
    fn to_ast(&self) -> Module {
        Module {
            name: self.name.clone(),
            documentation: None,
            imports: self.imports.clone(),
            items: self.items.iter().map(|item| item.to_ast()).collect(),
            exports: self.exports.clone(),
            span: self.span,
        }
    }
}

impl AnnotatedItem {
    fn to_ast(&self) -> Item {
        match self {
            AnnotatedItem::ValueDef(def) => Item::ValueDef(def.to_ast()),
            AnnotatedItem::TypeDef(def) => Item::TypeDef(def.clone()),
            AnnotatedItem::EffectDef(def) => Item::EffectDef(def.clone()),
            AnnotatedItem::LetRec(_, _) => panic!("LetRec not supported in regular AST"),
            AnnotatedItem::Open(_, _) => panic!("Open not supported in regular AST"),
        }
    }
}

impl AnnotatedValueDef {
    pub fn to_ast(&self) -> ValueDef {
        ValueDef {
            name: self.name,
            documentation: None,
            type_annotation: self.type_annotation.clone()
                .or_else(|| self.inferred_type.as_ref().map(|ts| {
                    // Convert inferred type to AST type annotation
                    self.type_scheme_to_ast_type(ts)
                })),
            parameters: self.parameters.iter().map(|p| p.pattern.clone()).collect(),
            body: self.body.expr.clone(),
            visibility: self.visibility.clone(),
            purity: self.purity.clone(),
            imports: Vec::new(),
            span: self.span,
        }
    }
    
    fn type_scheme_to_ast_type(&self, scheme: &TypeScheme) -> Type {
        // Simplified conversion - would be more complex in practice
        match &scheme.body {
            InferredType::Con(name) => Type::Con(*name, self.span),
            InferredType::Fun { params, return_type, effects: _ } => Type::Fun {
                params: params.iter().map(|p| self.inferred_to_ast_type(p)).collect(),
                return_type: Box::new(self.inferred_to_ast_type(return_type)),
                effects: EffectSet::empty(self.span), // Simplified
                span: self.span,
            },
            InferredType::App(con, args) => {
                if let InferredType::Con(name) = con.as_ref() {
                    // For simple type applications like List Int
                    Type::App(
                        Box::new(Type::Con(*name, self.span)),
                        args.iter().map(|a| self.inferred_to_ast_type(a)).collect(),
                        self.span,
                    )
                } else {
                    Type::Con(Symbol::intern("Unknown"), self.span)
                }
            }
            _ => Type::Con(Symbol::intern("Unknown"), self.span),
        }
    }
    
    fn inferred_to_ast_type(&self, ty: &InferredType) -> Type {
        match ty {
            InferredType::Con(name) => Type::Con(*name, self.span),
            InferredType::Fun { params, return_type, .. } => Type::Fun {
                params: params.iter().map(|p| self.inferred_to_ast_type(p)).collect(),
                return_type: Box::new(self.inferred_to_ast_type(return_type)),
                effects: EffectSet::empty(self.span),
                span: self.span,
            },
            _ => Type::Con(Symbol::intern("Unknown"), self.span),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{FileId, span::ByteOffset};
    
    #[test]
    fn test_type_environment() {
        let mut env = TypeEnvironment::new();
        
        let int_type = InferredType::Con(Symbol::intern("Int"));
        env.locals.insert(Symbol::intern("x"), int_type.clone());
        
        assert!(env.locals.contains_key(&Symbol::intern("x")));
    }
    
    #[test]
    fn test_annotated_expr() {
        let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1));
        let expr = Expr::Literal(Literal::Integer(42), span);
        let inferred = InferredType::Con(Symbol::intern("Int"));
        
        let annotated = AnnotatedExpr {
            expr: expr.clone(),
            inferred_type: Some(inferred),
            span,
        };
        
        assert!(annotated.inferred_type.is_some());
    }
}