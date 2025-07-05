//! Type system implementation for x Language
//! 
//! This module implements a Hindley-Milner type system extended with:
//! - Algebraic effects and handlers
//! - Row polymorphism for effects
//! - Kind system for higher-kinded types

use x_parser::Symbol;
use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Deserialize, Serialize};

/// Type variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeVar(pub u32);

/// Effect variable identifier  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EffectVar(pub u32);

/// Row variable identifier (for open effect rows)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RowVar(pub u32);

/// Types in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Type variable (unification variable)
    Var(TypeVar),
    
    /// Type constructor (Int, String, Bool, etc.)
    Con(Symbol),
    
    /// Type application (List Int, Maybe String, etc.)
    App(Box<Type>, Vec<Type>),
    
    /// Function type with effects
    Fun {
        params: Vec<Type>,
        return_type: Box<Type>,
        effects: EffectSet,
    },
    
    /// Universal quantification (forall a. a -> a)
    Forall {
        type_vars: Vec<TypeVar>,
        effect_vars: Vec<EffectVar>,
        body: Box<Type>,
    },
    
    /// Record type {x: Int, y: String}
    Record(Vec<(Symbol, Type)>),
    
    /// Variant type (|A Int | B String|)
    Variant(Vec<(Symbol, Vec<Type>)>),
    
    /// Tuple type (Int, String, Bool)
    Tuple(Vec<Type>),
    
    /// Type hole for inference
    Hole,
    
    /// Recursive type (μα. α -> List α)
    Rec {
        var: TypeVar,
        body: Box<Type>,
    },
    
    /// Unknown type (used during type checking)
    Unknown,
}

/// Effect sets with row polymorphism
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectSet {
    /// Empty effect set
    Empty,
    
    /// Effect variable
    Var(EffectVar),
    
    /// Concrete effects with optional tail
    Row {
        effects: Vec<Effect>,
        tail: Option<Box<EffectSet>>,
    },
}

/// Individual effects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Effect {
    pub name: Symbol,
    pub operations: Vec<Operation>,
}

/// Effect operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: Symbol,
    pub params: Vec<Type>,
    pub return_type: Type,
}

/// Kinds for higher-kinded types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    /// Base kind (*)
    Star,
    
    /// Arrow kind (* -> *)
    Arrow(Box<Kind>, Box<Kind>),
    
    /// Effect kind (Effect)
    Effect,
    
    /// Row kind (Row Effect)
    Row(Box<Kind>),
}

/// Type scheme (polymorphic type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScheme {
    pub type_vars: Vec<TypeVar>,
    pub effect_vars: Vec<EffectVar>,
    pub constraints: Vec<Constraint>,
    pub body: Type,
}

impl TypeScheme {
    /// Create a monomorphic type scheme (no type variables)
    pub fn monotype(typ: Type) -> Self {
        TypeScheme {
            type_vars: Vec::new(),
            effect_vars: Vec::new(),
            constraints: Vec::new(),
            body: typ,
        }
    }
}

/// Type constraints for qualified types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constraint {
    /// Type class constraint
    Class {
        class: Symbol,
        types: Vec<Type>,
    },
    
    /// Effect constraint (effect E on type T)
    Effect {
        effect: Effect,
        type_: Type,
    },
    
    /// Row constraint (extensible records/effects)
    Row {
        lacks: Symbol,
        row: EffectSet,
    },
}

/// Type environment
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Variable bindings
    pub vars: HashMap<Symbol, TypeScheme>,
    
    /// Type constructor bindings
    pub type_cons: HashMap<Symbol, (Kind, Vec<TypeVar>)>,
    
    /// Effect bindings
    pub effects: HashMap<Symbol, Effect>,
    
    /// Type class instances
    pub instances: HashMap<Symbol, Vec<Instance>>,
}

/// Type class instance
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub constraints: Vec<Constraint>,
    pub head: Constraint,
}

/// Type variable generator
#[derive(Debug, Clone)]
pub struct VarGen {
    next_type_var: u32,
    next_effect_var: u32,
    next_row_var: u32,
}

impl VarGen {
    pub fn new() -> Self {
        VarGen {
            next_type_var: 0,
            next_effect_var: 0,
            next_row_var: 0,
        }
    }
    
    pub fn fresh_type_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_type_var);
        self.next_type_var += 1;
        var
    }
    
    pub fn fresh_effect_var(&mut self) -> EffectVar {
        let var = EffectVar(self.next_effect_var);
        self.next_effect_var += 1;
        var
    }
    
    pub fn fresh_row_var(&mut self) -> RowVar {
        let var = RowVar(self.next_row_var);
        self.next_row_var += 1;
        var
    }
}

impl Default for VarGen {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            vars: HashMap::new(),
            type_cons: HashMap::new(),
            effects: HashMap::new(),
            instances: HashMap::new(),
        };
        
        // Add built-in types
        env.add_builtin_types();
        env
    }
    
    fn add_builtin_types(&mut self) {
        use x_parser::symbol::symbols;
        
        // Basic types
        self.type_cons.insert(symbols::INT(), (Kind::Star, vec![]));
        self.type_cons.insert(symbols::FLOAT(), (Kind::Star, vec![]));
        self.type_cons.insert(symbols::STRING(), (Kind::Star, vec![]));
        self.type_cons.insert(symbols::BOOL(), (Kind::Star, vec![]));
        self.type_cons.insert(symbols::UNIT_TYPE(), (Kind::Star, vec![]));
        
        // Built-in effects
        let io_effect = Effect {
            name: symbols::IO(),
            operations: vec![
                Operation {
                    name: symbols::PRINT(),
                    params: vec![Type::Con(symbols::STRING())],
                    return_type: Type::Con(symbols::UNIT_TYPE()),
                },
                Operation {
                    name: symbols::READ(),
                    params: vec![],
                    return_type: Type::Con(symbols::STRING()),
                },
            ],
        };
        self.effects.insert(symbols::IO(), io_effect);
        
        let state_effect = Effect {
            name: symbols::STATE(),
            operations: vec![
                Operation {
                    name: symbols::GET(),
                    params: vec![],
                    return_type: Type::Var(TypeVar(0)), // polymorphic
                },
                Operation {
                    name: symbols::PUT(),
                    params: vec![Type::Var(TypeVar(0))],
                    return_type: Type::Con(symbols::UNIT_TYPE()),
                },
            ],
        };
        self.effects.insert(symbols::STATE(), state_effect);
    }
    
    pub fn lookup_var(&self, name: Symbol) -> Option<&TypeScheme> {
        self.vars.get(&name)
    }
    
    pub fn insert_var(&mut self, name: Symbol, scheme: TypeScheme) {
        self.vars.insert(name, scheme);
    }
    
    pub fn lookup_type_con(&self, name: Symbol) -> Option<&(Kind, Vec<TypeVar>)> {
        self.type_cons.get(&name)
    }
    
    pub fn lookup_effect(&self, name: Symbol) -> Option<&Effect> {
        self.effects.get(&name)
    }
    
    pub fn enter_scope(&self) -> Self {
        self.clone()
    }
    
    pub fn extend(&mut self, other: &TypeEnv) {
        self.vars.extend(other.vars.clone());
        self.type_cons.extend(other.type_cons.clone());
        self.effects.extend(other.effects.clone());
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Type {
    /// Get the kind of a type
    pub fn kind(&self, env: &TypeEnv) -> Kind {
        match self {
            Type::Var(_) => Kind::Star,
            Type::Con(name) => {
                env.lookup_type_con(*name)
                    .map(|(kind, _)| kind.clone())
                    .unwrap_or(Kind::Star)
            }
            Type::App(constructor, args) => {
                let con_kind = constructor.kind(env);
                apply_kind(con_kind, args.len())
            }
            Type::Fun { .. } => Kind::Star,
            Type::Forall { body, .. } => body.kind(env),
            Type::Record(_) => Kind::Star,
            Type::Variant(_) => Kind::Star,
            Type::Tuple(_) => Kind::Star,
            Type::Hole => Kind::Star,
            Type::Rec { body, .. } => body.kind(env),
            Type::Unknown => Kind::Star,
        }
    }
    
    /// Get free type variables
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut vars = HashSet::new();
        self.collect_free_vars(&mut vars);
        vars
    }
    
    fn collect_free_vars(&self, vars: &mut HashSet<TypeVar>) {
        match self {
            Type::Var(v) => {
                vars.insert(*v);
            }
            Type::Con(_) => {}
            Type::App(con, args) => {
                con.collect_free_vars(vars);
                for arg in args {
                    arg.collect_free_vars(vars);
                }
            }
            Type::Fun { params, return_type, effects } => {
                for param in params {
                    param.collect_free_vars(vars);
                }
                return_type.collect_free_vars(vars);
                effects.collect_free_vars(vars);
            }
            Type::Forall { type_vars, body, .. } => {
                let mut body_vars = HashSet::new();
                body.collect_free_vars(&mut body_vars);
                for var in body_vars {
                    if !type_vars.contains(&var) {
                        vars.insert(var);
                    }
                }
            }
            Type::Record(fields) => {
                for (_, typ) in fields {
                    typ.collect_free_vars(vars);
                }
            }
            Type::Variant(variants) => {
                for (_, types) in variants {
                    for typ in types {
                        typ.collect_free_vars(vars);
                    }
                }
            }
            Type::Tuple(types) => {
                for typ in types {
                    typ.collect_free_vars(vars);
                }
            }
            Type::Hole => {}
            Type::Rec { var, body } => {
                let mut body_vars = HashSet::new();
                body.collect_free_vars(&mut body_vars);
                // Remove the bound variable
                body_vars.remove(var);
                vars.extend(body_vars);
            }
            Type::Unknown => {}
        }
    }
    
    /// Apply a substitution to this type
    pub fn apply_subst(&self, subst: &Substitution) -> Type {
        match self {
            Type::Var(v) => {
                subst.lookup_type(*v).cloned().unwrap_or_else(|| self.clone())
            }
            Type::Con(_) => self.clone(),
            Type::App(con, args) => {
                Type::App(
                    Box::new(con.apply_subst(subst)),
                    args.iter().map(|t| t.apply_subst(subst)).collect(),
                )
            }
            Type::Fun { params, return_type, effects } => {
                Type::Fun {
                    params: params.iter().map(|t| t.apply_subst(subst)).collect(),
                    return_type: Box::new(return_type.apply_subst(subst)),
                    effects: effects.apply_subst(subst),
                }
            }
            Type::Forall { type_vars, effect_vars, body } => {
                // Remove bound variables from substitution
                let mut filtered_subst = subst.clone();
                for var in type_vars {
                    filtered_subst.remove_type(*var);
                }
                for var in effect_vars {
                    filtered_subst.remove_effect(*var);
                }
                Type::Forall {
                    type_vars: type_vars.clone(),
                    effect_vars: effect_vars.clone(),
                    body: Box::new(body.apply_subst(&filtered_subst)),
                }
            }
            Type::Record(fields) => {
                Type::Record(
                    fields.iter()
                        .map(|(name, typ)| (*name, typ.apply_subst(subst)))
                        .collect()
                )
            }
            Type::Variant(variants) => {
                Type::Variant(
                    variants.iter()
                        .map(|(name, types)| {
                            (*name, types.iter().map(|t| t.apply_subst(subst)).collect())
                        })
                        .collect()
                )
            }
            Type::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| t.apply_subst(subst)).collect())
            }
            Type::Hole => Type::Hole,
            Type::Rec { var, body } => {
                // Don't substitute the bound variable
                let mut filtered_subst = subst.clone();
                filtered_subst.remove_type(*var);
                Type::Rec {
                    var: *var,
                    body: Box::new(body.apply_subst(&filtered_subst)),
                }
            },
            Type::Unknown => Type::Unknown,
        }
    }
}

impl EffectSet {
    pub fn empty() -> Self {
        EffectSet::Empty
    }
    
    pub fn collect_free_vars(&self, vars: &mut HashSet<TypeVar>) {
        match self {
            EffectSet::Empty => {}
            EffectSet::Var(_) => {} // Effect vars are separate
            EffectSet::Row { effects, tail } => {
                for effect in effects {
                    effect.collect_free_vars(vars);
                }
                if let Some(tail) = tail {
                    tail.collect_free_vars(vars);
                }
            }
        }
    }
    
    pub fn apply_subst(&self, subst: &Substitution) -> EffectSet {
        match self {
            EffectSet::Empty => EffectSet::Empty,
            EffectSet::Var(v) => {
                subst.lookup_effect(*v).cloned().unwrap_or_else(|| self.clone())
            }
            EffectSet::Row { effects, tail } => {
                EffectSet::Row {
                    effects: effects.iter().map(|e| e.apply_subst(subst)).collect(),
                    tail: tail.as_ref().map(|t| Box::new(t.apply_subst(subst))),
                }
            }
        }
    }
    
    pub fn contains_effect(&self, effect_name: Symbol) -> bool {
        match self {
            EffectSet::Empty => false,
            EffectSet::Var(_) => false, // Can't determine
            EffectSet::Row { effects, tail } => {
                effects.iter().any(|e| e.name == effect_name) ||
                tail.as_ref().map_or(false, |t| t.contains_effect(effect_name))
            }
        }
    }
}

impl Effect {
    pub fn collect_free_vars(&self, vars: &mut HashSet<TypeVar>) {
        for op in &self.operations {
            op.collect_free_vars(vars);
        }
    }
    
    pub fn apply_subst(&self, subst: &Substitution) -> Effect {
        Effect {
            name: self.name,
            operations: self.operations.iter().map(|op| op.apply_subst(subst)).collect(),
        }
    }
}

impl Operation {
    pub fn collect_free_vars(&self, vars: &mut HashSet<TypeVar>) {
        for param in &self.params {
            param.collect_free_vars(vars);
        }
        self.return_type.collect_free_vars(vars);
    }
    
    pub fn apply_subst(&self, subst: &Substitution) -> Operation {
        Operation {
            name: self.name,
            params: self.params.iter().map(|t| t.apply_subst(subst)).collect(),
            return_type: self.return_type.apply_subst(subst),
        }
    }
}

/// Type substitution
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub type_subst: HashMap<TypeVar, Type>,
    pub effect_subst: HashMap<EffectVar, EffectSet>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            type_subst: HashMap::new(),
            effect_subst: HashMap::new(),
        }
    }
    
    pub fn insert_type(&mut self, var: TypeVar, typ: Type) {
        self.type_subst.insert(var, typ);
    }
    
    pub fn insert_effect(&mut self, var: EffectVar, effects: EffectSet) {
        self.effect_subst.insert(var, effects);
    }
    
    pub fn lookup_type(&self, var: TypeVar) -> Option<&Type> {
        self.type_subst.get(&var)
    }
    
    pub fn lookup_effect(&self, var: EffectVar) -> Option<&EffectSet> {
        self.effect_subst.get(&var)
    }
    
    pub fn remove_type(&mut self, var: TypeVar) {
        self.type_subst.remove(&var);
    }
    
    pub fn remove_effect(&mut self, var: EffectVar) {
        self.effect_subst.remove(&var);
    }
    
    /// Compose two substitutions
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();
        
        // Apply other to our types, then add other's types
        for (var, typ) in &self.type_subst {
            result.insert_type(*var, typ.apply_subst(other));
        }
        for (var, typ) in &other.type_subst {
            if !result.type_subst.contains_key(var) {
                result.insert_type(*var, typ.clone());
            }
        }
        
        // Same for effects
        for (var, effects) in &self.effect_subst {
            result.insert_effect(*var, effects.apply_subst(other));
        }
        for (var, effects) in &other.effect_subst {
            if !result.effect_subst.contains_key(var) {
                result.insert_effect(*var, effects.clone());
            }
        }
        
        result
    }
}

/// Helper functions for recursive types
impl Type {
    /// Unfold a recursive type (μα.T → T[μα.T/α])
    pub fn unfold_rec(&self) -> Type {
        match self {
            Type::Rec { var, body } => {
                let mut subst = Substitution::new();
                subst.insert_type(*var, self.clone());
                body.apply_subst(&subst)
            }
            _ => self.clone(),
        }
    }
    
    /// Try to fold a type into a recursive type if it matches a pattern
    pub fn try_fold_rec(typ: &Type, rec_var: TypeVar, rec_body: &Type) -> Option<Type> {
        // Check if typ matches the unfolded form of μrec_var.rec_body
        let unfolded = Type::Rec { 
            var: rec_var, 
            body: Box::new(rec_body.clone()) 
        }.unfold_rec();
        
        if Self::structurally_equal(typ, &unfolded) {
            Some(Type::Rec { 
                var: rec_var, 
                body: Box::new(rec_body.clone()) 
            })
        } else {
            None
        }
    }
    
    /// Check structural equality (ignoring variable names)
    pub fn structurally_equal(t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Var(_), Type::Var(_)) => true, // All variables are considered equal for structure
            (Type::Con(n1), Type::Con(n2)) => n1 == n2,
            (Type::App(c1, args1), Type::App(c2, args2)) => {
                Self::structurally_equal(c1, c2) && 
                args1.len() == args2.len() &&
                args1.iter().zip(args2.iter()).all(|(a1, a2)| Self::structurally_equal(a1, a2))
            }
            (Type::Fun { params: p1, return_type: r1, .. }, 
             Type::Fun { params: p2, return_type: r2, .. }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(a1, a2)| Self::structurally_equal(a1, a2)) &&
                Self::structurally_equal(r1, r2)
            }
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                types1.len() == types2.len() &&
                types1.iter().zip(types2.iter()).all(|(t1, t2)| Self::structurally_equal(t1, t2))
            }
            (Type::Record(fields1), Type::Record(fields2)) => {
                fields1.len() == fields2.len() &&
                fields1.iter().zip(fields2.iter()).all(|((n1, t1), (n2, t2))| {
                    n1 == n2 && Self::structurally_equal(t1, t2)
                })
            }
            (Type::Rec { body: b1, .. }, Type::Rec { body: b2, .. }) => {
                Self::structurally_equal(b1, b2)
            }
            (Type::Hole, Type::Hole) => true,
            _ => false,
        }
    }
}

/// Helper function to apply kind arrows
fn apply_kind(kind: Kind, arity: usize) -> Kind {
    match (kind, arity) {
        (kind, 0) => kind,
        (Kind::Arrow(_, result), n) if n > 0 => apply_kind(*result, n - 1),
        _ => Kind::Star, // Default fallback
    }
}

/// Pretty printing for types
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Var(TypeVar(n)) => write!(f, "a{}", n),
            Type::Con(name) => write!(f, "{}", name),
            Type::App(con, args) => {
                write!(f, "{}", con)?;
                if !args.is_empty() {
                    write!(f, " ")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, " ")?; }
                        write!(f, "{}", arg)?;
                    }
                }
                Ok(())
            }
            Type::Fun { params, return_type, effects } => {
                if params.len() == 1 {
                    write!(f, "{} -> {}", params[0], return_type)?;
                } else {
                    write!(f, "(")?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ") -> {}", return_type)?;
                }
                if !matches!(effects, EffectSet::Empty) {
                    write!(f, " / {}", effects)?;
                }
                Ok(())
            }
            Type::Forall { type_vars, body, .. } => {
                write!(f, "∀")?;
                for var in type_vars {
                    write!(f, " {}", Type::Var(*var))?;
                }
                write!(f, ". {}", body)
            }
            Type::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, typ)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, typ)?;
                }
                write!(f, "}}")
            }
            Type::Variant(variants) => {
                write!(f, "|")?;
                for (i, (name, types)) in variants.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, " {}", name)?;
                    for typ in types {
                        write!(f, " {}", typ)?;
                    }
                }
                write!(f, " |")
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, typ) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", typ)?;
                }
                write!(f, ")")
            }
            Type::Hole => write!(f, "_"),
            Type::Rec { var, body } => {
                write!(f, "μ{}.{}", Type::Var(*var), body)
            },
            Type::Unknown => write!(f, "?")
        }
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectSet::Empty => write!(f, "{{}}"),
            EffectSet::Var(EffectVar(n)) => write!(f, "e{}", n),
            EffectSet::Row { effects, tail } => {
                write!(f, "{{")?;
                for (i, effect) in effects.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", effect.name)?;
                }
                if let Some(tail) = tail {
                    if !effects.is_empty() { write!(f, " | ")?; }
                    write!(f, "{}", tail)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::Symbol;
    
    #[test]
    fn test_type_construction() {
        let int_type = Type::Con(Symbol::intern("Int"));
        let string_type = Type::Con(Symbol::intern("String"));
        
        // Function type: Int -> String
        let fun_type = Type::Fun {
            params: vec![int_type.clone()],
            return_type: Box::new(string_type.clone()),
            effects: EffectSet::Empty,
        };
        
        assert_eq!(format!("{}", fun_type), "Int -> String / {}");
    }
    
    #[test]
    fn test_free_variables() {
        let var_a = TypeVar(0);
        let var_b = TypeVar(1);
        
        let typ = Type::Fun {
            params: vec![Type::Var(var_a)],
            return_type: Box::new(Type::Var(var_b)),
            effects: EffectSet::Empty,
        };
        
        let free_vars = typ.free_vars();
        assert!(free_vars.contains(&var_a));
        assert!(free_vars.contains(&var_b));
        assert_eq!(free_vars.len(), 2);
    }
    
    #[test]
    fn test_substitution() {
        let var_a = TypeVar(0);
        let int_type = Type::Con(Symbol::intern("Int"));
        
        let mut subst = Substitution::new();
        subst.insert_type(var_a, int_type.clone());
        
        let original = Type::Fun {
            params: vec![Type::Var(var_a)],
            return_type: Box::new(Type::Con(Symbol::intern("String"))),
            effects: EffectSet::Empty,
        };
        
        let substituted = original.apply_subst(&subst);
        
        match substituted {
            Type::Fun { params, .. } => {
                assert_eq!(params[0], int_type);
            }
            _ => panic!("Expected function type"),
        }
    }
}