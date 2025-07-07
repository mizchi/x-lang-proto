use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};

/// Git-like namespace management for x-lang
pub struct NamespaceManager {
    root: Namespace,
    current_path: Vec<String>,
    version_store: VersionStore,
}

#[derive(Debug, Clone)]
pub struct Namespace {
    path: Vec<String>,
    entries: HashMap<String, Entry>,
    parent: Option<Box<Namespace>>,
}

#[derive(Debug, Clone)]
pub enum Entry {
    Function {
        name: String,
        versions: Vec<Version>,
        current: Hash,
    },
    Module {
        namespace: Namespace,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash(pub String);

#[derive(Debug, Clone)]
pub struct Version {
    hash: Hash,
    timestamp: DateTime<Utc>,
    author: String,
    message: String,
    content: String,
    dependencies: HashSet<Hash>,
}

pub struct VersionStore {
    versions: HashMap<Hash, Version>,
}

impl NamespaceManager {
    pub fn new() -> Self {
        Self {
            root: Namespace::new(vec![]),
            current_path: vec![],
            version_store: VersionStore::new(),
        }
    }

    /// Get current working directory
    pub fn pwd(&self) -> String {
        if self.current_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", self.current_path.join("/"))
        }
    }

    /// Change directory
    pub fn cd(&mut self, path: &str) -> Result<(), String> {
        let new_path = self.resolve_path(path)?;
        
        // Verify the path exists
        if self.get_namespace(&new_path).is_some() {
            self.current_path = new_path;
            Ok(())
        } else {
            Err(format!("No such namespace: {}", path))
        }
    }

    /// List entries in current namespace
    pub fn ls(&self) -> Vec<String> {
        if let Some(ns) = self.get_current_namespace() {
            ns.entries.iter().map(|(name, entry)| {
                match entry {
                    Entry::Function { current, .. } => {
                        format!("{}#{}", name, &current.0[..8])
                    }
                    Entry::Module { .. } => {
                        format!("{}/", name)
                    }
                }
            }).collect()
        } else {
            vec![]
        }
    }

    /// Show content of a function
    pub fn cat(&self, name: &str) -> Result<String, String> {
        let ns = self.get_current_namespace()
            .ok_or("Invalid namespace")?;
        
        match ns.entries.get(name) {
            Some(Entry::Function { current, .. }) => {
                self.version_store.get_content(current)
                    .ok_or("Version not found".to_string())
            }
            Some(Entry::Module { .. }) => {
                Err(format!("{} is a namespace", name))
            }
            None => {
                Err(format!("No such function: {}", name))
            }
        }
    }

    /// Edit a function (returns new hash if changed)
    pub fn edit(&mut self, name: &str, new_content: String) -> Result<Option<Hash>, String> {
        // Check if entry exists and get old content first
        let (entry_exists, entry_type, old_content) = {
            let ns = self.get_current_namespace()
                .ok_or("Invalid namespace")?;
            
            match ns.entries.get(name) {
                Some(Entry::Function { current, .. }) => {
                    let content = self.version_store.get_content(current)
                        .ok_or("Version not found")?;
                    (true, "function", Some(content))
                }
                Some(Entry::Module { .. }) => {
                    (true, "module", None)
                }
                None => {
                    (false, "", None)
                }
            }
        };
        
        if entry_type == "module" {
            return Err(format!("{} is a namespace", name));
        }
        
        if entry_exists {
            let old_content = old_content.unwrap();
            if old_content != new_content {
                // Create new version
                let new_hash = Hash::compute(&new_content);
                let new_version = Version {
                    hash: new_hash.clone(),
                    timestamp: Utc::now(),
                    author: get_current_user(),
                    message: format!("Edit {}", name),
                    content: new_content.clone(),
                    dependencies: analyze_dependencies(&new_content),
                };
                
                // Store version
                self.version_store.add_version(new_version.clone());
                
                // Update function
                let ns = self.get_current_namespace_mut()
                    .ok_or("Invalid namespace")?;
                
                if let Some(Entry::Function { versions, current, .. }) = ns.entries.get_mut(name) {
                    versions.push(new_version);
                    *current = new_hash.clone();
                }
                
                Ok(Some(new_hash))
            } else {
                Ok(None)
            }
        } else {
            // Create new function
            let hash = Hash::compute(&new_content);
            let version = Version {
                hash: hash.clone(),
                timestamp: Utc::now(),
                author: get_current_user(),
                message: format!("Create {}", name),
                content: new_content.clone(),
                dependencies: analyze_dependencies(&new_content),
            };
            
            self.version_store.add_version(version.clone());
            
            let ns = self.get_current_namespace_mut()
                .ok_or("Invalid namespace")?;
                
            ns.entries.insert(name.to_string(), Entry::Function {
                name: name.to_string(),
                versions: vec![version],
                current: hash.clone(),
            });
            
            Ok(Some(hash))
        }
    }

    /// Show specific version of a function
    pub fn show(&self, name: &str, hash: &str) -> Result<String, String> {
        let hash = Hash(hash.to_string());
        self.version_store.get_content(&hash)
            .ok_or(format!("No such version: {}#{}", name, hash.0))
    }

    /// Show version history of a function
    pub fn log(&self, name: &str) -> Result<Vec<String>, String> {
        let ns = self.get_current_namespace()
            .ok_or("Invalid namespace")?;
        
        match ns.entries.get(name) {
            Some(Entry::Function { versions, .. }) => {
                Ok(versions.iter().rev().map(|v| {
                    format!("{} - {} - {} - {}",
                        &v.hash.0[..8],
                        v.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        v.author,
                        v.message
                    )
                }).collect())
            }
            Some(Entry::Module { .. }) => {
                Err(format!("{} is a namespace", name))
            }
            None => {
                Err(format!("No such function: {}", name))
            }
        }
    }

    /// Export namespace to filesystem
    pub fn export(&self, target_dir: &Path) -> Result<(), String> {
        let ns = self.get_current_namespace()
            .ok_or("Invalid namespace")?;
        
        std::fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        
        for (name, entry) in &ns.entries {
            match entry {
                Entry::Function { current, .. } => {
                    let content = self.version_store.get_content(current)
                        .ok_or("Version not found")?;
                    let file_path = target_dir.join(format!("{}.x", name));
                    std::fs::write(file_path, content)
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                }
                Entry::Module { namespace } => {
                    let subdir = target_dir.join(name);
                    self.export_namespace(namespace, &subdir)?;
                }
            }
        }
        
        Ok(())
    }

    /// Import from filesystem
    pub fn import(&mut self, source_dir: &Path) -> Result<(), String> {
        let entries = std::fs::read_dir(source_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if path.is_file() && file_name.ends_with(".x") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                let name = file_name.trim_end_matches(".x");
                self.edit(name, content)?;
            } else if path.is_dir() {
                // Create sub-namespace and import recursively
                self.create_namespace(&file_name)?;
                self.cd(&file_name)?;
                self.import(&path)?;
                self.cd("..")?;
            }
        }
        
        Ok(())
    }

    // Helper methods
    
    fn resolve_path(&self, path: &str) -> Result<Vec<String>, String> {
        if path.starts_with('/') {
            // Absolute path
            Ok(path.trim_start_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect())
        } else if path == ".." {
            // Parent directory
            let mut new_path = self.current_path.clone();
            new_path.pop();
            Ok(new_path)
        } else if path == "." {
            // Current directory
            Ok(self.current_path.clone())
        } else {
            // Relative path
            let mut new_path = self.current_path.clone();
            for segment in path.split('/') {
                if segment == ".." {
                    new_path.pop();
                } else if !segment.is_empty() && segment != "." {
                    new_path.push(segment.to_string());
                }
            }
            Ok(new_path)
        }
    }

    fn get_namespace(&self, path: &[String]) -> Option<&Namespace> {
        let mut current = &self.root;
        
        for segment in path {
            match current.entries.get(segment) {
                Some(Entry::Module { namespace }) => {
                    current = namespace;
                }
                _ => return None,
            }
        }
        
        Some(current)
    }

    fn get_current_namespace(&self) -> Option<&Namespace> {
        self.get_namespace(&self.current_path)
    }

    fn get_current_namespace_mut(&mut self) -> Option<&mut Namespace> {
        let mut current = &mut self.root;
        
        for segment in &self.current_path {
            match current.entries.get_mut(segment) {
                Some(Entry::Module { namespace }) => {
                    current = namespace;
                }
                _ => return None,
            }
        }
        
        Some(current)
    }

    fn create_namespace(&mut self, name: &str) -> Result<(), String> {
        // Check if already exists
        {
            let ns = self.get_current_namespace()
                .ok_or("Invalid namespace")?;
            
            if ns.entries.contains_key(name) {
                return Err(format!("Entry already exists: {}", name));
            }
        }
        
        let mut new_path = self.current_path.clone();
        new_path.push(name.to_string());
        
        let ns = self.get_current_namespace_mut()
            .ok_or("Invalid namespace")?;
            
        ns.entries.insert(name.to_string(), Entry::Module {
            namespace: Namespace::new(new_path),
        });
        
        Ok(())
    }

    fn export_namespace(&self, ns: &Namespace, target_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        
        for (name, entry) in &ns.entries {
            match entry {
                Entry::Function { current, .. } => {
                    let content = self.version_store.get_content(current)
                        .ok_or("Version not found")?;
                    let file_path = target_dir.join(format!("{}.x", name));
                    std::fs::write(file_path, content)
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                }
                Entry::Module { namespace } => {
                    let subdir = target_dir.join(name);
                    self.export_namespace(namespace, &subdir)?;
                }
            }
        }
        
        Ok(())
    }
}

impl Namespace {
    fn new(path: Vec<String>) -> Self {
        Self {
            path,
            entries: HashMap::new(),
            parent: None,
        }
    }
}

impl VersionStore {
    fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    fn add_version(&mut self, version: Version) {
        self.versions.insert(version.hash.clone(), version);
    }

    fn get_content(&self, hash: &Hash) -> Option<String> {
        self.versions.get(hash).map(|v| v.content.clone())
    }
}

impl Hash {
    fn compute(content: &str) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Hash(format!("{:x}", result))
    }
}

fn get_current_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

fn analyze_dependencies(content: &str) -> HashSet<Hash> {
    use x_parser::{parse_source, SyntaxStyle, FileId, dependency::DependencyManager};
    
    // Parse the content
    match parse_source(content, FileId::new(0), SyntaxStyle::Haskell) {
        Ok(cu) => {
            let mut deps = HashSet::new();
            
            // Extract dependencies from all items
            for item in &cu.module.items {
                match item {
                    x_parser::ast::Item::ValueDef(def) => {
                        let dep_symbols = DependencyManager::extract_dependencies_from_def(def);
                        // Convert symbol names to hashes (simplified)
                        for symbol in dep_symbols {
                            // For now, use symbol name as hash
                            deps.insert(Hash(format!("dep_{}", symbol)));
                        }
                    }
                    _ => {}
                }
            }
            
            deps
        }
        Err(_) => {
            // If parsing fails, return empty set
            HashSet::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_navigation() {
        let mut mgr = NamespaceManager::new();
        
        // Test pwd
        assert_eq!(mgr.pwd(), "/");
        
        // Create namespace
        mgr.create_namespace("Core").unwrap();
        mgr.cd("Core").unwrap();
        assert_eq!(mgr.pwd(), "/Core");
        
        // Test relative paths
        mgr.cd("..").unwrap();
        assert_eq!(mgr.pwd(), "/");
    }

    #[test]
    fn test_function_management() {
        let mut mgr = NamespaceManager::new();
        
        // Create a function
        let content = "add x y = x + y";
        let hash = mgr.edit("add", content.to_string()).unwrap();
        assert!(hash.is_some());
        
        // List should show the function
        let entries = mgr.ls();
        assert!(entries.iter().any(|e| e.starts_with("add#")));
        
        // Cat should return the content
        let retrieved = mgr.cat("add").unwrap();
        assert_eq!(retrieved, content);
    }
    
    #[test]
    fn test_version_history() {
        let mut mgr = NamespaceManager::new();
        
        // Create multiple versions
        mgr.edit("map", "map f list = ...".to_string()).unwrap();
        mgr.edit("map", "map f list = ... # v2".to_string()).unwrap();
        mgr.edit("map", "map f list = ... # v3".to_string()).unwrap();
        
        // Check history
        let log = mgr.log("map").unwrap();
        assert_eq!(log.len(), 3);
        
        // History should be in reverse order (newest first)
        assert!(log[0].contains("Edit map"));
    }

    #[test]
    fn test_hash_computation() {
        let hash1 = Hash::compute("content1");
        let hash2 = Hash::compute("content2");
        let hash3 = Hash::compute("content1");
        
        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }
}