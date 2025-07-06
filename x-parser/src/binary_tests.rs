//! Tests for binary serialization/deserialization round-trip compatibility
//! 
//! These tests verify that the binary serializer and deserializer can correctly
//! round-trip AST structures without losing information.

#[cfg(test)]
mod tests {
    use crate::{
        ast::*,
        span::{Span, FileId, ByteOffset},
        symbol::Symbol,
        binary::{BinarySerializer, BinaryDeserializer},
    };

    /// Create a test span for testing purposes
    fn test_span() -> Span {
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(10))
    }

    /// Test round-trip serialization of simple literals
    #[test]
    fn test_literal_round_trip() {
        let test_cases = vec![
            Literal::Integer(42),
            Literal::Float(3.14),
            Literal::String("hello".to_string()),
            Literal::Bool(true),
            Literal::Bool(false),
            Literal::Unit,
        ];

        for literal in test_cases {
            let expr = Expr::Literal(literal.clone(), test_span());
            
            // Create a simple compilation unit with this expression
            let module = Module {
                name: ModulePath::single(Symbol::intern("test"), test_span()),
                exports: None,
                imports: Vec::new(),
                items: vec![Item::ValueDef(ValueDef {
                    name: Symbol::intern("test_value"),
                    documentation: None,
                    type_annotation: None,
                    parameters: Vec::new(),
                    body: expr,
                    visibility: Visibility::Public,
                    purity: Purity::Pure,
                    imports: Vec::new(),
                    span: test_span(),
                })],
                span: test_span(),
            };
            
            let compilation_unit = CompilationUnit {
                module,
                span: test_span(),
            };

            // Serialize
            let mut serializer = BinarySerializer::new();
            let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
                .expect("Failed to serialize compilation unit");

            // Deserialize
            let mut deserializer = BinaryDeserializer::new(binary_data)
                .expect("Failed to create deserializer");
            let restored_unit = deserializer.deserialize_compilation_unit()
                .expect("Failed to deserialize compilation unit");

            // Verify the literal is preserved
            if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
                if let Expr::Literal(restored_literal, _) = &value_def.body {
                    assert_eq!(restored_literal, &literal, "Literal not preserved: {:?}", literal);
                } else {
                    panic!("Expected literal expression");
                }
            } else {
                panic!("Expected value definition");
            }
        }
    }

    /// Test round-trip serialization of variable expressions
    #[test]
    fn test_variable_round_trip() {
        let var_name = Symbol::intern("my_variable");
        let expr = Expr::Var(var_name, test_span());
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_value"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the variable is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::Var(restored_var_name, _) = &value_def.body {
                assert_eq!(restored_var_name, &var_name, "Variable name not preserved");
            } else {
                panic!("Expected variable expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization of function application
    #[test]
    fn test_application_round_trip() {
        let func = Box::new(Expr::Var(Symbol::intern("add"), test_span()));
        let args = vec![
            Expr::Literal(Literal::Integer(1), test_span()),
            Expr::Literal(Literal::Integer(2), test_span()),
        ];
        let expr = Expr::App(func, args, test_span());
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_value"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the application is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::App(restored_func, restored_args, _) = &value_def.body {
                if let Expr::Var(func_name, _) = restored_func.as_ref() {
                    assert_eq!(func_name, &Symbol::intern("add"), "Function name not preserved");
                } else {
                    panic!("Expected variable function");
                }
                
                assert_eq!(restored_args.len(), 2, "Argument count not preserved");
                
                if let Expr::Literal(Literal::Integer(n1), _) = &restored_args[0] {
                    assert_eq!(*n1, 1, "First argument not preserved");
                } else {
                    panic!("Expected integer literal for first argument");
                }
                
                if let Expr::Literal(Literal::Integer(n2), _) = &restored_args[1] {
                    assert_eq!(*n2, 2, "Second argument not preserved");
                } else {
                    panic!("Expected integer literal for second argument");
                }
            } else {
                panic!("Expected application expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization of lambda expressions
    #[test]
    fn test_lambda_round_trip() {
        let param = Pattern::Variable(Symbol::intern("x"), test_span());
        let body = Box::new(Expr::Var(Symbol::intern("x"), test_span()));
        let expr = Expr::Lambda {
            parameters: vec![param],
            body,
            span: test_span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_value"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the lambda is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::Lambda { parameters, body, .. } = &value_def.body {
                assert_eq!(parameters.len(), 1, "Parameter count not preserved");
                
                if let Pattern::Variable(param_name, _) = &parameters[0] {
                    assert_eq!(param_name, &Symbol::intern("x"), "Parameter name not preserved");
                } else {
                    panic!("Expected variable pattern");
                }
                
                if let Expr::Var(body_var, _) = body.as_ref() {
                    assert_eq!(body_var, &Symbol::intern("x"), "Body variable not preserved");
                } else {
                    panic!("Expected variable in lambda body");
                }
            } else {
                panic!("Expected lambda expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization of let expressions
    #[test]
    fn test_let_round_trip() {
        let pattern = Pattern::Variable(Symbol::intern("y"), test_span());
        let value = Box::new(Expr::Literal(Literal::Integer(42), test_span()));
        let body = Box::new(Expr::Var(Symbol::intern("y"), test_span()));
        
        let expr = Expr::Let {
            pattern,
            type_annotation: None,
            value,
            body,
            span: test_span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_value"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the let expression is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::Let { pattern, value, body, .. } = &value_def.body {
                if let Pattern::Variable(var_name, _) = pattern {
                    assert_eq!(var_name, &Symbol::intern("y"), "Let pattern variable not preserved");
                } else {
                    panic!("Expected variable pattern in let");
                }
                
                if let Expr::Literal(Literal::Integer(n), _) = value.as_ref() {
                    assert_eq!(*n, 42, "Let value not preserved");
                } else {
                    panic!("Expected integer literal in let value");
                }
                
                if let Expr::Var(body_var, _) = body.as_ref() {
                    assert_eq!(body_var, &Symbol::intern("y"), "Let body variable not preserved");
                } else {
                    panic!("Expected variable in let body");
                }
            } else {
                panic!("Expected let expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization of function types
    #[test]
    fn test_function_type_round_trip() {
        let param_type = Type::Con(Symbol::intern("Int"), test_span());
        let return_type = Box::new(Type::Con(Symbol::intern("Int"), test_span()));
        let effects = EffectSet::empty(test_span());
        
        let func_type = Type::Fun {
            params: vec![param_type],
            return_type,
            effects,
            span: test_span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_func"),
                documentation: None,
                type_annotation: Some(func_type),
                parameters: Vec::new(),
                body: Expr::Literal(Literal::Unit, test_span()),
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the function type is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Some(Type::Fun { params, return_type, .. }) = &value_def.type_annotation {
                assert_eq!(params.len(), 1, "Function parameter count not preserved");
                
                if let Type::Con(param_name, _) = &params[0] {
                    assert_eq!(param_name, &Symbol::intern("Int"), "Function parameter type not preserved");
                } else {
                    panic!("Expected constructor type for parameter");
                }
                
                if let Type::Con(return_name, _) = return_type.as_ref() {
                    assert_eq!(return_name, &Symbol::intern("Int"), "Function return type not preserved");
                } else {
                    panic!("Expected constructor type for return");
                }
            } else {
                panic!("Expected function type annotation");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization of complex nested expressions
    #[test]
    fn test_complex_expression_round_trip() {
        // Create: let add = fun x y -> x + y in add 1 2
        let x_param = Pattern::Variable(Symbol::intern("x"), test_span());
        let y_param = Pattern::Variable(Symbol::intern("y"), test_span());
        
        let add_body = Box::new(Expr::App(
            Box::new(Expr::Var(Symbol::intern("+"), test_span())),
            vec![
                Expr::Var(Symbol::intern("x"), test_span()),
                Expr::Var(Symbol::intern("y"), test_span()),
            ],
            test_span(),
        ));
        
        let add_lambda = Expr::Lambda {
            parameters: vec![x_param, y_param],
            body: add_body,
            span: test_span(),
        };
        
        let add_pattern = Pattern::Variable(Symbol::intern("add"), test_span());
        let add_value = Box::new(add_lambda);
        
        let use_add = Box::new(Expr::App(
            Box::new(Expr::Var(Symbol::intern("add"), test_span())),
            vec![
                Expr::Literal(Literal::Integer(1), test_span()),
                Expr::Literal(Literal::Integer(2), test_span()),
            ],
            test_span(),
        ));
        
        let expr = Expr::Let {
            pattern: add_pattern,
            type_annotation: None,
            value: add_value,
            body: use_add,
            span: test_span(),
        };
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_complex"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the complex expression structure is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::Let { pattern, value, body, .. } = &value_def.body {
                // Check let pattern
                if let Pattern::Variable(let_var, _) = pattern {
                    assert_eq!(let_var, &Symbol::intern("add"), "Let variable not preserved");
                } else {
                    panic!("Expected variable pattern in complex let");
                }
                
                // Check lambda value
                if let Expr::Lambda { parameters, body: lambda_body, .. } = value.as_ref() {
                    assert_eq!(parameters.len(), 2, "Lambda parameter count not preserved");
                    
                    // Check lambda body (should be application of +)
                    if let Expr::App(func, args, _) = lambda_body.as_ref() {
                        if let Expr::Var(op_name, _) = func.as_ref() {
                            assert_eq!(op_name, &Symbol::intern("+"), "Lambda operator not preserved");
                        } else {
                            panic!("Expected operator variable in lambda");
                        }
                        assert_eq!(args.len(), 2, "Lambda body argument count not preserved");
                    } else {
                        panic!("Expected application in lambda body");
                    }
                } else {
                    panic!("Expected lambda in let value");
                }
                
                // Check use of add function
                if let Expr::App(use_func, use_args, _) = body.as_ref() {
                    if let Expr::Var(use_name, _) = use_func.as_ref() {
                        assert_eq!(use_name, &Symbol::intern("add"), "Use function name not preserved");
                    } else {
                        panic!("Expected variable in use function");
                    }
                    assert_eq!(use_args.len(), 2, "Use argument count not preserved");
                } else {
                    panic!("Expected application in let body");
                }
            } else {
                panic!("Expected complex let expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }

    /// Test round-trip serialization preserves spans
    #[test]
    fn test_span_preservation() {
        let custom_span = Span::new(FileId::new(42), ByteOffset::new(100), ByteOffset::new(200));
        let expr = Expr::Literal(Literal::Integer(123), custom_span);
        
        let module = Module {
            name: ModulePath::single(Symbol::intern("test"), test_span()),
            exports: None,
            imports: Vec::new(),
            items: vec![Item::ValueDef(ValueDef {
                name: Symbol::intern("test_value"),
                documentation: None,
                type_annotation: None,
                parameters: Vec::new(),
                body: expr,
                visibility: Visibility::Public,
                purity: Purity::Pure,
                imports: Vec::new(),
                span: test_span(),
            })],
            span: test_span(),
        };
        
        let compilation_unit = CompilationUnit {
            module,
            span: test_span(),
        };

        // Serialize
        let mut serializer = BinarySerializer::new();
        let binary_data = serializer.serialize_compilation_unit(&compilation_unit)
            .expect("Failed to serialize compilation unit");

        // Deserialize
        let mut deserializer = BinaryDeserializer::new(binary_data)
            .expect("Failed to create deserializer");
        let restored_unit = deserializer.deserialize_compilation_unit()
            .expect("Failed to deserialize compilation unit");

        // Verify the span is preserved
        if let Item::ValueDef(value_def) = &restored_unit.module.items[0] {
            if let Expr::Literal(_, restored_span) = &value_def.body {
                assert_eq!(restored_span.file_id.as_u32(), 42, "File ID not preserved");
                assert_eq!(restored_span.start.as_u32(), 100, "Start offset not preserved");
                assert_eq!(restored_span.end.as_u32(), 200, "End offset not preserved");
            } else {
                panic!("Expected literal expression");
            }
        } else {
            panic!("Expected value definition");
        }
    }
}