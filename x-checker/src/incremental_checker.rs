//! Incremental type checker using Salsa framework
//! 
//! This module provides high-performance incremental type checking that only
//! recomputes types for nodes that have changed or depend on changed nodes.

use crate::types::{Type, TypeScheme, TypeVar, Substitution};
use crate::constraints::TypeConstraint;
use crate::types::{Effect, EffectSet};
use crate::checker::EffectConstraint;
use x_parser::{
    persistent_ast::{PersistentAstNode, NodeId, TypeInfo, TypeId, EffectSet as AstEffectSet},
    symbol::Symbol,
};
use salsa::{Database, Cancelled};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use im::OrdMap;

/// Salsa database for incremental computation
#[salsa::query_group(IncrementalTypeCheckDatabase)]
pub trait IncrementalTypeCheckDb: Database {
    /// Get the AST for a given node ID
    #[salsa::input]
    fn ast_node(&self, node_id: NodeId) -> Arc<PersistentAstNode>;
    
    /// Get the parent node ID for a given node
    #[salsa::input] 
    fn parent_node(&self, node_id: NodeId) -> Option<NodeId>;
    
    /// Get child node IDs for a given node
    #[salsa::input]
    fn child_nodes(&self, node_id: NodeId) -> Arc<Vec<NodeId>>;
    
    /// Infer the type of a node
    fn infer_type(&self, node_id: NodeId) -> Result<TypeScheme, TypeError>;
    
    /// Resolve symbol in scope
    fn resolve_symbol(&self, symbol: Symbol, scope: ScopeId) -> Option<SymbolInfo>;
    
    /// Get the scope for a node
    fn node_scope(&self, node_id: NodeId) -> ScopeId;
    
    /// Check effect constraints for a node
    fn check_effects(&self, node_id: NodeId) -> Result<EffectSet, EffectError>;
    
    /// Solve type constraints
    fn solve_constraints(&self, constraints: Arc<Vec<TypeConstraint>>) -> Result<Substitution, TypeError>;
    
    /// Get all nodes in a module
    fn module_nodes(&self, module_id: NodeId) -> Arc<Vec<NodeId>>;
    
    /// Get dependency graph
    fn dependency_graph(&self) -> Arc<DependencyGraph>;
}

/// Implementation database
#[derive(Default)]
#[salsa::database(IncrementalTypeCheckDatabase)]
pub struct IncrementalTypeCheckDbImpl {
    storage: salsa::Storage<Self>,
    /// Cache for computed types
    type_cache: DashMap<NodeId, TypeScheme>,
    /// Cache for effect information
    effect_cache: DashMap<NodeId, EffectSet>,
    /// Symbol resolution cache
    symbol_cache: DashMap<(Symbol, ScopeId), Option<SymbolInfo>>,
    /// Constraint solving cache
    constraint_cache: DashMap<Vec<TypeConstraint>, Substitution>,
}

impl salsa::Database for IncrementalTypeCheckDbImpl {}

/// Type checking errors
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnificationError { 
        expected: Type, 
        found: Type, 
        location: NodeId 
    },
    UnboundVariable { 
        name: Symbol, 
        location: NodeId 
    },
    TypeMismatch { 
        expected: Type, 
        found: Type, 
        location: NodeId 
    },
    CircularType { 
        type_var: TypeId, 
        location: NodeId 
    },
    InvalidApplication { 
        function_type: Type, 
        arg_types: Vec<Type>, 
        location: NodeId 
    },
    EffectMismatch { 
        expected: EffectSet, 
        found: EffectSet, 
        location: NodeId 
    },
    ConstraintUnsatisfiable {
        constraint: TypeConstraint,
        location: NodeId,
    },
}

/// Effect checking errors
#[derive(Debug, Clone, PartialEq)]
pub enum EffectError {
    UnhandledEffect {
        effect: Effect,
        location: NodeId,
    },
    EffectNotInScope {
        effect: Effect,
        location: NodeId,
    },
    HandlerMismatch {
        expected: EffectSet,
        found: EffectSet,
        location: NodeId,
    },
}

/// Scope identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u64);

impl ScopeId {
    pub fn new(id: u64) -> Self {
        ScopeId(id)
    }
    
    pub fn root() -> Self {
        ScopeId(0)
    }
}

/// Symbol information
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub type_scheme: TypeScheme,
    pub definition_location: NodeId,
    pub visibility: Visibility,
}

/// Visibility levels
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Module,
    Crate,
}

/// Dependency graph for incremental recomputation
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Map from node to nodes it depends on
    dependencies: OrdMap<NodeId, im::OrdSet<NodeId>>,
    /// Map from node to nodes that depend on it
    dependents: OrdMap<NodeId, im::OrdSet<NodeId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: OrdMap::new(),
            dependents: OrdMap::new(),
        }
    }
    
    pub fn add_dependency(&mut self, dependent: NodeId, dependency: NodeId) {
        self.dependencies
            .entry(dependent)
            .or_insert_with(im::OrdSet::new)
            .update(dependency);
        
        self.dependents
            .entry(dependency)
            .or_insert_with(im::OrdSet::new)
            .update(dependent);
    }
    
    pub fn get_dependencies(&self, node: NodeId) -> Option<&im::OrdSet<NodeId>> {
        self.dependencies.get(&node)
    }
    
    pub fn get_dependents(&self, node: NodeId) -> Option<&im::OrdSet<NodeId>> {
        self.dependents.get(&node)
    }
}

/// Main incremental type checker
pub struct IncrementalTypeChecker {
    db: IncrementalTypeCheckDbImpl,
    /// Current typing context
    type_context: TypeContext,
    /// Effect context
    effect_context: EffectContext,
    /// Scope management
    scope_manager: ScopeManager,
}

impl IncrementalTypeChecker {
    pub fn new() -> Self {
        Self {
            db: IncrementalTypeCheckDbImpl::default(),
            type_context: TypeContext::new(),
            effect_context: EffectContext::new(),
            scope_manager: ScopeManager::new(),
        }
    }
    
    /// Check the type of a node and all its dependencies
    pub fn check_node(&mut self, node_id: NodeId) -> Result<TypeScheme, TypeError> {
        self.infer_type(node_id)
    }
    
    /// Update a node and incrementally recheck affected nodes
    pub fn update_node(&mut self, node: Arc<PersistentAstNode>) -> Result<Vec<NodeId>, TypeError> {
        let node_id = node.id();
        
        // Update the AST in the database
        self.db.set_ast_node(node_id, node.clone());
        
        // Rebuild dependencies for this node
        self.rebuild_dependencies(node_id);
        
        // Get all nodes that need to be rechecked
        let affected_nodes = self.get_affected_nodes(node_id);
        
        // Incrementally recheck affected nodes
        let mut rechecked = Vec::new();
        for affected_node in affected_nodes {
            match self.check_node(affected_node) {
                Ok(_) => rechecked.push(affected_node),
                Err(e) => return Err(e),
            }
        }
        
        Ok(rechecked)
    }
    
    /// Get all nodes affected by a change to the given node
    fn get_affected_nodes(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut affected = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        queue.push_back(node_id);
        
        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            affected.push(current);
            
            // Add dependents to queue
            if let Some(dependents) = self.db.dependency_graph().get_dependents(current) {
                for &dependent in dependents {
                    if !visited.contains(&dependent) {
                        queue.push_back(dependent);
                    }
                }
            }
        }
        
        affected
    }
    
    /// Rebuild dependency information for a node
    fn rebuild_dependencies(&mut self, node_id: NodeId) {
        // This would analyze the node and update the dependency graph
        // Implementation depends on the specific language semantics
    }
}

/// Implementation of type inference query
fn infer_type(db: &dyn IncrementalTypeCheckDb, node_id: NodeId) -> Result<TypeScheme, TypeError> {
    let node = db.ast_node(node_id);
    
    match &node.kind {
        x_parser::persistent_ast::AstNodeKind::Literal { value } => {
            infer_literal_type(value)
        },
        x_parser::persistent_ast::AstNodeKind::Variable { name } => {
            let scope = db.node_scope(node_id);
            match db.resolve_symbol(*name, scope) {
                Some(symbol_info) => Ok(symbol_info.type_scheme),
                None => Err(TypeError::UnboundVariable { 
                    name: *name, 
                    location: node_id 
                }),
            }
        },
        x_parser::persistent_ast::AstNodeKind::Application { function, arguments } => {
            infer_application_type(db, node_id, function, arguments)
        },
        x_parser::persistent_ast::AstNodeKind::Lambda { parameters, body, effect_annotation } => {
            infer_lambda_type(db, node_id, parameters, body, effect_annotation.as_ref())
        },
        x_parser::persistent_ast::AstNodeKind::Let { bindings, body } => {
            infer_let_type(db, node_id, bindings, body)
        },
        x_parser::persistent_ast::AstNodeKind::If { condition, then_branch, else_branch } => {
            infer_if_type(db, node_id, condition, then_branch, else_branch.as_ref())
        },
        _ => {
            // TODO: Implement other node types
            Ok(TypeScheme::monotype(Type::Con(Symbol::new("Unit"))))
        }
    }
}

/// Helper functions for type inference
fn infer_literal_type(value: &x_parser::persistent_ast::LiteralValue) -> Result<TypeScheme, TypeError> {
    use x_parser::persistent_ast::LiteralValue;
    
    let ty = match value {
        LiteralValue::Unit => Type::Con(Symbol::new("Unit")),
        LiteralValue::Boolean(_) => Type::Con(Symbol::new("Bool")),
        LiteralValue::Integer(_) => Type::Con(Symbol::new("Int")),
        LiteralValue::Float(_) => Type::Con(Symbol::new("Float")),
        LiteralValue::String(_) => Type::Con(Symbol::new("String")),
        LiteralValue::Char(_) => Type::Con(Symbol::new("Char")),
    };
    
    Ok(TypeScheme::monotype(ty))
}

fn infer_application_type(
    db: &dyn IncrementalTypeCheckDb,
    node_id: NodeId,
    function: &PersistentAstNode,
    arguments: &im::Vector<PersistentAstNode>,
) -> Result<TypeScheme, TypeError> {
    // Get function type
    let function_type = db.infer_type(function.id())?;
    
    // Get argument types
    let mut arg_types = Vec::new();
    for arg in arguments {
        arg_types.push(db.infer_type(arg.id())?);
    }
    
    // Unify function type with argument types
    match function_type.body {
        Type::Fun { params: parameters, return_type, effects: _ } => {
            if parameters.len() != arg_types.len() {
                return Err(TypeError::InvalidApplication {
                    function_type: function_type.body,
                    arg_types: arg_types.into_iter().map(|ts| ts.body).collect(),
                    location: node_id,
                });
            }
            
            // TODO: Implement proper unification
            Ok(TypeScheme::monotype(*return_type))
        },
        _ => Err(TypeError::InvalidApplication {
            function_type: function_type.body,
            arg_types: arg_types.into_iter().map(|ts| ts.body).collect(),
            location: node_id,
        }),
    }
}

fn infer_lambda_type(
    _db: &dyn IncrementalTypeCheckDb,
    _node_id: NodeId,
    _parameters: &im::Vector<x_parser::persistent_ast::Parameter>,
    _body: &PersistentAstNode,
    _effect_annotation: Option<&PersistentAstNode>,
) -> Result<TypeScheme, TypeError> {
    // TODO: Implement lambda type inference
    Ok(TypeScheme::monotype(Type::Con(Symbol::new("Unit"))))
}

fn infer_let_type(
    _db: &dyn IncrementalTypeCheckDb,
    _node_id: NodeId,
    _bindings: &im::Vector<x_parser::persistent_ast::Binding>,
    _body: &PersistentAstNode,
) -> Result<TypeScheme, TypeError> {
    // TODO: Implement let type inference
    Ok(TypeScheme::monotype(Type::Con(Symbol::new("Unit"))))
}

fn infer_if_type(
    _db: &dyn IncrementalTypeCheckDb,
    _node_id: NodeId,
    _condition: &PersistentAstNode,
    _then_branch: &PersistentAstNode,
    _else_branch: Option<&PersistentAstNode>,
) -> Result<TypeScheme, TypeError> {
    // TODO: Implement if type inference
    Ok(TypeScheme::monotype(Type::Con(Symbol::new("Unit"))))
}

/// Additional helper structures
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Type variable generator
    next_type_var: u64,
    /// Active type constraints
    constraints: Vec<TypeConstraint>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            next_type_var: 0,
            constraints: Vec::new(),
        }
    }
    
    pub fn fresh_type_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_type_var as u32);
        self.next_type_var += 1;
        var
    }
    
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
}

#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Effect variable generator
    next_effect_var: u64,
    /// Active effect constraints
    constraints: Vec<EffectConstraint>,
}

impl EffectContext {
    pub fn new() -> Self {
        Self {
            next_effect_var: 0,
            constraints: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScopeManager {
    /// Scope hierarchy
    scopes: HashMap<ScopeId, Scope>,
    /// Next scope ID
    next_scope_id: u64,
}

impl ScopeManager {
    pub fn new() -> Self {
        let mut scopes = HashMap::new();
        scopes.insert(ScopeId::root(), Scope::new(None));
        
        Self {
            scopes,
            next_scope_id: 1,
        }
    }
    
    pub fn new_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let scope_id = ScopeId::new(self.next_scope_id);
        self.next_scope_id += 1;
        
        self.scopes.insert(scope_id, Scope::new(parent));
        scope_id
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    /// Parent scope
    parent: Option<ScopeId>,
    /// Symbols defined in this scope
    symbols: HashMap<Symbol, SymbolInfo>,
}

impl Scope {
    pub fn new(parent: Option<ScopeId>) -> Self {
        Self {
            parent,
            symbols: HashMap::new(),
        }
    }
}