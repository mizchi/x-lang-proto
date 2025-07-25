use super::backend::*;
use super::wit::WitGenerator;
use crate::core::ast::CompilationUnit;
use crate::core::symbol::Symbol;
use crate::analysis::types::TypeScheme;
use crate::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// WIT (WebAssembly Interface Types) backend
pub struct WitBackend {
    generator: WitGenerator,
}

impl WitBackend {
    pub fn new() -> Self {
        Self {
            generator: WitGenerator::new(),
        }
    }
}

impl CodegenBackend for WitBackend {
    fn target_info(&self) -> CompilationTarget {
        CompilationTarget {
            name: "wit".to_string(),
            file_extension: "wit".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: false,
        }
    }

    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "interfaces" => true,
            "resources" => true,
            "components" => true,
            "imports" => true,
            "exports" => true,
            "wasm-types" => true,
            "gc" => false,
            "effects" => false, // WIT doesn't directly support effects
            _ => false,
        }
    }

    fn generate_code(
        &mut self,
        cu: &CompilationUnit,
        _type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<CodegenResult> {
        let start_time = std::time::Instant::now();
        let mut files = HashMap::new();
        let mut diagnostics = Vec::new();

        // Generate WIT file
        match self.generator.generate(cu) {
            Ok(wit_content) => {
                let mut output_path = options.output_dir.clone();
                output_path.push(format!("{}.wit", 
                    cu.package_name.as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("main")));
                
                files.insert(output_path, wit_content);
            }
            Err(e) => {
                diagnostics.push(CodegenDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("Failed to generate WIT: {}", e),
                    location: None,
                });
            }
        }

        // Generate cargo.toml for component
        let cargo_toml = self.generate_cargo_toml(cu)?;
        let mut cargo_path = options.output_dir.clone();
        cargo_path.push("Cargo.toml");
        files.insert(cargo_path, cargo_toml);

        // Calculate total size
        let total_size = files.values().map(|content| content.len()).sum();

        Ok(CodegenResult {
            files,
            source_maps: HashMap::new(),
            diagnostics,
            metadata: CodegenMetadata {
                target_info: self.target_info(),
                generated_files: files.len(),
                total_size,
                compilation_time: start_time.elapsed(),
            },
        })
    }

    fn generate_module(
        &mut self,
        module: &crate::core::ast::Module,
        _type_info: &HashMap<Symbol, TypeScheme>,
        _options: &CodegenOptions,
    ) -> Result<String> {
        // Create a minimal compilation unit for this module
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("module")),
            modules: vec![module.clone()],
            imports: vec![],
            exports: vec![],
        };

        self.generator.generate(&cu)
            .map_err(|e| Error::Type { message: e })
    }

    fn generate_runtime(&self, _options: &CodegenOptions) -> Result<String> {
        // WIT doesn't need runtime support
        Ok(String::new())
    }

    fn validate_output(&self, code: &str) -> Vec<CodegenDiagnostic> {
        let mut diagnostics = Vec::new();

        // Basic WIT validation
        if !code.contains("world") && !code.contains("interface") {
            diagnostics.push(CodegenDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message: "WIT file doesn't contain world or interface definitions".to_string(),
                location: None,
            });
        }

        // Check for common WIT syntax issues
        if code.contains("record {") && !code.contains("}") {
            diagnostics.push(CodegenDiagnostic {
                severity: DiagnosticSeverity::Error,
                message: "Unclosed record definition".to_string(),
                location: None,
            });
        }

        if code.contains("interface ") && !code.contains("func") {
            diagnostics.push(CodegenDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message: "Interface contains no function definitions".to_string(),
                location: None,
            });
        }

        diagnostics
    }
}

impl WitBackend {
    fn generate_cargo_toml(&self, cu: &CompilationUnit) -> Result<String> {
        let package_name = cu.package_name.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("effect-lang-component");

        let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = {{ version = "0.18", features = ["macros"] }}
wasm-bindgen = "0.2"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]

[package.metadata.component]
package = "{}"

[package.metadata.component.dependencies]
"#, package_name, package_name);

        Ok(cargo_toml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::Module;
    use crate::core::symbol::Symbol;

    #[test]
    fn test_wit_backend_creation() {
        let backend = WitBackend::new();
        let target_info = backend.target_info();
        
        assert_eq!(target_info.name, "wit");
        assert_eq!(target_info.file_extension, "wit");
        assert!(target_info.supports_modules);
        assert!(target_info.supports_effects);
        assert!(!target_info.supports_gc);
    }

    #[test]
    fn test_feature_support() {
        let backend = WitBackend::new();
        
        assert!(backend.supports_feature("interfaces"));
        assert!(backend.supports_feature("resources"));
        assert!(backend.supports_feature("components"));
        assert!(!backend.supports_feature("gc"));
        assert!(!backend.supports_feature("effects"));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let backend = WitBackend::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test-package")),
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };

        let cargo_toml = backend.generate_cargo_toml(&cu).unwrap();
        
        assert!(cargo_toml.contains("name = \"test-package\""));
        assert!(cargo_toml.contains("wit-bindgen"));
        assert!(cargo_toml.contains("wasm-bindgen"));
        assert!(cargo_toml.contains("[package.metadata.component]"));
    }
}