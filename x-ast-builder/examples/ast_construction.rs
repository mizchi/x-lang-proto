//! Example: Constructing x Language programs using AST builder

use x_ast_builder::*;
use x_parser::ast::*;
use x_parser::{Symbol};
use x_parser::syntax::haskell::HaskellPrinter;
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
                    |b| b.expr().int(2),
                    |b| b.expr()
                        .binop("*",
                            |b| b.expr().int(3),
                            |b| b.expr().int(4)
                        )
                )
        })
        ;
    
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
                        |b| b.expr().var("n"),
                        |b| b.expr().int(1)
                    ),
                    |b| b.expr().int(1),
                    |b| b.expr().binop("*",
                        |b| b.expr().var("n"),
                        |b| b.app("factorial",
                            vec![|b| b.expr().binop("-",
                                |b| b.expr().var("n"),
                                |b| b.expr().int(1)
                            )]
                        )
                    )
                )
        })
        ;
    
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
            
            b.match_expr(
                |b| b.expr().var("opt"),
                vec![
                    (none_pattern, |b| b.expr().var("None")),
                    (some_pattern, |b| {
                        b.app("Some",
                            vec![|b| b.app("f",
                                vec![|b| b.expr().var("x")]
                            )]
                        )
                    }),
                ]
            )
        })
        ;
    
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
                    |b| b.expr().string("Hello, "),
                    |b| b.expr().var("person")
                )
            ])
        })
        
        // Main function with let bindings
        .function("main", vec![], |b| {
            b.expr()
                .let_in("people", 
                    |b| b.expr().list(vec![
                        |b| b.expr().string("Alice"),
                        |b| b.expr().string("Bob"),
                        |b| b.expr().string("Charlie"),
                    ]),
                    |b| b.expr()
                        .let_in("count",
                            |b| b.expr().app("length", vec![
                                |b| b.expr().var("people")
                            ]),
                            |b| b.expr().do_block(vec![
                                |b| DoStatement::Expr(
                                    b.expr().app("print_endline", vec![
                                        |b| b.expr().string("Greeting people...")
                                    ])
                                ),
                                |b| DoStatement::Expr(
                                    b.expr().app("iter", vec![
                                        |b| b.expr().var("greet"),
                                        |b| b.expr().var("people")
                                    ])
                                ),
                                |b| DoStatement::Expr(
                                    b.expr().app("print_int", vec![
                                        |b| b.expr().var("count")
                                    ])
                                ),
                            ])
                        )
                )
        })
        ;
    
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
        style: SyntaxStyle::Haskell,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = HaskellPrinter::new();
    match printer.print(&cu, &config) {
        Ok(code) => println!("{}", code),
        Err(e) => println!("Error printing: {:?}", e),
    }
}