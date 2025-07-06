//! Content addressing system for types and functions
//! 
//! This module provides content-based addressing for code artifacts,
//! enabling similarity search and version management.

use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use x_parser::ast::*;
use x_parser::Symbol;
use x_checker::types::TypeScheme;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::tree_similarity::{TreeNode, CombinedSimilarity};
use crate::annotated_ast::{AnnotatedValueDef, AnnotatedCompilationUnit};

/// Content hash for addressing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        ContentHash(format!("{:x}", result))
    }
    
    pub fn short(&self) -> &str {
        &self.0[..8]
    }
}

/// Semantic fingerprint for similarity matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFingerprint {
    /// Structural hash (ignoring names)
    pub structure_hash: ContentHash,
    
    /// Type signature hash
    pub type_hash: Option<ContentHash>,
    
    /// Feature vector for similarity
    pub features: FeatureVector,
    
    /// Normalized form for comparison
    pub normalized_form: String,
}

/// Feature vector for similarity computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Number of parameters
    pub param_count: usize,
    
    /// Depth of expression tree
    pub expr_depth: usize,
    
    /// Types of operations used
    pub operations: HashSet<String>,
    
    /// Pattern matching complexity
    pub pattern_complexity: usize,
    
    /// Effect usage
    pub effects: HashSet<String>,
    
    /// Recursive structure
    pub is_recursive: bool,
}

/// Version information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }
    
    pub fn to_string(&self) -> String {
        if let Some(pre) = &self.prerelease {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Content entry in the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    /// Content hash
    pub hash: ContentHash,
    
    /// Semantic fingerprint
    pub fingerprint: SemanticFingerprint,
    
    /// User-assigned version
    pub version: Option<Version>,
    
    /// Human-readable name
    pub name: String,
    
    /// Module path
    pub module_path: Option<String>,
    
    /// Tags for categorization
    pub tags: HashSet<String>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// The actual content
    pub content: ContentItem,
}

/// Actual content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentItem {
    Function {
        ast: ValueDef,
        annotated_ast: Option<AnnotatedValueDef>,
        type_scheme: Option<TypeScheme>,
    },
    Type {
        ast: TypeDef,
        constructors: Vec<Symbol>,
    },
    Effect {
        ast: EffectDef,
        operations: Vec<Symbol>,
    },
    AnnotatedModule {
        ast: AnnotatedCompilationUnit,
    },
}

/// Content repository for storage and search
#[derive(Clone)]
pub struct ContentRepository {
    /// All entries by hash
    pub entries: HashMap<ContentHash, ContentEntry>,
    
    /// Index by structure hash for similarity
    structure_index: HashMap<ContentHash, Vec<ContentHash>>,
    
    /// Version index
    version_index: HashMap<String, HashMap<Version, ContentHash>>,
    
    /// Tag index
    tag_index: HashMap<String, HashSet<ContentHash>>,
    
    /// Tree similarity calculator
    similarity_calc: CombinedSimilarity,
}

impl ContentRepository {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            structure_index: HashMap::new(),
            version_index: HashMap::new(),
            tag_index: HashMap::new(),
            similarity_calc: CombinedSimilarity::default(),
        }
    }
    
    /// Add a function to the repository
    pub fn add_function(
        &mut self,
        name: &str,
        def: &ValueDef,
        type_scheme: Option<&TypeScheme>,
        version: Option<Version>,
        tags: HashSet<String>,
    ) -> Result<ContentHash> {
        let fingerprint = self.compute_function_fingerprint(def)?;
        let content_data = self.serialize_function(def)?;
        let hash = ContentHash::new(&content_data);
        
        let entry = ContentEntry {
            hash: hash.clone(),
            fingerprint: fingerprint.clone(),
            version: version.clone(),
            name: name.to_string(),
            module_path: None,
            tags,
            created_at: chrono::Utc::now(),
            content: ContentItem::Function {
                ast: def.clone(),
                annotated_ast: None,
                type_scheme: type_scheme.cloned(),
            },
        };
        
        // Update indices
        self.entries.insert(hash.clone(), entry);
        
        self.structure_index
            .entry(fingerprint.structure_hash)
            .or_insert_with(Vec::new)
            .push(hash.clone());
            
        if let Some(v) = version {
            self.version_index
                .entry(name.to_string())
                .or_insert_with(HashMap::new)
                .insert(v, hash.clone());
        }
        
        Ok(hash)
    }
    
    /// Add a function with type annotations to the repository
    pub fn add_annotated_function(
        &mut self,
        name: &str,
        def: &ValueDef,
        annotated_def: &AnnotatedValueDef,
        version: Option<Version>,
        tags: HashSet<String>,
    ) -> Result<ContentHash> {
        let fingerprint = self.compute_function_fingerprint(def)?;
        let content_data = self.serialize_annotated_function(annotated_def)?;
        let hash = ContentHash::new(&content_data);
        
        let entry = ContentEntry {
            hash: hash.clone(),
            fingerprint: fingerprint.clone(),
            version: version.clone(),
            name: name.to_string(),
            module_path: None,
            tags,
            created_at: chrono::Utc::now(),
            content: ContentItem::Function {
                ast: def.clone(),
                annotated_ast: Some(annotated_def.clone()),
                type_scheme: annotated_def.inferred_type.clone(),
            },
        };
        
        // Update indices
        self.entries.insert(hash.clone(), entry);
        
        self.structure_index
            .entry(fingerprint.structure_hash)
            .or_insert_with(Vec::new)
            .push(hash.clone());
            
        if let Some(v) = version {
            self.version_index
                .entry(name.to_string())
                .or_insert_with(HashMap::new)
                .insert(v, hash.clone());
        }
        
        Ok(hash)
    }
    
    /// Add an entire annotated module
    pub fn add_annotated_module(
        &mut self,
        name: &str,
        annotated_ast: &AnnotatedCompilationUnit,
        version: Option<Version>,
        tags: HashSet<String>,
    ) -> Result<ContentHash> {
        let content_data = self.serialize_annotated_module(annotated_ast)?;
        let hash = ContentHash::new(&content_data);
        
        // Extract all functions for indexing
        for item in &annotated_ast.module.items {
            if let crate::annotated_ast::AnnotatedItem::ValueDef(def) = item {
                // Create fingerprint for similarity search
                let regular_def = def.to_ast();
                let fingerprint = self.compute_function_fingerprint(&regular_def)?;
                
                self.structure_index
                    .entry(fingerprint.structure_hash)
                    .or_insert_with(Vec::new)
                    .push(hash.clone());
            }
        }
        
        let entry = ContentEntry {
            hash: hash.clone(),
            fingerprint: SemanticFingerprint {
                structure_hash: hash.clone(),
                type_hash: None,
                features: FeatureVector {
                    param_count: 0,
                    expr_depth: 0,
                    operations: HashSet::new(),
                    pattern_complexity: 0,
                    effects: HashSet::new(),
                    is_recursive: false,
                },
                normalized_form: format!("module:{}", name),
            },
            version: version.clone(),
            name: name.to_string(),
            module_path: None,
            tags,
            created_at: chrono::Utc::now(),
            content: ContentItem::AnnotatedModule {
                ast: annotated_ast.clone(),
            },
        };
        
        self.entries.insert(hash.clone(), entry);
        
        if let Some(v) = version {
            self.version_index
                .entry(name.to_string())
                .or_insert_with(HashMap::new)
                .insert(v, hash.clone());
        }
        
        Ok(hash)
    }
    
    /// Find similar functions using tree similarity algorithms
    pub fn find_similar_functions(
        &self,
        target: &ValueDef,
        threshold: f64,
    ) -> Result<Vec<(ContentHash, f64)>> {
        let target_fingerprint = self.compute_function_fingerprint(target)?;
        let target_tree = TreeNode::from_expr(&target.body);
        let mut results = Vec::new();
        
        // First, check exact structural matches
        if let Some(matches) = self.structure_index.get(&target_fingerprint.structure_hash) {
            for hash in matches {
                results.push((hash.clone(), 1.0));
            }
        }
        
        // Then, compute similarity for all functions using APTED/TSED
        for (hash, entry) in &self.entries {
            if let ContentItem::Function { ast, annotated_ast: _, .. } = &entry.content {
                // Use tree similarity algorithms
                let entry_tree = TreeNode::from_expr(&ast.body);
                let tree_similarity = self.similarity_calc.similarity(&target_tree, &entry_tree);
                
                // Also compute feature-based similarity
                let feature_similarity = self.compute_similarity(
                    &target_fingerprint,
                    &entry.fingerprint,
                );
                
                // Combine both similarities
                let combined_similarity = 0.6 * tree_similarity + 0.4 * feature_similarity;
                
                if combined_similarity >= threshold {
                    results.push((hash.clone(), combined_similarity));
                }
            }
        }
        
        // Sort by similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.dedup_by_key(|(hash, _)| hash.clone());
        
        Ok(results)
    }
    
    /// Find similar functions with detailed report
    pub fn find_similar_functions_detailed(
        &self,
        target: &ValueDef,
        threshold: f64,
    ) -> Result<Vec<(ContentHash, SimilarityDetails)>> {
        let target_fingerprint = self.compute_function_fingerprint(target)?;
        let target_tree = TreeNode::from_expr(&target.body);
        let mut results = Vec::new();
        
        for (hash, entry) in &self.entries {
            if let ContentItem::Function { ast, annotated_ast: _, .. } = &entry.content {
                let entry_tree = TreeNode::from_expr(&ast.body);
                
                // Get detailed similarity report
                let tree_report = self.similarity_calc.detailed_similarity(&target_tree, &entry_tree);
                let feature_similarity = self.compute_similarity(
                    &target_fingerprint,
                    &entry.fingerprint,
                );
                
                let details = SimilarityDetails {
                    overall_similarity: 0.6 * tree_report.combined_similarity + 0.4 * feature_similarity,
                    tree_similarity: tree_report.combined_similarity,
                    apted_similarity: tree_report.apted_similarity,
                    tsed_similarity: tree_report.tsed_similarity,
                    feature_similarity,
                    structural_match: target_fingerprint.structure_hash == entry.fingerprint.structure_hash,
                };
                
                if details.overall_similarity >= threshold {
                    results.push((hash.clone(), details));
                }
            }
        }
        
        results.sort_by(|a, b| b.1.overall_similarity.partial_cmp(&a.1.overall_similarity).unwrap());
        
        Ok(results)
    }
    
    /// Search by semantic query
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<ContentEntry>> {
        let mut results = Vec::new();
        
        for entry in self.entries.values() {
            if self.matches_query(entry, query) {
                results.push(entry.clone());
            }
        }
        
        Ok(results)
    }
    
    /// Get entry by hash
    pub fn get(&self, hash: &ContentHash) -> Option<&ContentEntry> {
        self.entries.get(hash)
    }
    
    /// Get by name and version
    pub fn get_by_version(&self, name: &str, version: &Version) -> Option<&ContentEntry> {
        self.version_index
            .get(name)?
            .get(version)
            .and_then(|hash| self.entries.get(hash))
    }
    
    /// List all versions of a named item
    pub fn list_versions(&self, name: &str) -> Vec<(Version, ContentHash)> {
        self.version_index
            .get(name)
            .map(|versions| {
                let mut list: Vec<_> = versions.iter()
                    .map(|(v, h)| (v.clone(), h.clone()))
                    .collect();
                list.sort_by(|a, b| {
                    a.0.major.cmp(&b.0.major)
                        .then(a.0.minor.cmp(&b.0.minor))
                        .then(a.0.patch.cmp(&b.0.patch))
                });
                list
            })
            .unwrap_or_default()
    }
    
    /// Compute fingerprint for a function
    fn compute_function_fingerprint(&self, def: &ValueDef) -> Result<SemanticFingerprint> {
        let normalized = self.normalize_function(def);
        let structure_data = self.serialize_normalized(&normalized)?;
        let structure_hash = ContentHash::new(&structure_data);
        
        let features = self.extract_features(def);
        
        let type_hash = def.type_annotation.as_ref().map(|ty| {
            let ty_data = format!("{:?}", ty); // Simple serialization
            ContentHash::new(ty_data.as_bytes())
        });
        
        Ok(SemanticFingerprint {
            structure_hash,
            type_hash,
            features,
            normalized_form: normalized,
        })
    }
    
    /// Normalize function for comparison (alpha-rename, etc.)
    fn normalize_function(&self, def: &ValueDef) -> String {
        // Simple normalization: replace all names with generic ones
        // In practice, this would do proper alpha-renaming
        format!(
            "fn({}) = {}",
            def.parameters.len(),
            self.normalize_expr(&def.body)
        )
    }
    
    /// Normalize expression
    fn normalize_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Var(name, _) => {
                // Map variable names to canonical forms
                if self.is_builtin(name) {
                    name.as_str().to_string()
                } else {
                    "var".to_string()
                }
            }
            Expr::App(f, args, _) => {
                format!(
                    "({} {})",
                    self.normalize_expr(f),
                    args.iter()
                        .map(|a| self.normalize_expr(a))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Expr::Lambda { parameters, body, .. } => {
                format!("λ{}.{}", parameters.len(), self.normalize_expr(body))
            }
            Expr::Literal(lit, _) => format!("{:?}", lit),
            _ => "expr".to_string(),
        }
    }
    
    /// Check if a symbol is a builtin
    fn is_builtin(&self, name: &Symbol) -> bool {
        matches!(
            name.as_str(),
            "+" | "-" | "*" | "/" | "==" | "!=" | ">" | "<" | ">=" | "<=" |
            "&&" | "||" | "not" | "print" | "print_endline"
        )
    }
    
    /// Extract features from a function
    fn extract_features(&self, def: &ValueDef) -> FeatureVector {
        let mut features = FeatureVector {
            param_count: def.parameters.len(),
            expr_depth: self.compute_depth(&def.body),
            operations: HashSet::new(),
            pattern_complexity: self.compute_pattern_complexity(&def.parameters),
            effects: HashSet::new(),
            is_recursive: self.is_recursive(&def.name, &def.body),
        };
        
        self.collect_operations(&def.body, &mut features.operations);
        
        features
    }
    
    /// Compute expression depth
    fn compute_depth(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Var(_, _) | Expr::Literal(_, _) => 1,
            Expr::App(f, args, _) => {
                let f_depth = self.compute_depth(f);
                let max_arg_depth = args.iter()
                    .map(|a| self.compute_depth(a))
                    .max()
                    .unwrap_or(0);
                1 + f_depth.max(max_arg_depth)
            }
            Expr::Lambda { body, .. } => 1 + self.compute_depth(body),
            Expr::Let { value, body, .. } => {
                1 + self.compute_depth(value).max(self.compute_depth(body))
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                1 + self.compute_depth(condition)
                    .max(self.compute_depth(then_branch))
                    .max(self.compute_depth(else_branch))
            }
            Expr::Match { scrutinee, arms, .. } => {
                let scrutinee_depth = self.compute_depth(scrutinee);
                let max_arm_depth = arms.iter()
                    .map(|arm| self.compute_depth(&arm.body))
                    .max()
                    .unwrap_or(0);
                1 + scrutinee_depth.max(max_arm_depth)
            }
            _ => 1,
        }
    }
    
    /// Compute pattern complexity
    fn compute_pattern_complexity(&self, patterns: &[Pattern]) -> usize {
        patterns.iter().map(|p| self.pattern_complexity(p)).sum()
    }
    
    fn pattern_complexity(&self, pattern: &Pattern) -> usize {
        match pattern {
            Pattern::Variable(_, _) | Pattern::Wildcard(_) => 1,
            Pattern::Literal(_, _) => 1,
            Pattern::Constructor { name: _, args, span: _ } => {
                1 + args.iter().map(|p| self.pattern_complexity(p)).sum::<usize>()
            }
            Pattern::Tuple { patterns, span: _ } => {
                1 + patterns.iter().map(|p| self.pattern_complexity(p)).sum::<usize>()
            }
            _ => 1,
        }
    }
    
    /// Check if function is recursive
    fn is_recursive(&self, name: &Symbol, expr: &Expr) -> bool {
        match expr {
            Expr::Var(n, _) => n == name,
            Expr::App(f, args, _) => {
                self.is_recursive(name, f) ||
                args.iter().any(|a| self.is_recursive(name, a))
            }
            Expr::Lambda { body, .. } => self.is_recursive(name, body),
            Expr::Let { value, body, .. } => {
                self.is_recursive(name, value) || self.is_recursive(name, body)
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.is_recursive(name, condition) ||
                self.is_recursive(name, then_branch) ||
                self.is_recursive(name, else_branch)
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.is_recursive(name, scrutinee) ||
                arms.iter().any(|arm| self.is_recursive(name, &arm.body))
            }
            _ => false,
        }
    }
    
    /// Collect operations used
    fn collect_operations(&self, expr: &Expr, ops: &mut HashSet<String>) {
        match expr {
            Expr::Var(name, _) if self.is_builtin(name) => {
                ops.insert(name.as_str().to_string());
            }
            Expr::App(f, args, _) => {
                self.collect_operations(f, ops);
                for arg in args {
                    self.collect_operations(arg, ops);
                }
            }
            Expr::Lambda { body, .. } => {
                self.collect_operations(body, ops);
            }
            Expr::Let { value, body, .. } => {
                self.collect_operations(value, ops);
                self.collect_operations(body, ops);
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.collect_operations(condition, ops);
                self.collect_operations(then_branch, ops);
                self.collect_operations(else_branch, ops);
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.collect_operations(scrutinee, ops);
                for arm in arms {
                    self.collect_operations(&arm.body, ops);
                }
            }
            _ => {}
        }
    }
    
    /// Compute similarity between fingerprints
    fn compute_similarity(
        &self,
        a: &SemanticFingerprint,
        b: &SemanticFingerprint,
    ) -> f64 {
        let mut score = 0.0;
        let mut weight = 0.0;
        
        // Exact structure match
        if a.structure_hash == b.structure_hash {
            return 1.0;
        }
        
        // Type similarity
        if let (Some(ta), Some(tb)) = (&a.type_hash, &b.type_hash) {
            if ta == tb {
                score += 0.3;
            }
            weight += 0.3;
        }
        
        // Feature similarity
        let param_sim = 1.0 - (a.features.param_count as f64 - b.features.param_count as f64).abs() / 10.0;
        score += param_sim * 0.2;
        weight += 0.2;
        
        let depth_sim = 1.0 - (a.features.expr_depth as f64 - b.features.expr_depth as f64).abs() / 20.0;
        score += depth_sim * 0.1;
        weight += 0.1;
        
        // Operation similarity (Jaccard index)
        let ops_intersection = a.features.operations.intersection(&b.features.operations).count();
        let ops_union = a.features.operations.union(&b.features.operations).count();
        if ops_union > 0 {
            let ops_sim = ops_intersection as f64 / ops_union as f64;
            score += ops_sim * 0.3;
        }
        weight += 0.3;
        
        // Recursion similarity
        if a.features.is_recursive == b.features.is_recursive {
            score += 0.1;
        }
        weight += 0.1;
        
        if weight > 0.0 {
            score / weight
        } else {
            0.0
        }
    }
    
    /// Serialize function for hashing
    fn serialize_function(&self, def: &ValueDef) -> Result<Vec<u8>> {
        // Use bincode or similar for consistent serialization
        Ok(format!("{:?}", def).into_bytes())
    }
    
    /// Serialize normalized form
    fn serialize_normalized(&self, normalized: &str) -> Result<Vec<u8>> {
        Ok(normalized.as_bytes().to_vec())
    }
    
    /// Check if entry matches query
    fn matches_query(&self, entry: &ContentEntry, query: &SearchQuery) -> bool {
        // Name match
        if let Some(name) = &query.name_pattern {
            if !entry.name.contains(name) {
                return false;
            }
        }
        
        // Tag match
        if !query.tags.is_empty() {
            let has_all_tags = query.tags.iter().all(|tag| entry.tags.contains(tag));
            if !has_all_tags {
                return false;
            }
        }
        
        // Feature constraints
        if let Some(constraints) = &query.feature_constraints {
            match &entry.content {
                ContentItem::Function { annotated_ast: _, .. } => {
                    let features = &entry.fingerprint.features;
                    
                    if let Some(min_params) = constraints.min_params {
                        if features.param_count < min_params {
                            return false;
                        }
                    }
                    
                    if let Some(max_params) = constraints.max_params {
                        if features.param_count > max_params {
                            return false;
                        }
                    }
                    
                    if let Some(must_be_recursive) = constraints.is_recursive {
                        if features.is_recursive != must_be_recursive {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        
        true
    }
    
    /// Serialize annotated function for hashing
    fn serialize_annotated_function(&self, def: &AnnotatedValueDef) -> Result<Vec<u8>> {
        let json = serde_json::to_string(def)?;
        Ok(json.into_bytes())
    }
    
    /// Serialize annotated module
    fn serialize_annotated_module(&self, module: &AnnotatedCompilationUnit) -> Result<Vec<u8>> {
        let json = serde_json::to_string(module)?;
        Ok(json.into_bytes())
    }
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Name pattern (substring match)
    pub name_pattern: Option<String>,
    
    /// Required tags
    pub tags: HashSet<String>,
    
    /// Feature constraints
    pub feature_constraints: Option<FeatureConstraints>,
    
    /// Similarity threshold
    pub similarity_threshold: Option<f64>,
}

/// Feature constraints for search
#[derive(Debug, Clone)]
pub struct FeatureConstraints {
    pub min_params: Option<usize>,
    pub max_params: Option<usize>,
    pub is_recursive: Option<bool>,
    pub required_operations: HashSet<String>,
}

/// Detailed similarity information
#[derive(Debug, Clone)]
pub struct SimilarityDetails {
    pub overall_similarity: f64,
    pub tree_similarity: f64,
    pub apted_similarity: f64,
    pub tsed_similarity: f64,
    pub feature_similarity: f64,
    pub structural_match: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::{FileId, Span, span::ByteOffset};
    
    #[test]
    fn test_content_hash() {
        let data1 = b"hello world";
        let data2 = b"hello world";
        let data3 = b"different";
        
        let hash1 = ContentHash::new(data1);
        let hash2 = ContentHash::new(data2);
        let hash3 = ContentHash::new(data3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 2, 0);
        let v3 = Version::new(2, 0, 0);
        
        assert_eq!(v1.to_string(), "1.0.0");
        assert_eq!(v2.to_string(), "1.2.0");
        assert_eq!(v3.to_string(), "2.0.0");
    }
}