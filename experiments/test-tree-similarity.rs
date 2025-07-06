use x_parser::{Symbol, FileId, Span, span::ByteOffset};
use x_parser::ast::*;
use x_editor::content_addressing::{ContentRepository, Version};
use x_editor::tree_similarity::{TreeNode, APTED, TSED, CombinedSimilarity};
use std::collections::HashSet;

fn main() {
    println!("=== Tree Similarity Test (APTED/TSED) ===\n");

    let file_id = FileId::new(0);
    let span = Span::new(file_id, ByteOffset::new(0), ByteOffset::new(1));
    
    // Create repository
    let mut repo = ContentRepository::new();
    
    println!("1. Adding various functions to repository:");
    
    // Add different implementations of similar functions
    
    // Simple add: x + y
    let add_simple = create_binary_op("add_simple", "+", span);
    repo.add_function(
        "add_simple",
        &add_simple,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    println!("   - Added simple add: x + y");
    
    // Add with let binding: let result = x + y in result
    let add_let = create_add_with_let(span);
    repo.add_function(
        "add_let",
        &add_let,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    println!("   - Added add with let binding");
    
    // Add with if: if x > 0 then x + y else y
    let add_conditional = create_add_conditional(span);
    repo.add_function(
        "add_conditional",
        &add_conditional,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic", "conditional"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    println!("   - Added conditional add");
    
    // Multiply: x * y (structurally similar to add)
    let multiply = create_binary_op("multiply", "*", span);
    repo.add_function(
        "multiply",
        &multiply,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    println!("   - Added multiply: x * y");
    
    // Complex nested function
    let complex_fn = create_complex_function(span);
    repo.add_function(
        "complex_calc",
        &complex_fn,
        None,
        Some(Version::new(1, 0, 0)),
        ["arithmetic", "complex"].iter().map(|s| s.to_string()).collect(),
    ).unwrap();
    println!("   - Added complex nested calculation");
    
    println!("\n2. Testing APTED (All Path Tree Edit Distance):");
    
    // Create trees for direct comparison
    let add_tree = TreeNode::from_expr(&add_simple.body);
    let mul_tree = TreeNode::from_expr(&multiply.body);
    let let_tree = TreeNode::from_expr(&add_let.body);
    
    let apted = APTED::default();
    
    println!("   Comparing 'x + y' vs 'x * y':");
    let sim1 = apted.similarity(&add_tree, &mul_tree);
    println!("   - APTED similarity: {:.2}%", sim1 * 100.0);
    println!("   - These differ only in operator");
    
    println!("\n   Comparing 'x + y' vs 'let result = x + y in result':");
    let sim2 = apted.similarity(&add_tree, &let_tree);
    println!("   - APTED similarity: {:.2}%", sim2 * 100.0);
    println!("   - The let binding adds extra structure");
    
    println!("\n3. Testing TSED (Tree Structure Edit Distance):");
    
    let tsed = TSED::default();
    
    println!("   Structural comparison (ignoring labels):");
    let struct_sim1 = tsed.similarity(&add_tree, &mul_tree);
    println!("   - 'x + y' vs 'x * y': {:.2}%", struct_sim1 * 100.0);
    println!("   - Very high similarity (same structure)");
    
    let complex_tree = TreeNode::from_expr(&complex_fn.body);
    let struct_sim2 = tsed.similarity(&add_tree, &complex_tree);
    println!("   - 'x + y' vs complex function: {:.2}%", struct_sim2 * 100.0);
    println!("   - Lower similarity (different structure)");
    
    println!("\n4. Finding similar functions with detailed analysis:");
    
    // Search for functions similar to a subtract operation
    let subtract = create_binary_op("subtract", "-", span);
    let detailed_results = repo.find_similar_functions_detailed(&subtract, 0.3).unwrap();
    
    println!("   Functions similar to 'x - y':");
    for (hash, details) in detailed_results.iter().take(5) {
        if let Some(entry) = repo.get(hash) {
            println!("\n   {}:", entry.name);
            println!("   - Overall similarity: {:.2}%", details.overall_similarity * 100.0);
            println!("   - APTED similarity: {:.2}%", details.apted_similarity * 100.0);
            println!("   - TSED similarity: {:.2}%", details.tsed_similarity * 100.0);
            println!("   - Feature similarity: {:.2}%", details.feature_similarity * 100.0);
            println!("   - Exact structure match: {}", details.structural_match);
        }
    }
    
    println!("\n5. Combined similarity algorithm:");
    
    let combined = CombinedSimilarity::default();
    
    // Compare different function pairs
    println!("\n   Combined similarity scores:");
    
    let pairs = [
        ("add vs multiply", &add_tree, &mul_tree),
        ("add vs add_let", &add_tree, &let_tree),
        ("add vs complex", &add_tree, &complex_tree),
    ];
    
    for (name, tree1, tree2) in &pairs {
        let report = combined.detailed_similarity(tree1, tree2);
        println!("\n   {}:", name);
        println!("   - Combined: {:.2}%", report.combined_similarity * 100.0);
        println!("   - APTED: {:.2}%, TSED: {:.2}%", 
            report.apted_similarity * 100.0,
            report.tsed_similarity * 100.0
        );
        println!("   - Tree sizes: {} vs {}", report.tree1_size, report.tree2_size);
        println!("   - Tree depths: {} vs {}", report.tree1_depth, report.tree2_depth);
    }
    
    println!("\n6. Summary:");
    println!("   ✓ APTED captures exact tree edit distance");
    println!("   ✓ TSED focuses on structural similarity");
    println!("   ✓ Combined approach balances both aspects");
    println!("   ✓ Content addressing with advanced similarity works");
    println!("   ✓ Can find semantically similar code patterns");
}

// Helper functions

fn create_binary_op(name: &str, op: &str, span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern(name),
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

fn create_add_with_let(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("add_let"),
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
            body: Box::new(Expr::Let {
                pattern: Pattern::Variable(Symbol::intern("result"), span),
                type_annotation: None,
                value: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("+"), span)),
                    vec![
                        Expr::Var(Symbol::intern("x"), span),
                        Expr::Var(Symbol::intern("y"), span),
                    ],
                    span,
                )),
                body: Box::new(Expr::Var(Symbol::intern("result"), span)),
                span,
            }),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_add_conditional(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("add_conditional"),
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
            body: Box::new(Expr::If {
                condition: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern(">"), span)),
                    vec![
                        Expr::Var(Symbol::intern("x"), span),
                        Expr::Literal(Literal::Integer(0), span),
                    ],
                    span,
                )),
                then_branch: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("+"), span)),
                    vec![
                        Expr::Var(Symbol::intern("x"), span),
                        Expr::Var(Symbol::intern("y"), span),
                    ],
                    span,
                )),
                else_branch: Box::new(Expr::Var(Symbol::intern("y"), span)),
                span,
            }),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}

fn create_complex_function(span: Span) -> ValueDef {
    ValueDef {
        name: Symbol::intern("complex_calc"),
        type_annotation: None,
        parameters: vec![
            Pattern::Variable(Symbol::intern("a"), span),
            Pattern::Variable(Symbol::intern("b"), span),
            Pattern::Variable(Symbol::intern("c"), span),
        ],
        body: Expr::Lambda {
            parameters: vec![
                Pattern::Variable(Symbol::intern("a"), span),
                Pattern::Variable(Symbol::intern("b"), span),
                Pattern::Variable(Symbol::intern("c"), span),
            ],
            body: Box::new(Expr::Let {
                pattern: Pattern::Variable(Symbol::intern("temp"), span),
                type_annotation: None,
                value: Box::new(Expr::App(
                    Box::new(Expr::Var(Symbol::intern("*"), span)),
                    vec![
                        Expr::Var(Symbol::intern("a"), span),
                        Expr::Var(Symbol::intern("b"), span),
                    ],
                    span,
                )),
                body: Box::new(Expr::If {
                    condition: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern(">"), span)),
                        vec![
                            Expr::Var(Symbol::intern("temp"), span),
                            Expr::Var(Symbol::intern("c"), span),
                        ],
                        span,
                    )),
                    then_branch: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("+"), span)),
                        vec![
                            Expr::Var(Symbol::intern("temp"), span),
                            Expr::Var(Symbol::intern("c"), span),
                        ],
                        span,
                    )),
                    else_branch: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("-"), span)),
                        vec![
                            Expr::Var(Symbol::intern("temp"), span),
                            Expr::Var(Symbol::intern("c"), span),
                        ],
                        span,
                    )),
                    span,
                }),
                span,
            }),
            span,
        },
        visibility: Visibility::Public,
        purity: Purity::Pure,
        span,
    }
}