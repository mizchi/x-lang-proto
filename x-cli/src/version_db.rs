//! Simple file-based version database
//! 
//! Stores version metadata in a `.x-versions` directory

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use x_parser::metadata::ContentHash;
use x_parser::symbol::Symbol;
use x_parser::versioning::{Version, VersionMetadata, FunctionSignature};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDatabase {
    pub functions: HashMap<String, FunctionVersions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionVersions {
    pub name: String,
    pub versions: Vec<StoredVersion>,
    pub latest: Option<Version>,
    pub stable: Option<Version>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredVersion {
    pub version: Version,
    pub hash: ContentHash,
    pub signature: StoredSignature,
    pub dependencies: HashMap<String, String>, // name -> version spec
    pub release_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSignature {
    pub param_count: usize,
    pub return_type: String,
    pub effects: Vec<String>,
}

impl VersionDatabase {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::new())
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn load_default() -> Result<Self> {
        let path = Path::new(".x-versions/versions.json");
        Self::load(path)
    }

    fn add_version(
        &mut self,
        name: &str,
        version: Version,
        hash: ContentHash,
        signature: &FunctionSignature,
        notes: Option<String>,
    ) {
        let stored_sig = StoredSignature {
            param_count: signature.params.len(),
            return_type: format!("{:?}", signature.return_type), // Simple format
            effects: signature.effects.iter().map(|s| s.as_str().to_string()).collect(),
        };

        let stored_version = StoredVersion {
            version: version.clone(),
            hash,
            signature: stored_sig,
            dependencies: HashMap::new(), // TODO: Extract from function
            release_notes: notes,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let func_versions = self.functions.entry(name.to_string())
            .or_insert_with(|| FunctionVersions {
                name: name.to_string(),
                versions: Vec::new(),
                latest: None,
                stable: None,
            });

        // Insert in sorted order
        let pos = func_versions.versions
            .binary_search_by(|v| v.version.cmp(&version))
            .unwrap_or_else(|e| e);
        func_versions.versions.insert(pos, stored_version);

        // Update latest
        if func_versions.latest.as_ref().map_or(true, |l| version > *l) {
            func_versions.latest = Some(version.clone());
        }

        // Update stable if this is a stable version (no pre-release)
        if version.pre_release.is_none() {
            if func_versions.stable.as_ref().map_or(true, |s| version > *s) {
                func_versions.stable = Some(version);
            }
        }
    }

    fn get_versions(&self, name: &str) -> Option<&FunctionVersions> {
        self.functions.get(name)
    }
}

/// Get the version database path for a project
pub fn get_db_path(project_root: &Path) -> PathBuf {
    project_root.join(".x-versions").join("versions.json")
}

/// Load the version database for a project
pub fn load_db(project_root: &Path) -> Result<VersionDatabase> {
    let path = get_db_path(project_root);
    VersionDatabase::load(&path)
}

/// Save a version to the database
pub fn save_version(
    project_root: &Path,
    name: &str,
    version: Version,
    hash: ContentHash,
    signature: &FunctionSignature,
    notes: Option<String>,
) -> Result<()> {
    let path = get_db_path(project_root);
    let mut db = VersionDatabase::load(&path)?;
    db.add_version(name, version, hash, signature, notes);
    db.save(&path)?;
    Ok(())
}

/// Get all versions for a function
pub fn get_function_versions(project_root: &Path, name: &str) -> Result<Option<FunctionVersions>> {
    let db = load_db(project_root)?;
    Ok(db.get_versions(name).cloned())
}