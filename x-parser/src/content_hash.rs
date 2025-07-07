//! Content hashing for AST nodes
//! 
//! Provides content-based hashing for functions and definitions,
//! enabling content-addressed storage like Unison.

use crate::ast::*;
use crate::symbol::Symbol;
use sha2::{Sha256, Digest};

/// Compute content hash for a value definition
pub fn hash_value_def(def: &ValueDef) -> String {
    let mut hasher = ContentHasher::new();
    hasher.hash_value_def(def);
    hasher.finalize()
}

/// Compute content hash for an expression
pub fn hash_expr(expr: &Expr) -> String {
    let mut hasher = ContentHasher::new();
    hasher.hash_expr(expr);
    hasher.finalize()
}

/// Content hasher that produces deterministic hashes
struct ContentHasher {
    hasher: Sha256,
}

impl ContentHasher {
    fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    fn finalize(self) -> String {
        let result = self.hasher.finalize();
        hex::encode(result)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    fn write_u8(&mut self, byte: u8) {
        self.hasher.update([byte]);
    }

    fn write_string(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
        self.write_u8(0); // null terminator for disambiguation
    }

    fn write_symbol(&mut self, sym: &Symbol) {
        self.write_string(sym.as_str());
    }

    fn hash_value_def(&mut self, def: &ValueDef) {
        // Hash the structure tag
        self.write_u8(b'V'); // ValueDef
        
        // Hash parameters (not the name)
        self.write_u8(def.parameters.len() as u8);
        for param in &def.parameters {
            self.hash_pattern(param);
        }
        
        // Hash the body
        self.hash_expr(&def.body);
        
        // Hash type annotation if present
        if let Some(ty) = &def.type_annotation {
            self.write_u8(1);
            self.hash_type(ty);
        } else {
            self.write_u8(0);
        }
        
        // Hash visibility
        self.hash_visibility(&def.visibility);
        
        // Hash purity
        self.hash_purity(&def.purity);
    }

    fn hash_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit, _) => {
                self.write_u8(b'L');
                self.hash_literal(lit);
            }
            Expr::Var(name, _) => {
                self.write_u8(b'V');
                self.write_symbol(name);
            }
            Expr::App(func, args, _) => {
                self.write_u8(b'A');
                self.hash_expr(func);
                self.write_u8(args.len() as u8);
                for arg in args {
                    self.hash_expr(arg);
                }
            }
            Expr::Lambda { parameters, body, .. } => {
                self.write_u8(b'F');
                self.write_u8(parameters.len() as u8);
                for param in parameters {
                    self.hash_pattern(param);
                }
                self.hash_expr(body);
            }
            Expr::Let { pattern, type_annotation, value, body, .. } => {
                self.write_u8(b'L');
                self.write_u8(b'e');
                self.hash_pattern(pattern);
                if let Some(ty) = type_annotation {
                    self.write_u8(1);
                    self.hash_type(ty);
                } else {
                    self.write_u8(0);
                }
                self.hash_expr(value);
                self.hash_expr(body);
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.write_u8(b'I');
                self.hash_expr(condition);
                self.hash_expr(then_branch);
                self.hash_expr(else_branch);
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.write_u8(b'M');
                self.hash_expr(scrutinee);
                self.write_u8(arms.len() as u8);
                for arm in arms {
                    self.hash_match_arm(arm);
                }
            }
            Expr::Do { statements, .. } => {
                self.write_u8(b'D');
                self.write_u8(statements.len() as u8);
                for stmt in statements {
                    self.hash_do_statement(stmt);
                }
            }
            Expr::Handle { expr, handlers, return_clause, .. } => {
                self.write_u8(b'H');
                self.hash_expr(expr);
                self.write_u8(handlers.len() as u8);
                for handler in handlers {
                    self.hash_effect_handler(handler);
                }
                if let Some(ret) = return_clause {
                    self.write_u8(1);
                    self.hash_return_clause(ret);
                } else {
                    self.write_u8(0);
                }
            }
            Expr::Resume { value, .. } => {
                self.write_u8(b'R');
                self.hash_expr(value);
            }
            Expr::Perform { effect, operation, args, .. } => {
                self.write_u8(b'P');
                self.write_symbol(effect);
                self.write_symbol(operation);
                self.write_u8(args.len() as u8);
                for arg in args {
                    self.hash_expr(arg);
                }
            }
            Expr::Ann { expr, type_annotation, .. } => {
                self.write_u8(b'T');
                self.hash_expr(expr);
                self.hash_type(type_annotation);
            }
        }
    }

    fn hash_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard(_) => {
                self.write_u8(b'_');
            }
            Pattern::Variable(name, _) => {
                self.write_u8(b'v');
                self.write_symbol(name);
            }
            Pattern::Literal(lit, _) => {
                self.write_u8(b'l');
                self.hash_literal(lit);
            }
            Pattern::Constructor { name, args, .. } => {
                self.write_u8(b'c');
                self.write_symbol(name);
                self.write_u8(args.len() as u8);
                for arg in args {
                    self.hash_pattern(arg);
                }
            }
            Pattern::Tuple { patterns, .. } => {
                self.write_u8(b't');
                self.write_u8(patterns.len() as u8);
                for p in patterns {
                    self.hash_pattern(p);
                }
            }
            Pattern::Record { fields, rest, .. } => {
                self.write_u8(b'r');
                self.write_u8(fields.len() as u8);
                // Sort fields for deterministic hashing
                let mut sorted_fields: Vec<_> = fields.iter().collect();
                sorted_fields.sort_by_key(|(k, _)| k.as_str());
                for (k, v) in sorted_fields {
                    self.write_symbol(k);
                    self.hash_pattern(v);
                }
                if let Some(rest) = rest {
                    self.write_u8(1);
                    self.hash_pattern(rest);
                } else {
                    self.write_u8(0);
                }
            }
            Pattern::Or { left, right, .. } => {
                self.write_u8(b'o');
                self.hash_pattern(left);
                self.hash_pattern(right);
            }
            Pattern::As { pattern, name, .. } => {
                self.write_u8(b'a');
                self.hash_pattern(pattern);
                self.write_symbol(name);
            }
            Pattern::Ann { pattern, type_annotation, .. } => {
                self.write_u8(b'n');
                self.hash_pattern(pattern);
                self.hash_type(type_annotation);
            }
        }
    }

    fn hash_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Integer(n) => {
                self.write_u8(b'i');
                self.write_string(&n.to_string());
            }
            Literal::Float(f) => {
                self.write_u8(b'f');
                // Use a canonical representation for floats
                self.write_string(&format!("{f:?}"));
            }
            Literal::String(s) => {
                self.write_u8(b's');
                self.write_string(s);
            }
            Literal::Bool(b) => {
                self.write_u8(if *b { b't' } else { b'f' });
            }
            Literal::Unit => {
                self.write_u8(b'u');
            }
        }
    }

    fn hash_type(&mut self, ty: &Type) {
        match ty {
            Type::Var(name, _) => {
                self.write_u8(b'v');
                self.write_symbol(name);
            }
            Type::Con(name, _) => {
                self.write_u8(b'c');
                self.write_symbol(name);
            }
            Type::App(func, args, _) => {
                self.write_u8(b'a');
                self.hash_type(func);
                self.write_u8(args.len() as u8);
                for arg in args {
                    self.hash_type(arg);
                }
            }
            Type::Fun { params, return_type, effects, .. } => {
                self.write_u8(b'f');
                self.write_u8(params.len() as u8);
                for param in params {
                    self.hash_type(param);
                }
                self.hash_type(return_type);
                self.hash_effect_set(effects);
            }
            Type::Record { fields, rest, .. } => {
                self.write_u8(b'r');
                self.write_u8(fields.len() as u8);
                // Sort fields for deterministic hashing
                let mut sorted_fields: Vec<_> = fields.iter().collect();
                sorted_fields.sort_by_key(|(k, _)| k.as_str());
                for (k, v) in sorted_fields {
                    self.write_symbol(k);
                    self.hash_type(v);
                }
                if let Some(rest) = rest {
                    self.write_u8(1);
                    self.hash_type(rest);
                } else {
                    self.write_u8(0);
                }
            }
            Type::Tuple { types, .. } => {
                self.write_u8(b't');
                self.write_u8(types.len() as u8);
                for elem in types {
                    self.hash_type(elem);
                }
            }
            Type::Effects(eff_set, _) => {
                self.write_u8(b'e');
                self.hash_effect_set(eff_set);
            }
            Type::Forall { type_params, body, .. } => {
                self.write_u8(b'A');
                self.write_u8(type_params.len() as u8);
                for param in type_params {
                    self.hash_type_param(param);
                }
                self.hash_type(body);
            }
            Type::Exists { type_params, body, .. } => {
                self.write_u8(b'E');
                self.write_u8(type_params.len() as u8);
                for param in type_params {
                    self.hash_type_param(param);
                }
                self.hash_type(body);
            }
            Type::Variant { variants, rest, .. } => {
                self.write_u8(b'V');
                self.write_u8(variants.len() as u8);
                // Sort variants for deterministic hashing
                let mut sorted_variants: Vec<_> = variants.iter().collect();
                sorted_variants.sort_by_key(|(k, _)| k.as_str());
                for (k, v) in sorted_variants {
                    self.write_symbol(k);
                    self.hash_type(v);
                }
                if let Some(rest) = rest {
                    self.write_u8(1);
                    self.hash_type(rest);
                } else {
                    self.write_u8(0);
                }
            }
            Type::Row { fields, rest, .. } => {
                self.write_u8(b'R');
                self.write_u8(fields.len() as u8);
                // Sort fields for deterministic hashing
                let mut sorted_fields: Vec<_> = fields.iter().collect();
                sorted_fields.sort_by_key(|(k, _)| k.as_str());
                for (k, v) in sorted_fields {
                    self.write_symbol(k);
                    self.hash_type(v);
                }
                if let Some(rest) = rest {
                    self.write_u8(1);
                    self.hash_type(rest);
                } else {
                    self.write_u8(0);
                }
            }
            Type::Hole(_) => {
                self.write_u8(b'?');
            }
        }
    }

    fn hash_visibility(&mut self, vis: &Visibility) {
        match vis {
            Visibility::Public => self.write_u8(0),
            Visibility::Private => self.write_u8(1),
            Visibility::Crate => self.write_u8(2),
            Visibility::Package => self.write_u8(3),
            Visibility::Super => self.write_u8(4),
            Visibility::InPath(path) => {
                self.write_u8(5);
                self.write_string(&path.to_string());
            }
            Visibility::SelfModule => self.write_u8(6),
            Visibility::Component { export, import, interface } => {
                self.write_u8(7);
                self.write_u8(if *export { 1 } else { 0 });
                self.write_u8(if *import { 1 } else { 0 });
                if let Some(iface) = interface {
                    self.write_u8(1);
                    self.write_symbol(iface);
                } else {
                    self.write_u8(0);
                }
            }
        }
    }

    fn hash_purity(&mut self, purity: &Purity) {
        match purity {
            Purity::Pure => self.write_u8(0),
            Purity::Impure => self.write_u8(1),
            Purity::Inferred => self.write_u8(2),
        }
    }

    fn hash_match_arm(&mut self, arm: &MatchArm) {
        self.hash_pattern(&arm.pattern);
        if let Some(guard) = &arm.guard {
            self.write_u8(1);
            self.hash_expr(guard);
        } else {
            self.write_u8(0);
        }
        self.hash_expr(&arm.body);
    }

    fn hash_do_statement(&mut self, stmt: &DoStatement) {
        match stmt {
            DoStatement::Let { pattern, expr, .. } => {
                self.write_u8(b'l');
                self.hash_pattern(pattern);
                self.hash_expr(expr);
            }
            DoStatement::Bind { pattern, expr, .. } => {
                self.write_u8(b'b');
                self.hash_pattern(pattern);
                self.hash_expr(expr);
            }
            DoStatement::Expr(expr) => {
                self.write_u8(b'e');
                self.hash_expr(expr);
            }
        }
    }

    fn hash_effect_handler(&mut self, handler: &EffectHandler) {
        self.hash_effect_ref(&handler.effect);
        self.write_symbol(&handler.operation);
        self.write_u8(handler.parameters.len() as u8);
        for param in &handler.parameters {
            self.hash_pattern(param);
        }
        if let Some(cont) = &handler.continuation {
            self.write_u8(1);
            self.write_symbol(cont);
        } else {
            self.write_u8(0);
        }
        self.hash_expr(&handler.body);
    }

    fn hash_return_clause(&mut self, clause: &ReturnClause) {
        self.hash_pattern(&clause.parameter);
        self.hash_expr(&clause.body);
    }

    fn hash_effect_set(&mut self, effects: &EffectSet) {
        let mut sorted_effects = effects.effects.clone();
        sorted_effects.sort_by_key(|e| e.name.as_str());
        self.write_u8(sorted_effects.len() as u8);
        for effect in &sorted_effects {
            self.hash_effect_ref(effect);
        }
        if let Some(row_var) = &effects.row_var {
            self.write_u8(1);
            self.write_symbol(row_var);
        } else {
            self.write_u8(0);
        }
    }

    fn hash_kind(&mut self, kind: &Kind) {
        match kind {
            Kind::Type => self.write_u8(0),
            Kind::Effect => self.write_u8(1),
            Kind::Row => self.write_u8(2),
            Kind::Arrow(k1, k2) => {
                self.write_u8(3);
                self.hash_kind(k1);
                self.hash_kind(k2);
            }
        }
    }

    fn hash_effect_ref(&mut self, effect: &EffectRef) {
        self.write_symbol(&effect.name);
        self.write_u8(effect.args.len() as u8);
        for arg in &effect.args {
            self.hash_type(arg);
        }
    }

    fn hash_type_param(&mut self, param: &TypeParam) {
        self.write_symbol(&param.name);
        if let Some(kind) = &param.kind {
            self.write_u8(1);
            self.hash_kind(kind);
        } else {
            self.write_u8(0);
        }
        self.write_u8(param.constraints.len() as u8);
        for constraint in &param.constraints {
            self.hash_type_constraint(constraint);
        }
    }

    fn hash_type_constraint(&mut self, constraint: &TypeConstraint) {
        self.write_symbol(&constraint.class);
        self.write_u8(constraint.types.len() as u8);
        for ty in &constraint.types {
            self.hash_type(ty);
        }
    }
}