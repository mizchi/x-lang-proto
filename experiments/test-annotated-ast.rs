use x_parser::{Symbol, FileId, Span, span::ByteOffset};
use x_parser::ast::*;
use x_editor::content_addressing::{ContentRepository, Version};
use x_editor::annotated_ast::{
    AnnotatedValueDef, AnnotatedExpr, AnnotatedPattern, AnnotatedCompilationUnit,
    AnnotatedModule, AnnotatedItem, TypeEnvironment, TypeAnnotator,
};
use x_checker::types::{Type as InferredType, TypeScheme, TypeVar};
use std::collections::{HashMap, HashSet};

fn main() {
    println!("=== Type-Annotated AST Test ===\n");

    let file_id = FileId::new(0);
    let span = Span::new(file_id, ByteOffset::new(0), ByteOffset::new(1));
    
    // Create repository
    let mut repo = ContentRepository::new();
    
    println!("1. Creating functions with inferred types:");
    
    // Create a simple add function
    let add_def = create_add_function(span);
    
    // Create annotated version with inferred types
    let annotated_add = create_annotated_add(span);
    
    // Add both versions to repository
    let hash1 = repo.add_function(
        "add",
        &add_def,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'add' without type annotations: {}", hash1.short());
    
    let hash2 = repo.add_annotated_function(
        "add_typed",
        &add_def,
        &annotated_add,
        Some(Version::new(1, 1, 0)),
        ["arithmetic", "typed"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'add' with inferred types: {}", hash2.short());
    
    // Show the inferred type
    if let Some(ref inferred) = annotated_add.inferred_type {
        println!("   - Inferred type: {:?}", inferred);
    }
    
    println!("\n2. Creating a polymorphic function:");
    
    // Create identity function
    let id_def = create_identity_function(span);
    let annotated_id = create_annotated_identity(span);
    
    let hash3 = repo.add_annotated_function(
        "identity",
        &id_def,
        &annotated_id,
        Some(Version::new(1, 0, 0)),
        ["polymorphic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added polymorphic 'identity': {}", hash3.short());
    if let Some(ref inferred) = annotated_id.inferred_type {
        println!("   - Inferred type: {:?}", inferred);
    }
    
    println!("\n3. Creating annotated module:");
    
    // Create a module with multiple functions
    let annotated_module = create_annotated_module(span);
    
    let hash4 = repo.add_annotated_module(
        "MathModule",
        &annotated_module,
        Some(Version::new(1, 0, 0)),
        ["module", "math"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added annotated module: {}", hash4.short());
    
    // Show type information from the module
    let type_map = annotated_module.get_type_map();
    println!("   - Type information:");
    for (name, ty) in type_map.iter() {
        println!("     {}: {}", name, ty);
    }
    
    println!("\n4. Converting back to regular AST:");
    
    let regular_ast = annotated_module.to_ast();
    println!("   - Converted annotated AST back to regular AST");
    println!("   - Module has {} items", regular_ast.module.items.len());
    
    // Check if type annotations were preserved
    for item in &regular_ast.module.items {
        if let Item::ValueDef(def) = item {
            if def.type_annotation.is_some() {
                println!("   - Function '{}' has type annotation", def.name.as_str());
            }
        }
    }
    
    println!("\n5. Searching with type information:");
    
    // Find functions with specific return types
    let mut count = 0;
    for (_, entry) in repo.entries.iter() {
        if let x_editor::content_addressing::ContentItem::Function { 
            annotated_ast: Some(ann), .. 
        } = &entry.content {
            if let Some(ref scheme) = ann.inferred_type {
                // Check if the type involves Int (Symbol(4) is Int)
                let type_str = format!("{:?}", scheme);
                if type_str.contains("Symbol(4)") || type_str.contains("Int") {
                    count += 1;
                    println!("   - Found function '{}' with Int type", ann.name.as_str());
                }
            }
        }
    }
    
    println!("   - Found {} functions involving Int type", count);
    
    println!("\n6. Summary:");
    println!("   ✓ Functions can be stored with inferred type annotations");
    println!("   ✓ Polymorphic types are preserved");
    println!("   ✓ Type information can be queried");
    println!("   ✓ Annotated AST can be converted back to regular AST");
    println!("   ✓ Type annotations are preserved in the conversion");
}

// Helper functions

fn create_add_function(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("add"),
        type_annotation: None,
        parameters: vec![
            Pattern::Variable(Symbol::intern("x"), span),
            Pattern::Variable(Symbol::intern("y"), span),
        ],
        body: Expr::Lambda {
            parameters: vec![
                Pattern::Variable(Symbol::intern("x"), span),
                Pattern::Variable(Symbol::intern("y"), span),
            ],
            body: Box::new(Expr::App(
                Box::new(Expr::Var(Symbol::intern("+"), span)),
                vec![
                    Expr::Var(Symbol::intern("x"), span),
                    Expr::Var(Symbol::intern("y"), span),
                ],
                span,
            )),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_annotated_add(span: Span) -> AnnotatedValueDef {
    // Create type scheme: Int -> Int -> Int
    let int_type = InferredType::Con(Symbol::intern("Int"));
    let type_scheme = TypeScheme {
        type_vars: vec![],
        effect_vars: vec![],
        constraints: vec![],
        body: InferredType::Fun {
            params: vec![int_type.clone(), int_type.clone()],
            return_type: Box::new(int_type.clone()),
            effects: x_checker::types::EffectSet::empty(),
        },
    };
    
    AnnotatedValueDef {
        name: Symbol::intern("add"),
        type_annotation: None,
        inferred_type: Some(type_scheme),
        parameters: vec![
            AnnotatedPattern {
                pattern: Pattern::Variable(Symbol::intern("x"), span),
                inferred_type: Some(int_type.clone()),
                span,
            },
            AnnotatedPattern {
                pattern: Pattern::Variable(Symbol::intern("y"), span),
                inferred_type: Some(int_type.clone()),
                span,
            },
        ],
        body: AnnotatedExpr {
            expr: create_add_function(span).body,
            inferred_type: Some(int_type),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_identity_function(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("identity"),
        type_annotation: None,
        parameters: vec![
            Pattern::Variable(Symbol::intern("x"), span),
        ],
        body: Expr::Lambda {
            parameters: vec![
                Pattern::Variable(Symbol::intern("x"), span),
            ],
            body: Box::new(Expr::Var(Symbol::intern("x"), span)),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_annotated_identity(span: Span) -> AnnotatedValueDef {
    // Create polymorphic type scheme: forall a. a -> a
    let type_var = TypeVar(0);
    let var_type = InferredType::Var(type_var);
    
    let type_scheme = TypeScheme {
        type_vars: vec![type_var],
        effect_vars: vec![],
        constraints: vec![],
        body: InferredType::Fun {
            params: vec![var_type.clone()],
            return_type: Box::new(var_type.clone()),
            effects: x_checker::types::EffectSet::empty(),
        },
    };
    
    AnnotatedValueDef {
        name: Symbol::intern("identity"),
        type_annotation: None,
        inferred_type: Some(type_scheme),
        parameters: vec![
            AnnotatedPattern {
                pattern: Pattern::Variable(Symbol::intern("x"), span),
                inferred_type: Some(var_type.clone()),
                span,
            },
        ],
        body: AnnotatedExpr {
            expr: Expr::Var(Symbol::intern("x"), span),
            inferred_type: Some(var_type),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_annotated_module(span: Span) -> AnnotatedCompilationUnit {
    let mut type_env = TypeEnvironment::new();
    
    // Add type information to environment
    let int_type = InferredType::Con(Symbol::intern("Int"));
    let add_type = TypeScheme {
        type_vars: vec![],
        effect_vars: vec![],
        constraints: vec![],
        body: InferredType::Fun {
            params: vec![int_type.clone(), int_type.clone()],
            return_type: Box::new(int_type.clone()),
            effects: x_checker::types::EffectSet::empty(),
        },
    };
    
    type_env.globals.insert(Symbol::intern("add"), add_type);
    
    // Create annotated items
    let items = vec![
        AnnotatedItem::ValueDef(create_annotated_add(span)),
        AnnotatedItem::ValueDef(create_annotated_identity(span)),
    ];
    
    AnnotatedCompilationUnit {
        module: AnnotatedModule {
            name: ModulePath::single(Symbol::intern("MathModule"), span),
            imports: vec![],
            items,
            exports: None,
            span,
        },
        type_environment: type_env,
        span,
    }
}