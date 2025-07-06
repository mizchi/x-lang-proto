use x_parser::{Symbol, FileId, Span, span::ByteOffset};
use x_parser::ast::*;
use x_editor::content_addressing::{
    ContentRepository, ContentHash, Version, SearchQuery, FeatureConstraints,
};
use std::collections::HashSet;

fn main() {
    println!("=== Content Addressing Test ===\n");

    let file_id = FileId::new(0);
    let span = Span::new(file_id, ByteOffset::new(0), ByteOffset::new(1));
    
    // Create repository
    let mut repo = ContentRepository::new();
    
    println!("1. Adding functions to repository:");
    
    // Add version 1.0.0 of add function
    let add_v1 = create_add_function("+", span);
    let hash_v1 = repo.add_function(
        "add",
        &add_v1,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic", "basic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'add' v1.0.0: {}", hash_v1.short());
    
    // Add version 1.1.0 with type annotation
    let add_v2 = create_add_function_typed("+", span);
    let hash_v2 = repo.add_function(
        "add",
        &add_v2,
        None,
        Some(Version::new(1, 1, 0)),
        ["arithmetic", "basic", "typed"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'add' v1.1.0: {}", hash_v2.short());
    
    // Add similar function (multiply)
    let multiply = create_add_function("*", span);
    let hash_mul = repo.add_function(
        "multiply",
        &multiply,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic", "basic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'multiply' v1.0.0: {}", hash_mul.short());
    
    // Add recursive function
    let factorial = create_factorial_function(span);
    let hash_fac = repo.add_function(
        "factorial",
        &factorial,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic", "recursive"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    
    println!("   - Added 'factorial' v1.0.0: {}", hash_fac.short());
    
    println!("\n2. Version listing:");
    
    let add_versions = repo.list_versions("add");
    for (version, hash) in add_versions {
        println!("   - add v{}: {}", version.to_string(), hash.short());
    }
    
    println!("\n3. Finding similar functions:");
    
    // Create a subtract function to find similar ones
    let subtract = create_add_function("-", span);
    let similar = repo.find_similar_functions(&subtract, 0.5).unwrap();
    
    println!("   Similar to 'subtract' function:");
    for (hash, similarity) in similar.iter().take(3) {
        if let Some(entry) = repo.get(hash) {
            println!("   - {} ({}): {:.2}% similar", 
                entry.name, 
                entry.version.as_ref().map(|v| v.to_string()).unwrap_or("unversioned".to_string()),
                similarity * 100.0
            );
        }
    }
    
    println!("\n4. Searching by features:");
    
    // Search for recursive functions
    let mut query = SearchQuery {
        name_pattern: None,
        tags: HashSet::new(),
        feature_constraints: Some(FeatureConstraints {
            min_params: None,
            max_params: None,
            is_recursive: Some(true),
            required_operations: HashSet::new(),
        }),
        similarity_threshold: None,
    };
    
    let recursive_results = repo.search(&query).unwrap();
    println!("   Recursive functions:");
    for entry in recursive_results {
        println!("   - {} v{}", 
            entry.name,
            entry.version.as_ref().map(|v| v.to_string()).unwrap_or("unversioned".to_string())
        );
    }
    
    // Search by tags
    query.feature_constraints = None;
    query.tags = ["arithmetic", "basic"].iter().map(|s| s.to_string()).collect();
    
    let tag_results = repo.search(&query).unwrap();
    println!("\n   Functions tagged 'arithmetic' and 'basic':");
    for entry in tag_results {
        println!("   - {} v{}", 
            entry.name,
            entry.version.as_ref().map(|v| v.to_string()).unwrap_or("unversioned".to_string())
        );
    }
    
    println!("\n5. Content hash properties:");
    
    // Show that identical structures have same hash
    let add_copy = create_add_function("+", span);
    let similar_to_v1 = repo.find_similar_functions(&add_copy, 0.99).unwrap();
    
    println!("   Functions identical to 'add' structure:");
    for (hash, similarity) in similar_to_v1 {
        if similarity == 1.0 {
            if let Some(entry) = repo.get(&hash) {
                println!("   - {} has exact same structure", entry.name);
            }
        }
    }
    
    println!("\n6. Semantic fingerprinting:");
    
    // Show normalized forms
    if let Some(add_entry) = repo.get(&hash_v1) {
        println!("   'add' normalized form: {}", add_entry.fingerprint.normalized_form);
    }
    if let Some(mul_entry) = repo.get(&hash_mul) {
        println!("   'multiply' normalized form: {}", mul_entry.fingerprint.normalized_form);
    }
    
    println!("\n7. Summary:");
    println!("   ✓ Functions can be stored with content-based addresses");
    println!("   ✓ Version management is supported");
    println!("   ✓ Similar functions can be found by structure");
    println!("   ✓ Search by features and tags is available");
    println!("   ✓ Semantic fingerprinting enables similarity matching");
}

// Helper functions to create AST nodes

fn create_add_function(op: &str, span: Span) -> ValueDef {
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
                Box::new(Expr::Var(Symbol::intern(op), span)),
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

fn create_add_function_typed(op: &str, span: Span) -> ValueDef {
    let mut def = create_add_function(op, span);
    
    // Add type annotation: Int -> Int -> Int
    def.type_annotation = Some(Type::Fun {
        params: vec![
            Type::Con(Symbol::intern("Int"), span),
            Type::Con(Symbol::intern("Int"), span),
        ],
        return_type: Box::new(Type::Con(Symbol::intern("Int"), span)),
        effects: EffectSet::empty(span),
        span,
    });
    
    def
}

fn create_factorial_function(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("factorial"),
        type_annotation: None,
        parameters: vec![
            Pattern::Variable(Symbol::intern("n"), span),
        ],
        body: Expr::Lambda {
            parameters: vec![
                Pattern::Variable(Symbol::intern("n"), span),
            ],
            body: Box::new(Expr::If {
                condition: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("=="), span)),
                    vec![
                        Expr::Var(Symbol::intern("n"), span),
                        Expr::Literal(Literal::Integer(0), span),
                    ],
                    span,
                )),
                then_branch: Box::new(Expr::Literal(Literal::Integer(1), span)),
                else_branch: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("*"), span)),
                    vec![
                        Expr::Var(Symbol::intern("n"), span),
                        Expr::App(
                            Box::new(Expr::Var(Symbol::intern("factorial"), span)),
                            vec![
                                Expr::App(
                                    Box::new(Expr::Var(Symbol::intern("-"), span)),
                                    vec![
                                        Expr::Var(Symbol::intern("n"), span),
                                        Expr::Literal(Literal::Integer(1), span),
                                    ],
                                    span,
                                ),
                            ],
                            span,
                        ),
                    ],
                    span,
                )),
                span,
            }),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}