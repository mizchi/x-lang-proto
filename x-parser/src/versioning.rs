//! Version management and compatibility tracking for functions
//! 
//! This module provides version management for content-addressed functions,
//! allowing human-friendly references while maintaining type safety.

use crate::symbol::Symbol;
use crate::metadata::ContentHash;
use crate::ast::Type;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A versioned reference to a function
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionedRef {
    /// Human-readable name
    pub name: Symbol,
    /// Version specification
    pub version: VersionSpec,
    /// Content hash (optional for flexible references)
    pub hash: Option<ContentHash>,
}

/// Version specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionSpec {
    /// Exact version
    Exact(Version),
    /// Any compatible version (same major)
    Compatible(Version),
    /// Version range
    Range {
        min: Option<Version>,
        max: Option<Version>,
    },
    /// Latest version
    Latest,
    /// Specific content hash
    Hash(ContentHash),
}

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// Optional pre-release identifier
    pub pre_release: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Check if this version is compatible with another
    /// (same major version, newer or equal minor/patch)
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major && 
        (self.minor > other.minor || 
         (self.minor == other.minor && self.patch >= other.patch))
    }
}

/// Function signature for compatibility checking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type
    pub return_type: Type,
    /// Effects
    pub effects: Vec<Symbol>,
}

/// Version metadata for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// The version
    pub version: Version,
    /// Content hash
    pub hash: ContentHash,
    /// Function signature
    pub signature: FunctionSignature,
    /// Dependencies with their version requirements
    pub dependencies: HashMap<Symbol, VersionSpec>,
    /// Release notes
    pub release_notes: Option<String>,
    /// Deprecation info
    pub deprecation: Option<DeprecationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    pub since: Version,
    pub reason: String,
    pub alternative: Option<Symbol>,
}

/// Reference set - tracks all versions of a function
#[derive(Debug, Clone)]
pub struct ReferenceSet {
    /// Function name
    pub name: Symbol,
    /// All versions, sorted by version number
    pub versions: Vec<VersionMetadata>,
    /// Latest stable version
    pub latest: Option<Version>,
    /// Version aliases (e.g., "stable", "beta")
    pub aliases: HashMap<String, Version>,
}

impl ReferenceSet {
    pub fn new(name: Symbol) -> Self {
        Self {
            name,
            versions: Vec::new(),
            latest: None,
            aliases: HashMap::new(),
        }
    }

    /// Add a new version
    pub fn add_version(&mut self, metadata: VersionMetadata) {
        // Insert in sorted order
        let pos = self.versions.binary_search_by(|v| v.version.cmp(&metadata.version))
            .unwrap_or_else(|e| e);
        self.versions.insert(pos, metadata.clone());
        
        // Update latest if this is newer
        if self.latest.as_ref().is_none_or(|l| metadata.version > *l) {
            self.latest = Some(metadata.version);
        }
    }

    /// Find compatible versions for a given spec
    pub fn find_compatible(&self, spec: &VersionSpec) -> Vec<&VersionMetadata> {
        match spec {
            VersionSpec::Exact(v) => {
                self.versions.iter()
                    .find(|m| &m.version == v)
                    .into_iter()
                    .collect()
            }
            VersionSpec::Compatible(v) => {
                self.versions.iter()
                    .filter(|m| m.version.is_compatible_with(v))
                    .collect()
            }
            VersionSpec::Range { min, max } => {
                self.versions.iter()
                    .filter(|m| {
                        let above_min = min.as_ref().is_none_or(|min| m.version >= *min);
                        let below_max = max.as_ref().is_none_or(|max| m.version <= *max);
                        above_min && below_max
                    })
                    .collect()
            }
            VersionSpec::Latest => {
                self.latest.as_ref()
                    .and_then(|l| self.versions.iter().find(|m| &m.version == l))
                    .into_iter()
                    .collect()
            }
            VersionSpec::Hash(h) => {
                self.versions.iter()
                    .find(|m| &m.hash == h)
                    .into_iter()
                    .collect()
            }
        }
    }

    /// Check if two versions are signature-compatible
    pub fn are_compatible(&self, v1: &Version, v2: &Version) -> Option<CompatibilityStatus> {
        let meta1 = self.versions.iter().find(|m| &m.version == v1)?;
        let meta2 = self.versions.iter().find(|m| &m.version == v2)?;
        
        Some(check_signature_compatibility(&meta1.signature, &meta2.signature))
    }
}

/// Compatibility status between two function signatures
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityStatus {
    /// Fully compatible - can be used interchangeably
    Compatible,
    /// Forward compatible - newer version can replace older
    ForwardCompatible {
        changes: Vec<CompatibilityChange>,
    },
    /// Incompatible - breaking changes
    Incompatible {
        breaking_changes: Vec<BreakingChange>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityChange {
    /// New optional parameter added
    OptionalParamAdded { position: usize, param_type: Type },
    /// Effect removed (making function more pure)
    EffectRemoved { effect: Symbol },
    /// Return type widened (covariant)
    ReturnTypeWidened { from: Type, to: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreakingChange {
    /// Parameter count changed
    ParamCountChanged { from: usize, to: usize },
    /// Parameter type changed (contravariant)
    ParamTypeChanged { position: usize, from: Type, to: Type },
    /// Return type narrowed (contravariant) 
    ReturnTypeNarrowed { from: Type, to: Type },
    /// New effect added
    EffectAdded { effect: Symbol },
    /// Required parameter added
    RequiredParamAdded { position: usize, param_type: Type },
}

/// Check signature compatibility between two functions
pub fn check_signature_compatibility(
    old: &FunctionSignature,
    new: &FunctionSignature,
) -> CompatibilityStatus {
    let mut changes = Vec::new();
    let mut breaking_changes = Vec::new();

    // Check parameter compatibility
    if old.params.len() != new.params.len() {
        // Could be compatible if new params are optional
        if new.params.len() > old.params.len() {
            // For now, treat as breaking
            breaking_changes.push(BreakingChange::ParamCountChanged {
                from: old.params.len(),
                to: new.params.len(),
            });
        } else {
            breaking_changes.push(BreakingChange::ParamCountChanged {
                from: old.params.len(),
                to: new.params.len(),
            });
        }
    } else {
        // Check each parameter type
        for (i, (old_param, new_param)) in old.params.iter().zip(&new.params).enumerate() {
            if !are_types_compatible(old_param, new_param, Variance::Contravariant) {
                breaking_changes.push(BreakingChange::ParamTypeChanged {
                    position: i,
                    from: old_param.clone(),
                    to: new_param.clone(),
                });
            }
        }
    }

    // Check return type compatibility (covariant)
    if !are_types_compatible(&old.return_type, &new.return_type, Variance::Covariant) {
        breaking_changes.push(BreakingChange::ReturnTypeNarrowed {
            from: old.return_type.clone(),
            to: new.return_type.clone(),
        });
    } else if old.return_type != new.return_type {
        changes.push(CompatibilityChange::ReturnTypeWidened {
            from: old.return_type.clone(),
            to: new.return_type.clone(),
        });
    }

    // Check effects
    let old_effects: HashSet<_> = old.effects.iter().collect();
    let new_effects: HashSet<_> = new.effects.iter().collect();

    // New effects are breaking changes
    for effect in new_effects.difference(&old_effects) {
        breaking_changes.push(BreakingChange::EffectAdded {
            effect: **effect,
        });
    }

    // Removed effects are non-breaking improvements
    for effect in old_effects.difference(&new_effects) {
        changes.push(CompatibilityChange::EffectRemoved {
            effect: **effect,
        });
    }

    if !breaking_changes.is_empty() {
        CompatibilityStatus::Incompatible { breaking_changes }
    } else if !changes.is_empty() {
        CompatibilityStatus::ForwardCompatible { changes }
    } else {
        CompatibilityStatus::Compatible
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Variance {
    Covariant,
    Contravariant,
}

/// Check if two types are compatible given variance
fn are_types_compatible(old: &Type, new: &Type, _variance: Variance) -> bool {
    // TODO: Implement proper subtyping rules
    // For now, just check exact equality
    old == new
}

/// Version repository for managing all function versions
#[derive(Debug, Clone)]
pub struct VersionRepository {
    /// All reference sets by name
    pub references: HashMap<Symbol, ReferenceSet>,
    /// Reverse index: hash to (name, version)
    pub hash_index: HashMap<ContentHash, (Symbol, Version)>,
}

impl Default for VersionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionRepository {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
            hash_index: HashMap::new(),
        }
    }

    /// Register a new version of a function
    pub fn register_version(
        &mut self,
        name: Symbol,
        metadata: VersionMetadata,
    ) {
        let hash = metadata.hash.clone();
        let version = metadata.version.clone();

        self.references
            .entry(name)
            .or_insert_with(|| ReferenceSet::new(name))
            .add_version(metadata);

        self.hash_index.insert(hash, (name, version));
    }

    /// Resolve a versioned reference to a specific hash
    pub fn resolve(&self, vref: &VersionedRef) -> Option<ContentHash> {
        if let Some(hash) = &vref.hash {
            return Some(hash.clone());
        }

        let refset = self.references.get(&vref.name)?;
        let compatible = refset.find_compatible(&vref.version);
        
        // Return the latest compatible version
        compatible.last().map(|m| m.hash.clone())
    }

    /// Find all functions that depend on a specific version
    pub fn find_dependents(&self, name: Symbol, version: &Version) -> Vec<(Symbol, Version)> {
        let mut dependents = Vec::new();

        for (dep_name, refset) in &self.references {
            for dep_meta in &refset.versions {
                if dep_meta.dependencies.contains_key(&name) {
                    // Check if this version would be affected
                    if let Some(spec) = dep_meta.dependencies.get(&name) {
                        let compatible = refset.find_compatible(spec);
                        if compatible.iter().any(|m| &m.version == version) {
                            dependents.push((*dep_name, dep_meta.version.clone()));
                        }
                    }
                }
            }
        }

        dependents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v2.is_compatible_with(&v1));
        assert!(!v3.is_compatible_with(&v1));
    }
}