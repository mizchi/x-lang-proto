//! Metadata for functions and definitions
//! 
//! This module provides structures to store human-readable metadata
//! alongside content-addressed definitions.

use crate::symbol::Symbol;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Metadata for a definition (function, type, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionMetadata {
    /// Human-readable name
    pub name: Symbol,
    /// Content hash of the definition
    pub hash: ContentHash,
    /// Direct dependencies (by name)
    pub dependencies: HashSet<Symbol>,
    /// Source location information
    pub source_info: Option<SourceInfo>,
    /// Documentation
    pub documentation: Option<String>,
    /// Type signature (if available)
    pub type_signature: Option<String>,
    /// Whether this is exported
    pub is_exported: bool,
}

/// Content hash for content-addressed storage
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        ContentHash(hex::encode(result))
    }
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

/// Module metadata containing all definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// Module name
    pub name: Symbol,
    /// All definitions in the module
    pub definitions: HashMap<Symbol, DefinitionMetadata>,
    /// Export list
    pub exports: Vec<Symbol>,
    /// Import statements (for code generation)
    pub imports: Vec<ImportMetadata>,
}

/// Import metadata for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    /// Module to import from
    pub module_path: Vec<Symbol>,
    /// Specific items to import
    pub items: Vec<ImportItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: Symbol,
    pub alias: Option<Symbol>,
}

/// Repository for storing metadata
#[derive(Debug, Clone)]
pub struct MetadataRepository {
    /// Map from content hash to definition metadata
    definitions: HashMap<ContentHash, DefinitionMetadata>,
    /// Map from human-readable name to content hash
    name_index: HashMap<Symbol, ContentHash>,
    /// Reverse index: hash to names (multiple names can point to same content)
    hash_to_names: HashMap<ContentHash, HashSet<Symbol>>,
}

impl Default for MetadataRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataRepository {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            name_index: HashMap::new(),
            hash_to_names: HashMap::new(),
        }
    }

    /// Store a definition with its metadata
    pub fn store_definition(&mut self, metadata: DefinitionMetadata) {
        let hash = metadata.hash.clone();
        let name = metadata.name;
        
        // Store the definition
        self.definitions.insert(hash.clone(), metadata);
        
        // Update indices
        self.name_index.insert(name, hash.clone());
        self.hash_to_names
            .entry(hash)
            .or_default()
            .insert(name);
    }

    /// Look up definition by name
    pub fn lookup_by_name(&self, name: &Symbol) -> Option<&DefinitionMetadata> {
        self.name_index
            .get(name)
            .and_then(|hash| self.definitions.get(hash))
    }

    /// Look up definition by hash
    pub fn lookup_by_hash(&self, hash: &ContentHash) -> Option<&DefinitionMetadata> {
        self.definitions.get(hash)
    }

    /// Get all names for a given hash
    pub fn get_names_for_hash(&self, hash: &ContentHash) -> Option<&HashSet<Symbol>> {
        self.hash_to_names.get(hash)
    }

    /// Extract all dependencies for a set of definitions
    pub fn extract_dependencies(&self, roots: &[Symbol]) -> HashMap<Symbol, ContentHash> {
        let mut result = HashMap::new();
        let mut queue = roots.to_vec();
        let mut visited = HashSet::new();

        while let Some(name) = queue.pop() {
            if visited.contains(&name) {
                continue;
            }
            visited.insert(name);

            if let Some(metadata) = self.lookup_by_name(&name) {
                result.insert(name, metadata.hash.clone());
                
                // Add dependencies to queue
                for dep in &metadata.dependencies {
                    if !visited.contains(dep) {
                        queue.push(*dep);
                    }
                }
            }
        }

        result
    }

    /// Generate import statements for a set of definitions
    pub fn generate_imports(&self, definitions: &[Symbol]) -> Vec<ImportMetadata> {
        let mut imports = HashMap::new();
        
        for name in definitions {
            if let Some(metadata) = self.lookup_by_name(name) {
                for dep in &metadata.dependencies {
                    // Skip if dependency is in the same set
                    if definitions.contains(dep) {
                        continue;
                    }
                    
                    // For now, assume all external deps come from "Prelude"
                    // This will be improved with proper module system
                    let module_path = vec![Symbol::intern("Prelude")];
                    
                    imports
                        .entry(module_path.clone())
                        .or_insert_with(|| ImportMetadata {
                            module_path,
                            items: Vec::new(),
                        })
                        .items
                        .push(ImportItem {
                            name: *dep,
                            alias: None,
                        });
                }
            }
        }
        
        imports.into_values().collect()
    }
    
    /// Get all content hashes stored in the repository
    pub fn all_hashes(&self) -> Vec<ContentHash> {
        self.definitions.keys().cloned().collect()
    }
}