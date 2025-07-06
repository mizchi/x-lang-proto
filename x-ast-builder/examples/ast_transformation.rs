//! Example: AST transformation and manipulation

use x_ast_builder::*;
use x_parser::ast::*;
use x_parser::{Symbol, parse_source, FileId, SyntaxStyle};
use x_parser::syntax::ocaml::OCamlPrinter;
use x_parser::syntax::{SyntaxPrinter, SyntaxConfig};
use std::collections::HashMap;

fn main() {
    println!("=== AST Transformation Examples ===\n");
    
    // Example 1: Variable renaming
    example_rename_variables();
    
    // Example 2: Function inlining
    example_inline_function();
    
    // Example 3: Constant folding
    example_constant_folding();
    
    // Example 4: Dead code elimination
    example_dead_code_elimination();
}

fn example_rename_variables() {
    println!("Example 1: Variable Renaming");
    println!("----------------------------");
    
    let source = r#"
module Rename

let main = fun () ->
  let x = 10 in
  let y = x + 5 in
  x + y
"#;
    
    let cu = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
    
    println!("Original:");
    print_compilation_unit(&cu);
    
    // Transform: rename x to a, y to b
    let mut renaming = HashMap::new();
    renaming.insert(Symbol::intern("x"), Symbol::intern("a"));
    renaming.insert(Symbol::intern("y"), Symbol::intern("b"));
    
    let transformed = transform_compilation_unit(cu, |expr| {
        rename_variables_in_expr(expr, &renaming)
    });
    
    println!("\nAfter renaming (x->a, y->b):");
    print_compilation_unit(&transformed);
    println!();
}

fn example_inline_function() {
    println!("Example 2: Function Inlining");
    println!("-----------------------------");
    
    let source = r#"
module Inline

let double = fun x -> x * 2
let main = fun () ->
  double 5 + double 3
"#;
    
    let cu = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
    
    println!("Original:");
    print_compilation_unit(&cu);
    
    // Transform: inline all calls to 'double'
    let transformed = transform_compilation_unit(cu, |expr| {
        inline_function_calls(expr, "double", |args| {
            if args.len() == 1 {
                // Replace double(x) with x * 2
                let x = args[0].clone();
                let mut builder = AstBuilder::new();
                builder.expr().binop("*",
                    |_| x.clone(),
                    |b| b.expr().int(2).build()
                ).build()
            } else {
                expr.clone()
            }
        })
    });
    
    println!("\nAfter inlining 'double':");
    print_compilation_unit(&transformed);
    println!();
}

fn example_constant_folding() {
    println!("Example 3: Constant Folding");
    println!("----------------------------");
    
    let source = r#"
module ConstFold

let main = fun () ->
  let a = 2 + 3 in
  let b = 4 * 5 in
  let c = a + b in
  c
"#;
    
    let cu = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
    
    println!("Original:");
    print_compilation_unit(&cu);
    
    // Transform: fold constant expressions
    let transformed = transform_compilation_unit(cu, |expr| {
        fold_constants(expr)
    });
    
    println!("\nAfter constant folding:");
    print_compilation_unit(&transformed);
    println!();
}

fn example_dead_code_elimination() {
    println!("Example 4: Dead Code Elimination");
    println!("---------------------------------");
    
    let source = r#"
module DeadCode

let main = fun () ->
  let unused = 42 in
  let used = 10 in
  let also_unused = unused + 5 in
  used * 2
"#;
    
    let cu = parse_source(source, FileId::new(0), SyntaxStyle::OCaml).unwrap();
    
    println!("Original:");
    print_compilation_unit(&cu);
    
    // Transform: eliminate unused bindings
    let transformed = transform_compilation_unit(cu, |expr| {
        eliminate_dead_code(expr)
    });
    
    println!("\nAfter dead code elimination:");
    print_compilation_unit(&transformed);
    println!();
}

// Transformation functions

fn transform_compilation_unit<F>(cu: CompilationUnit, transform: F) -> CompilationUnit
where
    F: Fn(&Expr) -> Expr + Clone,
{
    let mut module = cu.module;
    module.items = module.items.into_iter().map(|item| {
        transform_item(item, transform.clone())
    }).collect();
    
    CompilationUnit {
        module,
        span: cu.span,
    }
}

fn transform_item<F>(item: Item, transform: F) -> Item
where
    F: Fn(&Expr) -> Expr,
{
    match item {
        Item::ValueDef(mut def) => {
            def.body = transform(&def.body);
            Item::ValueDef(def)
        }
        other => other,
    }
}

fn rename_variables_in_expr(expr: &Expr, renaming: &HashMap<Symbol, Symbol>) -> Expr {
    match expr {
        Expr::Var(name, span) => {
            if let Some(new_name) = renaming.get(name) {
                Expr::Var(*new_name, *span)
            } else {
                expr.clone()
            }
        }
        Expr::Let { pattern, type_annotation, value, body, span } => {
            let new_value = Box::new(rename_variables_in_expr(value, renaming));
            let new_body = Box::new(rename_variables_in_expr(body, renaming));
            Expr::Let {
                pattern: pattern.clone(),
                type_annotation: type_annotation.clone(),
                value: new_value,
                body: new_body,
                span: *span,
            }
        }
        Expr::App(func, args, span) => {
            let new_func = Box::new(rename_variables_in_expr(func, renaming));
            let new_args = args.iter()
                .map(|arg| rename_variables_in_expr(arg, renaming))
                .collect();
            Expr::App(new_func, new_args, *span)
        }
        Expr::Lambda { parameters, body, span } => {
            let new_body = Box::new(rename_variables_in_expr(body, renaming));
            Expr::Lambda {
                parameters: parameters.clone(),
                body: new_body,
                span: *span,
            }
        }
        Expr::If { condition, then_branch, else_branch, span } => {
            Expr::If {
                condition: Box::new(rename_variables_in_expr(condition, renaming)),
                then_branch: Box::new(rename_variables_in_expr(then_branch, renaming)),
                else_branch: Box::new(rename_variables_in_expr(else_branch, renaming)),
                span: *span,
            }
        }
        _ => expr.clone(),
    }
}

fn inline_function_calls<F>(expr: &Expr, func_name: &str, inline_with: F) -> Expr
where
    F: Fn(&[Expr]) -> Expr + Clone,
{
    match expr {
        Expr::App(func, args, span) => {
            match func.as_ref() {
                Expr::Var(name, _) if name.as_str() == func_name => {
                    // Inline the function call
                    inline_with(args)
                }
                _ => {
                    // Recursively transform function and arguments
                    let new_func = Box::new(inline_function_calls(func, func_name, inline_with.clone()));
                    let new_args = args.iter()
                        .map(|arg| inline_function_calls(arg, func_name, inline_with.clone()))
                        .collect();
                    Expr::App(new_func, new_args, *span)
                }
            }
        }
        Expr::Let { pattern, type_annotation, value, body, span } => {
            Expr::Let {
                pattern: pattern.clone(),
                type_annotation: type_annotation.clone(),
                value: Box::new(inline_function_calls(value, func_name, inline_with.clone())),
                body: Box::new(inline_function_calls(body, func_name, inline_with)),
                span: *span,
            }
        }
        _ => expr.clone(),
    }
}

fn fold_constants(expr: &Expr) -> Expr {
    match expr {
        Expr::App(func, args, span) => {
            match (func.as_ref(), args.as_slice()) {
                (Expr::Var(op, _), [left, right]) => {
                    match (fold_constants(left), fold_constants(right)) {
                        (Expr::Literal(Literal::Integer(l), _), 
                         Expr::Literal(Literal::Integer(r), _)) => {
                            match op.as_str() {
                                "+" => Expr::Literal(Literal::Integer(l + r), *span),
                                "-" => Expr::Literal(Literal::Integer(l - r), *span),
                                "*" => Expr::Literal(Literal::Integer(l * r), *span),
                                "/" if r != 0 => Expr::Literal(Literal::Integer(l / r), *span),
                                _ => expr.clone(),
                            }
                        }
                        (new_left, new_right) => {
                            Expr::App(func.clone(), vec![new_left, new_right], *span)
                        }
                    }
                }
                _ => expr.clone(),
            }
        }
        Expr::Let { pattern, type_annotation, value, body, span } => {
            Expr::Let {
                pattern: pattern.clone(),
                type_annotation: type_annotation.clone(),
                value: Box::new(fold_constants(value)),
                body: Box::new(fold_constants(body)),
                span: *span,
            }
        }
        _ => expr.clone(),
    }
}

fn eliminate_dead_code(expr: &Expr) -> Expr {
    match expr {
        Expr::Let { pattern, value, body, .. } => {
            let used_vars = collect_used_variables(body);
            
            if let Pattern::Variable(name, _) = pattern {
                if !used_vars.contains(name) {
                    // Skip this binding
                    return eliminate_dead_code(body);
                }
            }
            
            // Keep the binding but recursively eliminate in body
            Expr::Let {
                pattern: pattern.clone(),
                type_annotation: None,
                value: value.clone(),
                body: Box::new(eliminate_dead_code(body)),
                span: expr.span(),
            }
        }
        _ => expr.clone(),
    }
}

fn collect_used_variables(expr: &Expr) -> HashSet<Symbol> {
    let mut used = HashSet::new();
    collect_used_variables_impl(expr, &mut used);
    used
}

fn collect_used_variables_impl(expr: &Expr, used: &mut HashSet<Symbol>) {
    match expr {
        Expr::Var(name, _) => {
            used.insert(*name);
        }
        Expr::App(func, args, _) => {
            collect_used_variables_impl(func, used);
            for arg in args {
                collect_used_variables_impl(arg, used);
            }
        }
        Expr::Let { value, body, .. } => {
            collect_used_variables_impl(value, used);
            collect_used_variables_impl(body, used);
        }
        Expr::Lambda { body, .. } => {
            collect_used_variables_impl(body, used);
        }
        Expr::If { condition, then_branch, else_branch, .. } => {
            collect_used_variables_impl(condition, used);
            collect_used_variables_impl(then_branch, used);
            collect_used_variables_impl(else_branch, used);
        }
        _ => {}
    }
}

fn print_compilation_unit(cu: &CompilationUnit) {
    let config = SyntaxConfig {
        style: SyntaxStyle::OCaml,
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
        preserve_comments: true,
    };
    
    let printer = OCamlPrinter::new();
    match printer.print(cu, &config) {
        Ok(code) => print!("{}", code),
        Err(e) => println!("Error printing: {:?}", e),
    }
}