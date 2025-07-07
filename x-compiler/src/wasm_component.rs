use crate::backend::*;
use crate::wit::WitGenerator;
use x_parser::{CompilationUnit, Module, Symbol, TypeDefKind, Type, Visibility, Item, TypeDef, ValueDef};
use x_checker::TypeScheme;
use crate::{CompilerError, Result};
use std::collections::HashMap;
use std::fmt::Write;

/// WebAssembly Component Model backend
pub struct WasmComponentBackend {
    wit_generator: WitGenerator,
}

impl Default for WasmComponentBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmComponentBackend {
    pub fn new() -> Self {
        Self {
            wit_generator: WitGenerator::new(),
        }
    }
}

impl CodegenBackend for WasmComponentBackend {
    fn target_info(&self) -> CompilationTarget {
        CompilationTarget {
            name: "wasm-component".to_string(),
            file_extension: "wasm".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        }
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, "components" | "interfaces" | "resources" | "imports" | "exports" | "gc" | "effects" | "wasm-types")
    }

    fn generate_code(
        &mut self,
        cu: &CompilationUnit,
        type_info: &HashMap<Symbol, TypeScheme>,
        options: &CodegenOptions,
    ) -> Result<CodegenResult> {
        let start_time = std::time::Instant::now();
        let mut files = HashMap::new();
        let mut diagnostics = Vec::new();

        // Generate WIT file
        match self.wit_generator.generate(cu) {
            Ok(wit_content) => {
                let mut wit_path = options.output_dir.clone();
                wit_path.push(format!("{}.wit", 
                    cu.module.name.segments.first()
                        .map(|s| s.as_str())
                        .unwrap_or("main")));
                
                files.insert(wit_path, wit_content);
            }
            Err(e) => {
                diagnostics.push(CodegenDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("Failed to generate WIT: {e}"),
                    location: None,
                });
            }
        }

        // Generate Rust source code for the component
        match self.generate_rust_component(cu, type_info) {
            Ok(rust_content) => {
                let mut rust_path = options.output_dir.clone();
                rust_path.push("src");
                rust_path.push("lib.rs");
                files.insert(rust_path, rust_content);
            }
            Err(e) => {
                diagnostics.push(CodegenDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("Failed to generate Rust component: {e}"),
                    location: None,
                });
            }
        }

        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml(cu)?;
        let mut cargo_path = options.output_dir.clone();
        cargo_path.push("Cargo.toml");
        files.insert(cargo_path, cargo_toml);

        // Generate build script
        let build_script = self.generate_build_script()?;
        let mut build_path = options.output_dir.clone();
        build_path.push("build.rs");
        files.insert(build_path, build_script);

        // Calculate total size
        let total_size = files.values().map(|content| content.len()).sum();
        let file_count = files.len();

        Ok(CodegenResult {
            files,
            source_maps: HashMap::new(),
            diagnostics,
            metadata: CodegenMetadata {
                target_info: self.target_info(),
                generated_files: file_count,
                total_size,
                compilation_time: start_time.elapsed(),
            },
        })
    }

    fn generate_module(
        &mut self,
        module: &Module,
        type_info: &HashMap<Symbol, TypeScheme>,
        _options: &CodegenOptions,
    ) -> Result<String> {
        let cu = CompilationUnit {
            module: module.clone(),
            span: module.span,
        };

        self.generate_rust_component(&cu, type_info)
    }

    fn generate_runtime(&self, _options: &CodegenOptions) -> Result<String> {
        Ok(r#"
// WebAssembly Component Model runtime support
use wit_bindgen::{generate, Resource};

// Effect system runtime
pub struct EffectRuntime {
    stack: Vec<Effect>,
}

#[derive(Debug, Clone)]
pub enum Effect {
    IO(IOEffect),
    State(StateEffect),
    Console(ConsoleEffect),
}

#[derive(Debug, Clone)]
pub enum IOEffect {
    Read(String),
    Write(String, String),
}

#[derive(Debug, Clone)]
pub enum StateEffect {
    Get,
    Put(String),
}

#[derive(Debug, Clone)]
pub enum ConsoleEffect {
    Print(String),
    Log(String),
}

impl EffectRuntime {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }

    pub fn handle_effect(&mut self, effect: Effect) -> Result<String, String> {
        match effect {
            Effect::IO(io_effect) => self.handle_io(io_effect),
            Effect::State(state_effect) => self.handle_state(state_effect),
            Effect::Console(console_effect) => self.handle_console(console_effect),
        }
    }

    fn handle_io(&mut self, effect: IOEffect) -> Result<String, String> {
        match effect {
            IOEffect::Read(path) => {
                // In a real implementation, this would read from WASI filesystem
                Ok(format!("Reading from {}", path))
            }
            IOEffect::Write(path, content) => {
                // In a real implementation, this would write to WASI filesystem
                Ok(format!("Writing to {}: {}", path, content))
            }
        }
    }

    fn handle_state(&mut self, effect: StateEffect) -> Result<String, String> {
        match effect {
            StateEffect::Get => {
                // Return current state
                Ok("current_state".to_string())
            }
            StateEffect::Put(new_state) => {
                // Update state
                Ok(format!("State updated to: {}", new_state))
            }
        }
    }

    fn handle_console(&mut self, effect: ConsoleEffect) -> Result<String, String> {
        match effect {
            ConsoleEffect::Print(message) => {
                // Print to console
                println!("{}", message);
                Ok("()".to_string())
            }
            ConsoleEffect::Log(message) => {
                // Log message
                eprintln!("{}", message);
                Ok("()".to_string())
            }
        }
    }
}

// Memory management for WebAssembly Component Model
pub struct ComponentMemory {
    heap: Vec<u8>,
    free_list: Vec<usize>,
}

impl ComponentMemory {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn allocate(&mut self, size: usize) -> usize {
        if let Some(ptr) = self.free_list.pop() {
            ptr
        } else {
            let ptr = self.heap.len();
            self.heap.resize(ptr + size, 0);
            ptr
        }
    }

    pub fn deallocate(&mut self, ptr: usize) {
        self.free_list.push(ptr);
    }
}
"#.to_string())
    }
}

impl WasmComponentBackend {
    fn generate_rust_component(&self, cu: &CompilationUnit, type_info: &HashMap<Symbol, TypeScheme>) -> Result<String> {
        let mut code = String::new();
        
        // Generate use statements
        writeln!(code, "use wit_bindgen::generate;").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "use std::collections::HashMap;").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        // Generate WIT bindings
        let package_name = cu.module.name.segments.first()
            .map(|s| s.as_str())
            .unwrap_or("effect-lang");
        
        writeln!(code, "generate!({{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    world: \"effect-lang\",").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    path: \"{package_name}.wit\",").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "}});").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        // Generate component struct
        writeln!(code, "struct Component {{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    runtime: EffectRuntime,").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    memory: ComponentMemory,").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "}}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        // Generate component implementation
        writeln!(code, "impl Component {{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    fn new() -> Self {{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        Self {{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "            runtime: EffectRuntime::new(),").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "            memory: ComponentMemory::new(),").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        }}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    }}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "}}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        // Generate module
        self.generate_rust_module(&mut code, &cu.module, type_info)?;

        // Generate export implementations
        writeln!(code, "impl exports::Guest for Component {{").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        // Generate exported functions
        for item in &cu.module.items {
            if let Item::ValueDef(value_def) = item {
                if self.is_exported(&value_def.visibility) {
                    self.generate_export_function(&mut code, value_def)?;
                }
            }
        }

        writeln!(code, "}}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        // Generate component export
        writeln!(code, "export!(Component);").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;

        Ok(code)
    }

    fn generate_rust_module(&self, code: &mut String, module: &Module, _type_info: &HashMap<Symbol, TypeScheme>) -> Result<()> {
        writeln!(code, "// Module: {}", module.name).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "mod {} {{", sanitize_rust_identifier(&module.name.to_string())).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        // Generate type definitions
        for item in &module.items {
            match item {
                Item::TypeDef(type_def) => {
                    self.generate_rust_type_def(code, type_def)?;
                }
                Item::ValueDef(value_def) => {
                    self.generate_rust_value_def(code, value_def)?;
                }
                _ => {} // Skip other items for now
            }
        }
        
        writeln!(code, "}}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        Ok(())
    }

    fn generate_rust_type_def(&self, code: &mut String, type_def: &TypeDef) -> Result<()> {
        writeln!(code, "    #[derive(Debug, Clone)]").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        match &type_def.kind {
            TypeDefKind::Data(constructors) => {
                writeln!(code, "    pub enum {} {{", type_def.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                for constructor in constructors {
                    if constructor.fields.is_empty() {
                        writeln!(code, "        {},", constructor.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                    } else if constructor.fields.len() == 1 {
                        writeln!(code, "        {}({}),", 
                            constructor.name.as_str(),
                            self.type_to_rust_type(&constructor.fields[0])
                        ).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                    } else {
                        writeln!(code, "        {}(", constructor.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                        for field_type in &constructor.fields {
                            writeln!(code, "            {},", self.type_to_rust_type(field_type))
                                .map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                        }
                        writeln!(code, "        ),").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
                    }
                }
                writeln!(code, "    }}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
            }
            TypeDefKind::Alias(alias_type) => {
                writeln!(code, "    pub type {} = {};", 
                    type_def.name.as_str(),
                    self.type_to_rust_type(alias_type)
                ).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
            }
            TypeDefKind::Abstract => {
                writeln!(code, "    // Abstract type: {}", type_def.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
            }
        }
        
        Ok(())
    }

    fn generate_rust_value_def(&self, code: &mut String, value_def: &ValueDef) -> Result<()> {
        let visibility = match &value_def.visibility {
            Visibility::Public => "pub",
            _ => "",
        };
        
        writeln!(code, "    {} fn {}() -> String {{", visibility, value_def.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        // TODO: Implement function body").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        \"not implemented\".to_string()").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    }}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        Ok(())
    }

    fn generate_export_function(&self, code: &mut String, value_def: &ValueDef) -> Result<()> {
        writeln!(code, "    fn {}(&mut self) -> String {{", value_def.name.as_str()).map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        // TODO: Implement exported function").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "        \"exported function result\".to_string()").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        writeln!(code, "    }}").map_err(|e| CompilerError::CodeGen { message: e.to_string() })?;
        
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn type_to_rust_type(&self, type_expr: &Type) -> String {
        match type_expr {
            Type::Var(name, _) => name.as_str().to_string(),
            Type::Con(name, _) => {
                // Map x-lang type constructors to Rust types
                match name.as_str() {
                    "Unit" => "()".to_string(),
                    "Bool" => "bool".to_string(),
                    "Int" => "i32".to_string(),
                    "Float" => "f64".to_string(),
                    "String" => "String".to_string(),
                    "Char" => "char".to_string(),
                    _ => name.as_str().to_string(),
                }
            }
            Type::App(con, args, _) => {
                if let Type::Con(name, _) = con.as_ref() {
                    match name.as_str() {
                        "List" if args.len() == 1 => {
                            format!("Vec<{}>", self.type_to_rust_type(&args[0]))
                        }
                        "Option" if args.len() == 1 => {
                            format!("Option<{}>", self.type_to_rust_type(&args[0]))
                        }
                        "Result" if args.len() == 2 => {
                            format!("Result<{}, {}>", self.type_to_rust_type(&args[0]), self.type_to_rust_type(&args[1]))
                        }
                        _ => name.as_str().to_string(),
                    }
                } else {
                    "String".to_string()
                }
            }
            Type::Fun { .. } => "Box<dyn Fn>".to_string(), // Simplified function type
            _ => "String".to_string(), // Default fallback
        }
    }

    fn is_exported(&self, visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public | Visibility::Component { export: true, .. })
    }

    fn generate_cargo_toml(&self, cu: &CompilationUnit) -> Result<String> {
        let package_name = cu.module.name.segments.first()
            .map(|s| s.as_str())
            .unwrap_or("effect-lang-component");

        let cargo_toml = format!(r#"[package]
name = "{package_name}"
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
package = "{package_name}"
target = {{ path = "{package_name}.wit" }}

[package.metadata.component.dependencies]
"#);

        Ok(cargo_toml)
    }

    fn generate_build_script(&self) -> Result<String> {
        let build_script = r#"fn main() {
    // Build script for WebAssembly Component Model
    println!("cargo:rerun-if-changed=*.wit");
    println!("cargo:rerun-if-changed=build.rs");
    
    // Add any custom build logic here
}
"#;
        Ok(build_script.to_string())
    }
}

fn sanitize_rust_identifier(name: &str) -> String {
    // Handle Rust reserved keywords and naming conventions
    match name {
        "type" | "impl" | "trait" | "struct" | "enum" | "fn" | "let" | "mut" | "ref" | "match" => {
            format!("{name}_")
        }
        _ => name.replace("-", "_").replace("?", "_q").replace("!", "_e"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::Module;

    #[test]
    fn test_wasm_component_backend_creation() {
        let backend = WasmComponentBackend::new();
        let target_info = backend.target_info();
        
        assert_eq!(target_info.name, "wasm-component");
        assert_eq!(target_info.file_extension, "wasm");
        assert!(target_info.supports_modules);
        assert!(target_info.supports_effects);
        assert!(target_info.supports_gc);
    }

    #[test]
    fn test_rust_identifier_sanitization() {
        assert_eq!(sanitize_rust_identifier("type"), "type_");
        assert_eq!(sanitize_rust_identifier("my-function"), "my_function");
        assert_eq!(sanitize_rust_identifier("valid_name"), "valid_name");
    }

    #[test]
    fn test_type_conversion() {
        let backend = WasmComponentBackend::new();
        
        assert_eq!(backend.type_to_rust_type(&Type::Int), "i32");
        assert_eq!(backend.type_to_rust_type(&Type::String), "String");
        assert_eq!(backend.type_to_rust_type(&Type::Bool), "bool");
        assert_eq!(backend.type_to_rust_type(&Type::List(Box::new(Type::Int))), "Vec<i32>");
    }
}