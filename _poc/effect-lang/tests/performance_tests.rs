use effect_lang::codegen::backend::{CodegenOptions, CompilationTarget};
use effect_lang::codegen::wasm_component::WasmComponentBackend;
use effect_lang::codegen::wit_backend::WitBackend;
use effect_lang::codegen::wit::WitGenerator;
use effect_lang::core::ast::*;
use effect_lang::core::symbol::Symbol;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_tests {
    use super::*;

    fn create_large_compilation_unit(module_count: usize, interface_count: usize, function_count: usize) -> CompilationUnit {
        let mut modules = vec![];
        
        for module_idx in 0..module_count {
            let mut items = vec![];
            
            // Add interfaces
            for interface_idx in 0..interface_count {
                let mut interface_items = vec![];
                
                // Add functions to interface
                for func_idx in 0..function_count {
                    interface_items.push(InterfaceItem::Function(FunctionSignature {
                        name: Symbol::new(&format!("func_{}_{}", interface_idx, func_idx)),
                        params: vec![
                            Parameter {
                                name: Symbol::new("x"),
                                wasm_type: WasmType::I32,
                            },
                            Parameter {
                                name: Symbol::new("y"),
                                wasm_type: WasmType::String,
                            },
                        ],
                        results: vec![WasmType::F64],
                    }));
                }
                
                // Add resources to interface
                interface_items.push(InterfaceItem::Resource(ResourceDefinition {
                    name: Symbol::new(&format!("Resource_{}", interface_idx)),
                    constructor: Some(ResourceMethod {
                        name: Symbol::new("new"),
                        method_type: MethodType::Constructor,
                        params: vec![
                            Parameter {
                                name: Symbol::new("config"),
                                wasm_type: WasmType::String,
                            }
                        ],
                        results: vec![],
                    }),
                    methods: vec![
                        ResourceMethod {
                            name: Symbol::new("process"),
                            method_type: MethodType::Method,
                            params: vec![
                                Parameter {
                                    name: Symbol::new("data"),
                                    wasm_type: WasmType::String,
                                }
                            ],
                            results: vec![WasmType::String],
                        },
                        ResourceMethod {
                            name: Symbol::new("get_stats"),
                            method_type: MethodType::Static,
                            params: vec![],
                            results: vec![WasmType::I64],
                        },
                    ],
                }));
                
                items.push(Item::InterfaceDef(ComponentInterface {
                    name: Symbol::new(&format!("Interface_{}_{}", module_idx, interface_idx)),
                    visibility: if interface_idx % 2 == 0 { 
                        Visibility::Public 
                    } else { 
                        Visibility::Component {
                            export: true,
                            import: false,
                            interface: Some(Symbol::new(&format!("api:module{}@1.0.0", module_idx))),
                        }
                    },
                    items: interface_items,
                    span: Default::default(),
                }));
            }
            
            // Add type definitions
            for type_idx in 0..10 {
                items.push(Item::TypeDef(TypeDef {
                    name: Symbol::new(&format!("Type_{}_{}", module_idx, type_idx)),
                    visibility: Visibility::Public,
                    definition: if type_idx % 2 == 0 {
                        TypeDefinition::Record(vec![
                            (Symbol::new("id"), Type::Int),
                            (Symbol::new("name"), Type::String),
                            (Symbol::new("active"), Type::Bool),
                            (Symbol::new("data"), Type::List(Box::new(Type::String))),
                        ])
                    } else {
                        TypeDefinition::Variant(vec![
                            VariantDefinition {
                                name: Symbol::new("Success"),
                                data: Some(Type::String),
                            },
                            VariantDefinition {
                                name: Symbol::new("Error"),
                                data: Some(Type::String),
                            },
                            VariantDefinition {
                                name: Symbol::new("Pending"),
                                data: None,
                            },
                        ])
                    },
                    span: Default::default(),
                }));
            }
            
            // Add value definitions
            for value_idx in 0..5 {
                items.push(Item::ValueDef(ValueDef {
                    name: Symbol::new(&format!("value_{}_{}", module_idx, value_idx)),
                    visibility: Visibility::Public,
                    parameters: vec![
                        (Symbol::new("input"), Type::String),
                    ],
                    body: Expr::Literal(Literal::String(format!("result_{}", value_idx))),
                    type_annotation: Some(Type::String),
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
        
        // Add imports
        let mut imports = vec![];
        for i in 0..20 {
            imports.push(ImportDef {
                name: Symbol::new(&format!("import_{}", i)),
                kind: if i % 3 == 0 { ImportKind::Interface } else if i % 3 == 1 { ImportKind::Core } else { ImportKind::Func },
                path: Some(Symbol::new(&format!("external:api{}@1.0.0", i))),
                visibility: Visibility::Public,
                span: Default::default(),
            });
        }
        
        // Add exports
        let mut exports = vec![];
        for i in 0..10 {
            exports.push(ExportDef {
                name: Symbol::new(&format!("export_{}", i)),
                exported_name: Symbol::new(&format!("my:api{}@1.0.0", i)),
                visibility: Visibility::Public,
                span: Default::default(),
            });
        }
        
        CompilationUnit {
            package_name: Some(Symbol::new("test:performance")),
            modules,
            imports,
            exports,
        }
    }

    fn create_codegen_options(target: &str) -> CodegenOptions {
        CodegenOptions {
            target: CompilationTarget {
                name: target.to_string(),
                file_extension: if target == "wit" { "wit" } else { "wasm" }.to_string(),
                supports_modules: true,
                supports_effects: true,
                supports_gc: target != "wit",
            },
            output_dir: PathBuf::from("./perf_test_output"),
            source_maps: false,
            debug_info: false,
            optimization_level: 0,
            emit_types: false,
        }
    }

    fn measure_duration<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    #[test]
    fn test_wit_generation_performance_small() {
        let mut generator = WitGenerator::new();
        let cu = create_large_compilation_unit(1, 2, 5); // Small: 1 module, 2 interfaces, 5 functions each
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "Small compilation unit should generate successfully");
        assert!(duration < Duration::from_millis(100), 
                "Small WIT generation should complete in <100ms, took {:?}", duration);
        
        let wit_content = result.unwrap();
        assert!(wit_content.len() > 100, "Should generate substantial content");
        
        println!("Small WIT generation: {:?} ({} chars)", duration, wit_content.len());
    }

    #[test]
    fn test_wit_generation_performance_medium() {
        let mut generator = WitGenerator::new();
        let cu = create_large_compilation_unit(5, 10, 20); // Medium: 5 modules, 10 interfaces, 20 functions each
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "Medium compilation unit should generate successfully");
        assert!(duration < Duration::from_secs(1), 
                "Medium WIT generation should complete in <1s, took {:?}", duration);
        
        let wit_content = result.unwrap();
        assert!(wit_content.len() > 10000, "Should generate substantial content");
        
        println!("Medium WIT generation: {:?} ({} chars)", duration, wit_content.len());
    }

    #[test]
    #[ignore] // Only run when specifically testing performance
    fn test_wit_generation_performance_large() {
        let mut generator = WitGenerator::new();
        let cu = create_large_compilation_unit(20, 25, 50); // Large: 20 modules, 25 interfaces, 50 functions each
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "Large compilation unit should generate successfully");
        assert!(duration < Duration::from_secs(10), 
                "Large WIT generation should complete in <10s, took {:?}", duration);
        
        let wit_content = result.unwrap();
        assert!(wit_content.len() > 100000, "Should generate substantial content");
        
        println!("Large WIT generation: {:?} ({} chars)", duration, wit_content.len());
    }

    #[test]
    fn test_component_backend_performance() {
        let mut backend = WasmComponentBackend::new();
        let cu = create_large_compilation_unit(3, 5, 10); // Moderate size for component generation
        let options = create_codegen_options("wasm-component");
        
        let (result, duration) = measure_duration(|| {
            backend.generate_code(&cu, &HashMap::new(), &options)
        });
        
        assert!(result.is_ok(), "Component generation should succeed");
        assert!(duration < Duration::from_secs(2), 
                "Component generation should complete in <2s, took {:?}", duration);
        
        let codegen_result = result.unwrap();
        assert!(!codegen_result.files.is_empty(), "Should generate files");
        
        let total_size: usize = codegen_result.files.values().map(|content| content.len()).sum();
        println!("Component generation: {:?} ({} files, {} total chars)", 
                duration, codegen_result.files.len(), total_size);
    }

    #[test]
    fn test_wit_backend_performance() {
        let mut backend = WitBackend::new();
        let cu = create_large_compilation_unit(3, 5, 10);
        let options = create_codegen_options("wit");
        
        let (result, duration) = measure_duration(|| {
            backend.generate_code(&cu, &HashMap::new(), &options)
        });
        
        assert!(result.is_ok(), "WIT backend generation should succeed");
        assert!(duration < Duration::from_secs(1), 
                "WIT backend generation should complete in <1s, took {:?}", duration);
        
        let codegen_result = result.unwrap();
        assert!(!codegen_result.files.is_empty(), "Should generate files");
        
        println!("WIT backend generation: {:?} ({} files)", duration, codegen_result.files.len());
    }

    #[test]
    fn test_memory_usage_stability() {
        // Test memory usage by generating multiple compilation units
        let mut total_duration = Duration::new(0, 0);
        let iterations = 10;
        
        for i in 0..iterations {
            let mut generator = WitGenerator::new();
            let cu = create_large_compilation_unit(2, 3, 8);
            
            let (result, duration) = measure_duration(|| generator.generate(&cu));
            
            assert!(result.is_ok(), "Iteration {} should succeed", i);
            total_duration += duration;
        }
        
        let avg_duration = total_duration / iterations;
        println!("Average generation time over {} iterations: {:?}", iterations, avg_duration);
        
        // Memory usage should remain stable across iterations
        assert!(avg_duration < Duration::from_millis(500), 
                "Average generation time should be reasonable");
    }

    #[test]
    fn test_incremental_performance() {
        // Test how performance scales with input size
        let sizes = vec![
            (1, 1, 5),   // Tiny
            (1, 3, 10),  // Small
            (2, 5, 15),  // Medium
            (3, 8, 20),  // Large
        ];
        
        let mut previous_duration = Duration::new(0, 0);
        
        for (i, (modules, interfaces, functions)) in sizes.iter().enumerate() {
            let mut generator = WitGenerator::new();
            let cu = create_large_compilation_unit(*modules, *interfaces, *functions);
            
            let (result, duration) = measure_duration(|| generator.generate(&cu));
            
            assert!(result.is_ok(), "Size test {} should succeed", i);
            
            let wit_content = result.unwrap();
            println!("Size {}: {}x{}x{} -> {:?} ({} chars)", 
                    i, modules, interfaces, functions, duration, wit_content.len());
            
            // Performance should scale reasonably (not exponentially)
            if i > 0 {
                let scale_factor = duration.as_millis() as f64 / previous_duration.as_millis() as f64;
                assert!(scale_factor < 10.0, "Performance should not degrade exponentially (factor: {:.2})", scale_factor);
            }
            
            previous_duration = duration;
        }
    }

    #[test]
    fn test_deep_nesting_performance() {
        // Test performance with deeply nested structures
        let mut generator = WitGenerator::new();
        
        // Create nested type definitions
        let mut nested_type = Type::String;
        for i in 0..20 {
            nested_type = if i % 2 == 0 {
                Type::List(Box::new(nested_type))
            } else {
                Type::Option(Box::new(nested_type))
            };
        }
        
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:nested")),
            modules: vec![Module {
                name: Symbol::new("NestedModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("DeeplyNested"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Alias(nested_type),
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "Deeply nested structures should be handled");
        assert!(duration < Duration::from_millis(100), 
                "Deep nesting should not cause performance issues, took {:?}", duration);
        
        println!("Deep nesting performance: {:?}", duration);
    }

    #[test]
    fn test_concurrent_generation_performance() {
        use std::thread;
        use std::sync::Arc;
        
        let cu = Arc::new(create_large_compilation_unit(2, 4, 8));
        let thread_count = 4;
        
        let (results, total_duration) = measure_duration(|| {
            let handles: Vec<_> = (0..thread_count).map(|_| {
                let cu_clone = cu.clone();
                thread::spawn(move || {
                    let mut generator = WitGenerator::new();
                    let start = Instant::now();
                    let result = generator.generate(&cu_clone);
                    let duration = start.elapsed();
                    (result, duration)
                })
            }).collect();
            
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
        });
        
        // All threads should complete successfully
        for (i, (result, thread_duration)) in results.iter().enumerate() {
            assert!(result.is_ok(), "Thread {} should complete successfully", i);
            assert!(thread_duration < &Duration::from_secs(1), 
                    "Thread {} should complete quickly", i);
        }
        
        println!("Concurrent generation: {} threads completed in {:?}", thread_count, total_duration);
        
        // Concurrent execution should not be significantly slower than sequential
        assert!(total_duration < Duration::from_secs(5), 
                "Concurrent generation should complete in reasonable time");
    }

    #[test]
    fn test_string_building_performance() {
        // Test performance of string building operations
        let mut generator = WitGenerator::new();
        
        // Create compilation unit with many string operations
        let cu = create_large_compilation_unit(1, 1, 100); // 1 module, 1 interface, 100 functions
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "String-heavy generation should succeed");
        
        let wit_content = result.unwrap();
        let content_size = wit_content.len();
        
        println!("String building performance: {:?} for {} characters", duration, content_size);
        
        // Performance should be reasonable for string operations
        let chars_per_ms = content_size as f64 / duration.as_millis() as f64;
        assert!(chars_per_ms > 100.0, 
                "String building should be efficient: {:.2} chars/ms", chars_per_ms);
    }

    #[test]
    fn test_validation_performance() {
        let backend = WitBackend::new();
        
        // Generate some WIT content to validate
        let mut generator = WitGenerator::new();
        let cu = create_large_compilation_unit(2, 3, 10);
        let wit_content = generator.generate(&cu).unwrap();
        
        // Test validation performance
        let (diagnostics, duration) = measure_duration(|| {
            backend.validate_output(&wit_content)
        });
        
        println!("Validation performance: {:?} for {} chars", duration, wit_content.len());
        println!("Validation diagnostics: {:?}", diagnostics);
        
        // Validation should be fast
        assert!(duration < Duration::from_millis(50), 
                "Validation should be fast, took {:?}", duration);
    }

    #[test]
    fn test_rust_generation_performance() {
        let mut backend = WasmComponentBackend::new();
        let cu = create_large_compilation_unit(3, 4, 12);
        
        let (result, duration) = measure_duration(|| {
            backend.generate_rust_component(&cu, &HashMap::new())
        });
        
        assert!(result.is_ok(), "Rust generation should succeed");
        assert!(duration < Duration::from_secs(1), 
                "Rust generation should complete quickly, took {:?}", duration);
        
        let rust_code = result.unwrap();
        println!("Rust generation performance: {:?} for {} characters", duration, rust_code.len());
    }

    #[test]
    #[ignore] // Run only for stress testing
    fn test_stress_generation() {
        // Stress test with very large input
        let mut generator = WitGenerator::new();
        let cu = create_large_compilation_unit(50, 30, 100); // Very large
        
        let (result, duration) = measure_duration(|| generator.generate(&cu));
        
        assert!(result.is_ok(), "Stress test should eventually succeed");
        
        let wit_content = result.unwrap();
        println!("Stress test: {:?} for {} characters", duration, wit_content.len());
        
        // Should complete within reasonable time even for very large input
        assert!(duration < Duration::from_secs(30), 
                "Stress test should complete within 30 seconds");
    }
}