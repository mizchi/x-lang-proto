use effect_lang::codegen::wit::WitGenerator;
use effect_lang::core::ast::*;
use effect_lang::core::symbol::Symbol;

#[cfg(test)]
mod wit_generation_tests {
    use super::*;

    #[test]
    fn test_empty_compilation_unit() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:empty")),
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("package test:empty;"));
        assert!(wit_content.contains("world effect-lang {"));
        assert!(wit_content.contains("}"));
    }

    #[test]
    fn test_simple_interface_generation() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:interface")),
            modules: vec![Module {
                name: Symbol::new("TestModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("SimpleInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("greet"),
                                params: vec![
                                    Parameter {
                                        name: Symbol::new("name"),
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("interface SimpleInterface {"));
        assert!(wit_content.contains("greet: func(name: string) -> string"));
    }

    #[test]
    fn test_complex_interface_with_resources() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:resources")),
            modules: vec![Module {
                name: Symbol::new("ResourceModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("DatabaseInterface"),
                        visibility: Visibility::Component {
                            export: true,
                            import: false,
                            interface: Some(Symbol::new("database:api")),
                        },
                        items: vec![
                            InterfaceItem::Resource(ResourceDefinition {
                                name: Symbol::new("Connection"),
                                constructor: Some(ResourceMethod {
                                    name: Symbol::new("new"),
                                    method_type: MethodType::Constructor,
                                    params: vec![
                                        Parameter {
                                            name: Symbol::new("url"),
                                            wasm_type: WasmType::String,
                                        }
                                    ],
                                    results: vec![],
                                }),
                                methods: vec![
                                    ResourceMethod {
                                        name: Symbol::new("query"),
                                        method_type: MethodType::Method,
                                        params: vec![
                                            Parameter {
                                                name: Symbol::new("sql"),
                                                wasm_type: WasmType::String,
                                            }
                                        ],
                                        results: vec![WasmType::String],
                                    },
                                    ResourceMethod {
                                        name: Symbol::new("close"),
                                        method_type: MethodType::Method,
                                        params: vec![],
                                        results: vec![],
                                    },
                                ],
                            }),
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("connect"),
                                params: vec![
                                    Parameter {
                                        name: Symbol::new("url"),
                                        wasm_type: WasmType::String,
                                    }
                                ],
                                results: vec![WasmType::ExternRef], // Connection resource
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        println!("Generated WIT:\n{}", wit_content);
        
        assert!(wit_content.contains("interface DatabaseInterface {"));
        assert!(wit_content.contains("resource Connection {"));
        assert!(wit_content.contains("constructor new: func(url: string)"));
        assert!(wit_content.contains("query: func(sql: string) -> string"));
        assert!(wit_content.contains("close: func()"));
        assert!(wit_content.contains("connect: func(url: string) -> externref"));
    }

    #[test]
    fn test_wasm_type_conversions() {
        let generator = WitGenerator::new();
        
        assert_eq!(generator.wasm_type_to_wit(&WasmType::I32), "s32");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::I64), "s64");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::F32), "f32");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::F64), "f64");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::String), "string");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::Bool), "bool");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::List), "list");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::Option), "option");
        assert_eq!(generator.wasm_type_to_wit(&WasmType::Result), "result");
    }

    #[test]
    fn test_multiple_interfaces() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:multi")),
            modules: vec![Module {
                name: Symbol::new("MultiModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("MathInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("add"),
                                params: vec![
                                    Parameter { name: Symbol::new("x"), wasm_type: WasmType::F64 },
                                    Parameter { name: Symbol::new("y"), wasm_type: WasmType::F64 },
                                ],
                                results: vec![WasmType::F64],
                            }),
                        ],
                        span: Default::default(),
                    }),
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("StringInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("concat"),
                                params: vec![
                                    Parameter { name: Symbol::new("a"), wasm_type: WasmType::String },
                                    Parameter { name: Symbol::new("b"), wasm_type: WasmType::String },
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("interface MathInterface {"));
        assert!(wit_content.contains("add: func(x: f64, y: f64) -> f64"));
        assert!(wit_content.contains("interface StringInterface {"));
        assert!(wit_content.contains("concat: func(a: string, b: string) -> string"));
    }

    #[test]
    fn test_type_definitions() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:types")),
            modules: vec![Module {
                name: Symbol::new("TypeModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("Person"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Record(vec![
                            (Symbol::new("name"), Type::String),
                            (Symbol::new("age"), Type::Int),
                            (Symbol::new("email"), Type::Option(Box::new(Type::String))),
                        ]),
                        span: Default::default(),
                    }),
                    Item::TypeDef(TypeDef {
                        name: Symbol::new("Status"),
                        visibility: Visibility::Public,
                        definition: TypeDefinition::Variant(vec![
                            VariantDefinition {
                                name: Symbol::new("Active"),
                                data: None,
                            },
                            VariantDefinition {
                                name: Symbol::new("Inactive"),
                                data: Some(Type::String), // reason
                            },
                        ]),
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        println!("Generated WIT with types:\n{}", wit_content);
        
        assert!(wit_content.contains("record Person {"));
        assert!(wit_content.contains("name: string,"));
        assert!(wit_content.contains("age: s32,"));
        assert!(wit_content.contains("email: option<string>,"));
        
        assert!(wit_content.contains("variant Status {"));
        assert!(wit_content.contains("Active,"));
        assert!(wit_content.contains("Inactive(string),"));
    }

    #[test]
    fn test_imports_and_exports() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:import-export")),
            modules: vec![Module {
                name: Symbol::new("ImportExportModule"),
                visibility: Visibility::Public,
                items: vec![],
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
                    name: Symbol::new("env"),
                    kind: ImportKind::Core,
                    path: Some(Symbol::new("env")),
                    visibility: Visibility::Private,
                    span: Default::default(),
                },
            ],
            exports: vec![
                ExportDef {
                    name: Symbol::new("api"),
                    exported_name: Symbol::new("my:api@1.0.0"),
                    visibility: Visibility::Public,
                    span: Default::default(),
                },
            ],
        };

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("import wasi-filesystem: interface;"));
        assert!(wit_content.contains("import env: core;"));
        assert!(wit_content.contains("export api: my:api@1.0.0;"));
    }

    #[test]
    fn test_visibility_modifiers() {
        let mut generator = WitGenerator::new();
        
        // Test different visibility modifiers
        let public_vis = Visibility::Public;
        let component_export_vis = Visibility::Component {
            export: true,
            import: false,
            interface: Some(Symbol::new("test:api")),
        };
        let component_import_vis = Visibility::Component {
            export: false,
            import: true,
            interface: Some(Symbol::new("wasi:filesystem")),
        };
        let private_vis = Visibility::Private;

        assert!(generator.is_public_visibility(&public_vis));
        assert!(generator.is_public_visibility(&component_export_vis));
        assert!(!generator.is_public_visibility(&component_import_vis));
        assert!(!generator.is_public_visibility(&private_vis));

        // Test WIT export generation
        let export_stmt = generator.visibility_to_wit_export(&component_export_vis);
        assert!(export_stmt.is_some());
        assert!(export_stmt.unwrap().contains("export test:api"));

        let no_export = generator.visibility_to_wit_export(&private_vis);
        assert!(no_export.is_none());
    }

    #[test]
    fn test_function_signatures_with_multiple_returns() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:multi-return")),
            modules: vec![Module {
                name: Symbol::new("MultiReturnModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("MultiReturnInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("divide"),
                                params: vec![
                                    Parameter { name: Symbol::new("dividend"), wasm_type: WasmType::F64 },
                                    Parameter { name: Symbol::new("divisor"), wasm_type: WasmType::F64 },
                                ],
                                results: vec![WasmType::F64, WasmType::Bool], // quotient, success
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("divide: func(dividend: f64, divisor: f64) -> (f64, bool)"));
    }

    #[test]
    fn test_resource_with_static_methods() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:static-methods")),
            modules: vec![Module {
                name: Symbol::new("StaticModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("UtilInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Resource(ResourceDefinition {
                                name: Symbol::new("Utils"),
                                constructor: None,
                                methods: vec![
                                    ResourceMethod {
                                        name: Symbol::new("random"),
                                        method_type: MethodType::Static,
                                        params: vec![],
                                        results: vec![WasmType::F64],
                                    },
                                    ResourceMethod {
                                        name: Symbol::new("seed"),
                                        method_type: MethodType::Static,
                                        params: vec![
                                            Parameter { name: Symbol::new("value"), wasm_type: WasmType::I64 }
                                        ],
                                        results: vec![],
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("resource Utils {"));
        assert!(wit_content.contains("static random: func() -> f64"));
        assert!(wit_content.contains("static seed: func(value: s64)"));
    }

    #[test]
    fn test_error_handling() {
        let mut generator = WitGenerator::new();
        
        // Test with invalid compilation unit (this should not cause a panic)
        let invalid_cu = CompilationUnit {
            package_name: None, // Missing package name
            modules: vec![],
            imports: vec![],
            exports: vec![],
        };

        let result = generator.generate(&invalid_cu);
        assert!(result.is_ok()); // Should handle gracefully
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("world effect-lang"));
    }

    #[test]
    fn test_large_interface() {
        let mut generator = WitGenerator::new();
        
        // Create a large interface with many functions
        let mut functions = vec![];
        for i in 0..100 {
            functions.push(InterfaceItem::Function(FunctionSignature {
                name: Symbol::new(&format!("func_{}", i)),
                params: vec![
                    Parameter { name: Symbol::new("param"), wasm_type: WasmType::I32 }
                ],
                results: vec![WasmType::I32],
            }));
        }

        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:large")),
            modules: vec![Module {
                name: Symbol::new("LargeModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("LargeInterface"),
                        visibility: Visibility::Public,
                        items: functions,
                        span: Default::default(),
                    }),
                ],
                span: Default::default(),
            }],
            imports: vec![],
            exports: vec![],
        };

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("func_0: func(param: s32) -> s32"));
        assert!(wit_content.contains("func_99: func(param: s32) -> s32"));
        
        // Check that all functions are present
        for i in 0..100 {
            assert!(wit_content.contains(&format!("func_{}", i)));
        }
    }
}

#[cfg(test)]
mod wit_edge_cases {
    use super::*;

    #[test]
    fn test_empty_interface() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:empty-interface")),
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("interface EmptyInterface {"));
        assert!(wit_content.contains("}"));
    }

    #[test]
    fn test_resource_without_constructor() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:no-constructor")),
            modules: vec![Module {
                name: Symbol::new("NoConstructorModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("NoConstructorInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Resource(ResourceDefinition {
                                name: Symbol::new("NoConstructor"),
                                constructor: None, // No constructor
                                methods: vec![
                                    ResourceMethod {
                                        name: Symbol::new("method"),
                                        method_type: MethodType::Method,
                                        params: vec![],
                                        results: vec![WasmType::String],
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("resource NoConstructor {"));
        assert!(wit_content.contains("method: func() -> string"));
        assert!(!wit_content.contains("constructor"));
    }

    #[test]
    fn test_special_characters_in_names() {
        let mut generator = WitGenerator::new();
        let cu = CompilationUnit {
            package_name: Some(Symbol::new("test:special-chars")),
            modules: vec![Module {
                name: Symbol::new("SpecialModule"),
                visibility: Visibility::Public,
                items: vec![
                    Item::InterfaceDef(ComponentInterface {
                        name: Symbol::new("SpecialInterface"),
                        visibility: Visibility::Public,
                        items: vec![
                            InterfaceItem::Function(FunctionSignature {
                                name: Symbol::new("kebab-case-function"),
                                params: vec![
                                    Parameter { 
                                        name: Symbol::new("snake_case_param"), 
                                        wasm_type: WasmType::String 
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

        let result = generator.generate(&cu);
        assert!(result.is_ok());
        
        let wit_content = result.unwrap();
        assert!(wit_content.contains("kebab-case-function: func(snake_case_param: string) -> string"));
    }
}