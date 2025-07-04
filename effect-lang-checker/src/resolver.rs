//! Module resolution and dependency management
//! 
//! This module implements the core logic for resolving module paths,
//! managing dependencies, and providing incremental module analysis.

use crate::core::{
    ast::{ModulePath, Import, ImportKind},
    span::{FileId, Span},
    symbol::Symbol,
};
// use crate::analysis::database::Database;
use crate::{Error, Result};

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Module resolver responsible for finding and loading modules
#[derive(Debug, Clone)]
pub struct ModuleResolver {
    /// Workspace root directory
    workspace_root: PathBuf,
    /// Source directories to search for modules
    source_dirs: Vec<PathBuf>,
    /// External dependencies
    dependencies: HashMap<String, DependencyInfo>,
    /// Module path to file ID mapping cache
    module_cache: HashMap<ModulePath, FileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Registry { registry: String },
    Git { url: String, rev: Option<String> },
    Path { path: PathBuf },
    Local { path: PathBuf },
}

/// Module resolution result
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleResolution {
    pub file_id: FileId,
    pub module_path: ModulePath,
    pub source_type: ModuleSourceType,
    pub dependencies: Vec<ModulePath>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSourceType {
    Local,
    Dependency(String),
    Standard,
}

/// Dependency graph for managing module relationships
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Module -> Direct dependencies
    direct_deps: HashMap<FileId, Vec<FileId>>,
    /// Module -> All transitive dependencies
    transitive_deps: HashMap<FileId, HashSet<FileId>>,
    /// Reverse dependencies (what depends on this module)
    reverse_deps: HashMap<FileId, Vec<FileId>>,
}

impl ModuleResolver {
    pub fn new(workspace_root: PathBuf) -> Self {
        let mut source_dirs = vec![
            workspace_root.join("src"),
            workspace_root.join("lib"),
        ];
        
        // Add standard library paths
        if let Ok(stdlib_path) = std::env::var("EFFECT_STDLIB_PATH") {
            source_dirs.push(PathBuf::from(stdlib_path));
        }
        
        ModuleResolver {
            workspace_root,
            source_dirs,
            dependencies: HashMap::new(),
            module_cache: HashMap::new(),
        }
    }
    
    /// Load workspace configuration from effect.toml
    pub fn load_workspace_config(&mut self, config_path: &Path) -> Result<()> {
        let config_content = std::fs::read_to_string(config_path)?;
        let config: WorkspaceConfig = toml::from_str(&config_content)
            .map_err(|e| Error::Parse { message: format!("Invalid effect.toml: {}", e) })?;
        
        self.dependencies = config.dependencies;
        
        // Add additional source directories
        for source_dir in config.source_dirs.unwrap_or_default() {
            self.source_dirs.push(self.workspace_root.join(source_dir));
        }
        
        Ok(())
    }
    
    /// Resolve a module path to a file ID
    pub fn resolve_module(&mut self, module_path: &ModulePath) -> Result<FileId> {
        // Check cache first
        if let Some(&file_id) = self.module_cache.get(module_path) {
            return Ok(file_id);
        }
        
        // Try different resolution strategies
        let file_path = self.resolve_local_module(module_path)
            .or_else(|| self.resolve_dependency_module(module_path))
            .or_else(|| self.resolve_standard_module(module_path))
            .ok_or_else(|| Error::Parse { 
                message: format!("Module not found: {}", module_path.to_string()) 
            })?;
        
        // Register file with database
        let file_id = FileId::new(self.module_cache.len() as u32);
        self.module_cache.insert(module_path.clone(), file_id);
        
        Ok(file_id)
    }
    
    /// Resolve module imports and build dependency graph
    pub fn resolve_imports(
        &mut self, 
        file_id: FileId, 
        imports: &[Import]
    ) -> Result<Vec<FileId>> {
        let mut resolved_deps = Vec::new();
        
        for import in imports {
            // Handle different import kinds
            match &import.kind {
                ImportKind::Qualified | ImportKind::Selective(_) | ImportKind::Wildcard => {
                    let dep_file_id = self.resolve_module(&import.module_path)?;
                    resolved_deps.push(dep_file_id);
                }
                ImportKind::Lazy => {
                    // Lazy imports are resolved on-demand
                    let dep_file_id = self.resolve_module(&import.module_path)?;
                    resolved_deps.push(dep_file_id);
                }
                ImportKind::Conditional(_condition) => {
                    // Conditional imports are resolved based on compile-time conditions
                    // For now, always resolve them
                    let dep_file_id = self.resolve_module(&import.module_path)?;
                    resolved_deps.push(dep_file_id);
                }
            }
        }
        
        Ok(resolved_deps)
    }
    
    /// Check for circular dependencies
    pub fn check_circular_dependencies(&self, graph: &DependencyGraph) -> Result<()> {
        for (&module, deps) in &graph.direct_deps {
            if self.has_circular_dependency(graph, module, deps, &mut HashSet::new()) {
                return Err(Error::Parse {
                    message: format!("Circular dependency detected involving module {:?}", module),
                });
            }
        }
        Ok(())
    }
    
    /// Get all modules that depend on the given module (for incremental recompilation)
    pub fn get_dependent_modules(&self, graph: &DependencyGraph, file_id: FileId) -> Vec<FileId> {
        graph.reverse_deps.get(&file_id).cloned().unwrap_or_default()
    }
    
    /// Resolve local module (within workspace)
    fn resolve_local_module(&self, module_path: &ModulePath) -> Option<PathBuf> {
        for source_dir in &self.source_dirs {
            // Try different file extensions and naming conventions
            let candidates = self.generate_path_candidates(source_dir, module_path);
            
            for candidate in candidates {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
        None
    }
    
    /// Resolve dependency module (external crate)
    fn resolve_dependency_module(&self, module_path: &ModulePath) -> Option<PathBuf> {
        if let Some(first_segment) = module_path.segments.first() {
            let dep_name = first_segment.as_str();
            
            if let Some(dep_info) = self.dependencies.get(dep_name) {
                let dep_path = self.get_dependency_path(dep_info);
                
                // Create module path within dependency
                let mut relative_path = PathBuf::new();
                for segment in &module_path.segments[1..] {
                    relative_path.push(segment.as_str());
                }
                
                let full_path = dep_path.join("src").join(relative_path);
                return self.find_module_file(&full_path);
            }
        }
        None
    }
    
    /// Resolve standard library module
    fn resolve_standard_module(&self, module_path: &ModulePath) -> Option<PathBuf> {
        // Standard library modules start with "Std" or "Core"
        if let Some(first_segment) = module_path.segments.first() {
            let first_name = first_segment.as_str();
            if first_name == "Std" || first_name == "Core" || first_name == "Prelude" {
                // Look in standard library paths
                if let Ok(stdlib_path) = std::env::var("EFFECT_STDLIB_PATH") {
                    let stdlib_dir = PathBuf::from(stdlib_path);
                    let candidates = self.generate_path_candidates(&stdlib_dir, module_path);
                    
                    for candidate in candidates {
                        if candidate.exists() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Generate possible file paths for a module
    fn generate_path_candidates(&self, base_dir: &Path, module_path: &ModulePath) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        
        // Convert module path to file path
        let mut path = base_dir.to_path_buf();
        for segment in &module_path.segments {
            path.push(segment.as_str());
        }
        
        // Try different extensions and conventions
        candidates.push(path.with_extension("eff"));
        candidates.push(path.with_extension("effect"));
        
        // Try mod.eff in directory
        candidates.push(path.join("mod.eff"));
        candidates.push(path.join("mod.effect"));
        
        // Try lowercase versions
        let mut lowercase_path = base_dir.to_path_buf();
        for segment in &module_path.segments {
            lowercase_path.push(segment.as_str().to_lowercase());
        }
        candidates.push(lowercase_path.with_extension("eff"));
        
        candidates
    }
    
    /// Find module file with different naming conventions
    fn find_module_file(&self, base_path: &Path) -> Option<PathBuf> {
        let extensions = ["eff", "effect"];
        
        for ext in &extensions {
            let path = base_path.with_extension(ext);
            if path.exists() {
                return Some(path);
            }
        }
        
        // Try mod.eff in directory
        for ext in &extensions {
            let mod_path = base_path.join(format!("mod.{}", ext));
            if mod_path.exists() {
                return Some(mod_path);
            }
        }
        
        None
    }
    
    /// Get the file system path for a dependency
    fn get_dependency_path(&self, dep_info: &DependencyInfo) -> PathBuf {
        match &dep_info.source {
            DependencySource::Path { path } => path.clone(),
            DependencySource::Local { path } => self.workspace_root.join(path),
            DependencySource::Git { .. } => {
                // In a real implementation, this would resolve to the git checkout directory
                self.workspace_root.join(".effect").join("git").join(&dep_info.name)
            }
            DependencySource::Registry { .. } => {
                // In a real implementation, this would resolve to the registry cache
                self.workspace_root.join(".effect").join("registry").join(&dep_info.name)
            }
        }
    }
    
    /// Check for circular dependencies using DFS
    fn has_circular_dependency(
        &self,
        graph: &DependencyGraph,
        current: FileId,
        deps: &[FileId],
        visited: &mut HashSet<FileId>,
    ) -> bool {
        if visited.contains(&current) {
            return true;
        }
        
        visited.insert(current);
        
        for &dep in deps {
            if let Some(dep_deps) = graph.direct_deps.get(&dep) {
                if self.has_circular_dependency(graph, dep, dep_deps, visited) {
                    return true;
                }
            }
        }
        
        visited.remove(&current);
        false
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            direct_deps: HashMap::new(),
            transitive_deps: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }
    
    /// Add a dependency relationship
    pub fn add_dependency(&mut self, from: FileId, to: FileId) {
        self.direct_deps.entry(from).or_default().push(to);
        self.reverse_deps.entry(to).or_default().push(from);
    }
    
    /// Compute transitive dependencies
    pub fn compute_transitive_deps(&mut self) {
        for &module in self.direct_deps.keys() {
            let mut transitive = HashSet::new();
            let mut queue = VecDeque::new();
            
            if let Some(direct) = self.direct_deps.get(&module) {
                for &dep in direct {
                    queue.push_back(dep);
                    transitive.insert(dep);
                }
            }
            
            while let Some(current) = queue.pop_front() {
                if let Some(deps) = self.direct_deps.get(&current) {
                    for &dep in deps {
                        if transitive.insert(dep) {
                            queue.push_back(dep);
                        }
                    }
                }
            }
            
            self.transitive_deps.insert(module, transitive);
        }
    }
    
    /// Get topological order for compilation
    pub fn topological_order(&self) -> Result<Vec<FileId>> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut result = Vec::new();
        
        for &module in self.direct_deps.keys() {
            if !visited.contains(&module) {
                self.topological_visit(module, &mut visited, &mut temp_visited, &mut result)?;
            }
        }
        
        result.reverse();
        Ok(result)
    }
    
    fn topological_visit(
        &self,
        module: FileId,
        visited: &mut HashSet<FileId>,
        temp_visited: &mut HashSet<FileId>,
        result: &mut Vec<FileId>,
    ) -> Result<()> {
        if temp_visited.contains(&module) {
            return Err(Error::Parse {
                message: "Circular dependency detected".to_string(),
            });
        }
        
        if visited.contains(&module) {
            return Ok(());
        }
        
        temp_visited.insert(module);
        
        if let Some(deps) = self.direct_deps.get(&module) {
            for &dep in deps {
                self.topological_visit(dep, visited, temp_visited, result)?;
            }
        }
        
        temp_visited.remove(&module);
        visited.insert(module);
        result.push(module);
        
        Ok(())
    }
}

/// Workspace configuration structure
#[derive(Debug, Deserialize)]
struct WorkspaceConfig {
    #[serde(default)]
    dependencies: HashMap<String, DependencyInfo>,
    #[serde(default)]
    source_dirs: Option<Vec<PathBuf>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_module_path_resolution() {
        let temp_dir = tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        
        // Create a test module file
        let module_file = src_dir.join("test.eff");
        fs::write(&module_file, "module Test\n").unwrap();
        
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        
        let module_path = ModulePath::new(
            vec![Symbol::intern("Test")],
            Span::new(FileId::new(0), crate::core::span::ByteOffset(0), crate::core::span::ByteOffset(4))
        );
        
        let resolved = resolver.resolve_local_module(&module_path);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), module_file);
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();
        
        let file1 = FileId::new(1);
        let file2 = FileId::new(2);
        let file3 = FileId::new(3);
        
        graph.add_dependency(file1, file2);
        graph.add_dependency(file2, file3);
        
        graph.compute_transitive_deps();
        
        let transitive = graph.transitive_deps.get(&file1).unwrap();
        assert!(transitive.contains(&file2));
        assert!(transitive.contains(&file3));
        
        let order = graph.topological_order().unwrap();
        assert_eq!(order, vec![file3, file2, file1]);
    }
}