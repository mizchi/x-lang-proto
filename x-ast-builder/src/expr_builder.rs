//! Complete expression builder implementation

use x_parser::ast::*;
use x_parser::{Symbol, Span};
use crate::AstBuilder;

/// Expression builder with state tracking
pub struct ExprBuilder<'a> {
    builder: &'a mut AstBuilder,
    expr: Option<Expr>,
}

impl<'a> ExprBuilder<'a> {
    pub fn new(builder: &'a mut AstBuilder) -> Self {
        Self { 
            builder,
            expr: None,
        }
    }
    
    /// Integer literal
    pub fn int(mut self, value: i64) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Literal(Literal::Integer(value), span));
        self
    }
    
    /// Float literal
    pub fn float(mut self, value: f64) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Literal(Literal::Float(value), span));
        self
    }
    
    /// String literal
    pub fn string(mut self, value: &str) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Literal(Literal::String(value.to_string()), span));
        self
    }
    
    /// Boolean literal
    pub fn bool(mut self, value: bool) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Literal(Literal::Bool(value), span));
        self
    }
    
    /// Unit literal
    pub fn unit(mut self) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Literal(Literal::Unit, span));
        self
    }
    
    /// Variable reference
    pub fn var(mut self, name: &str) -> Self {
        let span = self.builder.span();
        self.expr = Some(Expr::Var(Symbol::intern(name), span));
        self
    }
    
    /// Binary operation
    pub fn binop<F1, F2>(mut self, op: &str, left: F1, right: F2) -> Self
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
    {
        let left_expr = left(self.builder);
        let right_expr = right(self.builder);
        let span = self.builder.span();
        
        // Create application of the operator
        let op_expr = Expr::Var(Symbol::intern(op), span);
        self.expr = Some(Expr::App(
            Box::new(op_expr),
            vec![left_expr, right_expr],
            span,
        ));
        self
    }
    
    /// Function application
    pub fn app<F>(mut self, func: &str, args: Vec<F>) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let func_expr = Expr::Var(Symbol::intern(func), span);
        let arg_exprs: Vec<Expr> = args.into_iter()
            .map(|f| f(self.builder))
            .collect();
        
        self.expr = Some(Expr::App(
            Box::new(func_expr),
            arg_exprs,
            span,
        ));
        self
    }
    
    /// Direct application with expression
    pub fn app_expr<F1, F2>(mut self, func: F1, args: Vec<F2>) -> Self
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let func_expr = func(self.builder);
        let arg_exprs: Vec<Expr> = args.into_iter()
            .map(|f| f(self.builder))
            .collect();
        
        self.expr = Some(Expr::App(
            Box::new(func_expr),
            arg_exprs,
            span,
        ));
        self
    }
    
    /// Lambda expression
    pub fn lambda<F>(mut self, params: Vec<&str>, body: F) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let parameters: Vec<Pattern> = params.into_iter()
            .map(|p| Pattern::Variable(Symbol::intern(p), self.builder.span()))
            .collect();
        
        let body_expr = body(self.builder);
        
        self.expr = Some(Expr::Lambda {
            parameters,
            body: Box::new(body_expr),
            span,
        });
        self
    }
    
    /// Let expression
    pub fn let_in<F1, F2>(mut self, name: &str, value: F1, body: F2) -> Self
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let pattern = Pattern::Variable(Symbol::intern(name), self.builder.span());
        let value_expr = value(self.builder);
        let body_expr = body(self.builder);
        
        self.expr = Some(Expr::Let {
            pattern,
            type_annotation: None,
            value: Box::new(value_expr),
            body: Box::new(body_expr),
            span,
        });
        self
    }
    
    /// If-then-else expression
    pub fn if_then_else<F1, F2, F3>(mut self, cond: F1, then_branch: F2, else_branch: F3) -> Self
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
        F3: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let cond_expr = cond(self.builder);
        let then_expr = then_branch(self.builder);
        let else_expr = else_branch(self.builder);
        
        self.expr = Some(Expr::If {
            condition: Box::new(cond_expr),
            then_branch: Box::new(then_expr),
            else_branch: Box::new(else_expr),
            span,
        });
        self
    }
    
    /// Match expression
    pub fn match_expr<F1, F2>(mut self, scrutinee: F1, arms: Vec<(Pattern, F2)>) -> Self
    where
        F1: FnOnce(&mut AstBuilder) -> Expr,
        F2: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let scrutinee_expr = scrutinee(self.builder);
        
        let match_arms: Vec<MatchArm> = arms.into_iter()
            .map(|(pattern, body_fn)| {
                let body = body_fn(self.builder);
                MatchArm {
                    pattern,
                    guard: None,
                    body,
                    span: self.builder.span(),
                }
            })
            .collect();
        
        self.expr = Some(Expr::Match {
            scrutinee: Box::new(scrutinee_expr),
            arms: match_arms,
            span,
        });
        self
    }
    
    /// List literal using cons operations
    pub fn list<F>(mut self, elements: Vec<F>) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let element_exprs: Vec<Expr> = elements.into_iter()
            .map(|f| f(self.builder))
            .collect();
        
        // Build list from right to left: [1; 2; 3] becomes 1 :: 2 :: 3 :: []
        let mut list_expr = Expr::Var(Symbol::intern("[]"), span);
        
        for elem in element_exprs.into_iter().rev() {
            let cons_span = self.builder.span();
            let cons_op = Expr::Var(Symbol::intern("::"), cons_span);
            list_expr = Expr::App(Box::new(cons_op), vec![elem, list_expr], cons_span);
        }
        
        self.expr = Some(list_expr);
        self
    }
    
    /// Do notation
    pub fn do_block<F>(mut self, statements: Vec<F>) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> DoStatement,
    {
        let span = self.builder.span();
        let stmts: Vec<DoStatement> = statements.into_iter()
            .map(|f| f(self.builder))
            .collect();
        
        self.expr = Some(Expr::Do {
            statements: stmts,
            span,
        });
        self
    }
    
    /// Type annotation
    pub fn annotate<F>(mut self, expr: F, typ: Type) -> Self
    where
        F: FnOnce(&mut AstBuilder) -> Expr,
    {
        let span = self.builder.span();
        let annotated_expr = expr(self.builder);
        
        self.expr = Some(Expr::Ann {
            expr: Box::new(annotated_expr),
            type_annotation: typ,
            span,
        });
        self
    }
    
    /// Build the expression
    pub fn build(self) -> Expr {
        self.expr.unwrap_or_else(|| Expr::Literal(Literal::Unit, Span::default()))
    }
}

/// Helper functions for building expressions
impl<'a> ExprBuilder<'a> {
    /// Helper to create a simple binary operation
    pub fn simple_binop(builder: &mut AstBuilder, op: &str, left: Expr, right: Expr) -> Expr {
        let span = builder.span();
        let op_expr = Expr::Var(Symbol::intern(op), span);
        Expr::App(Box::new(op_expr), vec![left, right], span)
    }
    
    /// Helper to create a function call
    pub fn simple_app(builder: &mut AstBuilder, func: &str, args: Vec<Expr>) -> Expr {
        let span = builder.span();
        let func_expr = Expr::Var(Symbol::intern(func), span);
        Expr::App(Box::new(func_expr), args, span)
    }
}

/// Extension trait for easier expression building
pub trait ExprBuilderExt<'a> {
    fn expr(&'a mut self) -> ExprBuilder<'a>;
}

impl ExprBuilderExt<'_> for AstBuilder {
    fn expr(&mut self) -> ExprBuilder {
        ExprBuilder::new(self)
    }
}