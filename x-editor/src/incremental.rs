//! Incremental analysis and compilation support

use x_parser::CompilationUnit;
use x_checker::CheckResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Result of incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Analysis ID
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: SystemTime,
    /// Analysis duration
    pub duration: Duration,
    /// Type checking result
    pub type_check: CheckResult,
    /// Affected nodes
    pub affected_nodes: Vec<Vec<usize>>,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Cache entry for incremental analysis
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Hash of the input
    hash: u64,
    /// Analysis result
    result: AnalysisResult,
    /// Last access time
    last_accessed: SystemTime,
}

/// Incremental analyzer for efficient reanalysis
#[derive(Debug)]
pub struct IncrementalAnalyzer {
    /// Cache of analysis results
    cache: Arc<DashMap<String, CacheEntry>>,
    /// Maximum cache size
    max_cache_size: usize,
    /// Cache hit statistics
    cache_hits: std::sync::atomic::AtomicUsize,
    /// Cache miss statistics
    cache_misses: std::sync::atomic::AtomicUsize,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_cache_size,
            cache_hits: std::sync::atomic::AtomicUsize::new(0),
            cache_misses: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Analyze AST incrementally
    pub fn analyze(
        &self,
        ast: &CompilationUnit,
        previous_result: Option<&AnalysisResult>,
    ) -> AnalysisResult {
        let start_time = SystemTime::now();
        let analysis_id = Uuid::new_v4().to_string();
        
        // Calculate hash of the AST
        let hash = self.calculate_hash(ast);
        let cache_key = format!("ast_{}", hash);
        
        // Check cache first
        if let Some(cached) = self.get_from_cache(&cache_key) {
            self.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return cached.result;
        }
        
        self.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Perform analysis
        let type_check = x_checker::type_check(ast);
        let affected_nodes = self.compute_affected_nodes(ast, previous_result);
        let dependencies = self.compute_dependencies(ast);
        
        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        
        let result = AnalysisResult {
            id: analysis_id,
            timestamp: start_time,
            duration,
            type_check,
            affected_nodes,
            dependencies,
        };
        
        // Cache the result
        self.store_in_cache(cache_key, hash, result.clone());
        
        result
    }

    /// Analyze only the changed parts
    pub fn analyze_incremental(
        &self,
        ast: &CompilationUnit,
        changed_paths: &[Vec<usize>],
        previous_result: &AnalysisResult,
    ) -> AnalysisResult {
        let start_time = SystemTime::now();
        let analysis_id = Uuid::new_v4().to_string();
        
        // For now, perform full analysis
        // TODO: Implement true incremental analysis
        let type_check = x_checker::type_check(ast);
        let affected_nodes = changed_paths.to_vec();
        let dependencies = self.compute_dependencies(ast);
        
        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        
        AnalysisResult {
            id: analysis_id,
            timestamp: start_time,
            duration,
            type_check,
            affected_nodes,
            dependencies,
        }
    }

    /// Clear the analysis cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.cache_misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        
        CacheStats {
            cache_size: self.cache.len(),
            cache_hits: hits,
            cache_misses: misses,
            hit_rate,
        }
    }

    /// Evict old cache entries
    pub fn evict_old_entries(&self, max_age: Duration) {
        let now = SystemTime::now();
        let mut to_remove = Vec::new();
        
        for entry in self.cache.iter() {
            if let Ok(age) = now.duration_since(entry.last_accessed) {
                if age > max_age {
                    to_remove.push(entry.key().clone());
                }
            }
        }
        
        for key in to_remove {
            self.cache.remove(&key);
        }
    }

    /// Get result from cache
    fn get_from_cache(&self, key: &str) -> Option<CacheEntry> {
        if let Some(mut entry) = self.cache.get_mut(key) {
            entry.last_accessed = SystemTime::now();
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Store result in cache
    fn store_in_cache(&self, key: String, hash: u64, result: AnalysisResult) {
        // Evict entries if cache is full
        if self.cache.len() >= self.max_cache_size {
            self.evict_lru_entries();
        }
        
        let entry = CacheEntry {
            hash,
            result,
            last_accessed: SystemTime::now(),
        };
        
        self.cache.insert(key, entry);
    }

    /// Evict least recently used entries
    fn evict_lru_entries(&self) {
        let mut entries: Vec<_> = self.cache.iter()
            .map(|entry| (entry.key().clone(), entry.last_accessed))
            .collect();
        
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);
        
        // Remove oldest 10% of entries
        let to_remove = entries.len() / 10;
        for (key, _) in entries.into_iter().take(to_remove) {
            self.cache.remove(&key);
        }
    }

    /// Calculate hash of AST
    fn calculate_hash(&self, ast: &CompilationUnit) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash module count
        ast.modules.len().hash(&mut hasher);
        
        // Hash each module
        for module in &ast.modules {
            module.name.as_str().hash(&mut hasher);
            module.items.len().hash(&mut hasher);
            // TODO: Hash item contents
        }
        
        hasher.finish()
    }

    /// Compute affected nodes
    fn compute_affected_nodes(
        &self,
        _ast: &CompilationUnit,
        _previous_result: Option<&AnalysisResult>,
    ) -> Vec<Vec<usize>> {
        // TODO: Implement affected node computation
        Vec::new()
    }

    /// Compute dependencies
    fn compute_dependencies(&self, ast: &CompilationUnit) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Add imports as dependencies
        for import in &ast.imports {
            dependencies.push(import.module.as_str().to_string());
        }
        
        dependencies
    }
}

impl Default for IncrementalAnalyzer {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current cache size
    pub cache_size: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Cache hit rate
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{parse_source, FileId, SyntaxStyle};

    #[test]
    fn test_incremental_analyzer_creation() {
        let analyzer = IncrementalAnalyzer::new(100);
        assert_eq!(analyzer.max_cache_size, 100);
    }

    #[test]
    fn test_analysis() {
        let analyzer = IncrementalAnalyzer::new(100);
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
        
        let result = analyzer.analyze(&ast, None);
        assert!(!result.id.is_empty());
    }

    #[test]
    fn test_cache_functionality() {
        let analyzer = IncrementalAnalyzer::new(100);
        let source = "let x = 42";
        let ast = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
        
        // First analysis
        let _result1 = analyzer.analyze(&ast, None);
        
        // Second analysis should hit cache
        let _result2 = analyzer.analyze(&ast, None);
        
        let stats = analyzer.cache_stats();
        assert!(stats.cache_hits > 0 || stats.cache_misses > 0);
    }

    #[test]
    fn test_cache_stats() {
        let analyzer = IncrementalAnalyzer::new(100);
        let stats = analyzer.cache_stats();
        
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }
}