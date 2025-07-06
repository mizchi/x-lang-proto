//! Configuration management

use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default syntax style
    #[serde(default = "default_syntax_style")]
    pub default_syntax: String,
    
    /// Default output format
    #[serde(default = "default_output_format")]
    pub default_format: String,
    
    /// Editor configuration
    #[serde(default)]
    pub editor: EditorConfig,
    
    /// Compiler configuration
    #[serde(default)]
    pub compiler: CompilerConfig,
    
    /// LSP configuration
    #[serde(default)]
    pub lsp: LspConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Show type information by default
    #[serde(default = "default_true")]
    pub show_types: bool,
    
    /// Show spans by default
    #[serde(default)]
    pub show_spans: bool,
    
    /// Maximum tree display depth
    #[serde(default = "default_max_depth")]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Default target language
    #[serde(default = "default_target")]
    pub default_target: String,
    
    /// Optimization level
    #[serde(default = "default_optimization")]
    pub optimization_level: u8,
    
    /// Generate source maps
    #[serde(default = "default_true")]
    pub source_maps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    /// LSP server port
    #[serde(default = "default_lsp_port")]
    pub port: u16,
    
    /// Enable LSP features
    #[serde(default = "default_lsp_features")]
    pub features: Vec<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_syntax: default_syntax_style(),
            default_format: default_output_format(),
            editor: EditorConfig::default(),
            compiler: CompilerConfig::default(),
            lsp: LspConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            show_types: default_true(),
            show_spans: false,
            max_depth: default_max_depth(),
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            default_target: default_target(),
            optimization_level: default_optimization(),
            source_maps: default_true(),
        }
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            port: default_lsp_port(),
            features: default_lsp_features(),
        }
    }
}

impl CliConfig {
    /// Load configuration from file
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let config_path = match config_path {
            Some(path) => path.to_owned(),
            None => Self::default_config_path()?,
        };
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        
        let config: CliConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    #[allow(dead_code)]
    pub fn save(&self, config_path: Option<&Path>) -> Result<()> {
        let config_path = match config_path {
            Some(path) => path.to_owned(),
            None => Self::default_config_path()?,
        };
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
        Ok(())
    }
    
    /// Get default configuration file path
    fn default_config_path() -> Result<std::path::PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Cannot determine config directory")?;
        
        Ok(config_dir.join("x-lang").join("config.toml"))
    }
}

// Default value functions
fn default_syntax_style() -> String {
    "haskell".to_string()
}

fn default_output_format() -> String {
    "auto".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_depth() -> Option<usize> {
    Some(10)
}

fn default_target() -> String {
    "typescript".to_string()
}

fn default_optimization() -> u8 {
    2
}

fn default_lsp_port() -> u16 {
    9257
}

fn default_lsp_features() -> Vec<String> {
    vec![
        "completions".to_string(),
        "hover".to_string(),
        "goto-definition".to_string(),
        "find-references".to_string(),
        "rename".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_config_serialization() {
        let config = CliConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CliConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.default_syntax, parsed.default_syntax);
        assert_eq!(config.default_format, parsed.default_format);
    }
    
    #[test]
    fn test_config_load_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config = CliConfig::default();
        config.save(Some(&config_path)).unwrap();
        
        let loaded = CliConfig::load(Some(&config_path)).unwrap();
        assert_eq!(config.default_syntax, loaded.default_syntax);
    }
}