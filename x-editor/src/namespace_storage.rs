//! Namespace storage and content addressing integration
//!
//! This module provides persistent storage for namespaces and integrates
//! with the content addressing system.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::namespace::{
    Namespace, NamespacePath, NameBinding,
    NamespaceResolver,
};
use crate::content_addressing::{ContentRepository, ContentHash};

/// Namespace storage backend
pub struct NamespaceStorage {
    /// Root directory for namespace storage
    root_dir: PathBuf,
    
    /// Content repository
    content_repo: ContentRepository,
    
    /// Cached namespaces
    cache: HashMap<NamespacePath, Namespace>,
    
    /// Index of all known namespaces
    namespace_index: NamespaceIndex,
}

/// Index of all namespaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceIndex {
    /// All namespace paths (stored as strings for serialization)
    pub namespaces: HashSet<String>,
    
    /// Namespace dependencies (keys are namespace path strings)
    pub dependencies: HashMap<String, HashSet<String>>,
    
    /// Reverse dependencies (who depends on this namespace)
    pub reverse_dependencies: HashMap<String, HashSet<String>>,
    
    /// Namespace versions (keys are namespace path strings)
    pub versions: HashMap<String, Vec<NamespaceVersion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceVersion {
    pub version: String,
    pub hash: ContentHash,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl NamespaceStorage {
    pub fn new(root_dir: PathBuf, content_repo: ContentRepository) -> Result<Self> {
        // Create root directory if it doesn't exist
        fs::create_dir_all(&root_dir)?;
        
        // Load or create index
        let index_path = root_dir.join("namespace_index.json");
        let namespace_index = if index_path.exists() {
            let data = fs::read_to_string(&index_path)?;
            serde_json::from_str(&data)?
        } else {
            NamespaceIndex {
                namespaces: HashSet::new(),
                dependencies: HashMap::new(),
                reverse_dependencies: HashMap::new(),
                versions: HashMap::new(),
            }
        };
        
        Ok(Self {
            root_dir,
            content_repo,
            cache: HashMap::new(),
            namespace_index,
        })
    }
    
    /// Save a namespace
    pub fn save_namespace(&mut self, namespace: &Namespace) -> Result<ContentHash> {
        // Serialize namespace
        let data = serde_json::to_vec_pretty(namespace)?;
        let hash = ContentHash::new(&data);
        
        // Save to file system
        let path = self.namespace_path(&namespace.path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, &data)?;
        
        // Update index
        self.namespace_index.namespaces.insert(namespace.path.to_string());
        
        // Extract and update dependencies
        let deps = self.extract_dependencies(namespace);
        let deps_strings: HashSet<String> = deps.iter().map(|p| p.to_string()).collect();
        self.namespace_index.dependencies.insert(namespace.path.to_string(), deps_strings.clone());
        
        // Update reverse dependencies
        for dep in &deps_strings {
            self.namespace_index.reverse_dependencies
                .entry(dep.clone())
                .or_default()
                .insert(namespace.path.to_string());
        }
        
        // Add version
        let version = namespace.metadata.version.clone()
            .unwrap_or_else(|| format!("0.0.0+{}", hash.short()));
        
        self.namespace_index.versions
            .entry(namespace.path.to_string())
            .or_default()
            .push(NamespaceVersion {
                version,
                hash: hash.clone(),
                created_at: chrono::Utc::now(),
            });
        
        // Save index
        self.save_index()?;
        
        // Update cache
        self.cache.insert(namespace.path.clone(), namespace.clone());
        
        Ok(hash)
    }
    
    /// Load a namespace
    pub fn load_namespace(&mut self, path: &NamespacePath) -> Result<Namespace> {
        // Check cache first
        if let Some(ns) = self.cache.get(path) {
            return Ok(ns.clone());
        }
        
        // Load from file
        let file_path = self.namespace_path(path);
        if !file_path.exists() {
            return Err(anyhow!("Namespace '{}' not found", path.to_string()));
        }
        
        let data = fs::read_to_string(&file_path)?;
        let namespace: Namespace = serde_json::from_str(&data)?;
        
        // Update cache
        self.cache.insert(path.clone(), namespace.clone());
        
        Ok(namespace)
    }
    
    /// Delete a namespace
    pub fn delete_namespace(&mut self, path: &NamespacePath) -> Result<()> {
        // Check for dependencies
        if let Some(deps) = self.namespace_index.reverse_dependencies.get(&path.to_string()) {
            if !deps.is_empty() {
                return Err(anyhow!(
                    "Cannot delete namespace '{}': depended on by {:?}",
                    path.to_string(),
                    deps
                ));
            }
        }
        
        // Remove from file system
        let file_path = self.namespace_path(path);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        
        // Update index
        let path_str = path.to_string();
        self.namespace_index.namespaces.remove(&path_str);
        self.namespace_index.dependencies.remove(&path_str);
        self.namespace_index.versions.remove(&path_str);
        
        // Remove from reverse dependencies
        for (_, deps) in self.namespace_index.reverse_dependencies.iter_mut() {
            deps.remove(&path_str);
        }
        
        // Save index
        self.save_index()?;
        
        // Remove from cache
        self.cache.remove(path);
        
        Ok(())
    }
    
    /// List all namespaces
    pub fn list_namespaces(&self) -> Vec<NamespacePath> {
        self.namespace_index.namespaces.iter()
            .map(|s| NamespacePath::from_str(s))
            .collect()
    }
    
    /// Get namespace versions
    pub fn get_versions(&self, path: &NamespacePath) -> Vec<NamespaceVersion> {
        self.namespace_index.versions
            .get(&path.to_string())
            .cloned()
            .unwrap_or_default()
    }
    
    /// Load a specific version of a namespace
    pub fn load_namespace_version(
        &mut self,
        path: &NamespacePath,
        version: &str,
    ) -> Result<Namespace> {
        let versions = self.get_versions(path);
        let version_info = versions.iter()
            .find(|v| v.version == version)
            .ok_or_else(|| anyhow!("Version '{}' not found for namespace '{}'", 
                version, path.to_string()))?;
        
        // Load from versioned file
        let file_path = self.namespace_version_path(path, &version_info.hash);
        if !file_path.exists() {
            return Err(anyhow!("Version file not found"));
        }
        
        let data = fs::read_to_string(&file_path)?;
        let namespace: Namespace = serde_json::from_str(&data)?;
        
        Ok(namespace)
    }
    
    /// Create a namespace resolver
    pub fn create_resolver(&mut self) -> Result<NamespaceResolver> {
        // Load root namespace
        let root = self.load_namespace(&NamespacePath::root())
            .unwrap_or_else(|_| Namespace::new(NamespacePath::root()));
        
        Ok(NamespaceResolver::new(root, self.content_repo.clone()))
    }
    
    /// Import a namespace from another storage
    pub fn import_namespace(
        &mut self,
        namespace: &Namespace,
        overwrite: bool,
    ) -> Result<()> {
        if !overwrite && self.namespace_index.namespaces.contains(&namespace.path.to_string()) {
            return Err(anyhow!("Namespace '{}' already exists", namespace.path.to_string()));
        }
        
        self.save_namespace(namespace)?;
        Ok(())
    }
    
    /// Export a namespace
    pub fn export_namespace(&mut self, path: &NamespacePath) -> Result<NamespaceExport> {
        let namespace = self.load_namespace(path)?;
        
        // Collect all content hashes referenced by this namespace
        let mut content_hashes = HashSet::new();
        self.collect_content_hashes(&namespace, &mut content_hashes);
        
        // Export content items
        let mut content_items = HashMap::new();
        for hash in &content_hashes {
            if let Some(entry) = self.content_repo.get(hash) {
                content_items.insert(hash.clone(), entry.clone());
            }
        }
        
        Ok(NamespaceExport {
            namespace,
            content_items,
            export_time: chrono::Utc::now(),
        })
    }
    
    /// Path to namespace file
    fn namespace_path(&self, path: &NamespacePath) -> PathBuf {
        if path.is_root() {
            self.root_dir.join("root.namespace.json")
        } else {
            let mut file_path = self.root_dir.clone();
            for segment in &path.segments {
                file_path = file_path.join(segment.as_str());
            }
            file_path.with_extension("namespace.json")
        }
    }
    
    /// Path to versioned namespace file
    fn namespace_version_path(&self, path: &NamespacePath, hash: &ContentHash) -> PathBuf {
        let base = self.namespace_path(path);
        base.with_extension(format!("{}.json", hash.short()))
    }
    
    /// Save namespace index
    fn save_index(&self) -> Result<()> {
        let index_path = self.root_dir.join("namespace_index.json");
        let data = serde_json::to_vec_pretty(&self.namespace_index)?;
        fs::write(index_path, data)?;
        Ok(())
    }
    
    /// Extract dependencies from namespace
    fn extract_dependencies(&self, namespace: &Namespace) -> HashSet<NamespacePath> {
        let mut deps = HashSet::new();
        
        // From imports
        for import in &namespace.imports {
            deps.insert(import.source.clone());
        }
        
        // From metadata
        for dep in &namespace.metadata.dependencies {
            deps.insert(dep.namespace.clone());
        }
        
        deps
    }
    
    /// Collect all content hashes referenced by a namespace
    fn collect_content_hashes(&self, namespace: &Namespace, hashes: &mut HashSet<ContentHash>) {
        for binding in namespace.bindings.values() {
            match binding {
                NameBinding::Value { hash, .. } |
                NameBinding::Type { hash, .. } |
                NameBinding::Effect { hash, .. } => {
                    hashes.insert(hash.clone());
                }
                NameBinding::Namespace { namespace } => {
                    self.collect_content_hashes(namespace, hashes);
                }
                NameBinding::Alias { .. } => {}
            }
        }
    }
}

/// Namespace export bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceExport {
    pub namespace: Namespace,
    pub content_items: HashMap<ContentHash, crate::content_addressing::ContentEntry>,
    pub export_time: chrono::DateTime<chrono::Utc>,
}

/// Namespace synchronization
pub struct NamespaceSync {
    local: NamespaceStorage,
    remote: NamespaceStorage,
}

impl NamespaceSync {
    pub fn new(local: NamespaceStorage, remote: NamespaceStorage) -> Self {
        Self { local, remote }
    }
    
    /// Sync namespaces from remote to local
    pub fn pull(&mut self, namespace_path: &NamespacePath) -> Result<()> {
        let remote_ns = self.remote.load_namespace(namespace_path)?;
        self.local.import_namespace(&remote_ns, true)?;
        Ok(())
    }
    
    /// Sync namespaces from local to remote
    pub fn push(&mut self, namespace_path: &NamespacePath) -> Result<()> {
        let local_ns = self.local.load_namespace(namespace_path)?;
        self.remote.import_namespace(&local_ns, true)?;
        Ok(())
    }
    
    /// Two-way sync
    pub fn sync(&mut self, namespace_path: &NamespacePath) -> Result<()> {
        // Simple implementation: last-write-wins based on modification time
        let local_ns = self.local.load_namespace(namespace_path)?;
        let remote_ns = self.remote.load_namespace(namespace_path)?;
        
        if local_ns.metadata.modified_at > remote_ns.metadata.modified_at {
            self.push(namespace_path)?;
        } else if remote_ns.metadata.modified_at > local_ns.metadata.modified_at {
            self.pull(namespace_path)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_namespace_storage() {
        let temp_dir = TempDir::new().unwrap();
        let content_repo = ContentRepository::new();
        let mut storage = NamespaceStorage::new(
            temp_dir.path().to_path_buf(),
            content_repo,
        ).unwrap();
        
        // Create and save namespace
        let ns_path = NamespacePath::from_str("Test.Module");
        let mut namespace = Namespace::new(ns_path.clone());
        namespace.add_value(
            Symbol::intern("test_fn"),
            ContentHash::new(b"test"),
            None,
            Visibility::Public,
        );
        
        let hash = storage.save_namespace(&namespace).unwrap();
        assert!(!hash.0.is_empty());
        
        // Load namespace
        let loaded = storage.load_namespace(&ns_path).unwrap();
        assert_eq!(loaded.path, ns_path);
        assert!(loaded.bindings.contains_key(&Symbol::intern("test_fn")));
    }
}