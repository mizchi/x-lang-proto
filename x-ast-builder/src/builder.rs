//! Builder implementations for AST construction

use x_parser::ast::*;
use x_parser::{Symbol, Visibility, Purity};
use super::AstBuilder;

/// Module builder
pub struct ModuleBuilder<'a> {
    builder: &'a mut AstBuilder,
    name: ModulePath,
    exports: Option<ExportList>,
    imports: Vec<Import>,
    items: Vec<Item>,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new(builder: &'a mut AstBuilder, name: &str) -> Self {
        let name_symbol = Symbol::intern(name);
        let name_path = ModulePath::single(name_symbol, builder.span());
        
        Self {
            builder,
            name: name_path,
            exports: None,
            imports: Vec::new(),
            items: Vec::new(),
        }
    }
    
    /// Set exports
    pub fn exports(mut self, exports: Vec<&str>) -> Self {
        let export_items = exports.into_iter().map(|name| {
            ExportItem {
                kind: ExportKind::Value,
                name: Symbol::intern(name),
                alias: None,
                span: self.builder.span(),
            }
        }).collect();
        
        self.exports = Some(ExportList {
            items: export_items,
            span: self.builder.span(),
        });
        self
    }
    
    /// Add an import
    pub fn import(mut self, module: &str) -> Self {
        let import = Import {
            module_path: ModulePath::single(Symbol::intern(module), self.builder.span()),
            kind: ImportKind::Wildcard,
            alias: None,
            version_spec: None,
            span: self.builder.span(),
        };
        self.imports.push(import);
        self
    }
    
    /// Add a value definition
    pub fn value<F>(mut self, name: &str, body: F) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let body_expr = body(self.builder);
        let value_def = ValueDef {
            name: Symbol::intern(name),
            documentation: None,
            type_annotation: None,
            parameters: Vec::new(),
            body: body_expr,
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            imports: Vec::new(),
            span: self.builder.span(),
        };
        self.items.push(Item::ValueDef(value_def));
        self
    }
    
    /// Add a function definition
    pub fn function<F>(mut self, name: &str, params: Vec<&str>, body: F) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let body_expr = body(self.builder);
        
        // Create lambda parameters
        let parameters: Vec<Pattern> = params.into_iter()
            .map(|p| Pattern::Variable(Symbol::intern(p), self.builder.span()))
            .collect();
        
        // Create lambda expression
        let lambda = Expr::Lambda {
            parameters,
            body: Box::new(body_expr),
            span: self.builder.span(),
        };
        
        let value_def = ValueDef {
            name: Symbol::intern(name),
            documentation: None,
            type_annotation: None,
            parameters: Vec::new(),
            body: lambda,
            visibility: Visibility::Public,
            purity: Purity::Inferred,
            imports: Vec::new(),
            span: self.builder.span(),
        };
        self.items.push(Item::ValueDef(value_def));
        self
    }
    
    /// Add a data type definition
    pub fn data_type(mut self, name: &str, constructors: Vec<(&str, Vec<&str>)>) -> Self {
        let ctors: Vec<Constructor> = constructors.into_iter()
            .map(|(ctor_name, fields)| {
                let field_types: Vec<Type> = fields.into_iter()
                    .map(|f| Type::Con(Symbol::intern(f), self.builder.span()))
                    .collect();
                
                Constructor {
                    name: Symbol::intern(ctor_name),
                    fields: field_types,
                    span: self.builder.span(),
                }
            })
            .collect();
        
        let type_def = TypeDef {
            name: Symbol::intern(name),
            documentation: None,
            type_params: Vec::new(),
            kind: TypeDefKind::Data(ctors),
            visibility: Visibility::Public,
            span: self.builder.span(),
        };
        self.items.push(Item::TypeDef(type_def));
        self
    }
    
    /// Add a record type definition using type alias
    pub fn record_type<F>(mut self, name: &str, fields: Vec<(&str, F)>) -> Self
    where
        F: Fn(&mut AstBuilder) -> Type,
    {
        use std::collections::HashMap;
        
        let mut field_map = HashMap::new();
        for (field_name, type_fn) in fields {
            let field_type = type_fn(self.builder);
            field_map.insert(Symbol::intern(field_name), field_type);
        }
        
        let record_type = Type::Record {
            fields: field_map,
            rest: None,
            span: self.builder.span(),
        };
        
        let type_def = TypeDef {
            name: Symbol::intern(name),
            documentation: None,
            type_params: Vec::new(),
            kind: TypeDefKind::Alias(record_type),
            visibility: Visibility::Public,
            span: self.builder.span(),
        };
        self.items.push(Item::TypeDef(type_def));
        self
    }
    
    /// Add a type alias
    pub fn type_alias<F>(mut self, name: &str, target: F) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Type,
    {
        let target_type = target(self.builder);
        let type_def = TypeDef {
            name: Symbol::intern(name),
            documentation: None,
            type_params: Vec::new(),
            kind: TypeDefKind::Alias(target_type),
            visibility: Visibility::Public,
            span: self.builder.span(),
        };
        self.items.push(Item::TypeDef(type_def));
        self
    }
    
    /// Add an effect definition
    pub fn effect(mut self, name: &str, operations: Vec<(&str, Vec<&str>, &str)>) -> Self {
        let ops: Vec<EffectOperation> = operations.into_iter()
            .map(|(op_name, params, return_type)| {
                let param_types: Vec<Type> = params.into_iter()
                    .map(|p| Type::Con(Symbol::intern(p), self.builder.span()))
                    .collect();
                
                EffectOperation {
                    name: Symbol::intern(op_name),
                    parameters: param_types,
                    return_type: Type::Con(Symbol::intern(return_type), self.builder.span()),
                    span: self.builder.span(),
                }
            })
            .collect();
        
        let effect_def = EffectDef {
            name: Symbol::intern(name),
            documentation: None,
            type_params: Vec::new(),
            operations: ops,
            visibility: Visibility::Public,
            span: self.builder.span(),
        };
        self.items.push(Item::EffectDef(effect_def));
        self
    }
    
    /// Build the module
    pub fn build(self) -> Module {
        Module {
            name: self.name,
            documentation: None,
            exports: self.exports,
            imports: self.imports,
            items: self.items,
            span: self.builder.span(),
        }
    }
}

/// Expression builder helper trait
impl AstBuilder {
    /// Integer literal
    pub fn int(&mut self, value: i64) -> Expr {
        Expr::Literal(Literal::Integer(value), self.span())
    }
    
    /// Float literal
    pub fn float(&mut self, value: f64) -> Expr {
        Expr::Literal(Literal::Float(value), self.span())
    }
    
    /// String literal
    pub fn string(&mut self, value: &str) -> Expr {
        Expr::Literal(Literal::String(value.to_string()), self.span())
    }
    
    /// Boolean literal
    pub fn bool(&mut self, value: bool) -> Expr {
        // x Language uses Haskell-style true/false as constructors
        Expr::Var(Symbol::intern(if value { "true" } else { "false" }), self.span())
    }
    
    /// Unit literal
    pub fn unit(&mut self) -> Expr {
        Expr::Literal(Literal::Unit, self.span())
    }
    
    /// Variable reference
    pub fn var(&mut self, name: &str) -> Expr {
        Expr::Var(Symbol::intern(name), self.span())
    }
    
    /// Binary operation
    pub fn binop<F1, F2>(&mut self, op: &str, left: F1, right: F2) -> Expr
    where
        F1: FnOnce(&mut Self) -> Expr,
        F2: FnOnce(&mut Self) -> Expr,
    {
        let left_expr = left(self);
        let right_expr = right(self);
        
        Expr::App(
            Box::new(Expr::Var(Symbol::intern(op), self.span())),
            vec![left_expr, right_expr],
            self.span(),
        )
    }
    
    /// Function application
    pub fn app(&mut self, func: &str, args: Vec<impl FnOnce(&mut Self) -> Expr>) -> Expr {
        let arg_exprs: Vec<Expr> = args.into_iter()
            .map(|arg_fn| arg_fn(self))
            .collect();
        
        Expr::App(
            Box::new(Expr::Var(Symbol::intern(func), self.span())),
            arg_exprs,
            self.span(),
        )
    }
    
    /// Direct application with expression
    pub fn app_expr(&mut self, func: Expr, args: Vec<Expr>) -> Expr {
        Expr::App(Box::new(func), args, self.span())
    }
    
    /// Lambda expression
    pub fn lambda(&mut self, params: Vec<&str>, body: impl FnOnce(&mut Self) -> Expr) -> Expr {
        let parameters: Vec<Pattern> = params.into_iter()
            .map(|p| Pattern::Variable(Symbol::intern(p), self.span()))
            .collect();
        
        let body_expr = body(self);
        
        Expr::Lambda {
            parameters,
            body: Box::new(body_expr),
            span: self.span(),
        }
    }
    
    /// Let expression
    pub fn let_in<F1, F2>(&mut self, name: &str, value: F1, body: F2) -> Expr
    where
        F1: FnOnce(&mut Self) -> Expr,
        F2: FnOnce(&mut Self) -> Expr,
    {
        let value_expr = value(self);
        let body_expr = body(self);
        
        Expr::Let {
            pattern: Pattern::Variable(Symbol::intern(name), self.span()),
            type_annotation: None,
            value: Box::new(value_expr),
            body: Box::new(body_expr),
            span: self.span(),
        }
    }
    
    /// Let rec expression
    pub fn let_rec<F1, F2>(&mut self, name: &str, params: Vec<&str>, value: F1, body: F2) -> Expr
    where
        F1: FnOnce(&mut Self) -> Expr,
        F2: FnOnce(&mut Self) -> Expr,
    {
        let value_expr = if params.is_empty() {
            value(self)
        } else {
            self.lambda(params, value)
        };
        
        let body_expr = body(self);
        
        // x Language doesn't have direct LetRec, use nested Let with recursive function
        Expr::Let {
            pattern: Pattern::Variable(Symbol::intern(name), self.span()),
            type_annotation: None,
            value: Box::new(value_expr),
            body: Box::new(body_expr),
            span: self.span(),
        }
    }
    
    /// If-then-else expression
    pub fn if_then_else<F1, F2, F3>(&mut self, cond: F1, then_branch: F2, else_branch: F3) -> Expr
    where
        F1: FnOnce(&mut Self) -> Expr,
        F2: FnOnce(&mut Self) -> Expr,
        F3: FnOnce(&mut Self) -> Expr,
    {
        let cond_expr = cond(self);
        let then_expr = then_branch(self);
        let else_expr = else_branch(self);
        
        Expr::If {
            condition: Box::new(cond_expr),
            then_branch: Box::new(then_expr),
            else_branch: Box::new(else_expr),
            span: self.span(),
        }
    }
    
    /// Match expression
    pub fn match_expr<F>(&mut self, scrutinee: F, arms: Vec<(impl FnOnce(&mut Self) -> Pattern, impl FnOnce(&mut Self) -> Expr)>) -> Expr
    where
        F: FnOnce(&mut Self) -> Expr,
    {
        let scrutinee_expr = scrutinee(self);
        
        let match_arms: Vec<MatchArm> = arms.into_iter()
            .map(|(pat_fn, expr_fn)| {
                let pattern = pat_fn(self);
                let body = expr_fn(self);
                
                MatchArm {
                    pattern,
                    guard: None,
                    body,
                    span: self.span(),
                }
            })
            .collect();
        
        Expr::Match {
            scrutinee: Box::new(scrutinee_expr),
            arms: match_arms,
            span: self.span(),
        }
    }
    
    /// List literal
    pub fn list(&mut self, elements: Vec<impl FnOnce(&mut Self) -> Expr>) -> Expr {
        let element_exprs: Vec<Expr> = elements.into_iter()
            .map(|elem_fn| elem_fn(self))
            .collect();
        
        // Build list using cons
        element_exprs.into_iter()
            .rev()
            .fold(
                Expr::Var(Symbol::intern("[]"), self.span()),
                |acc, elem| {
                    Expr::App(
                        Box::new(Expr::Var(Symbol::intern("::"), self.span())),
                        vec![elem, acc],
                        self.span(),
                    )
                }
            )
    }
    
    /// Cons operation
    pub fn cons(&mut self, head: impl FnOnce(&mut Self) -> Expr, tail: impl FnOnce(&mut Self) -> Expr) -> Expr {
        let head_expr = head(self);
        let tail_expr = tail(self);
        
        Expr::App(
            Box::new(Expr::Var(Symbol::intern("::"), self.span())),
            vec![head_expr, tail_expr],
            self.span(),
        )
    }
    
    /// Type constructor
    pub fn con(&mut self, name: &str) -> Type {
        Type::Con(Symbol::intern(name), self.span())
    }
    
    /// Type variable
    pub fn type_var(&mut self, name: &str) -> Type {
        Type::Var(Symbol::intern(name), self.span())
    }
    
    /// Function type
    pub fn fun_type(&mut self, params: Vec<Type>, result: Type) -> Type {
        Type::Fun {
            params,
            return_type: Box::new(result),
            effects: EffectSet::empty(self.span()),
            span: self.span(),
        }
    }
    
    /// Type application
    pub fn type_app(&mut self, con: &str, args: Vec<Type>) -> Type {
        Type::App(
            Box::new(Type::Con(Symbol::intern(con), self.span())),
            args,
            self.span(),
        )
    }
    
    /// Variable pattern
    pub fn var_pattern(&mut self, name: &str) -> Pattern {
        Pattern::Variable(Symbol::intern(name), self.span())
    }
    
    /// Wildcard pattern
    pub fn wildcard_pattern(&mut self) -> Pattern {
        Pattern::Wildcard(self.span())
    }
    
    /// Constructor pattern
    pub fn constructor_pattern(&mut self, name: &str, args: Vec<Pattern>) -> Pattern {
        Pattern::Constructor {
            name: Symbol::intern(name),
            args,
            span: self.span(),
        }
    }
    
    /// Tuple pattern
    pub fn tuple_pattern(&mut self, elements: Vec<Pattern>) -> Pattern {
        Pattern::Tuple { patterns: elements, span: self.span() }
    }
}

/// Standalone expression builder
pub struct ExprBuilder<'a> {
    builder: &'a mut AstBuilder,
}

impl<'a> ExprBuilder<'a> {
    pub fn new(builder: &'a mut AstBuilder) -> Self {
        Self { builder }
    }
    
    /// Delegate all methods to AstBuilder
    pub fn int(self, value: i64) -> Expr {
        self.builder.int(value)
    }
    
    pub fn var(self, name: &str) -> Expr {
        self.builder.var(name)
    }
    
    pub fn string(self, value: &str) -> Expr {
        self.builder.string(value)
    }
    
    pub fn app(self, func: &str, args: Vec<impl FnOnce(&mut AstBuilder) -> Expr>) -> Expr {
        self.builder.app(func, args)
    }
    
    pub fn binop<F1, F2>(self, op: &str, left: F1, right: F2) -> Expr
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
    {
        self.builder.binop(op, left, right)
    }
    
    pub fn if_then_else<F1, F2, F3>(self, cond: F1, then_branch: F2, else_branch: F3) -> Expr
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
        F3: FnOnce(&mut AstBuilder) -> Expr,
    {
        self.builder.if_then_else(cond, then_branch, else_branch)
    }
    
    pub fn build(self) -> Expr {
        // Default to unit
        self.builder.unit()
    }
}

/// Type builder
pub struct TypeBuilder<'a> {
    builder: &'a mut AstBuilder,
}

impl<'a> TypeBuilder<'a> {
    pub fn new(builder: &'a mut AstBuilder) -> Self {
        Self { builder }
    }
    
    pub fn con(self, name: &str) -> Type {
        self.builder.con(name)
    }
    
    pub fn var(self, name: &str) -> Type {
        self.builder.type_var(name)
    }
    
    pub fn fun(self, params: Vec<Type>, result: Type) -> Type {
        self.builder.fun_type(params, result)
    }
    
    pub fn app(self, con: &str, args: Vec<Type>) -> Type {
        self.builder.type_app(con, args)
    }
}

/// Pattern builder
pub struct PatternBuilder<'a> {
    builder: &'a mut AstBuilder,
}

impl<'a> PatternBuilder<'a> {
    pub fn new(builder: &'a mut AstBuilder) -> Self {
        Self { builder }
    }
    
    pub fn var(self, name: &str) -> Pattern {
        self.builder.var_pattern(name)
    }
    
    pub fn wildcard(self) -> Pattern {
        self.builder.wildcard_pattern()
    }
    
    pub fn constructor(self, name: &str, args: Vec<Pattern>) -> Pattern {
        self.builder.constructor_pattern(name, args)
    }
    
    pub fn tuple(self, elements: Vec<Pattern>) -> Pattern {
        self.builder.tuple_pattern(elements)
    }
}