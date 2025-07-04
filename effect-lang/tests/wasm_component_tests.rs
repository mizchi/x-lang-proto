use effect_lang::codegen::backend::{BackendFactory, CodegenOptions, CompilationTarget};
use effect_lang::codegen::wasm_component::WasmComponentBackend;
use effect_lang::codegen::wit_backend::WitBackend;
use effect_lang::core::ast::*;
use effect_lang::core::symbol::Symbol;
use std::collections::HashMap;
use std::path::PathBuf;

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
fn test_backend_factory_component() {
    let result = BackendFactory::create_backend("wasm-component");
    assert!(result.is_ok());
    
    let backend = result.unwrap();
    assert_eq!(backend.target_info().name, "wasm-component");
}

#[test]
fn test_backend_factory_wit() {
    let result = BackendFactory::create_backend("wit");
    assert!(result.is_ok());
    
    let backend = result.unwrap();
    assert_eq!(backend.target_info().name, "wit");
}

#[test]
fn test_simple_wit_generation() {
    let mut backend = WitBackend::new();
    
    // Create a simple compilation unit
    let cu = CompilationUnit {
        package_name: Some(Symbol::new("test:simple")),
        modules: vec![Module {
            name: Symbol::new("TestModule"),
            visibility: Visibility::Public,
            items: vec![
                Item::ValueDef(ValueDef {
                    name: Symbol::new("hello"),
                    visibility: Visibility::Public,
                    parameters: vec![],
                    body: Expr::Literal(Literal::String("Hello World".to_string())),
                    type_annotation: Some(Type::String),
                    span: Default::default(),
                }),
            ],
            span: Default::default(),
        }],
        imports: vec![],
        exports: vec![],
    };

    let options = CodegenOptions {
        target: CompilationTarget {
            name: "wit".to_string(),
            file_extension: "wit".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: false,
        },
        output_dir: PathBuf::from("./test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };

    let result = backend.generate_code(&cu, &HashMap::new(), &options);
    assert!(result.is_ok());
    
    let codegen_result = result.unwrap();
    assert!(!codegen_result.files.is_empty());
    
    // Check that WIT file was generated
    let wit_file = codegen_result.files.iter().find(|(path, _)| {
        path.extension().map_or(false, |ext| ext == "wit")
    });
    assert!(wit_file.is_some());
    
    let (_path, content) = wit_file.unwrap();
    assert!(content.contains("package test:simple"));
    assert!(content.contains("world effect-lang"));
}

#[test]
fn test_interface_wit_generation() {
    let mut backend = WitBackend::new();
    
    // Create a compilation unit with interface
    let cu = CompilationUnit {
        package_name: Some(Symbol::new("test:interface")),
        modules: vec![Module {
            name: Symbol::new("TestModule"),
            visibility: Visibility::Public,
            items: vec![
                Item::InterfaceDef(ComponentInterface {
                    name: Symbol::new("TestInterface"),
                    visibility: Visibility::Component {
                        export: true,
                        import: false,
                        interface: Some(Symbol::new("test:interface")),
                    },
                    items: vec![
                        InterfaceItem::Function(FunctionSignature {
                            name: Symbol::new("add"),
                            params: vec![
                                Parameter {
                                    name: Symbol::new("x"),
                                    wasm_type: WasmType::I32,
                                },
                                Parameter {
                                    name: Symbol::new("y"),
                                    wasm_type: WasmType::I32,
                                },
                            ],
                            results: vec![WasmType::I32],
                        }),
                        InterfaceItem::Type(Symbol::new("Counter"), WasmType::I32),
                    ],
                    span: Default::default(),
                }),
            ],
            span: Default::default(),
        }],
        imports: vec![],
        exports: vec![],
    };

    let options = CodegenOptions {
        target: CompilationTarget {
            name: "wit".to_string(),
            file_extension: "wit".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: false,
        },
        output_dir: PathBuf::from("./test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };

    let result = backend.generate_code(&cu, &HashMap::new(), &options);
    assert!(result.is_ok());
    
    let codegen_result = result.unwrap();
    let wit_file = codegen_result.files.iter().find(|(path, _)| {
        path.extension().map_or(false, |ext| ext == "wit")
    });
    assert!(wit_file.is_some());
    
    let (_path, content) = wit_file.unwrap();
    assert!(content.contains("interface TestInterface"));
    assert!(content.contains("add: func(x: s32, y: s32) -> s32"));
    assert!(content.contains("type Counter = s32"));
}

#[test]
fn test_component_rust_generation() {
    let mut backend = WasmComponentBackend::new();
    
    // Create a simple compilation unit
    let cu = CompilationUnit {
        package_name: Some(Symbol::new("test:component")),
        modules: vec![Module {
            name: Symbol::new("TestModule"),
            visibility: Visibility::Public,
            items: vec![
                Item::ValueDef(ValueDef {
                    name: Symbol::new("greet"),
                    visibility: Visibility::Public,
                    parameters: vec![],
                    body: Expr::Literal(Literal::String("Hello from component!".to_string())),
                    type_annotation: Some(Type::String),
                    span: Default::default(),
                }),
            ],
            span: Default::default(),
        }],
        imports: vec![],
        exports: vec![],
    };

    let options = CodegenOptions {
        target: CompilationTarget {
            name: "wasm-component".to_string(),
            file_extension: "wasm".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: true,
        },
        output_dir: PathBuf::from("./test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };

    let result = backend.generate_code(&cu, &HashMap::new(), &options);
    assert!(result.is_ok());
    
    let codegen_result = result.unwrap();
    
    // Check that Rust source file was generated
    let rust_file = codegen_result.files.iter().find(|(path, _)| {
        path.file_name().map_or(false, |name| name == "lib.rs")
    });
    assert!(rust_file.is_some());
    
    let (_path, content) = rust_file.unwrap();
    assert!(content.contains("use wit_bindgen::generate;"));
    assert!(content.contains("struct Component"));
    assert!(content.contains("impl exports::Guest for Component"));
    
    // Check that Cargo.toml was generated
    let cargo_file = codegen_result.files.iter().find(|(path, _)| {
        path.file_name().map_or(false, |name| name == "Cargo.toml")
    });
    assert!(cargo_file.is_some());
    
    let (_path, content) = cargo_file.unwrap();
    assert!(content.contains("wit-bindgen"));
    assert!(content.contains("[package.metadata.component]"));
    
    // Check that WIT file was generated
    let wit_file = codegen_result.files.iter().find(|(path, _)| {
        path.extension().map_or(false, |ext| ext == "wit")
    });
    assert!(wit_file.is_some());
}

#[test]
fn test_wasm_type_conversion() {
    let backend = WitBackend::new();
    
    // Test basic type conversions
    assert_eq!(backend.wasm_type_to_wit(&WasmType::I32), "s32");
    assert_eq!(backend.wasm_type_to_wit(&WasmType::I64), "s64");
    assert_eq!(backend.wasm_type_to_wit(&WasmType::F32), "f32");
    assert_eq!(backend.wasm_type_to_wit(&WasmType::F64), "f64");
    assert_eq!(backend.wasm_type_to_wit(&WasmType::String), "string");
    assert_eq!(backend.wasm_type_to_wit(&WasmType::Bool), "bool");
}

#[test]
fn test_visibility_export() {
    let backend = WitBackend::new();
    
    // Test visibility to WIT export conversion
    let component_export = Visibility::Component {
        export: true,
        import: false,
        interface: Some(Symbol::new("test:api")),
    };
    
    let export_stmt = backend.visibility_to_wit_export(&component_export);
    assert!(export_stmt.is_some());
    assert!(export_stmt.unwrap().contains("export test:api"));
    
    // Test non-export visibility
    let private_visibility = Visibility::Private;
    let no_export = backend.visibility_to_wit_export(&private_visibility);
    assert!(no_export.is_none());
}

#[test]
fn test_resource_generation() {
    let mut backend = WitBackend::new();
    
    // Create a compilation unit with resource
    let cu = CompilationUnit {
        package_name: Some(Symbol::new("test:resource")),
        modules: vec![Module {
            name: Symbol::new("TestModule"),
            visibility: Visibility::Public,
            items: vec![
                Item::InterfaceDef(ComponentInterface {
                    name: Symbol::new("ResourceInterface"),
                    visibility: Visibility::Public,
                    items: vec![
                        InterfaceItem::Resource(ResourceDefinition {
                            name: Symbol::new("Counter"),
                            constructor: Some(ResourceMethod {
                                name: Symbol::new("new"),
                                method_type: MethodType::Constructor,
                                params: vec![Parameter {
                                    name: Symbol::new("initial"),
                                    wasm_type: WasmType::I32,
                                }],
                                results: vec![],
                            }),
                            methods: vec![
                                ResourceMethod {
                                    name: Symbol::new("increment"),
                                    method_type: MethodType::Method,
                                    params: vec![],
                                    results: vec![WasmType::I32],
                                },
                            ],
                        }),
                    ],
                    span: Default::default(),
                }),
            ],
            span: Default::default(),
        }],
        imports: vec![],
        exports: vec![],
    };

    let options = CodegenOptions {
        target: CompilationTarget {
            name: "wit".to_string(),
            file_extension: "wit".to_string(),
            supports_modules: true,
            supports_effects: true,
            supports_gc: false,
        },
        output_dir: PathBuf::from("./test_output"),
        source_maps: false,
        debug_info: false,
        optimization_level: 0,
        emit_types: false,
    };

    let result = backend.generate_code(&cu, &HashMap::new(), &options);
    assert!(result.is_ok());
    
    let codegen_result = result.unwrap();
    let wit_file = codegen_result.files.iter().find(|(path, _)| {
        path.extension().map_or(false, |ext| ext == "wit")
    });
    assert!(wit_file.is_some());
    
    let (_path, content) = wit_file.unwrap();
    assert!(content.contains("resource Counter"));
    assert!(content.contains("constructor new: func(initial: s32)"));
    assert!(content.contains("increment: func() -> s32"));
}

#[test]
fn test_available_backends() {
    let backends = BackendFactory::available_backends();
    assert!(backends.contains(&"wasm-component"));
    assert!(backends.contains(&"wit"));
    assert!(backends.contains(&"typescript"));
    assert!(backends.contains(&"wasm-gc"));
}

#[test]
fn test_backend_feature_support() {
    let component_backend = WasmComponentBackend::new();
    let wit_backend = WitBackend::new();
    
    // Test component backend features
    assert!(component_backend.supports_feature("components"));
    assert!(component_backend.supports_feature("interfaces"));
    assert!(component_backend.supports_feature("gc"));
    assert!(component_backend.supports_feature("effects"));
    
    // Test WIT backend features
    assert!(wit_backend.supports_feature("interfaces"));
    assert!(wit_backend.supports_feature("resources"));
    assert!(wit_backend.supports_feature("components"));
    assert!(!wit_backend.supports_feature("gc"));
    assert!(!wit_backend.supports_feature("effects"));
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    
    #[test]
    #[ignore] // Requires wasm-tools to be installed
    fn test_wit_validation() {
        let mut backend = WitBackend::new();
        
        // Generate a WIT file
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:validation")),
            modules: vec![Module {
                name: Symbol::new("TestModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("TestInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("test_func"),
                                params: vec![],
                                results: vec![WasmType::String],
                            }),
                        ],
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };
        
        let options = CodegenOptions {
            target: CompilationTarget {
                name: "wit".to_string(),
                file_extension: "wit".to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: false,
            },
            output_dir: PathBuf::from("./test_output"),
            source_maps: false,
            debug_info: false,
            optimization_level: 0,
            emit_types: false,
        };
        
        let result = backend.generate_code(&cu, &HashMap::new(), &options);
        assert!(result.is_ok());
        
        let codegen_result = result.unwrap();
        let wit_file = codegen_result.files.iter().find(|(path, _)| {
            path.extension().map_or(false, |ext| ext == "wit")
        });
        assert!(wit_file.is_some());
        
        let (path, content) = wit_file.unwrap();
        
        // Create temporary directory and write WIT file
        let temp_dir = std::env::temp_dir().join("effect_lang_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let wit_path = temp_dir.join("test.wit");
        fs::write(&wit_path, content).unwrap();
        
        // Validate WIT file using wasm-tools (if available)
        if Command::new("wasm-tools").arg("--version").output().is_ok() {
            let output = Command::new("wasm-tools")
                .args(&["component", "wit", wit_path.to_str().unwrap()])
                .output()
                .expect("Failed to run wasm-tools");
            
            if !output.status.success() {
                panic!("WIT validation failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}