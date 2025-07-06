//! Example: Constructing x Language programs using AST builder

use x_ast_builder::*;
use x_parser::ast::*;
use x_parser::syntax::ocaml::OCamlPrinter;
use x_parser::syntax::{SyntaxPrinter, SyntaxConfig, SyntaxStyle};

fn main() {
    println!("=== AST Construction Examples ===\n");
    
    // Example 1: Simple arithmetic
    example_arithmetic();
    
    // Example 2: Function definition
    example_function();
    
    // Example 3: Data types and pattern matching
    example_pattern_matching();
    
    // Example 4: Complex program
    example_complex_program();
}

fn example_arithmetic() {
    println!("Example 1: Simple Arithmetic");
    println!("----------------------------");
    
    let mut builder = AstBuilder::new();
    
    // Build: module Math let result = 2 + 3 * 4
    let module = builder.module("Math")
        .value("result", |b| {
            b.expr()
                .binop("+", 
                    |b| b.expr().int(2).build(),
                    |b| b.expr()
                        .binop("*",
                            |b| b.expr().int(3).build(),
                            |b| b.expr().int(4).build()
                        ).build()
                ).build()
        })
        .build();
    
    print_module(&module);
    println!();
}

fn example_function() {
    println!("Example 2: Function Definition");
    println!("------------------------------");
    
    let mut builder = AstBuilder::new();
    
    // Build: module Functions
    //        let factorial = fun n ->
    //          if n <= 1 then 1 else n * factorial (n - 1)
    let module = builder.module("Functions")
        .function("factorial", vec!["n"], |b| {
            b.expr()
                .if_then_else(
                    |b| b.expr().binop("<=",
                        |b| b.expr().var("n").build(),
                        |b| b.expr().int(1).build()
                    ).build(),
                    |b| b.expr().int(1).build(),
                    |b| b.expr().binop("*",
                        |b| b.expr().var("n").build(),
                        |b| b.expr().app_expr(
                            |b| b.expr().var("factorial").build(),
                            vec![|b| b.expr().binop("-",
                                |b| b.expr().var("n").build(),
                                |b| b.expr().int(1).build()
                            ).build()]
                        ).build()
                    ).build()
                ).build()
        })
        .build();
    
    print_module(&module);
    println!();
}

fn example_pattern_matching() {
    println!("Example 3: Data Types and Pattern Matching");
    println!("------------------------------------------");
    
    let mut builder = AstBuilder::new();
    
    // Build: module Option
    //        data Option = None | Some value
    //        let map = fun f opt -> 
    //          match opt with
    //          | None -> None
    //          | Some x -> Some (f x)
    let module = builder.module("OptionModule")
        .data_type("Option", vec![
            ("None", vec![]),
            ("Some", vec!["'a"])
        ])
        .function("map", vec!["f", "opt"], |b| {
            // Create patterns
            let none_pattern = Pattern::Constructor {
                name: Symbol::intern("None"),
                args: vec![],
                span: b.span(),
            };
            let some_pattern = Pattern::Constructor {
                name: Symbol::intern("Some"),
                args: vec![Pattern::Variable(Symbol::intern("x"), b.span())],
                span: b.span(),
            };
            
            b.expr()
                .match_expr(
                    |b| b.expr().var("opt").build(),
                    vec![
                        (none_pattern, |b| b.expr().var("None").build()),
                        (some_pattern, |b| {
                            b.expr().app_expr(
                                |b| b.expr().var("Some").build(),
                                vec![|b| b.expr().app_expr(
                                    |b| b.expr().var("f").build(),
                                    vec![|b| b.expr().var("x").build()]
                                ).build()]
                            ).build()
                        }),
                    ]
                ).build()
        })
        .build();
    
    print_module(&module);
    println!();
}

fn example_complex_program() {
    println!("Example 4: Complex Program");
    println!("--------------------------");
    
    let mut builder = AstBuilder::new();
    
    // Build a more complex program with multiple features
    let module = builder.module("ComplexExample")
        // Import standard library
        .import("List")
        .import("IO")
        
        // Define a record type (using simple approach)
        .data_type("Person", vec![
            ("Person", vec!["String", "Int"])
        ])
        
        // Helper function
        .function("greet", vec!["person"], |b| {
            b.expr().app("print_endline", vec![
                |b| b.expr().binop("^",
                    |b| b.expr().string("Hello, ").build(),
                    |b| b.expr().var("person").build()
                ).build()
            ]).build()
        })
        
        // Main function with let bindings
        .function("main", vec![], |b| {
            b.expr()
                .let_in("people", 
                    |b| b.expr().list(vec![
                        |b| b.expr().string("Alice").build(),
                        |b| b.expr().string("Bob").build(),
                        |b| b.expr().string("Charlie").build(),
                    ]).build(),
                    |b| b.expr()
                        .let_in("count",
                            |b| b.expr().app("length", vec![
                                |b| b.expr().var("people").build()
                            ]).build(),
                            |b| b.expr().do_block(vec![
                                |b| DoStatement::Expr(
                                    b.expr().app("print_endline", vec![
                                        |b| b.expr().string("Greeting people...").build()
                                    ]).build()
                                ),
                                |b| DoStatement::Expr(
                                    b.expr().app("iter", vec![
                                        |b| b.expr().var("greet").build(),
                                        |b| b.expr().var("people").build()
                                    ]).build()
                                ),
                                |b| DoStatement::Expr(
                                    b.expr().app("print_int", vec![
                                        |b| b.expr().var("count").build()
                                    ]).build()
                                ),
                            ]).build()
                        ).build()
                ).build()
        })
        .build();
    
    print_module(&module);
    println!();
}

/// Helper function to print a module
fn print_module(module: &Module) {
    let cu = CompilationUnit {
        module: module.clone(),
        span: module.span,
    };
    
    let config = SyntaxConfig {
        style: SyntaxStyle::OCaml,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = OCamlPrinter::new();
    match printer.print(&cu, &config) {
        Ok(code) => println!("{}", code),
        Err(e) => println!("Error printing: {:?}", e),
    }
}