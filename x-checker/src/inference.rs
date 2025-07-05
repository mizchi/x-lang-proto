//! Type inference engine for x Language
//! 
//! Implements Algorithm W extended with:
//! - Effect inference and checking
//! - Row polymorphism for extensible effects
//! - Let-polymorphism with value restriction

use x_parser::{
    Expr, Pattern, Literal, Item, ValueDef, TypeDef, Module,
    LetBinding, MatchArm, DoStatement, EffectHandler, Type as AstType,
    Span,
    Symbol,
};
use crate::types::*;
use crate::error_reporting::*;
use crate::builtins::Builtins;
use std::result::Result as StdResult;

use std::collections::{HashMap, HashSet};

/// Type inference context
#[derive(Debug)]
pub struct InferenceContext {
    pub env: TypeEnv,
    pub var_gen: VarGen,
    pub constraints: Vec<Constraint>,
    pub errors: Vec<TypeError>,
    pub builtins: Builtins,
}

/// Inference result containing type and effects
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub typ: Type,
    pub effects: EffectSet,
    pub constraints: Vec<Constraint>,
}

impl InferenceContext {
    pub fn new() -> Self {
        InferenceContext {
            env: TypeEnv::new(),
            var_gen: VarGen::new(),
            constraints: Vec::new(),
            errors: Vec::new(),
            builtins: Builtins::new(),
        }
    }
    
    pub fn fresh_type_var(&mut self) -> Type {
        Type::Var(self.var_gen.fresh_type_var())
    }
    
    pub fn fresh_effect_var(&mut self) -> EffectSet {
        EffectSet::Var(self.var_gen.fresh_effect_var())
    }
    
    /// Instantiate a type scheme with fresh variables
    pub fn instantiate(&mut self, scheme: &TypeScheme) -> (Type, EffectSet) {
        let mut subst = Substitution::new();
        
        // Create fresh variables for bound type variables
        for &type_var in &scheme.type_vars {
            let fresh_var = self.var_gen.fresh_type_var();
            subst.insert_type(type_var, Type::Var(fresh_var));
        }
        
        // Create fresh variables for bound effect variables
        for &effect_var in &scheme.effect_vars {
            let fresh_var = self.var_gen.fresh_effect_var();
            subst.insert_effect(effect_var, EffectSet::Var(fresh_var));
        }
        
        let typ = scheme.body.apply_subst(&subst);
        let effects = extract_effects(&typ);
        
        (typ, effects)
    }
    
    /// Generalize a type to a type scheme
    pub fn generalize(&self, typ: &Type, _effects: &EffectSet) -> TypeScheme {
        let type_free_vars = typ.free_vars();
        let env_free_vars = self.env_free_vars();
        
        let generalizable_vars: Vec<TypeVar> = type_free_vars
            .difference(&env_free_vars)
            .cloned()
            .collect();
        
        // TODO: Properly extract effect variables
        let effect_vars = Vec::new();
        
        TypeScheme {
            type_vars: generalizable_vars,
            effect_vars,
            constraints: self.constraints.clone(),
            body: typ.clone(),
        }
    }
    
    fn env_free_vars(&self) -> HashSet<TypeVar> {
        let mut vars = HashSet::new();
        for scheme in self.env.vars.values() {
            let scheme_vars = scheme.body.free_vars();
            vars.extend(scheme_vars);
        }
        vars
    }
    
    /// Report a type error
    pub fn report_error(&mut self, error: TypeError) {
        self.errors.push(error);
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main type inference functions
impl InferenceContext {
    /// Infer the type of an expression
    pub fn infer_expr(&mut self, expr: &Expr) -> StdResult<InferenceResult, String> {
        match expr {
            Expr::Literal(lit, _span) => self.infer_literal(lit),
            
            Expr::Var(name, _span) => self.infer_var(*name),
            
            Expr::App(func, args, _span) => self.infer_app(func, args),
            
            Expr::Lambda { parameters, body, .. } => {
                self.infer_lambda(parameters, body, None)
            }
            
            Expr::Let { pattern: _, value, body, .. } => {
                // For now, handle simple let as a single binding
                let binding = LetBinding {
                    name: Symbol::intern("x"), // Placeholder
                    value: *value.clone(),
                    span: body.span(),
                };
                self.infer_let(&[binding], body)
            }
            
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.infer_if(condition, then_branch, else_branch)
            }
            
            Expr::Match { scrutinee, arms, .. } => self.infer_match(scrutinee, arms),
            
            Expr::Do { statements, .. } => self.infer_do_statements(statements),
            
            Expr::Handle { expr, handlers, .. } => {
                self.infer_handle(expr, handlers)
            }
            
            Expr::Resume { value, .. } => self.infer_resume(value),
            
            Expr::Perform { effect, operation, args, .. } => {
                self.infer_perform(*effect, *operation, args)
            }
            
            Expr::Ann { expr, type_annotation, .. } => self.infer_annotation(expr, type_annotation),
        }
    }
    
    fn infer_literal(&mut self, lit: &Literal) -> StdResult<InferenceResult, String> {
        let typ = match lit {
            Literal::Integer(_) => Type::Con(Symbol::intern("Int")),
            Literal::Float(_) => Type::Con(Symbol::intern("Float")),
            Literal::String(_) => Type::Con(Symbol::intern("String")),
            Literal::Bool(_) => Type::Con(Symbol::intern("Bool")),
            Literal::Unit => Type::Con(Symbol::intern("Unit")),
        };
        
        Ok(InferenceResult {
            typ,
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
    
    fn infer_var(&mut self, name: Symbol) -> StdResult<InferenceResult, String> {
        self.infer_var_with_span(name, None)
    }
    
    fn infer_var_with_span(&mut self, name: Symbol, span: Option<Span>) -> StdResult<InferenceResult, String> {
        // First check if it's a builtin
        if let Some(scheme) = self.builtins.get_type_scheme(&name) {
            let scheme = scheme.clone();
            let (typ, effects) = self.instantiate(&scheme);
            return Ok(InferenceResult {
                typ,
                effects,
                constraints: Vec::new(),
            });
        }
        
        // Then check the environment
        match self.env.lookup_var(name) {
            Some(scheme) => {
                let scheme = scheme.clone();
                let (typ, effects) = self.instantiate(&scheme);
                Ok(InferenceResult {
                    typ,
                    effects,
                    constraints: Vec::new(),
                })
            }
            None => {
                if let Some(span) = span {
                    self.report_error(TypeError::UnboundVariable {
                        name,
                        span,
                    });
                }
                Err(format!("Unbound variable: {}", name))
            }
        }
    }
    
    fn infer_app(&mut self, func: &Expr, args: &[Expr]) -> StdResult<InferenceResult, String> {
        // Infer function type
        let func_result = self.infer_expr(func)?;
        
        // Infer argument types
        let mut arg_results = Vec::new();
        for arg in args {
            arg_results.push(self.infer_expr(arg)?);
        }
        
        // Create fresh variables for result
        let result_type = self.fresh_type_var();
        let result_effects = self.fresh_effect_var();
        
        // Unify function type with expected signature
        let arg_types: Vec<Type> = arg_results.iter().map(|r| r.typ.clone()).collect();
        let expected_func_type = Type::Fun {
            params: arg_types,
            return_type: Box::new(result_type.clone()),
            effects: result_effects.clone(),
        };
        
        self.unify(&func_result.typ, &expected_func_type)?;
        
        // Combine effects from function and arguments
        let mut combined_effects = func_result.effects;
        for arg_result in &arg_results {
            combined_effects = self.combine_effects(combined_effects, arg_result.effects.clone())?;
        }
        
        Ok(InferenceResult {
            typ: result_type,
            effects: combined_effects,
            constraints: Vec::new(),
        })
    }
    
    fn infer_lambda(
        &mut self,
        params: &[Pattern],
        body: &Expr,
        _effects: Option<&()>,
    ) -> StdResult<InferenceResult, String> {
        // Enter new scope
        let saved_env = self.env.clone();
        
        // Add parameters to environment
        let mut param_types = Vec::new();
        for param in params {
            // For now, just handle variable patterns
            let param_name = match param {
                Pattern::Variable(name, _) => *name,
                _ => Symbol::intern("_"), // Placeholder for complex patterns
            };
            
            let param_type = self.fresh_type_var();
            
            let scheme = TypeScheme {
                type_vars: Vec::new(),
                effect_vars: Vec::new(),
                constraints: Vec::new(),
                body: param_type.clone(),
            };
            
            self.env.insert_var(param_name, scheme);
            param_types.push(param_type);
        }
        
        // Infer body type
        let body_result = self.infer_expr(body)?;
        
        // Restore environment
        self.env = saved_env;
        
        let lambda_type = Type::Fun {
            params: param_types,
            return_type: Box::new(body_result.typ),
            effects: body_result.effects.clone(),
        };
        
        Ok(InferenceResult {
            typ: lambda_type,
            effects: EffectSet::Empty, // Lambdas themselves are pure
            constraints: body_result.constraints,
        })
    }
    
    fn infer_let(
        &mut self,
        bindings: &[LetBinding],
        body: &Expr,
    ) -> StdResult<InferenceResult, String> {
        let saved_env = self.env.clone();
        
        // Process bindings
        for binding in bindings {
            let binding_result = self.infer_expr(&binding.value)?;
            
            // Generalize if it's a value (not a function with effects)
            let scheme = if self.is_value(&binding.value) {
                self.generalize(&binding_result.typ, &binding_result.effects)
            } else {
                TypeScheme {
                    type_vars: Vec::new(),
                    effect_vars: Vec::new(),
                    constraints: Vec::new(),
                    body: binding_result.typ,
                }
            };
            
            self.env.insert_var(binding.name, scheme);
        }
        
        // Infer body
        let body_result = self.infer_expr(body)?;
        
        // Restore environment for outer scope
        self.env = saved_env;
        
        Ok(body_result)
    }
    
    fn infer_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> StdResult<InferenceResult, String> {
        use x_parser::symbol::symbols;
        
        // Condition must be Bool
        let cond_result = self.infer_expr(condition)?;
        self.unify(&cond_result.typ, &Type::Con(symbols::BOOL()))?;
        
        // Both branches must have same type
        let then_result = self.infer_expr(then_branch)?;
        let else_result = self.infer_expr(else_branch)?;
        
        self.unify(&then_result.typ, &else_result.typ)?;
        
        // Combine effects
        let branch_effects = self.combine_effects(then_result.effects, else_result.effects)?;
        let combined_effects = self.combine_effects(cond_result.effects, branch_effects)?;
        
        Ok(InferenceResult {
            typ: then_result.typ,
            effects: combined_effects,
            constraints: Vec::new(),
        })
    }
    
    fn infer_match(
        &mut self,
        expr: &Expr,
        arms: &[MatchArm],
    ) -> StdResult<InferenceResult, String> {
        let expr_result = self.infer_expr(expr)?;
        
        if arms.is_empty() {
            return Err("Match expression must have at least one arm".to_string());
        }
        
        // Infer first arm to get result type
        let first_arm = &arms[0];
        let _pattern_result = self.infer_pattern(&first_arm.pattern, &expr_result.typ)?;
        let first_body_result = self.infer_expr(&first_arm.body)?;
        
        let result_type = first_body_result.typ;
        let mut combined_effects = self.combine_effects(
            expr_result.effects,
            first_body_result.effects,
        )?;
        
        // Check remaining arms
        for arm in &arms[1..] {
            let _pattern_result = self.infer_pattern(&arm.pattern, &expr_result.typ)?;
            let body_result = self.infer_expr(&arm.body)?;
            
            self.unify(&result_type, &body_result.typ)?;
            combined_effects = self.combine_effects(combined_effects, body_result.effects)?;
        }
        
        Ok(InferenceResult {
            typ: result_type,
            effects: combined_effects,
            constraints: Vec::new(),
        })
    }
    
    fn infer_do_statements(&mut self, _statements: &[DoStatement]) -> StdResult<InferenceResult, String> {
        // For now, just return unit type
        Ok(InferenceResult {
            typ: Type::Con(Symbol::intern("Unit")),
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
    
    fn infer_handle(
        &mut self,
        body: &Expr,
        _handlers: &[EffectHandler],
    ) -> StdResult<InferenceResult, String> {
        let body_result = self.infer_expr(body)?;
        
        // TODO: Implement proper effect handling
        // For now, just return the body type with empty effects
        Ok(InferenceResult {
            typ: body_result.typ,
            effects: EffectSet::Empty,
            constraints: Vec::new(),
        })
    }
    
    fn infer_resume(&mut self, expr: &Expr) -> StdResult<InferenceResult, String> {
        // Resume passes through the expression type
        self.infer_expr(expr)
    }
    
    fn infer_perform(
        &mut self,
        effect: Symbol,
        operation: Symbol,
        args: &[Expr],
    ) -> StdResult<InferenceResult, String> {
        // Look up effect and operation  
        let effect_def = self.env.lookup_effect(effect)
            .ok_or_else(|| format!("Unknown effect: {}", effect))?.clone();
        
        let operation_def = effect_def.operations.iter()
            .find(|op| op.name == operation)
            .ok_or_else(|| format!("Unknown operation: {} in effect {}", operation, effect))?.clone();
        
        // Check argument types
        if args.len() != operation_def.params.len() {
            return Err(format!(
                "Operation {} expects {} arguments, got {}",
                operation,
                operation_def.params.len(),
                args.len()
            ));
        }
        
        for (arg, expected_type) in args.iter().zip(&operation_def.params) {
            let arg_result = self.infer_expr(arg)?;
            self.unify(&arg_result.typ, &expected_type)?;
        }
        
        // Create effect set containing this effect
        let effect_set = EffectSet::Row {
            effects: vec![effect_def],
            tail: None,
        };
        
        Ok(InferenceResult {
            typ: operation_def.return_type,
            effects: effect_set,
            constraints: Vec::new(),
        })
    }
    
    fn infer_annotation(&mut self, expr: &Expr, typ: &AstType) -> StdResult<InferenceResult, String> {
        let expr_result = self.infer_expr(expr)?;
        let expected_type = self.ast_type_to_type(typ)?;
        
        self.unify(&expr_result.typ, &expected_type)?;
        
        Ok(InferenceResult {
            typ: expected_type,
            effects: expr_result.effects,
            constraints: expr_result.constraints,
        })
    }
    
    /// Infer pattern type and return bindings
    fn infer_pattern(&mut self, pattern: &Pattern, expected_type: &Type) -> StdResult<HashMap<Symbol, Type>, String> {
        match pattern {
            Pattern::Wildcard(_) => Ok(HashMap::new()),
            
            Pattern::Variable(name, _) => {
                let mut bindings = HashMap::new();
                bindings.insert(*name, expected_type.clone());
                Ok(bindings)
            }
            
            Pattern::Literal(lit, _) => {
                let lit_result = self.infer_literal(lit)?;
                self.unify(&lit_result.typ, expected_type)?;
                Ok(HashMap::new())
            }
            
            Pattern::Constructor { name: _, args, .. } => {
                // TODO: Implement constructor patterns
                let mut bindings = HashMap::new();
                for (_i, arg) in args.iter().enumerate() {
                    let arg_type = self.fresh_type_var();
                    let arg_bindings = self.infer_pattern(arg, &arg_type)?;
                    bindings.extend(arg_bindings);
                }
                Ok(bindings)
            }
            
            Pattern::Tuple { patterns, .. } => {
                match expected_type {
                    Type::Tuple(types) => {
                        if patterns.len() != types.len() {
                            return Err(format!(
                                "Tuple pattern expects {} elements, got {}",
                                types.len(),
                                patterns.len()
                            ));
                        }
                        
                        let mut bindings = HashMap::new();
                        for (pattern, typ) in patterns.iter().zip(types) {
                            let pattern_bindings = self.infer_pattern(pattern, typ)?;
                            bindings.extend(pattern_bindings);
                        }
                        Ok(bindings)
                    }
                    _ => {
                        let pattern_types: Vec<Type> = patterns.iter()
                            .map(|_| self.fresh_type_var())
                            .collect();
                        
                        let tuple_type = Type::Tuple(pattern_types.clone());
                        self.unify(&tuple_type, expected_type)?;
                        
                        let mut bindings = HashMap::new();
                        for (pattern, typ) in patterns.iter().zip(pattern_types) {
                            let pattern_bindings = self.infer_pattern(pattern, &typ)?;
                            bindings.extend(pattern_bindings);
                        }
                        Ok(bindings)
                    }
                }
            }
            
            _ => {
                // TODO: Implement remaining patterns
                Ok(HashMap::new())
            }
        }
    }
    
    /// Helper function to check if an expression is a value (for let-polymorphism)
    fn is_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_, _) => true,
            Expr::Var(_, _) => true,
            Expr::Lambda { .. } => true,
            _ => false,
        }
    }
    
    /// Convert AST type to internal type representation
    fn ast_type_to_type(&mut self, ast_type: &AstType) -> StdResult<Type, String> {
        match ast_type {
            AstType::Var(name, _) => {
                // For now, treat as type constructor
                Ok(Type::Con(*name))
            }
            AstType::Con(name, _) => {
                Ok(Type::Con(*name))
            }
            AstType::App(con, args, _) => {
                let con_type = self.ast_type_to_type(con)?;
                let arg_types: StdResult<Vec<Type>, String> = args.iter()
                    .map(|arg| self.ast_type_to_type(arg))
                    .collect();
                Ok(Type::App(Box::new(con_type), arg_types?))
            }
            AstType::Fun { params, return_type, effects, .. } => {
                let param_types: StdResult<Vec<Type>, String> = params.iter()
                    .map(|param| self.ast_type_to_type(param))
                    .collect();
                let return_typ = self.ast_type_to_type(return_type)?;
                let effect_set = self.ast_effects_to_effects(effects)?;
                
                Ok(Type::Fun {
                    params: param_types?,
                    return_type: Box::new(return_typ),
                    effects: effect_set,
                })
            }
            _ => {
                // TODO: Implement remaining AST type conversions
                Err("Unsupported AST type conversion".to_string())
            }
        }
    }
    
    fn ast_effects_to_effects(&mut self, _effects: &x_parser::ast::EffectSet) -> StdResult<EffectSet, String> {
        // TODO: Implement AST effect set conversion
        Ok(EffectSet::Empty)
    }
}

/// Unification and effect combination
impl InferenceContext {
    /// Unify two types with enhanced error reporting
    fn unify(&mut self, t1: &Type, t2: &Type) -> StdResult<(), String> {
        self.unify_with_span(t1, t2, None)
    }
    
    /// Unify two types with span information for error reporting
    fn unify_with_span(&mut self, t1: &Type, t2: &Type, span: Option<Span>) -> StdResult<(), String> {
        match (t1, t2) {
            (Type::Var(v1), Type::Var(v2)) if v1 == v2 => Ok(()),
            
            (Type::Var(var), typ) | (typ, Type::Var(var)) => {
                if typ.free_vars().contains(var) {
                    if let Some(span) = span {
                        self.report_error(TypeError::InfiniteType {
                            var: *var,
                            typ: typ.clone(),
                            span,
                        });
                    }
                    Err(format!("Occurs check failed: {} occurs in {}", 
                                Type::Var(*var), typ))
                } else {
                    // TODO: Apply substitution
                    Ok(())
                }
            }
            
            (Type::Con(n1), Type::Con(n2)) if n1 == n2 => Ok(()),
            
            (Type::App(c1, args1), Type::App(c2, args2)) => {
                self.unify(c1, c2)?;
                if args1.len() != args2.len() {
                    if let Some(span) = span {
                        self.report_error(TypeError::ArityMismatch {
                            expected: args1.len(),
                            found: args2.len(),
                            span,
                        });
                    }
                    return Err(format!("Type application arity mismatch: {} vs {}", 
                                     args1.len(), args2.len()));
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            }
            
            (Type::Fun { params: p1, return_type: r1, effects: e1 },
             Type::Fun { params: p2, return_type: r2, effects: e2 }) => {
                if p1.len() != p2.len() {
                    if let Some(span) = span {
                        self.report_error(TypeError::ArityMismatch {
                            expected: p1.len(),
                            found: p2.len(),
                            span,
                        });
                    }
                    return Err(format!("Function arity mismatch: {} vs {}", 
                                     p1.len(), p2.len()));
                }
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2)?;
                }
                self.unify(r1, r2)?;
                self.unify_effects(e1, e2)?;
                Ok(())
            }
            
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    if let Some(span) = span {
                        self.report_error(TypeError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            span,
                        });
                    }
                    return Err(format!("Tuple length mismatch: {} vs {}", 
                                     types1.len(), types2.len()));
                }
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify(t1, t2)?;
                }
                Ok(())
            }
            
            // Recursive type unification
            (Type::Rec { var: v1, body: b1 }, Type::Rec { var: v2, body: b2 }) => {
                // Simple structural comparison for now
                if v1 == v2 {
                    self.unify(b1, b2)
                } else {
                    // Could implement alpha equivalence here
                    self.unify(b1, b2)
                }
            }
            
            (Type::Hole, _) | (_, Type::Hole) => Ok(()),
            
            _ => {
                if let Some(span) = span {
                    self.report_error(TypeError::TypeMismatch {
                        expected: t1.clone(),
                        found: t2.clone(),
                        span,
                    });
                }
                Err(format!("Cannot unify {} with {}", t1, t2))
            },
        }
    }
    
    fn unify_effects(&mut self, _e1: &EffectSet, _e2: &EffectSet) -> StdResult<(), String> {
        // TODO: Implement proper effect unification
        Ok(())
    }
    
    fn combine_effects(&mut self, e1: EffectSet, e2: EffectSet) -> StdResult<EffectSet, String> {
        match (e1, e2) {
            (EffectSet::Empty, e) | (e, EffectSet::Empty) => Ok(e),
            
            (EffectSet::Row { mut effects, tail }, EffectSet::Row { effects: mut effects2, tail: tail2 }) => {
                effects.append(&mut effects2);
                let combined_tail = match (tail, tail2) {
                    (Some(t1), Some(t2)) => Some(Box::new(self.combine_effects(*t1, *t2)?)),
                    (Some(t), None) | (None, Some(t)) => Some(t),
                    (None, None) => None,
                };
                Ok(EffectSet::Row { effects, tail: combined_tail })
            }
            
            (e1, _) => {
                // TODO: Handle other effect combinations
                Ok(e1.clone())
            }
        }
    }
}

/// Extract effect set from a type (for functions)
fn extract_effects(typ: &Type) -> EffectSet {
    match typ {
        Type::Fun { effects, .. } => effects.clone(),
        _ => EffectSet::Empty,
    }
}

/// Public interface for type inference
pub fn infer_module(module: &Module) -> StdResult<TypeEnv, String> {
    let mut ctx = InferenceContext::new();
    
    // Process items in dependency order
    for item in &module.items {
        match item {
            Item::ValueDef(value_def) => {
                infer_value_def(&mut ctx, value_def)?;
            }
            Item::TypeDef(type_def) => {
                infer_type_def(&mut ctx, type_def)?;
            }
            _ => {
                // TODO: Handle other item types
            }
        }
    }
    
    Ok(ctx.env)
}

fn infer_value_def(ctx: &mut InferenceContext, value_def: &ValueDef) -> StdResult<(), String> {
    let result = ctx.infer_expr(&value_def.body)?;
    
    // Check against annotation if present
    if let Some(ann) = &value_def.type_annotation {
        let expected_type = ctx.ast_type_to_type(ann)?;
        ctx.unify(&result.typ, &expected_type)?;
    }
    
    // Generalize and add to environment
    let scheme = ctx.generalize(&result.typ, &result.effects);
    ctx.env.insert_var(value_def.name, scheme);
    
    Ok(())
}

fn infer_type_def(_ctx: &mut InferenceContext, _type_def: &TypeDef) -> StdResult<(), String> {
    // TODO: Process type definitions
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{ast::Parameter, span::Span, symbol::Symbol};
    
    fn test_span() -> Span {
        Span::new(FileId::INVALID, ByteOffset(0), ByteOffset(0))
    }
    
    #[test]
    fn test_literal_inference() {
        let mut ctx = InferenceContext::new();
        let lit = Literal::Integer(42);
        let result = ctx.infer_literal(&lit).unwrap();
        
        assert!(matches!(result.typ, Type::Con(_)));
        assert!(matches!(result.effects, EffectSet::Empty));
    }
    
    #[test]
    fn test_lambda_inference() {
        let mut ctx = InferenceContext::new();
        
        let param = Pattern::Variable(Symbol::intern("x"), test_span());
        let body = Expr::Var(Symbol::intern("x"), test_span());
        
        let result = ctx.infer_lambda(&[param], &body, None).unwrap();
        
        match result.typ {
            Type::Fun { params, return_type, .. } => {
                assert_eq!(params.len(), 1);
                // Identity function should have same input and output type
                assert_eq!(params[0], *return_type);
            }
            _ => panic!("Expected function type"),
        }
    }
    
    #[test]
    fn test_application_inference() {
        let mut ctx = InferenceContext::new();
        
        // Set up identity function in environment
        let id_type = Type::Fun {
            params: vec![Type::Var(TypeVar(0))],
            return_type: Box::new(Type::Var(TypeVar(0))),
            effects: EffectSet::Empty,
        };
        let id_scheme = TypeScheme {
            type_vars: vec![TypeVar(0)],
            effect_vars: vec![],
            constraints: vec![],
            body: id_type,
        };
        ctx.env.insert_var(Symbol::intern("id"), id_scheme);
        
        let func = Expr::Var(Symbol::intern("id"), test_span());
        let arg = Expr::Literal(Literal::Integer(42), test_span());
        
        let result = ctx.infer_app(&func, &[arg]).unwrap();
        
        // Applying identity to Int should give Int
        assert!(matches!(result.typ, Type::Con(_)));
    }
}