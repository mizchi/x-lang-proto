//! Example: Code generator using AST builder
//! 
//! This example demonstrates generating x Language code from
//! a simple configuration or schema.

use x_ast_builder::*;
use x_parser::ast::*;
use x_parser::{Symbol, FileId};
use x_parser::syntax::sexp::SExpPrinter;
use x_parser::syntax::{SyntaxPrinter, SyntaxConfig, SyntaxStyle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct FieldDef {
    name: String,
    typ: String,
    optional: bool,
}

#[derive(Debug, Clone)]
struct RecordDef {
    name: String,
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
struct ApiEndpoint {
    name: String,
    method: String,
    path: String,
    request_type: String,
    response_type: String,
}

fn main() {
    println!("=== Code Generation Examples ===\n");
    
    // Example 1: Generate record types with getters/setters
    example_record_generation();
    
    // Example 2: Generate API client from schema
    example_api_client_generation();
    
    // Example 3: Generate serialization/deserialization functions
    example_serialization_generation();
}

fn example_record_generation() {
    println!("Example 1: Record Type Generation");
    println!("---------------------------------");
    
    let user_record = RecordDef {
        name: "User".to_string(),
        fields: vec![
            FieldDef { name: "id".to_string(), typ: "Int".to_string(), optional: false },
            FieldDef { name: "name".to_string(), typ: "String".to_string(), optional: false },
            FieldDef { name: "email".to_string(), typ: "String".to_string(), optional: false },
            FieldDef { name: "age".to_string(), typ: "Int".to_string(), optional: true },
        ],
    };
    
    let module = generate_record_module(&user_record);
    print_module(&module);
    println!();
}

fn example_api_client_generation() {
    println!("Example 2: API Client Generation");
    println!("--------------------------------");
    
    let endpoints = vec![
        ApiEndpoint {
            name: "getUser".to_string(),
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
            request_type: "Int".to_string(),
            response_type: "User".to_string(),
        },
        ApiEndpoint {
            name: "createUser".to_string(),
            method: "POST".to_string(),
            path: "/users".to_string(),
            request_type: "CreateUserRequest".to_string(),
            response_type: "User".to_string(),
        },
        ApiEndpoint {
            name: "listUsers".to_string(),
            method: "GET".to_string(),
            path: "/users".to_string(),
            request_type: "Unit".to_string(),
            response_type: "List[User]".to_string(),
        },
    ];
    
    let module = generate_api_client("UserApi", &endpoints);
    print_module(&module);
    println!();
}

fn example_serialization_generation() {
    println!("Example 3: Serialization Function Generation");
    println!("--------------------------------------------");
    
    let types = vec![
        ("Point", vec![("x", "Int"), ("y", "Int")]),
        ("Color", vec![("r", "Int"), ("g", "Int"), ("b", "Int"), ("a", "Float")]),
    ];
    
    let module = generate_serialization_module(&types);
    print_module(&module);
    println!();
}

// Code generation functions

fn generate_record_module(record: &RecordDef) -> Module {
    let mut builder = AstBuilder::new();
    let mut module_builder = builder.module(&format!("{}Module", record.name));
    
    // Generate the record type constructor
    let field_types: Vec<(&str, Vec<&str>)> = vec![
        (&record.name, record.fields.iter()
            .map(|f| f.typ.as_str())
            .collect())
    ];
    
    module_builder = module_builder.data_type(&record.name, field_types);
    
    // Generate create function
    let param_names: Vec<&str> = record.fields.iter()
        .map(|f| f.name.as_str())
        .collect();
    
    module_builder = module_builder.function(
        &format!("create{}", record.name),
        param_names.clone(),
        |b| {
            // Create constructor application
            let args: Vec<Box<dyn Fn(&mut AstBuilder) -> Expr>> = record.fields.iter()
                .map(|f| {
                    let name = f.name.clone();
                    Box::new(move |b: &mut AstBuilder| b.expr().var(&name)) as Box<dyn Fn(&mut AstBuilder) -> Expr>
                })
                .collect();
            
            b.app(&record.name, args)
        }
    );
    
    // Generate getter functions
    for (i, field) in record.fields.iter().enumerate() {
        let field_name = field.name.clone();
        let record_name = record.name.clone();
        let index = i;
        
        module_builder = module_builder.function(
            &format!("get{}", capitalize(&field_name)),
            vec!["record"],
            move |b| {
                // Pattern match to extract field
                let pattern = Pattern::Constructor {
                    name: Symbol::intern(&record_name),
                    args: (0..record.fields.len()).map(|j| {
                        if j == index {
                            Pattern::Variable(Symbol::intern(&field_name), b.span())
                        } else {
                            Pattern::Wildcard(b.span())
                        }
                    }).collect(),
                    span: b.span(),
                };
                
                b.expr().match_expr(
                    |b| b.expr().var("record"),
                    vec![(pattern, |b| b.expr().var(&field_name))]
                )
            }
        );
    }
    
    module_builder
}

fn generate_api_client(name: &str, endpoints: &[ApiEndpoint]) -> Module {
    let mut builder = AstBuilder::new();
    let mut module_builder = builder.module(name);
    
    // Import HTTP library
    module_builder = module_builder
        .import("Http")
        .import("Json");
    
    // Generate base URL constant
    module_builder = module_builder.value("baseUrl", |b| {
        b.expr().string("https://api.example.com")
    });
    
    // Generate endpoint functions
    for endpoint in endpoints {
        let method = endpoint.method.clone();
        let path = endpoint.path.clone();
        let response_type = endpoint.response_type.clone();
        
        module_builder = module_builder.function(
            &endpoint.name,
            if endpoint.request_type == "Unit" { vec![] } else { vec!["request"] },
            move |b| {
                // Build URL
                let url_expr = if path.contains("{") {
                    // Template substitution needed
                    b.app("formatUrl", vec![
                        |b| b.expr().string(&path),
                        |b| b.expr().var("request"),
                    ])
                } else {
                    b.expr().binop("^",
                        |b| b.expr().var("baseUrl"),
                        |b| b.expr().string(&path)
                    )
                };
                
                // Make HTTP request
                b.expr().let_in("response",
                    move |b| {
                        b.app(&format!("Http.{}", method.to_lowercase()), vec![
                            move |_| url_expr.clone(),
                            |b| if endpoint.request_type == "Unit" {
                                b.expr().unit()
                            } else {
                                b.app("Json.encode", vec![
                                    |b| b.expr().var("request")
                                ])
                            },
                        ])
                    },
                    |b| {
                        // Decode response
                        b.app(&format!("Json.decode{}", response_type.replace("List[", "List").replace("]", "")), vec![
                            |b| b.expr().var("response")
                        ])
                    }
                )
            }
        );
    }
    
    module_builder
}

fn generate_serialization_module(types: &[(&str, Vec<(&str, &str)>)]) -> Module {
    let mut builder = AstBuilder::new();
    let mut module_builder = builder.module("Serialization");
    
    // Import JSON library
    module_builder = module_builder.import("Json");
    
    for (type_name, fields) in types {
        // Generate toJson function
        let type_name_owned = type_name.to_string();
        let fields_owned: Vec<(String, String)> = fields.iter()
            .map(|(n, t)| (n.to_string(), t.to_string()))
            .collect();
        
        module_builder = module_builder.function(
            &format!("{}ToJson", lowercase(&type_name_owned)),
            vec!["value"],
            move |b| {
                // Pattern match to extract all fields
                let field_names: Vec<String> = fields_owned.iter()
                    .map(|(name, _)| name.clone())
                    .collect();
                
                let pattern = Pattern::Constructor {
                    name: Symbol::intern(&type_name_owned),
                    args: field_names.iter()
                        .map(|name| Pattern::Variable(Symbol::intern(name), b.span()))
                        .collect(),
                    span: b.span(),
                };
                
                b.expr().match_expr(
                    |b| b.expr().var("value"),
                    vec![(pattern, move |b| {
                        // Create JSON object
                        let field_exprs: Vec<Box<dyn Fn(&mut AstBuilder) -> Expr>> = field_names.iter()
                            .map(|name| {
                                let n = name.clone();
                                Box::new(move |b: &mut AstBuilder| {
                                    b.app("Json.field", vec![
                                        move |b| b.expr().string(&n),
                                        move |b| b.expr().var(&n),
                                    ])
                                }) as Box<dyn Fn(&mut AstBuilder) -> Expr>
                            })
                            .collect();
                        
                        b.app("Json.object", field_exprs)
                    })]
                )
            }
        );
        
        // Generate fromJson function
        let type_name_owned = type_name.to_string();
        let fields_owned: Vec<(String, String)> = fields.iter()
            .map(|(n, t)| (n.to_string(), t.to_string()))
            .collect();
        
        module_builder = module_builder.function(
            &format!("{}FromJson", lowercase(&type_name_owned)),
            vec!["json"],
            move |b| {
                // Extract each field and construct the record
                let mut expr = b.expr();
                
                // Build nested let expressions for each field
                for (i, (field_name, field_type)) in fields_owned.iter().enumerate() {
                    let is_last = i == fields_owned.len() - 1;
                    let field_name_owned = field_name.clone();
                    let field_type_owned = field_type.clone();
                    
                    if is_last {
                        // Last field: construct the record
                        expr = expr.let_in(&field_name_owned,
                            move |b| {
                                b.app(&format!("Json.get{}", capitalize(&field_type_owned)), vec![
                                    |b| b.expr().string(&field_name_owned),
                                    |b| b.expr().var("json"),
                                ])
                            },
                            move |b| {
                                // Construct the record
                                let args: Vec<Box<dyn Fn(&mut AstBuilder) -> Expr>> = fields_owned.iter()
                                    .map(|(name, _)| {
                                        let n = name.clone();
                                        Box::new(move |b: &mut AstBuilder| b.expr().var(&n)) as Box<dyn Fn(&mut AstBuilder) -> Expr>
                                    })
                                    .collect();
                                
                                b.app(&type_name_owned, args)
                            }
                        );
                    } else {
                        expr = expr.let_in(&field_name_owned,
                            move |b| {
                                b.app(&format!("Json.get{}", capitalize(&field_type_owned)), vec![
                                    |b| b.expr().string(&field_name_owned),
                                    |b| b.expr().var("json"),
                                ])
                            },
                            |b| b.expr().unit() // Placeholder, will be replaced
                        );
                    }
                }
                
                expr
            }
        );
    }
    
    module_builder
}

// Helper functions

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn lowercase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

fn print_module(module: &Module) {
    let cu = CompilationUnit {
        module: module.clone(),
        span: module.span,
    };
    
    let config = SyntaxConfig {
        style: SyntaxStyle::SExpression,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = SExpPrinter::new();
    match printer.print(&cu, &config) {
        Ok(code) => println!("{}", code),
        Err(e) => println!("Error printing: {:?}", e),
    }
}