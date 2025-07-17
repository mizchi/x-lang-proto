//! Enhanced namespace resolution system
//!
//! This module provides a complete namespace resolution system that can
//! load namespaces on-demand and resolve names across the entire hierarchy.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use x_parser::Symbol;

use crate::namespace::{
    Namespace, NamespacePath, NameBinding, FullyQualifiedName,
    Import, ImportKind, Visibility,
};
use crate::namespace_storage::NamespaceStorage;
use crate::content_addressing::{ContentRepository, ContentHash};

/// Enhanced namespace resolver with lazy loading
pub struct LazyNamespaceResolver {
    /// Storage backend
    storage: Arc<RwLock<NamespaceStorage>>,
    
    /// Loaded namespaces cache
    loaded_namespaces: RwLock<HashMap<NamespacePath, Namespace>>,
    
    /// Resolution cache
    resolution_cache: RwLock<HashMap<(NamespacePath, Symbol), Option<ResolvedName>>>,
    
    /// Current namespace context stack (for nested resolutions)
    context_stack: RwLock<Vec<NamespacePath>>,
}

/// A resolved name with its full context
#[derive(Debug, Clone)]
pub struct ResolvedName {
    pub fully_qualified: FullyQualifiedName,
    pub binding: NameBinding,
    pub source_namespace: NamespacePath,
}

impl LazyNamespaceResolver {
    pub fn new(storage: Arc<RwLock<NamespaceStorage>>) -> Self {
        Self {
            storage,
            loaded_namespaces: RwLock::new(HashMap::new()),
            resolution_cache: RwLock::new(HashMap::new()),
            context_stack: RwLock::new(Vec::new()),
        }
    }
    
    /// Push a namespace context
    pub fn push_context(&self, namespace: NamespacePath) -> Result<()> {
        let mut stack = self.context_stack.write().unwrap();
        stack.push(namespace);
        Ok(())
    }
    
    /// Pop a namespace context
    pub fn pop_context(&self) -> Result<NamespacePath> {
        let mut stack = self.context_stack.write().unwrap();
        stack.pop().ok_or_else(|| anyhow!("Context stack is empty"))
    }
    
    /// Get current context
    pub fn current_context(&self) -> Result<NamespacePath> {
        let stack = self.context_stack.read().unwrap();
        stack.last().cloned()
            .ok_or_else(|| anyhow!("No current context"))
    }
    
    /// Resolve a name in the current context
    pub fn resolve_current(&self, name: Symbol) -> Result<ResolvedName> {
        let context = self.current_context()?;
        self.resolve(&context, name)
    }
    
    /// Resolve a name in a given namespace context
    pub fn resolve(
        &self,
        context: &NamespacePath,
        name: Symbol,
    ) -> Result<ResolvedName> {
        // Check cache first
        {
            let cache = self.resolution_cache.read().unwrap();
            if let Some(cached) = cache.get(&(context.clone(), name)) {
                return cached.clone()
                    .ok_or_else(|| anyhow!("Name '{}' not found in namespace '{}'", 
                        name.as_str(), context.to_string()));
            }
        }
        
        // Try to resolve
        let result = self.resolve_uncached(context, name);
        
        // Cache the result
        {
            let mut cache = self.resolution_cache.write().unwrap();
            cache.insert(
                (context.clone(), name),
                result.as_ref().ok().cloned(),
            );
        }
        
        result
    }
    
    /// Resolve a fully qualified name
    pub fn resolve_qualified(&self, fqn: &FullyQualifiedName) -> Result<ResolvedName> {
        self.resolve(&fqn.namespace, fqn.name)
    }
    
    /// Resolve a path starting from a context
    pub fn resolve_path(
        &self,
        context: &NamespacePath,
        path: &[Symbol],
    ) -> Result<ResolvedName> {
        if path.is_empty() {
            return Err(anyhow!("Empty path"));
        }
        
        if path.len() == 1 {
            return self.resolve(context, path[0]);
        }
        
        // Multi-segment path: resolve step by step
        let mut current_context = context.clone();
        
        for (i, segment) in path.iter().enumerate() {
            if i == path.len() - 1 {
                // Last segment: resolve as a name
                return self.resolve(&current_context, *segment);
            } else {
                // Intermediate segment: must be a namespace
                match self.resolve(&current_context, *segment)? {
                    ResolvedName { binding: NameBinding::Namespace { namespace }, .. } => {
                        current_context = namespace.path.clone();
                    }
                    _ => {
                        return Err(anyhow!(
                            "'{}' is not a namespace in path {:?}",
                            segment.as_str(),
                            path
                        ));
                    }
                }
            }
        }
        
        unreachable!("Should have returned in loop");
    }
    
    fn resolve_uncached(
        &self,
        context: &NamespacePath,
        name: Symbol,
    ) -> Result<ResolvedName> {
        // Load the namespace if not already loaded
        let namespace = self.load_namespace(context)?;
        
        // 1. Check direct bindings
        if let Some(binding) = namespace.bindings.get(&name) {
            return Ok(ResolvedName {
                fully_qualified: FullyQualifiedName::new(context.clone(), name),
                binding: binding.clone(),
                source_namespace: context.clone(),
            });
        }
        
        // 2. Check imports
        for import in &namespace.imports {
            if let Some(resolved) = self.check_import(import, name)? {
                return Ok(resolved);
            }
        }
        
        // 3. Check parent namespaces (for protected/public bindings)
        if let Some(parent_path) = context.parent() {
            if let Ok(resolved) = self.resolve_in_parent(&parent_path, name, context) {
                return Ok(resolved);
            }
        }
        
        // 4. Check child namespaces (direct children are visible)
        let child_ns_path = context.child(name);
        if self.namespace_exists(&child_ns_path)? {
            let child_ns = self.load_namespace(&child_ns_path)?;
            return Ok(ResolvedName {
                fully_qualified: FullyQualifiedName::new(
                    context.clone(),
                    name,
                ),
                binding: NameBinding::Namespace {
                    namespace: Box::new(child_ns),
                },
                source_namespace: context.clone(),
            });
        }
        
        Err(anyhow!("Name '{}' not found in namespace '{}'", 
            name.as_str(), context.to_string()))
    }
    
    fn load_namespace(&self, path: &NamespacePath) -> Result<Namespace> {
        // Check loaded cache first
        {
            let loaded = self.loaded_namespaces.read().unwrap();
            if let Some(ns) = loaded.get(path) {
                return Ok(ns.clone());
            }
        }
        
        // Load from storage
        let namespace = {
            let mut storage = self.storage.write().unwrap();
            storage.load_namespace(path)?
        };
        
        // Cache the loaded namespace
        {
            let mut loaded = self.loaded_namespaces.write().unwrap();
            loaded.insert(path.clone(), namespace.clone());
        }
        
        Ok(namespace)
    }
    
    fn namespace_exists(&self, path: &NamespacePath) -> Result<bool> {
        let storage = self.storage.read().unwrap();
        Ok(storage.list_namespaces().contains(path))
    }
    
    fn check_import(
        &self,
        import: &Import,
        name: Symbol,
    ) -> Result<Option<ResolvedName>> {
        match &import.kind {
            ImportKind::All => {
                // Import all public names
                let source_ns = self.load_namespace(&import.source)?;
                if let Some(binding) = source_ns.bindings.get(&name) {
                    if self.is_importable(binding, &source_ns) {
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.clone(),
                                name,
                            ),
                            binding: binding.clone(),
                            source_namespace: import.source.clone(),
                        }));
                    }
                }
            }
            ImportKind::Selective(names) => {
                // Import specific names
                if names.contains(&name) {
                    let source_ns = self.load_namespace(&import.source)?;
                    if let Some(binding) = source_ns.bindings.get(&name) {
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.clone(),
                                name,
                            ),
                            binding: binding.clone(),
                            source_namespace: import.source.clone(),
                        }));
                    }
                }
            }
            ImportKind::Namespace => {
                // Check if using the namespace alias
                if let Some(alias) = &import.alias {
                    if name == *alias {
                        // Return the namespace itself
                        let source_ns = self.load_namespace(&import.source)?;
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.parent().unwrap_or_else(NamespacePath::root),
                                import.source.segments.last().copied()
                                    .unwrap_or_else(|| Symbol::intern("root")),
                            ),
                            binding: NameBinding::Namespace {
                                namespace: Box::new(source_ns),
                            },
                            source_namespace: import.source.clone(),
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    fn resolve_in_parent(
        &self,
        parent_path: &NamespacePath,
        name: Symbol,
        original_context: &NamespacePath,
    ) -> Result<ResolvedName> {
        let parent_ns = self.load_namespace(parent_path)?;
        
        if let Some(binding) = parent_ns.bindings.get(&name) {
            match binding {
                NameBinding::Value { visibility, .. } |
                NameBinding::Type { visibility, .. } |
                NameBinding::Effect { visibility, .. } => {
                    if matches!(visibility, Visibility::Public | Visibility::Protected) {
                        return Ok(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                parent_path.clone(),
                                name,
                            ),
                            binding: binding.clone(),
                            source_namespace: parent_path.clone(),
                        });
                    }
                }
                NameBinding::Namespace { .. } => {
                    // Namespaces are always visible
                    return Ok(ResolvedName {
                        fully_qualified: FullyQualifiedName::new(
                            parent_path.clone(),
                            name,
                        ),
                        binding: binding.clone(),
                        source_namespace: parent_path.clone(),
                    });
                }
                NameBinding::Alias { .. } => {
                    // Aliases are always visible
                    return Ok(ResolvedName {
                        fully_qualified: FullyQualifiedName::new(
                            parent_path.clone(),
                            name,
                        ),
                        binding: binding.clone(),
                        source_namespace: parent_path.clone(),
                    });
                }
            }
        }
        
        // Continue searching in grandparent
        if let Some(grandparent) = parent_path.parent() {
            self.resolve_in_parent(&grandparent, name, original_context)
        } else {
            Err(anyhow!("Name '{}' not found in parent namespaces", name.as_str()))
        }
    }
    
    fn is_importable(&self, binding: &NameBinding, source_ns: &Namespace) -> bool {
        // Check if the binding is exported by the source namespace
        if let Some(ref exports) = source_ns.exports {
            if exports.export_all {
                return match binding {
                    NameBinding::Value { visibility, .. } |
                    NameBinding::Type { visibility, .. } |
                    NameBinding::Effect { visibility, .. } => {
                        *visibility == Visibility::Public
                    }
                    _ => true,
                };
            } else {
                // Check explicit exports
                // This would need the name, which we don't have here
                // In a real implementation, we'd pass the name as well
                return true; // Simplified for now
            }
        }
        
        // If no exports specified, check visibility
        match binding {
            NameBinding::Value { visibility, .. } |
            NameBinding::Type { visibility, .. } |
            NameBinding::Effect { visibility, .. } => {
                *visibility == Visibility::Public
            }
            _ => true,
        }
    }
    
    /// Clear all caches
    pub fn clear_caches(&self) {
        self.loaded_namespaces.write().unwrap().clear();
        self.resolution_cache.write().unwrap().clear();
    }
    
    /// Get all visible names in a namespace
    pub fn list_visible_names(&self, context: &NamespacePath) -> Result<Vec<VisibleName>> {
        let mut visible = Vec::new();
        let namespace = self.load_namespace(context)?;
        
        // Direct bindings
        for (name, binding) in &namespace.bindings {
            visible.push(VisibleName {
                name: *name,
                fully_qualified: FullyQualifiedName::new(context.clone(), *name),
                kind: NameKind::from_binding(binding),
                source: NameSource::Direct,
            });
        }
        
        // Imported names
        for import in &namespace.imports {
            match &import.kind {
                ImportKind::All => {
                    if let Ok(source_ns) = self.load_namespace(&import.source) {
                        for (name, binding) in &source_ns.bindings {
                            if self.is_importable(binding, &source_ns) {
                                visible.push(VisibleName {
                                    name: *name,
                                    fully_qualified: FullyQualifiedName::new(
                                        import.source.clone(),
                                        *name,
                                    ),
                                    kind: NameKind::from_binding(binding),
                                    source: NameSource::Imported(import.source.clone()),
                                });
                            }
                        }
                    }
                }
                ImportKind::Selective(names) => {
                    for name in names {
                        visible.push(VisibleName {
                            name: *name,
                            fully_qualified: FullyQualifiedName::new(
                                import.source.clone(),
                                *name,
                            ),
                            kind: NameKind::Unknown, // Would need to load to know
                            source: NameSource::Imported(import.source.clone()),
                        });
                    }
                }
                ImportKind::Namespace => {
                    if let Some(alias) = &import.alias {
                        visible.push(VisibleName {
                            name: *alias,
                            fully_qualified: FullyQualifiedName::new(
                                import.source.parent().unwrap_or_else(NamespacePath::root),
                                import.source.segments.last().copied()
                                    .unwrap_or_else(|| Symbol::intern("root")),
                            ),
                            kind: NameKind::Namespace,
                            source: NameSource::Imported(import.source.clone()),
                        });
                    }
                }
            }
        }
        
        // Child namespaces
        let all_namespaces = {
            let storage = self.storage.read().unwrap();
            storage.list_namespaces()
        };
        
        for ns_path in all_namespaces {
            if let Some(parent) = ns_path.parent() {
                if parent == *context {
                    if let Some(child_name) = ns_path.segments.last() {
                        visible.push(VisibleName {
                            name: *child_name,
                            fully_qualified: FullyQualifiedName::new(
                                context.clone(),
                                *child_name,
                            ),
                            kind: NameKind::Namespace,
                            source: NameSource::Child,
                        });
                    }
                }
            }
        }
        
        Ok(visible)
    }
}

/// Information about a visible name
#[derive(Debug, Clone)]
pub struct VisibleName {
    pub name: Symbol,
    pub fully_qualified: FullyQualifiedName,
    pub kind: NameKind,
    pub source: NameSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameKind {
    Value,
    Type,
    Effect,
    Namespace,
    Alias,
    Unknown,
}

impl NameKind {
    fn from_binding(binding: &NameBinding) -> Self {
        match binding {
            NameBinding::Value { .. } => NameKind::Value,
            NameBinding::Type { .. } => NameKind::Type,
            NameBinding::Effect { .. } => NameKind::Effect,
            NameBinding::Namespace { .. } => NameKind::Namespace,
            NameBinding::Alias { .. } => NameKind::Alias,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NameSource {
    Direct,
    Imported(NamespacePath),
    Parent,
    Child,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_lazy_resolver() {
        let temp_dir = TempDir::new().unwrap();
        let content_repo = ContentRepository::new();
        let storage = NamespaceStorage::new(
            temp_dir.path().to_path_buf(),
            content_repo,
        ).unwrap();
        
        let storage = Arc::new(RwLock::new(storage));
        let resolver = LazyNamespaceResolver::new(storage.clone());
        
        // Create test namespace
        let mut ns = Namespace::new(NamespacePath::from_str("Test"));
        ns.add_value(
            Symbol::intern("test_value"),
            ContentHash::new(b"test"),
            None,
            Visibility::Public,
        );
        
        storage.write().unwrap().save_namespace(&ns).unwrap();
        
        // Test resolution
        resolver.push_context(NamespacePath::from_str("Test")).unwrap();
        let resolved = resolver.resolve_current(Symbol::intern("test_value")).unwrap();
        
        assert_eq!(resolved.fully_qualified.name, Symbol::intern("test_value"));
    }
}