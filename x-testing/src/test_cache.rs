//! Test result caching system
//!
//! This module implements persistent caching of test results based on
//! content hashes, allowing tests to be skipped if their code hasn't changed.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use x_editor::content_addressing::ContentHash;
use crate::test_runner::TestResult;

/// Cached test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTestResult {
    /// Hash of the test function
    pub test_hash: ContentHash,
    
    /// The test result
    pub result: TestResult,
    
    /// Dependencies and their hashes at the time of execution
    pub dependencies: HashMap<String, ContentHash>,
    
    /// When the test was executed
    pub executed_at: chrono::DateTime<chrono::Utc>,
    
    /// Version of x Language used
    pub x_version: String,
}

/// Test cache
#[derive(Clone)]
pub struct TestCache {
    cache_dir: PathBuf,
}

impl TestCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        fs::create_dir_all(cache_dir)
            .context("Failed to create test cache directory")?;
        
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
        })
    }
    
    /// Get cached test result
    pub fn get(&self, test_hash: &ContentHash) -> Result<Option<CachedTestResult>> {
        let cache_file = self.cache_file_path(test_hash);
        
        if !cache_file.exists() {
            return Ok(None);
        }
        
        let data = fs::read(&cache_file)
            .context("Failed to read cache file")?;
        
        let cached: CachedTestResult = serde_json::from_slice(&data)
            .context("Failed to deserialize cached test result")?;
        
        // Verify the hash matches
        if cached.test_hash != *test_hash {
            return Ok(None);
        }
        
        Ok(Some(cached))
    }
    
    /// Store test result in cache
    pub fn put(&self, test_hash: &ContentHash, result: &CachedTestResult) -> Result<()> {
        let cache_file = self.cache_file_path(test_hash);
        
        // Ensure parent directory exists
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create cache subdirectory")?;
        }
        
        let data = serde_json::to_vec_pretty(result)
            .context("Failed to serialize test result")?;
        
        fs::write(&cache_file, data)
            .context("Failed to write cache file")?;
        
        Ok(())
    }
    
    /// Clear cache for a specific test
    pub fn clear(&self, test_hash: &ContentHash) -> Result<()> {
        let cache_file = self.cache_file_path(test_hash);
        
        if cache_file.exists() {
            fs::remove_file(&cache_file)
                .context("Failed to remove cache file")?;
        }
        
        Ok(())
    }
    
    /// Clear all cached results
    pub fn clear_all(&self) -> Result<()> {
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                fs::remove_file(&path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats::default();
        
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                stats.total_entries += 1;
                
                if let Ok(data) = fs::read(&path) {
                    stats.total_size_bytes += data.len() as u64;
                    
                    if let Ok(cached) = serde_json::from_slice::<CachedTestResult>(&data) {
                        match cached.result {
                            TestResult::Pass { .. } => stats.passed_tests += 1,
                            TestResult::Fail { .. } => stats.failed_tests += 1,
                            TestResult::Skipped { .. } => stats.skipped_tests += 1,
                            TestResult::Cached { .. } => {} // Shouldn't happen in cache
                        }
                    }
                }
            }
        }
        
        Ok(stats)
    }
    
    /// Generate cache file path for a test hash
    fn cache_file_path(&self, test_hash: &ContentHash) -> PathBuf {
        // Use first 2 characters as subdirectory for better file system performance
        let hash_str = &test_hash.0;
        let subdir = &hash_str[..2];
        let filename = format!("{}.json", hash_str);
        
        self.cache_dir.join(subdir).join(filename)
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
}

impl CacheStats {
    pub fn hit_rate(&self, total_runs: usize) -> f64 {
        if total_runs == 0 {
            0.0
        } else {
            (self.total_entries as f64) / (total_runs as f64) * 100.0
        }
    }
    
    pub fn size_mb(&self) -> f64 {
        (self.total_size_bytes as f64) / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache = TestCache::new(temp_dir.path()).unwrap();
        
        let test_hash = ContentHash::new(b"test_function");
        let result = CachedTestResult {
            test_hash: test_hash.clone(),
            result: TestResult::Pass {
                duration_ms: 100,
                output: Some("Test passed".to_string()),
            },
            dependencies: HashMap::new(),
            executed_at: chrono::Utc::now(),
            x_version: "0.1.0".to_string(),
        };
        
        // Put and get
        cache.put(&test_hash, &result).unwrap();
        let cached = cache.get(&test_hash).unwrap();
        assert!(cached.is_some());
        
        // Clear
        cache.clear(&test_hash).unwrap();
        let cached = cache.get(&test_hash).unwrap();
        assert!(cached.is_none());
    }
}