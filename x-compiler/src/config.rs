//! Compiler configuration and settings

use x_parser::SyntaxStyle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main compiler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub syntax_style: SyntaxStyle,
    pub optimization_level: u8,
    pub debug_info: bool,
    pub source_maps: bool,
    pub emit_types: bool,
    pub target_configs: HashMap<String, TargetConfig>,
    pub output_format: OutputFormat,
    pub incremental: bool,
    pub cache_dir: Option<PathBuf>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            syntax_style: SyntaxStyle::Haskell,
            optimization_level: 0,
            debug_info: false,
            source_maps: false,
            emit_types: false,
            target_configs: HashMap::new(),
            output_format: OutputFormat::Files,
            incremental: false,
            cache_dir: None,
        }
    }
}

/// Target-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub enabled: bool,
    pub options: HashMap<String, ConfigValue>,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            options: HashMap::new(),
        }
    }
}

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    String(String),
    Number(f64),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Bool(value)
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Number(value)
    }
}

impl From<i32> for ConfigValue {
    fn from(value: i32) -> Self {
        ConfigValue::Number(value as f64)
    }
}

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Generate separate files
    Files,
    /// Bundle into single file
    Bundle,
    /// In-memory only (for testing)
    Memory,
}

impl CompilerConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io { path: path.clone(), error: e })?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::Parse { path: path.clone(), error: e })
    }

    /// Save configuration to TOML file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Serialize { error: e })?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::Io { path: parent.to_path_buf(), error: e })?;
        }
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::Io { path: path.clone(), error: e })
    }

    /// Get target-specific configuration
    pub fn target_config(&self, target: &str) -> TargetConfig {
        self.target_configs.get(target).cloned().unwrap_or_default()
    }

    /// Set target-specific configuration
    pub fn set_target_config(&mut self, target: &str, config: TargetConfig) {
        self.target_configs.insert(target.to_string(), config);
    }

    /// Check if target is enabled
    pub fn is_target_enabled(&self, target: &str) -> bool {
        self.target_config(target).enabled
    }

    /// Get target option
    pub fn get_target_option(&self, target: &str, key: &str) -> Option<&ConfigValue> {
        self.target_configs.get(target)?.options.get(key)
    }

    /// Set target option
    pub fn set_target_option(&mut self, target: &str, key: &str, value: ConfigValue) {
        let mut config = self.target_config(target);
        config.options.insert(key.to_string(), value);
        self.set_target_config(target, config);
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check optimization level
        if self.optimization_level > 3 {
            return Err(ConfigError::Invalid {
                field: "optimization_level".to_string(),
                message: "Optimization level must be 0-3".to_string(),
            });
        }

        // Check cache directory if incremental is enabled
        if self.incremental && self.cache_dir.is_none() {
            return Err(ConfigError::Invalid {
                field: "cache_dir".to_string(),
                message: "Cache directory required for incremental compilation".to_string(),
            });
        }

        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge(&mut self, other: CompilerConfig) {
        if other.syntax_style != SyntaxStyle::default() {
            self.syntax_style = other.syntax_style;
        }
        if other.optimization_level != 0 {
            self.optimization_level = other.optimization_level;
        }
        if other.debug_info {
            self.debug_info = other.debug_info;
        }
        if other.source_maps {
            self.source_maps = other.source_maps;
        }
        if other.emit_types {
            self.emit_types = other.emit_types;
        }
        if other.incremental {
            self.incremental = other.incremental;
        }
        if other.cache_dir.is_some() {
            self.cache_dir = other.cache_dir;
        }

        // Merge target configs
        for (target, config) in other.target_configs {
            self.target_configs.insert(target, config);
        }
    }
}

impl TargetConfig {
    /// Get boolean option
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.options.get(key) {
            Some(ConfigValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    /// Get string option
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.options.get(key) {
            Some(ConfigValue::String(value)) => Some(value),
            _ => None,
        }
    }

    /// Get number option
    pub fn get_number(&self, key: &str) -> Option<f64> {
        match self.options.get(key) {
            Some(ConfigValue::Number(value)) => Some(*value),
            _ => None,
        }
    }

    /// Set boolean option
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.options.insert(key.to_string(), ConfigValue::Bool(value));
    }

    /// Set string option
    pub fn set_string(&mut self, key: &str, value: &str) {
        self.options.insert(key.to_string(), ConfigValue::String(value.to_string()));
    }

    /// Set number option
    pub fn set_number(&mut self, key: &str, value: f64) {
        self.options.insert(key.to_string(), ConfigValue::Number(value));
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error for {path:?}: {error}")]
    Io { path: PathBuf, error: std::io::Error },

    #[error("Parse error for {path:?}: {error}")]
    Parse { path: PathBuf, error: toml::de::Error },

    #[error("Serialization error: {error}")]
    Serialize { error: toml::ser::Error },

    #[error("Invalid configuration for {field}: {message}")]
    Invalid { field: String, message: String },
}

/// Predefined target configurations
pub mod presets {
    use super::*;

    /// TypeScript development configuration
    pub fn typescript_dev() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_string("module_system", "es2020");
        config.set_bool("emit_types", true);
        config.set_bool("strict", false);
        config
    }

    /// TypeScript production configuration
    pub fn typescript_prod() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_string("module_system", "es2020");
        config.set_bool("emit_types", true);
        config.set_bool("strict", true);
        config.set_bool("minify", true);
        config
    }

    /// WebAssembly development configuration
    pub fn wasm_dev() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_string("optimization", "none");
        config.set_bool("debug_info", true);
        config.set_string("gc_strategy", "conservative");
        config
    }

    /// WebAssembly production configuration
    pub fn wasm_prod() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_string("optimization", "aggressive");
        config.set_bool("debug_info", false);
        config.set_string("gc_strategy", "precise");
        config
    }

    /// WebAssembly Component Model configuration
    pub fn wasm_component() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_bool("with_wit", true);
        config.set_bool("generate_bindings", true);
        config.set_string("wit_package", "effect-lang");
        config
    }

    /// WIT generation configuration
    pub fn wit_only() -> TargetConfig {
        let mut config = TargetConfig::default();
        config.set_bool("validate", true);
        config.set_bool("generate_docs", true);
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = CompilerConfig::default();
        assert_eq!(config.syntax_style, SyntaxStyle::Haskell);
        assert_eq!(config.optimization_level, 0);
        assert!(!config.debug_info);
    }

    #[test]
    fn test_target_config() {
        let mut config = CompilerConfig::default();
        
        let ts_config = presets::typescript_dev();
        config.set_target_config("typescript", ts_config);
        
        assert!(config.is_target_enabled("typescript"));
        assert_eq!(config.get_target_option("typescript", "module_system"), 
                  Some(&ConfigValue::String("es2020".to_string())));
    }

    #[test]
    fn test_config_validation() {
        let mut config = CompilerConfig::default();
        config.optimization_level = 5; // Invalid
        
        assert!(config.validate().is_err());
        
        config.optimization_level = 2; // Valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_file_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = CompilerConfig::default();
        config.debug_info = true;
        config.set_target_config("typescript", presets::typescript_dev());
        
        // Save and load
        config.to_file(&config_path).unwrap();
        let loaded_config = CompilerConfig::from_file(&config_path).unwrap();
        
        assert_eq!(config.debug_info, loaded_config.debug_info);
        assert!(loaded_config.is_target_enabled("typescript"));
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = CompilerConfig::default();
        
        let mut override_config = CompilerConfig::default();
        override_config.debug_info = true;
        override_config.optimization_level = 2;
        
        base_config.merge(override_config);
        
        assert!(base_config.debug_info);
        assert_eq!(base_config.optimization_level, 2);
    }

    #[test]
    fn test_target_config_options() {
        let mut config = TargetConfig::default();
        
        config.set_bool("debug", true);
        config.set_string("format", "esm");
        config.set_number("version", 2.0);
        
        assert_eq!(config.get_bool("debug"), Some(true));
        assert_eq!(config.get_string("format"), Some("esm"));
        assert_eq!(config.get_number("version"), Some(2.0));
    }
}