//! Unison-style namespace system for x Language
//!
//! This module implements a hierarchical namespace system inspired by Unison,
//! where names are organized in a tree structure and can be imported/exported.

use std::collections::{HashMap, HashSet};
use x_parser::Symbol;
use x_checker::types::TypeScheme;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use crate::content_addressing::{ContentHash, ContentRepository};

/// A path in the namespace hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespacePath {
    /// Segments of the path (e.g., ["Data", "List"] for Data.List)
    pub segments: Vec<Symbol>,
}

impl NamespacePath {
    pub fn new(segments: Vec<Symbol>) -> Self {
        Self { segments }
    }
    
    pub fn root() -> Self {
        Self { segments: vec![] }
    }
    
    pub fn from_str(path: &str) -> Self {
        if path.is_empty() {
            Self::root()
        } else {
            let segments = path.split('.')
                .map(|s| Symbol::intern(s))
                .collect();
            Self { segments }
        }
    }
    
    pub fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
    
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }
    
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            None
        } else {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }
    
    pub fn child(&self, name: Symbol) -> Self {
        let mut segments = self.segments.clone();
        segments.push(name);
        Self { segments }
    }
    
    pub fn relative_to(&self, base: &NamespacePath) -> Option<Self> {
        if self.segments.starts_with(&base.segments) {
            Some(Self {
                segments: self.segments[base.segments.len()..].to_vec(),
            })
        } else {
            None
        }
    }
    
    pub fn join(&self, other: &NamespacePath) -> Self {
        let mut segments = self.segments.clone();
        segments.extend(other.segments.clone());
        Self { segments }
    }
}

/// A name binding in the namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NameBinding {
    /// Value/function binding
    Value {
        hash: ContentHash,
        type_scheme: Option<TypeScheme>,
        visibility: Visibility,
    },
    /// Type binding
    Type {
        hash: ContentHash,
        kind: TypeKind,
        visibility: Visibility,
    },
    /// Effect binding
    Effect {
        hash: ContentHash,
        visibility: Visibility,
    },
    /// Subnamespace
    Namespace {
        namespace: Box<Namespace>,
    },
    /// Alias to another name
    Alias {
        target: FullyQualifiedName,
    },
}

/// Visibility of a name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Public - can be imported by other namespaces
    Public,
    /// Private - only visible within this namespace
    Private,
    /// Protected - visible to child namespaces
    Protected,
}

/// Type kind for type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    /// Data type with constructors
    Data { constructors: Vec<Symbol> },
    /// Type alias
    Alias,
    /// Abstract type
    Abstract,
}

/// Fully qualified name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullyQualifiedName {
    pub namespace: NamespacePath,
    pub name: Symbol,
}

impl FullyQualifiedName {
    pub fn new(namespace: NamespacePath, name: Symbol) -> Self {
        Self { namespace, name }
    }
    
    pub fn to_string(&self) -> String {
        if self.namespace.is_root() {
            self.name.as_str().to_string()
        } else {
            format!("{}.{}", self.namespace.to_string(), self.name.as_str())
        }
    }
}

/// A namespace containing bindings and sub-namespaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    /// Path of this namespace
    pub path: NamespacePath,
    
    /// Direct bindings in this namespace
    pub bindings: HashMap<Symbol, NameBinding>,
    
    /// Import statements
    pub imports: Vec<Import>,
    
    /// Export specifications
    pub exports: Option<ExportSpec>,
    
    /// Metadata
    pub metadata: NamespaceMetadata,
}

/// Import specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    /// Source namespace path
    pub source: NamespacePath,
    
    /// Import kind
    pub kind: ImportKind,
    
    /// Optional alias for the import
    pub alias: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportKind {
    /// Import all public names from namespace
    All,
    /// Import specific names
    Selective(Vec<Symbol>),
    /// Import namespace itself (for qualified access)
    Namespace,
}

/// Export specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSpec {
    /// Export all public bindings
    pub export_all: bool,
    
    /// Specific exports (if not exporting all)
    pub exports: HashSet<Symbol>,
    
    /// Re-exports from imported namespaces
    pub reexports: Vec<ReExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReExport {
    pub from: NamespacePath,
    pub names: Vec<Symbol>,
}

/// Namespace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceMetadata {
    /// Version of this namespace
    pub version: Option<String>,
    
    /// Author information
    pub author: Option<String>,
    
    /// Description
    pub description: Option<String>,
    
    /// Dependencies on other namespaces
    pub dependencies: Vec<NamespaceDependency>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDependency {
    pub namespace: NamespacePath,
    pub version: Option<String>,
}

impl Namespace {
    pub fn new(path: NamespacePath) -> Self {
        Self {
            path,
            bindings: HashMap::new(),
            imports: Vec::new(),
            exports: None,
            metadata: NamespaceMetadata {
                version: None,
                author: None,
                description: None,
                dependencies: Vec::new(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
            },
        }
    }
    
    /// Add a value binding
    pub fn add_value(
        &mut self,
        name: Symbol,
        hash: ContentHash,
        type_scheme: Option<TypeScheme>,
        visibility: Visibility,
    ) {
        self.bindings.insert(name, NameBinding::Value {
            hash,
            type_scheme,
            visibility,
        });
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Add a type binding
    pub fn add_type(
        &mut self,
        name: Symbol,
        hash: ContentHash,
        kind: TypeKind,
        visibility: Visibility,
    ) {
        self.bindings.insert(name, NameBinding::Type {
            hash,
            kind,
            visibility,
        });
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Add an effect binding
    pub fn add_effect(
        &mut self,
        name: Symbol,
        hash: ContentHash,
        visibility: Visibility,
    ) {
        self.bindings.insert(name, NameBinding::Effect {
            hash,
            visibility,
        });
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Add a sub-namespace
    pub fn add_namespace(&mut self, name: Symbol, namespace: Namespace) {
        self.bindings.insert(name, NameBinding::Namespace {
            namespace: Box::new(namespace),
        });
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Add an alias
    pub fn add_alias(&mut self, name: Symbol, target: FullyQualifiedName) {
        self.bindings.insert(name, NameBinding::Alias { target });
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Add an import
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Set export specification
    pub fn set_exports(&mut self, exports: ExportSpec) {
        self.exports = Some(exports);
        self.metadata.modified_at = chrono::Utc::now();
    }
    
    /// Get all exported names
    pub fn exported_names(&self) -> HashSet<Symbol> {
        let mut names = HashSet::new();
        
        if let Some(ref export_spec) = self.exports {
            if export_spec.export_all {
                // Export all public bindings
                for (name, binding) in &self.bindings {
                    if self.is_public(binding) {
                        names.insert(*name);
                    }
                }
            } else {
                // Export only specified names
                names.extend(&export_spec.exports);
            }
        }
        
        names
    }
    
    /// Check if a binding is public
    fn is_public(&self, binding: &NameBinding) -> bool {
        match binding {
            NameBinding::Value { visibility, .. } |
            NameBinding::Type { visibility, .. } |
            NameBinding::Effect { visibility, .. } => {
                *visibility == Visibility::Public
            }
            NameBinding::Namespace { .. } => true, // Namespaces are always accessible
            NameBinding::Alias { .. } => true, // Aliases follow target visibility
        }
    }
}

/// Namespace resolution context
pub struct NamespaceResolver {
    /// Root namespace
    root: Namespace,
    
    /// Content repository for looking up definitions
    content_repo: ContentRepository,
    
    /// Cache of resolved names
    resolution_cache: HashMap<(NamespacePath, Symbol), Option<ResolvedName>>,
}

/// A resolved name with its full context
#[derive(Debug, Clone)]
pub struct ResolvedName {
    pub fully_qualified: FullyQualifiedName,
    pub binding: NameBinding,
}

impl NamespaceResolver {
    pub fn new(root: Namespace, content_repo: ContentRepository) -> Self {
        Self {
            root,
            content_repo,
            resolution_cache: HashMap::new(),
        }
    }
    
    /// Resolve a name in a given namespace context
    pub fn resolve(
        &mut self,
        context: &NamespacePath,
        name: Symbol,
    ) -> Result<ResolvedName> {
        // Check cache
        if let Some(cached) = self.resolution_cache.get(&(context.clone(), name)) {
            return cached.clone()
                .ok_or_else(|| anyhow!("Name '{}' not found in namespace '{}'", 
                    name.as_str(), context.to_string()));
        }
        
        // Try to resolve
        let result = self.resolve_uncached(context, name);
        
        // Cache the result
        self.resolution_cache.insert(
            (context.clone(), name),
            result.as_ref().ok().cloned(),
        );
        
        result
    }
    
    fn resolve_uncached(
        &self,
        context: &NamespacePath,
        name: Symbol,
    ) -> Result<ResolvedName> {
        // Navigate to the context namespace
        let namespace = self.navigate_to_namespace(context)?;
        
        // 1. Check direct bindings
        if let Some(binding) = namespace.bindings.get(&name) {
            return Ok(ResolvedName {
                fully_qualified: FullyQualifiedName::new(context.clone(), name),
                binding: binding.clone(),
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
            if let Ok(resolved) = self.resolve_in_parent(&parent_path, name) {
                return Ok(resolved);
            }
        }
        
        Err(anyhow!("Name '{}' not found in namespace '{}'", 
            name.as_str(), context.to_string()))
    }
    
    fn navigate_to_namespace(&self, path: &NamespacePath) -> Result<&Namespace> {
        let mut current = &self.root;
        
        for segment in &path.segments {
            match current.bindings.get(segment) {
                Some(NameBinding::Namespace { namespace }) => {
                    current = namespace;
                }
                _ => return Err(anyhow!("Namespace '{}' not found", path.to_string())),
            }
        }
        
        Ok(current)
    }
    
    fn check_import(
        &self,
        import: &Import,
        name: Symbol,
    ) -> Result<Option<ResolvedName>> {
        match &import.kind {
            ImportKind::All => {
                // Import all public names
                let source_ns = self.navigate_to_namespace(&import.source)?;
                if let Some(binding) = source_ns.bindings.get(&name) {
                    if self.is_importable(binding, &source_ns) {
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.clone(),
                                name,
                            ),
                            binding: binding.clone(),
                        }));
                    }
                }
            }
            ImportKind::Selective(names) => {
                // Import specific names
                if names.contains(&name) {
                    let source_ns = self.navigate_to_namespace(&import.source)?;
                    if let Some(binding) = source_ns.bindings.get(&name) {
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.clone(),
                                name,
                            ),
                            binding: binding.clone(),
                        }));
                    }
                }
            }
            ImportKind::Namespace => {
                // Check if using qualified name
                if let Some(alias) = &import.alias {
                    if name == *alias {
                        // Return the namespace itself
                        let source_ns = self.navigate_to_namespace(&import.source)?;
                        return Ok(Some(ResolvedName {
                            fully_qualified: FullyQualifiedName::new(
                                import.source.parent().unwrap_or_else(NamespacePath::root),
                                import.source.segments.last().copied()
                                    .unwrap_or_else(|| Symbol::intern("root")),
                            ),
                            binding: NameBinding::Namespace {
                                namespace: Box::new(source_ns.clone()),
                            },
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
    ) -> Result<ResolvedName> {
        let parent_ns = self.navigate_to_namespace(parent_path)?;
        
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
                        });
                    }
                }
                _ => {}
            }
        }
        
        // Continue searching in grandparent
        if let Some(grandparent) = parent_path.parent() {
            self.resolve_in_parent(&grandparent, name)
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
}

/// Builder for creating namespaces
pub struct NamespaceBuilder {
    namespace: Namespace,
}

impl NamespaceBuilder {
    pub fn new(path: NamespacePath) -> Self {
        Self {
            namespace: Namespace::new(path),
        }
    }
    
    pub fn with_metadata(mut self, metadata: NamespaceMetadata) -> Self {
        self.namespace.metadata = metadata;
        self
    }
    
    pub fn add_value(
        mut self,
        name: Symbol,
        hash: ContentHash,
        type_scheme: Option<TypeScheme>,
        visibility: Visibility,
    ) -> Self {
        self.namespace.add_value(name, hash, type_scheme, visibility);
        self
    }
    
    pub fn add_type(
        mut self,
        name: Symbol,
        hash: ContentHash,
        kind: TypeKind,
        visibility: Visibility,
    ) -> Self {
        self.namespace.add_type(name, hash, kind, visibility);
        self
    }
    
    pub fn add_import(mut self, import: Import) -> Self {
        self.namespace.add_import(import);
        self
    }
    
    pub fn with_exports(mut self, exports: ExportSpec) -> Self {
        self.namespace.set_exports(exports);
        self
    }
    
    pub fn build(self) -> Namespace {
        self.namespace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_namespace_path() {
        let path = NamespacePath::from_str("Data.List");
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.to_string(), "Data.List");
        
        let child = path.child(Symbol::intern("map"));
        assert_eq!(child.to_string(), "Data.List.map");
        
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "Data");
    }
    
    #[test]
    fn test_fully_qualified_name() {
        let fqn = FullyQualifiedName::new(
            NamespacePath::from_str("Data.List"),
            Symbol::intern("map"),
        );
        assert_eq!(fqn.to_string(), "Data.List.map");
    }
}