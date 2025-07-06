//! Simple demonstration of AST construction without the builder API
//! 
//! This shows how to construct x Language AST nodes directly

use x_parser::ast::*;
use x_parser::{Symbol, Span, FileId, span::ByteOffset, Visibility, Purity};
use x_parser::syntax::ocaml::OCamlPrinter;
use x_parser::syntax::{SyntaxPrinter, SyntaxConfig, SyntaxStyle};

fn main() {
    println!("=== Direct AST Construction Demo ===\n");
    
    example_simple_program();
    example_function_with_pattern_matching();
    example_algebraic_effects();
}

fn example_simple_program() {
    println!("Example 1: Simple Program");
    println!("-------------------------");
    
    let span = make_span();
    
    // Build: module Main
    //        let x = 42
    //        let y = x + 10
    //        let main = fun () -> print_endline (string_of_int y)
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("Main"), span),
        exports: None,
        imports: Vec::new(),
        items: vec![
            // let x = 42
            Item::ValueDef(ValueDef {
                name: Symbol::intern("x"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Literal(Literal::Integer(42), span),
                visibility: Visibility::Private,
                purity: Purity::Inferred,
                span,
            }),
            // let y = x + 10
            Item::ValueDef(ValueDef {
                name: Symbol::intern("y"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::App(
                    Box::new(Expr::Var(Symbol::intern("+"), span)),
                    vec![
                        Expr::Var(Symbol::intern("x"), span),
                        Expr::Literal(Literal::Integer(10), span),
                    ],
                    span
                ),
                visibility: Visibility::Private,
                purity: Purity::Inferred,
                span,
            }),
            // let main = fun () -> print_endline (string_of_int y)
            Item::ValueDef(ValueDef {
                name: Symbol::intern("main"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Lambda {
                    parameters: vec![Pattern::Variable(Symbol::intern("_"), span)],
                    body: Box::new(Expr::App(
                        Box::new(Expr::Var(Symbol::intern("print_endline"), span)),
                        vec![
                            Expr::App(
                                Box::new(Expr::Var(Symbol::intern("string_of_int"), span)),
                                vec![Expr::Var(Symbol::intern("y"), span)],
                                span
                            )
                        ],
                        span
                    )),
                    span,
                },
                visibility: Visibility::Public,
                purity: Purity::Inferred,
                span,
            }),
        ],
        span,
    };
    
    print_module(&module);
    println!();
}

fn example_function_with_pattern_matching() {
    println!("Example 2: Pattern Matching");
    println!("---------------------------");
    
    let span = make_span();
    
    // Build: module ListOps
    //        data List = Nil | Cons 'a (List 'a)
    //        let length = fun lst ->
    //          match lst with
    //          | Nil -> 0
    //          | Cons _ tail -> 1 + length tail
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("ListOps"), span),
        exports: None,
        imports: Vec::new(),
        items: vec![
            // data List = Nil | Cons 'a (List 'a)
            Item::TypeDef(TypeDef {
                name: Symbol::intern("List"),
                type_params: vec![TypeParam {
                    name: Symbol::intern("'a"),
                    kind: None,
                    constraints: Vec::new(),
                    span,
                }],
                kind: TypeDefKind::Data(vec![
                    Constructor {
                        name: Symbol::intern("Nil"),
                        fields: Vec::new(),
                        span,
                    },
                    Constructor {
                        name: Symbol::intern("Cons"),
                        fields: vec![
                            Type::Var(Symbol::intern("'a"), span),
                            Type::App(
                                Box::new(Type::Con(Symbol::intern("List"), span)),
                                vec![Type::Var(Symbol::intern("'a"), span)],
                                span
                            ),
                        ],
                        span,
                    },
                ]),
                visibility: Visibility::Public,
                span,
            }),
            // let length = fun lst -> match lst with ...
            Item::ValueDef(ValueDef {
                name: Symbol::intern("length"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Lambda {
                    parameters: vec![Pattern::Variable(Symbol::intern("lst"), span)],
                    body: Box::new(Expr::Match {
                        scrutinee: Box::new(Expr::Var(Symbol::intern("lst"), span)),
                        arms: vec![
                            // | Nil -> 0
                            MatchArm {
                                pattern: Pattern::Constructor {
                                    name: Symbol::intern("Nil"),
                                    args: Vec::new(),
                                    span,
                                },
                                guard: None,
                                body: Expr::Literal(Literal::Integer(0), span),
                                span,
                            },
                            // | Cons _ tail -> 1 + length tail
                            MatchArm {
                                pattern: Pattern::Constructor {
                                    name: Symbol::intern("Cons"),
                                    args: vec![
                                        Pattern::Wildcard(span),
                                        Pattern::Variable(Symbol::intern("tail"), span),
                                    ],
                                    span,
                                },
                                guard: None,
                                body: Expr::App(
                                    Box::new(Expr::Var(Symbol::intern("+"), span)),
                                    vec![
                                        Expr::Literal(Literal::Integer(1), span),
                                        Expr::App(
                                            Box::new(Expr::Var(Symbol::intern("length"), span)),
                                            vec![Expr::Var(Symbol::intern("tail"), span)],
                                            span
                                        ),
                                    ],
                                    span
                                ),
                                span,
                            },
                        ],
                        span,
                    }),
                    span,
                },
                visibility: Visibility::Public,
                purity: Purity::Inferred,
                span,
            }),
        ],
        span,
    };
    
    print_module(&module);
    println!();
}

fn example_algebraic_effects() {
    println!("Example 3: Algebraic Effects");
    println!("-----------------------------");
    
    let span = make_span();
    
    // Build: module State
    //        effect State = get : unit -> int | put : int -> unit
    //        let run_state = handler
    //          | get () k -> fun s -> k s s
    //          | put s' k -> fun _ -> k () s'
    //          | return x -> fun s -> (x, s)
    
    let module = Module {
        name: ModulePath::single(Symbol::intern("State"), span),
        exports: None,
        imports: Vec::new(),
        items: vec![
            // effect State = get : unit -> int | put : int -> unit
            Item::EffectDef(EffectDef {
                name: Symbol::intern("State"),
                type_params: Vec::new(),
                operations: vec![
                    EffectOperation {
                        name: Symbol::intern("get"),
                        parameters: vec![Type::Con(Symbol::intern("Unit"), span)],
                        return_type: Type::Con(Symbol::intern("Int"), span),
                        span,
                    },
                    EffectOperation {
                        name: Symbol::intern("put"),
                        parameters: vec![Type::Con(Symbol::intern("Int"), span)],
                        return_type: Type::Con(Symbol::intern("Unit"), span),
                        span,
                    },
                ],
                visibility: Visibility::Public,
                span,
            }),
            // Handler definition (simplified)
            Item::ValueDef(ValueDef {
                name: Symbol::intern("run_state"),
                type_annotation: None,
                parameters: Vec::new(),
                body: Expr::Literal(Literal::Unit, span), // Simplified
                visibility: Visibility::Public,
                purity: Purity::Inferred,
                span,
            }),
        ],
        span,
    };
    
    print_module(&module);
    println!();
}

// Helper functions

fn make_span() -> Span {
    Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1))
}

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