//! Function signature extraction and analysis
//! 
//! This module provides utilities for extracting and analyzing
//! function signatures for version compatibility checking.

use crate::ast::*;
use crate::symbol::Symbol;
use crate::versioning::FunctionSignature;

/// Extract signature from a value definition
pub fn extract_signature(def: &ValueDef) -> Option<FunctionSignature> {
    // For now, we'll infer simple signatures from the AST
    // In a real implementation, this would use the type checker
    
    let params = infer_param_types(&def.parameters);
    let return_type = def.type_annotation.as_ref()
        .and_then(extract_return_type)
        .unwrap_or(Type::Hole(def.span)); // Unknown type
    
    // Extract effects from the body
    let effects = extract_effects(&def.body);
    
    Some(FunctionSignature {
        params,
        return_type,
        effects,
    })
}

/// Infer parameter types from patterns
fn infer_param_types(params: &[Pattern]) -> Vec<Type> {
    params.iter().map(|p| {
        match p {
            Pattern::Variable(name, span) => {
                // Without type annotation, we can't know the type
                Type::Var(*name, *span)
            }
            Pattern::Ann { type_annotation, .. } => {
                type_annotation.clone()
            }
            _ => {
                // Complex patterns - would need full type inference
                Type::Hole(p.span())
            }
        }
    }).collect()
}

/// Extract return type from a type annotation
fn extract_return_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Fun { return_type, .. } => Some(return_type.as_ref().clone()),
        _ => Some(ty.clone()),
    }
}

/// Extract effects used in an expression
fn extract_effects(expr: &Expr) -> Vec<Symbol> {
    let mut effects = Vec::new();
    collect_effects(expr, &mut effects);
    effects.sort();
    effects.dedup();
    effects
}

fn collect_effects(expr: &Expr, effects: &mut Vec<Symbol>) {
    match expr {
        Expr::Perform { effect, .. } => {
            effects.push(*effect);
        }
        Expr::Handle { expr, handlers, .. } => {
            // Effects in the handled expression are captured
            collect_effects(expr, effects);
            // But we should subtract handled effects
            let handled: Vec<_> = handlers.iter().map(|h| h.effect.name).collect();
            effects.retain(|e| !handled.contains(e));
            
            // Add effects from handler bodies
            for handler in handlers {
                collect_effects(&handler.body, effects);
            }
        }
        Expr::Let { value, body, .. } => {
            collect_effects(value, effects);
            collect_effects(body, effects);
        }
        Expr::App(func, args, _) => {
            collect_effects(func, effects);
            for arg in args {
                collect_effects(arg, effects);
            }
        }
        Expr::Lambda { body, .. } => {
            collect_effects(body, effects);
        }
        Expr::If { condition, then_branch, else_branch, .. } => {
            collect_effects(condition, effects);
            collect_effects(then_branch, effects);
            collect_effects(else_branch, effects);
        }
        Expr::Match { scrutinee, arms, .. } => {
            collect_effects(scrutinee, effects);
            for arm in arms {
                collect_effects(&arm.body, effects);
            }
        }
        Expr::Do { statements, .. } => {
            for stmt in statements {
                match stmt {
                    DoStatement::Let { expr, .. } |
                    DoStatement::Bind { expr, .. } => collect_effects(expr, effects),
                    DoStatement::Expr(expr) => collect_effects(expr, effects),
                }
            }
        }
        Expr::Ann { expr, .. } => {
            collect_effects(expr, effects);
        }
        _ => {}
    }
}

/// Compare two signatures for compatibility
pub fn compare_signatures(old: &FunctionSignature, new: &FunctionSignature) -> SignatureComparison {
    SignatureComparison {
        param_changes: compare_params(&old.params, &new.params),
        return_change: compare_types(&old.return_type, &new.return_type),
        effect_changes: compare_effects(&old.effects, &new.effects),
    }
}

#[derive(Debug, Clone)]
pub struct SignatureComparison {
    pub param_changes: Vec<ParamChange>,
    pub return_change: TypeChange,
    pub effect_changes: EffectChanges,
}

#[derive(Debug, Clone)]
pub enum ParamChange {
    TypeChanged { position: usize, from: Type, to: Type },
    Added { position: usize, param_type: Type },
    Removed { position: usize, param_type: Type },
}

#[derive(Debug, Clone)]
pub enum TypeChange {
    Unchanged,
    Changed { from: Type, to: Type },
}

#[derive(Debug, Clone)]
pub struct EffectChanges {
    pub added: Vec<Symbol>,
    pub removed: Vec<Symbol>,
}

fn compare_params(old: &[Type], new: &[Type]) -> Vec<ParamChange> {
    let mut changes = Vec::new();
    let min_len = old.len().min(new.len());
    
    // Compare common parameters
    for i in 0..min_len {
        if old[i] != new[i] {
            changes.push(ParamChange::TypeChanged {
                position: i,
                from: old[i].clone(),
                to: new[i].clone(),
            });
        }
    }
    
    // Handle added parameters
    for (i, param) in new.iter().enumerate().skip(old.len()) {
        changes.push(ParamChange::Added {
            position: i,
            param_type: param.clone(),
        });
    }
    
    // Handle removed parameters
    for (i, param) in old.iter().enumerate().skip(new.len()) {
        changes.push(ParamChange::Removed {
            position: i,
            param_type: param.clone(),
        });
    }
    
    changes
}

fn compare_types(old: &Type, new: &Type) -> TypeChange {
    if old == new {
        TypeChange::Unchanged
    } else {
        TypeChange::Changed {
            from: old.clone(),
            to: new.clone(),
        }
    }
}

fn compare_effects(old: &[Symbol], new: &[Symbol]) -> EffectChanges {
    let old_set: std::collections::HashSet<_> = old.iter().cloned().collect();
    let new_set: std::collections::HashSet<_> = new.iter().cloned().collect();
    
    EffectChanges {
        added: new_set.difference(&old_set).cloned().collect(),
        removed: old_set.difference(&new_set).cloned().collect(),
    }
}