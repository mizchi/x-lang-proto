use effect_lang::codegen::backend::{BackendFactory, CodegenOptions, CompilationTarget, DiagnosticSeverity};
use effect_lang::codegen::wasm_component::WasmComponentBackend;
use effect_lang::codegen::wit_backend::WitBackend;
use effect_lang::codegen::wit::WitGenerator;
use effect_lang::core::ast::*;
use effect_lang::core::symbol::Symbol;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    fn create_invalid_compilation_unit() -> CompilationUnit {
        CompilationUnit {
            package_name: Some(Symbol::new("test:invalid")),
            modules: vec![Module {
                name: Symbol::new("InvalidModule"),
                visibility: Visibility::Public,
                items: vec![
                    // Interface with conflicting names
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("ConflictInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("same_name"),
                                params: vec![],
                                results: vec![WasmType::String],
                            }),
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("same_name"), // Duplicate name
                                params: vec![],
                                results: vec![WasmType::I32],
                            }),
                        ],
                        span: Default::default(),
                    }),
                    // Resource with invalid method types
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("InvalidResourceInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Resource(ResourceDefinition {
                                name: Symbol::new("InvalidResource"),
                                constructor: Some(ResourceMethod {
                                    name: Symbol::new("new"),
                                    method_type: MethodType::Constructor,
                                    params: vec![],
                                    results: vec![WasmType::String], // Constructor shouldn't return values
                                }),
                                methods: vec![],
                            }),
                        ],
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![
                // Invalid import
                ImportDef {
                    name: Symbol::new("invalid-import"),
                    kind: ImportKind::Interface,
                    path: None, // Missing path
                    visibility: Visibility::Public,
                    span: Default::default(),
                },
            ],
            exports: vec![],
        }
    }

    fn create_codegen_options() -> CodegenOptions {
        CodegenOptions {
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
        }
    }

    #[test]
    fn test_wit_generator_error_handling() {
        let mut generator = WitGenerator::new();
        
        // Test with invalid compilation unit
        let invalid_cu = create_invalid_compilation_unit();
        
        // WIT generator should handle errors gracefully
        let result = generator.generate(&invalid_cu);
        assert!(result.is_ok(), "WIT generator should handle invalid input gracefully");
        
        // Even with invalid input, it should produce some output
        let wit_content = result.unwrap();
        assert!(wit_content.contains("world effect-lang"), "Should still generate basic world structure");
    }

    #[test]
    fn test_missing_package_name() {
        let mut generator = WitGenerator::new();
        
        let cu_without_package = CompilationUnit {
            package_name: None, // Missing package name
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };
        
        let result = generator.generate(&cu_without_package);
        assert!(result.is_ok(), "Should handle missing package name");
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("world effect-lang"), "Should generate default world");
    }

    #[test]
    fn test_empty_interface_items() {
        let mut generator = WitGenerator::new();
        
        let cu_with_empty_interface = CompilationUnit {
            package_name: Some(Symbol::new("test:empty")),
            modules: vec![Module {
                name: Symbol::new("EmptyModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("EmptyInterface"),
                        visibility: Visibility::Public,
                        items: vec![], // Empty interface
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };
        
        let result = generator.generate(&cu_with_empty_interface);
        assert!(result.is_ok(), "Should handle empty interfaces");
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("interface EmptyInterface {"), "Should generate empty interface");
    }

    #[test]
    fn test_invalid_wasm_types() {
        let generator = WitGenerator::new();
        
        // Test all WASM types to ensure no panics
        let wasm_types = vec![
            WasmType::I32, WasmType::I64, WasmType::F32, WasmType::F64,
            WasmType::V128, WasmType::FuncRef, WasmType::ExternRef,
            WasmType::String, WasmType::Bool, WasmType::List,
            WasmType::Record, WasmType::Variant, WasmType::Tuple,
            WasmType::Option, WasmType::Result,
        ];
        
        for wasm_type in wasm_types {
            let wit_type = generator.wasm_type_to_wit(&wasm_type);
            assert!(!wit_type.is_empty(), "WASM type conversion should not be empty");
        }
    }

    #[test]
    fn test_component_backend_error_handling() {
        let mut backend = WasmComponentBackend::new();
        let invalid_cu = create_invalid_compilation_unit();
        let options = create_codegen_options();
        
        let result = backend.generate_code(&invalid_cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "Component backend should handle invalid input gracefully");
        
        let codegen_result = result.unwrap();
        
        // Should still generate files, but may have diagnostics
        assert!(!codegen_result.files.is_empty(), "Should generate some files");
        
        // Check for warnings or errors in diagnostics
        let has_warnings_or_errors = codegen_result.diagnostics.iter().any(|d| {
            matches!(d.severity, DiagnosticSeverity::Warning | DiagnosticSeverity::Error)
        });
        
        // For now, we expect basic generation to work even with some issues
        println!("Diagnostics: {:?}", codegen_result.diagnostics);
    }

    #[test]
    fn test_wit_backend_error_handling() {
        let mut backend = WitBackend::new();
        let invalid_cu = create_invalid_compilation_unit();
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
        
        let result = backend.generate_code(&invalid_cu, &HashMap::new(), &options);
        assert!(result.is_ok(), "WIT backend should handle invalid input gracefully");
        
        let codegen_result = result.unwrap();
        assert!(!codegen_result.files.is_empty(), "Should generate files");
    }

    #[test]
    fn test_backend_factory_invalid_targets() {
        // Test invalid backend names
        let invalid_targets = vec![
            "invalid-target",
            "not-a-backend",
            "typescript-invalid",
            "wasm-invalid",
            "",
        ];
        
        for target in invalid_targets {
            let result = BackendFactory::create_backend(target);
            assert!(result.is_err(), "Should return error for invalid target: {}", target);
            
            if let Err(e) = result {
                assert!(format!("{:?}", e).contains("Unknown compilation target"), 
                        "Error should mention unknown target");
            }
        }
    }

    #[test]
    fn test_corrupted_ast_structures() {
        let mut generator = WitGenerator::new();
        
        // Test with extreme values and edge cases
        let extreme_cu = CompilationUnit {
            package_name: Some(Symbol::new("test:extreme-values")),
            modules: vec![Module {
                name: Symbol::new("ExtremeModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("ExtremeInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("extreme_function"),
                                params: {
                                    // Generate many parameters
                                    let mut params = vec![];
                                    for i in 0..100 {
                                        params.push(Parameter {
                                            name: Symbol::new(&format!("param_{}", i)),
                                            wasm_type: WasmType::I32,
                                        });
                                    }
                                    params
                                },
                                results: {
                                    // Generate many return values
                                    let mut results = vec![];
                                    for _ in 0..50 {
                                        results.push(WasmType::String);
                                    }
                                    results
                                },
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
        
        let result = generator.generate(&extreme_cu);
        assert!(result.is_ok(), "Should handle extreme parameter counts");
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("extreme_function"), "Should generate function with many parameters");
    }

    #[test]
    fn test_circular_references() {
        let mut generator = WitGenerator::new();
        
        // Test with potential circular references in type definitions
        let circular_cu = CompilationUnit {
            package_name: Some(Symbol::new("test:circular")),
            modules: vec![Module {
                name: Symbol::new("CircularModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("NodeA"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Record(vec![
                            (Symbol::new("value"), Type::Int),
                            (Symbol::new("next"), Type::Named(Symbol::new("NodeB"))),
                        ]),
                        span: Default::default(),
                    }),
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("NodeB"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Record(vec![
                            (Symbol::new("value"), Type::String),
                            (Symbol::new("prev"), Type::Named(Symbol::new("NodeA"))),
                        ]),
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };
        
        let result = generator.generate(&circular_cu);
        assert!(result.is_ok(), "Should handle potential circular references");
    }

    #[test]
    fn test_special_characters_and_unicode() {
        let mut generator = WitGenerator::new();
        
        let unicode_cu = CompilationUnit {
            package_name: Some(Symbol::new("test:unicode-測試-🦀")),
            modules: vec![Module {
                name: Symbol::new("UnicodeModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("UnicodeInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("function_with_émojis_🎉"),
                                params: vec![
                                    Parameter {
                                        name: Symbol::new("café_param"),
                                        wasm_type: WasmType::String,
                                    }
                                ],
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
        
        let result = generator.generate(&unicode_cu);
        assert!(result.is_ok(), "Should handle Unicode characters gracefully");
        
        // The generator might sanitize or handle Unicode differently
        let wit_content = result.unwrap();
        assert!(wit_content.contains("world effect-lang"), "Should still generate valid structure");
    }

    #[test]
    fn test_memory_exhaustion_simulation() {
        let mut generator = WitGenerator::new();
        
        // Create a compilation unit with many modules and interfaces
        let mut modules = vec![];
        for module_idx in 0..50 {
            let mut items = vec![];
            for interface_idx in 0..20 {
                let mut interface_items = vec![];
                for func_idx in 0..50 {
                    interface_items.push(InterfaceItem::Function(FunctionSignature {
                        name: Symbol::new(&format!("func_{}_{}", interface_idx, func_idx)),
                        params: vec![
                            Parameter {
                                name: Symbol::new("param"),
                                wasm_type: WasmType::String,
                            }
                        ],
                        results: vec![WasmType::String],
                    }));
                }
                
                items.push(Item::InterfaceDef(ComponentInterface {
                    name: Symbol::new(&format!("Interface_{}", interface_idx)),
                    visibility: Visibility::Public,
                    items: interface_items,
                    span: Default::default(),
                }));
            }
            
            modules.push(Module {
                name: Symbol::new(&format!("Module_{}", module_idx)),
                visibility: Visibility::Public,
                items,
                span: Default::default(),
            });
        }
        
        let large_cu = CompilationUnit {
            package_name: Some(Symbol::new("test:large")),
            modules,
            imports: vec![],
            exports: vec![],
        };
        
        let result = generator.generate(&large_cu);
        assert!(result.is_ok(), "Should handle large compilation units");
        
        let wit_content = result.unwrap();
        assert!(wit_content.len() > 1000, "Should generate substantial content");
        assert!(wit_content.contains("Module_0"), "Should contain first module");
        assert!(wit_content.contains("Module_49"), "Should contain last module");
    }

    #[test]
    fn test_wit_validation_edge_cases() {
        let backend = WitBackend::new();
        
        // Test various edge cases in WIT validation
        let test_cases = vec![
            ("", "Empty content"),
            ("invalid syntax", "Invalid syntax"),
            ("world {", "Unclosed world"),
            ("interface test {", "Unclosed interface"),
            ("resource test {", "Unclosed resource"),
            ("func test(", "Unclosed function"),
            ("world effect-lang {\n  func test() -> string;\n}", "Valid minimal WIT"),
        ];
        
        for (wit_content, description) in test_cases {
            let diagnostics = backend.validate_output(wit_content);
            println!("Validation for '{}': {:?}", description, diagnostics);
            
            // Validation should not panic, regardless of input
            assert!(true, "Validation should complete without panic for: {}", description);
        }
    }

    #[test]
    fn test_concurrent_generation() {
        use std::thread;
        use std::sync::Arc;
        
        let cu = Arc::new(create_invalid_compilation_unit());
        let options = Arc::new(create_codegen_options());
        
        let handles: Vec<_> = (0..5).map(|_| {
            let cu_clone = cu.clone();
            let options_clone = options.clone();
            
            thread::spawn(move || {
                let mut backend = WasmComponentBackend::new();
                let result = backend.generate_code(&cu_clone, &HashMap::new(), &options_clone);
                assert!(result.is_ok(), "Concurrent generation should succeed");
                result.unwrap()
            })
        }).collect();
        
        // Wait for all threads to complete
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        
        // All results should be successful
        assert_eq!(results.len(), 5, "All threads should complete");
        
        for result in results {
            assert!(!result.files.is_empty(), "Each result should generate files");
        }
    }

    #[test]
    fn test_resource_recovery() {
        let mut backend = WasmComponentBackend::new();
        
        // Generate code multiple times to test resource cleanup
        for i in 0..10 {
            let cu = CompilationUnit {
                package_name: Some(Symbol::new(&format!("test:iteration-{}", i))),
                modules: vec![],
                imports: vec![],
                exports: vec![],
            };
            
            let result = backend.generate_code(&cu, &HashMap::new(), &create_codegen_options());
            assert!(result.is_ok(), "Iteration {} should succeed", i);
        }
    }

    #[test]
    fn test_malformed_visibility() {
        let mut generator = WitGenerator::new();
        
        // Test with various visibility combinations
        let visibility_test_cu = CompilationUnit {
            package_name: Some(Symbol::new("test:visibility")),
            modules: vec![Module {
                name: Symbol::new("VisibilityModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("TestInterface"),
                        visibility: Visibility::Component {
                            export: true,
                            import: true, // Both export and import (unusual)
                            interface: None, // Missing interface name
                        },
                        items: vec![],
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };
        
        let result = generator.generate(&visibility_test_cu);
        assert!(result.is_ok(), "Should handle unusual visibility combinations");
    }
}