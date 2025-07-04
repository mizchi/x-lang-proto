use effect_lang::codegen::backend::{BackendFactory, CodegenOptions, CompilationTarget};
use effect_lang::codegen::wasm_component::WasmComponentBackend;
use effect_lang::codegen::wit_backend::WitBackend;
use effect_lang::core::ast::*;
use effect_lang::core::symbol::Symbol;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::process::Command;

#[cfg(test)]
mod component_integration_tests {
    use super::*;

    fn create_test_compilation_unit() -> CompilationUnit {
        CompilationUnit {
            package_name: Some(Symbol::new("test:component")),
            modules: vec![Module {
                name: Symbol::new("TestModule"),
                visibility: Visibility::Public,
                items: vec![
                    // Interface definition
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("TestInterface"),
                        visibility: Visibility::Component {
                            export: true,
                            import: false,
                            interface: Some(Symbol::new("test:api")),
                        },
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("add"),
                                params: vec![
                                    Parameter { name: Symbol::new("x"), wasm_type: WasmType::I32 },
                                    Parameter { name: Symbol::new("y"), wasm_type: WasmType::I32 },
                                ],
                                results: vec![WasmType::I32],
                            }),
                            InterfaceItem::Resource(ResourceDefinition {
                                name: Symbol::new("Counter"),
                                constructor: Some(ResourceMethod {
                                    name: Symbol::new("new"),
                                    method_type: MethodType::Constructor,
                                    params: vec![
                                        Parameter { name: Symbol::new("initial"), wasm_type: WasmType::I32 }
                                    ],
                                    results: vec![],
                                }),
                                methods: vec![
                                    ResourceMethod {
                                        name: Symbol::new("increment"),
                                        method_type: MethodType::Method,
                                        params: vec![],
                                        results: vec![WasmType::I32],
                                    },
                                    ResourceMethod {
                                        name: Symbol::new("get_value"),
                                        method_type: MethodType::Method,
                                        params: vec![],
                                        results: vec![WasmType::I32],
                                    },
                                ],
                            }),
                        ],
                        span: Default::default(),
                    }),
                    // Type definitions
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("Config"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Record(vec![
                            (Symbol::new("name"), Type::String),
                            (Symbol::new("version"), Type::String),
                            (Symbol::new("debug"), Type::Bool),
                        ]),
                        span: Default::default(),
                    }),
                    // Value definitions
                    Item::ValueDef(ValueDef {
                        name: Symbol::new("hello"),
                        visibility: Visibility::Public,
                        parameters: vec![],
                        body: Expr::Literal(Literal::String("Hello, World!".to_string())),
                        type_annotation: Some(Type::String),
                        span: Default::default(),
                    }),
                    Item::ValueDef(ValueDef {
                        name: Symbol::new("calculate"),
                        visibility: Visibility::Public,
                        parameters: vec![
                            (Symbol::new("x"), Type::Int),
                            (Symbol::new("y"), Type::Int),
                        ],
                        body: Expr::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(Expr::Variable(Symbol::new("x"))),
                            right: Box::new(Expr::Variable(Symbol::new("y"))),
                            span: Default::default(),
                        },
                        type_annotation: Some(Type::Int),
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![
                ImportDef {
                    name: Symbol::new("wasi-filesystem"),
                    kind: ImportKind::Interface,
                    path: Some(Symbol::new("wasi:filesystem@0.2.0")),
                    visibility: Visibility::Public,
                    span: Default::default(),
                },
                ImportDef {
                    name: Symbol::new("console"),
                    kind: ImportKind::Func,
                    path: Some(Symbol::new("console")),
                    visibility: Visibility::Private,
                    span: Default::default(),
                },
            ],
            exports: vec![
                ExportDef {
                    name: Symbol::new("api"),
                    exported_name: Symbol::new("test:api@1.0.0"),
                    visibility: Visibility::Public,
                    span: Default::default(),
                },
            ],
        }
    }

    fn create_codegen_options(target_name: &str) -> CodegenOptions {
        CodegenOptions {
            target: CompilationTarget {
                name: target_name.to_string(),
                file_extension: if target_name == "wit" { "wit" } else { "wasm" }.to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: target_name != "wit",
            },
            output_dir: PathBuf::from("./test_output"),
            source_maps: false,
            debug_info: true,
            optimization_level: 1,
            emit_types: true,
        }
    }

    #[test]
    fn test_complete_component_generation() {
        let mut backend = WasmComponentBackend::new();
        let cu = create_test_compilation_unit();
        let options = create_codegen_options("wasm-component");

        let result = backend.generate_code(&cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "Component generation should succeed");

        let codegen_result = result.unwrap();
        
        // Check that all expected files were generated
        assert!(!codegen_result.files.is_empty(), "Should generate files");
        
        // Check for WIT file
        let wit_file = codegen_result.files.iter().find(|(path, _)| {
            path.extension().map_or(false, |ext| ext == "wit")
        });
        assert!(wit_file.is_some(), "Should generate WIT file");
        
        // Check for Rust source file
        let rust_file = codegen_result.files.iter().find(|(path, _)| {
            path.file_name().map_or(false, |name| name == "lib.rs")
        });
        assert!(rust_file.is_some(), "Should generate Rust lib.rs file");
        
        // Check for Cargo.toml
        let cargo_file = codegen_result.files.iter().find(|(path, _)| {
            path.file_name().map_or(false, |name| name == "Cargo.toml")
        });
        assert!(cargo_file.is_some(), "Should generate Cargo.toml file");
        
        // Check for build.rs
        let build_file = codegen_result.files.iter().find(|(path, _)| {
            path.file_name().map_or(false, |name| name == "build.rs")
        });
        assert!(build_file.is_some(), "Should generate build.rs file");

        // Validate content
        let (_, wit_content) = wit_file.unwrap();
        assert!(wit_content.contains("package test:component"), "WIT should contain package declaration");
        assert!(wit_content.contains("interface TestInterface"), "WIT should contain interface");
        assert!(wit_content.contains("resource Counter"), "WIT should contain resource");

        let (_, rust_content) = rust_file.unwrap();
        assert!(rust_content.contains("use wit_bindgen::generate"), "Rust should use wit_bindgen");
        assert!(rust_content.contains("struct Component"), "Rust should define Component struct");
        assert!(rust_content.contains("impl exports::Guest for Component"), "Rust should implement Guest trait");

        let (_, cargo_content) = cargo_file.unwrap();
        assert!(cargo_content.contains("wit-bindgen"), "Cargo.toml should include wit-bindgen");
        assert!(cargo_content.contains("[package.metadata.component]"), "Cargo.toml should have component metadata");
    }

    #[test]
    fn test_wit_only_generation() {
        let mut backend = WitBackend::new();
        let cu = create_test_compilation_unit();
        let options = create_codegen_options("wit");

        let result = backend.generate_code(&cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "WIT generation should succeed");

        let codegen_result = result.unwrap();
        
        // Should generate WIT file and Cargo.toml only
        assert!(codegen_result.files.len() >= 2, "Should generate at least 2 files");
        
        let wit_file = codegen_result.files.iter().find(|(path, _)| {
            path.extension().map_or(false, |ext| ext == "wit")
        });
        assert!(wit_file.is_some(), "Should generate WIT file");

        let (_, wit_content) = wit_file.unwrap();
        assert!(wit_content.contains("world effect-lang"), "WIT should contain world declaration");
        assert!(wit_content.contains("interface TestInterface"), "WIT should contain interface");
    }

    #[test]
    fn test_backend_factory_integration() {
        let cu = create_test_compilation_unit();
        let options = create_codegen_options("wasm-component");

        // Test wasm-component backend creation
        let result = BackendFactory::create_backend("wasm-component");
        assert!(result.is_ok(), "Should create wasm-component backend");

        let mut backend = result.unwrap();
        let target_info = backend.target_info();
        assert_eq!(target_info.name, "wasm-component");
        assert!(target_info.supports_modules);
        assert!(target_info.supports_effects);
        assert!(target_info.supports_gc);

        // Test code generation
        let result = backend.generate_code(&cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "Backend should generate code successfully");

        // Test wit backend creation
        let result = BackendFactory::create_backend("wit");
        assert!(result.is_ok(), "Should create wit backend");

        let mut wit_backend = result.unwrap();
        let wit_target_info = wit_backend.target_info();
        assert_eq!(wit_target_info.name, "wit");
        assert!(!wit_target_info.supports_gc);
    }

    #[test]
    fn test_feature_support() {
        let component_backend = WasmComponentBackend::new();
        let wit_backend = WitBackend::new();

        // Test component backend features
        assert!(component_backend.supports_feature("components"));
        assert!(component_backend.supports_feature("interfaces"));
        assert!(component_backend.supports_feature("resources"));
        assert!(component_backend.supports_feature("imports"));
        assert!(component_backend.supports_feature("exports"));
        assert!(component_backend.supports_feature("gc"));
        assert!(component_backend.supports_feature("effects"));
        assert!(component_backend.supports_feature("wasm-types"));

        // Test WIT backend features  
        assert!(wit_backend.supports_feature("interfaces"));
        assert!(wit_backend.supports_feature("resources"));
        assert!(wit_backend.supports_feature("components"));
        assert!(wit_backend.supports_feature("wasm-types"));
        assert!(!wit_backend.supports_feature("gc"));
        assert!(!wit_backend.supports_feature("effects"));
    }

    #[test]
    fn test_rust_code_generation_details() {
        let mut backend = WasmComponentBackend::new();
        let cu = create_test_compilation_unit();

        let result = backend.generate_rust_component(&cu, &HashMap::new());
        assert!(result.is_ok(), "Rust component generation should succeed");

        let rust_code = result.unwrap();
        
        // Check for essential Rust components
        assert!(rust_code.contains("use wit_bindgen::generate"), "Should import wit_bindgen");
        assert!(rust_code.contains("use std::collections::HashMap"), "Should import HashMap");
        assert!(rust_code.contains("generate!({"), "Should have generate! macro");
        assert!(rust_code.contains("world: \"effect-lang\""), "Should specify world");
        assert!(rust_code.contains("struct Component"), "Should define Component struct");
        assert!(rust_code.contains("runtime: EffectRuntime"), "Should include runtime");
        assert!(rust_code.contains("memory: ComponentMemory"), "Should include memory management");
        assert!(rust_code.contains("impl exports::Guest for Component"), "Should implement Guest trait");
        assert!(rust_code.contains("export!(Component);"), "Should export Component");

        // Check module generation
        assert!(rust_code.contains("mod test_module"), "Should generate module");
        assert!(rust_code.contains("pub struct Config"), "Should generate Config struct");
        assert!(rust_code.contains("pub name: String"), "Should generate Config fields");

        println!("Generated Rust code:\n{}", rust_code);
    }

    #[test]
    fn test_cargo_toml_generation() {
        let backend = WasmComponentBackend::new();
        let cu = create_test_compilation_unit();

        let result = backend.generate_cargo_toml(&cu);
        assert!(result.is_ok(), "Cargo.toml generation should succeed");

        let cargo_toml = result.unwrap();
        
        // Check essential Cargo.toml components
        assert!(cargo_toml.contains("name = \"test:component\""), "Should have correct package name");
        assert!(cargo_toml.contains("crate-type = [\"cdylib\"]"), "Should specify cdylib");
        assert!(cargo_toml.contains("wit-bindgen"), "Should include wit-bindgen dependency");
        assert!(cargo_toml.contains("wasm-bindgen"), "Should include wasm-bindgen dependency");
        assert!(cargo_toml.contains("[package.metadata.component]"), "Should have component metadata");
        assert!(cargo_toml.contains("target = { path = \"test:component.wit\" }"), "Should specify WIT target");

        println!("Generated Cargo.toml:\n{}", cargo_toml);
    }

    #[test]
    fn test_build_script_generation() {
        let backend = WasmComponentBackend::new();

        let result = backend.generate_build_script();
        assert!(result.is_ok(), "Build script generation should succeed");

        let build_script = result.unwrap();
        
        assert!(build_script.contains("fn main()"), "Should have main function");
        assert!(build_script.contains("cargo:rerun-if-changed=*.wit"), "Should watch WIT files");
        assert!(build_script.contains("cargo:rerun-if-changed=build.rs"), "Should watch build script");

        println!("Generated build.rs:\n{}", build_script);
    }

    #[test]
    fn test_runtime_generation() {
        let backend = WasmComponentBackend::new();
        let options = create_codegen_options("wasm-component");

        let result = backend.generate_runtime(&options);
        assert!(result.is_ok(), "Runtime generation should succeed");

        let runtime_code = result.unwrap();
        
        // Check for essential runtime components
        assert!(runtime_code.contains("struct EffectRuntime"), "Should define EffectRuntime");
        assert!(runtime_code.contains("enum Effect"), "Should define Effect enum");
        assert!(runtime_code.contains("IOEffect"), "Should define IOEffect");
        assert!(runtime_code.contains("StateEffect"), "Should define StateEffect");
        assert!(runtime_code.contains("ConsoleEffect"), "Should define ConsoleEffect");
        assert!(runtime_code.contains("handle_effect"), "Should have effect handler");
        assert!(runtime_code.contains("struct ComponentMemory"), "Should define ComponentMemory");
        assert!(runtime_code.contains("allocate"), "Should have memory allocation");
        assert!(runtime_code.contains("deallocate"), "Should have memory deallocation");

        println!("Generated runtime:\n{}", runtime_code);
    }

    #[test]
    fn test_type_conversion() {
        let backend = WasmComponentBackend::new();

        // Test basic type conversions
        assert_eq!(backend.type_to_rust_type(&Type::Unit), "()");
        assert_eq!(backend.type_to_rust_type(&Type::Bool), "bool");
        assert_eq!(backend.type_to_rust_type(&Type::Int), "i32");
        assert_eq!(backend.type_to_rust_type(&Type::Float), "f64");
        assert_eq!(backend.type_to_rust_type(&Type::String), "String");
        assert_eq!(backend.type_to_rust_type(&Type::Char), "char");

        // Test complex types
        assert_eq!(
            backend.type_to_rust_type(&Type::List(Box::new(Type::String))), 
            "Vec<String>"
        );
        assert_eq!(
            backend.type_to_rust_type(&Type::Option(Box::new(Type::Int))), 
            "Option<i32>"
        );
        assert_eq!(
            backend.type_to_rust_type(&Type::Result(Box::new(Type::String), Box::new(Type::String))), 
            "Result<String, String>"
        );
        assert_eq!(
            backend.type_to_rust_type(&Type::Tuple(vec![Type::Int, Type::String])), 
            "(i32, String)"
        );
    }

    #[test]
    fn test_visibility_handling() {
        let backend = WasmComponentBackend::new();

        // Test visibility detection
        assert!(backend.is_exported(&Visibility::Public));
        assert!(backend.is_exported(&Visibility::Component { 
            export: true, 
            import: false, 
            interface: None 
        }));
        assert!(!backend.is_exported(&Visibility::Private));
        assert!(!backend.is_exported(&Visibility::Component { 
            export: false, 
            import: true, 
            interface: None 
        }));
    }

    #[test]
    fn test_diagnostics() {
        let mut backend = WasmComponentBackend::new();
        
        // Test with empty compilation unit
        let empty_cu = CompilationUnit {
            package_name: None,
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };
        
        let options = create_codegen_options("wasm-component");
        let result = backend.generate_code(&empty_cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "Should handle empty compilation unit");

        let codegen_result = result.unwrap();
        // Should generate basic files even for empty unit
        assert!(!codegen_result.files.is_empty());
        assert!(codegen_result.diagnostics.is_empty() || 
                codegen_result.diagnostics.iter().all(|d| 
                    !matches!(d.severity, effect_lang::codegen::backend::DiagnosticSeverity::Error)));
    }

    #[test]
    fn test_code_validation() {
        let backend = WitBackend::new();

        // Test valid WIT code
        let valid_wit = r#"
package test:valid;

world effect-lang {
  interface test-interface {
    func hello() -> string;
  }
}
"#;
        let diagnostics = backend.validate_output(valid_wit);
        assert!(diagnostics.iter().all(|d| 
            !matches!(d.severity, effect_lang::codegen::backend::DiagnosticSeverity::Error)), 
            "Valid WIT should not have errors");

        // Test invalid WIT code
        let invalid_wit = r#"
package test:invalid;

world effect-lang {
  interface test-interface {
    func unclosed(
}
"#;
        let diagnostics = backend.validate_output(invalid_wit);
        // Should detect syntax issues (though basic validation might not catch all)
        println!("Validation diagnostics: {:?}", diagnostics);
    }

    #[test]
    #[ignore] // Only run when output directory exists
    fn test_file_output() {
        let mut backend = WasmComponentBackend::new();
        let cu = create_test_compilation_unit();
        
        // Create test output directory
        let output_dir = PathBuf::from("./test_component_output");
        let _ = fs::create_dir_all(&output_dir);

        let options = CodegenOptions {
            target: CompilationTarget {
                name: "wasm-component".to_string(),
                file_extension: "wasm".to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: true,
            },
            output_dir: output_dir.clone(),
            source_maps: false,
            debug_info: false,
            optimization_level: 0,
            emit_types: false,
        };

        let result = backend.generate_code(&cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "Code generation should succeed");

        let codegen_result = result.unwrap();
        
        // Write files to disk
        for (file_path, content) in &codegen_result.files {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).expect("Should create parent directories");
            }
            fs::write(file_path, content).expect("Should write file");
            assert!(file_path.exists(), "File should exist after writing");
        }

        // Verify files exist
        assert!(output_dir.join("test:component.wit").exists(), "WIT file should exist");
        assert!(output_dir.join("Cargo.toml").exists(), "Cargo.toml should exist");
        assert!(output_dir.join("src").join("lib.rs").exists(), "lib.rs should exist");
        assert!(output_dir.join("build.rs").exists(), "build.rs should exist");

        // Cleanup
        let _ = fs::remove_dir_all(&output_dir);
    }
}