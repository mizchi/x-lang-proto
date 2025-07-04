//! New project creation command

use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use colored::*;
use crate::utils::{ProgressIndicator, print_success};
use crate::format::{save_ast, Format};
use x_parser::{
    persistent_ast::{NodeBuilder, AstNodeKind, PersistentAstNode, Visibility, Purity, LiteralValue, Parameter, Binding},
    span::{Span, FileId, ByteOffset},
    symbol::Symbol,
};

pub async fn new_command(name: &str, dir: Option<&Path>) -> Result<()> {
    let progress = ProgressIndicator::new("Creating new project");
    
    let project_dir = match dir {
        Some(path) => path.to_owned(),
        None => std::env::current_dir()?.join(name),
    };
    
    progress.set_message("Creating directory structure");
    fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_dir.display()))?;
    
    progress.set_message("Generating main.x binary file");
    create_main_binary_file(&project_dir, name).await?;
    
    progress.set_message("Generating project files");
    create_project_files(&project_dir, name)?;
    
    progress.finish("Project created successfully");
    
    print_success(&format!("Created new x Language project: {}", name));
    println!("Project directory: {}", project_dir.display().to_string().cyan());
    println!();
    println!("Next steps:");
    println!("  {} cd {}", "1.".bold(), name);
    println!("  {} cargo run --bin x -- show main.x --format rustic", "2.".bold());
    println!("  {} cargo run --bin x -- check main.x", "3.".bold());
    println!("  {} cargo run --bin x -- compile main.x --target typescript", "4.".bold());
    
    Ok(())
}

/// Create the main.x binary AST file
async fn create_main_binary_file(project_dir: &Path, name: &str) -> Result<()> {
    let mut builder = NodeBuilder::new();
    
    // Create main function
    let main_body = create_main_function_body(&mut builder, name);
    let main_function = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(100)),
        AstNodeKind::ValueDef {
            name: Symbol::intern("main"),
            type_annotation: None,
            body: Box::new(main_body),
            visibility: Visibility::Public,
            purity: Purity::Impure,
        },
    );
    
    // Create add function
    let add_function = create_add_function(&mut builder);
    
    // Create module
    let module = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(200)),
        AstNodeKind::Module {
            name: Symbol::intern("main"),
            items: vec![main_function, add_function],
            visibility: Visibility::Public,
        },
    );
    
    // Create compilation unit
    let compilation_unit = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(200)),
        AstNodeKind::CompilationUnit {
            modules: vec![module],
            imports: Vec::new(),
            exports: Vec::new(),
        },
    );
    
    // Save as binary file
    let main_file = project_dir.join("main.x");
    save_ast(&main_file, &compilation_unit, Format::Binary).await
        .with_context(|| format!("Failed to create main.x file: {}", main_file.display()))?;
    
    Ok(())
}

/// Create the body of the main function
fn create_main_function_body(builder: &mut NodeBuilder, name: &str) -> PersistentAstNode {
    // Create: println("Hello from {name}!")
    let hello_string = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(10), ByteOffset::new(30)),
        AstNodeKind::Literal {
            value: LiteralValue::String(format!("Hello from {}!", name)),
        },
    );
    
    let println_var = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(7)),
        AstNodeKind::Variable { name: Symbol::intern("println") },
    );
    
    let println_call = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(40)),
        AstNodeKind::Application {
            function: Box::new(println_var),
            arguments: vec![hello_string],
        },
    );
    
    // Create: let result = add(21, 21)
    let add_var = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(50), ByteOffset::new(53)),
        AstNodeKind::Variable { name: Symbol::intern("add") },
    );
    
    let num_21_1 = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(54), ByteOffset::new(56)),
        AstNodeKind::Literal { value: LiteralValue::Integer(21) },
    );
    
    let num_21_2 = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(58), ByteOffset::new(60)),
        AstNodeKind::Literal { value: LiteralValue::Integer(21) },
    );
    
    let add_call = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(50), ByteOffset::new(70)),
        AstNodeKind::Application {
            function: Box::new(add_var),
            arguments: vec![num_21_1, num_21_2],
        },
    );
    
    let result_pattern = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(45), ByteOffset::new(51)),
        AstNodeKind::PatternVariable { name: Symbol::intern("result") },
    );
    
    let result_binding = Binding {
        pattern: Box::new(result_pattern),
        value: Box::new(add_call),
    };
    
    // Create function body with println and let binding
    builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(80)),
        AstNodeKind::Let {
            bindings: vec![result_binding],
            body: Box::new(println_call),
        },
    )
}

/// Create the add function
fn create_add_function(builder: &mut NodeBuilder) -> PersistentAstNode {
    // Create type references separately to avoid borrowing issues
    let i32_type_a = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(3)),
        AstNodeKind::TypeReference {
            name: Symbol::intern("i32"),
            type_args: Vec::new(),
        },
    );
    
    let i32_type_b = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(3)),
        AstNodeKind::TypeReference {
            name: Symbol::intern("i32"),
            type_args: Vec::new(),
        },
    );
    
    let param_a = Parameter {
        name: Symbol::intern("a"),
        type_annotation: Some(Box::new(i32_type_a)),
    };
    
    let param_b = Parameter {
        name: Symbol::intern("b"),
        type_annotation: Some(Box::new(i32_type_b)),
    };
    
    // Create variables separately
    let plus_op = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(2), ByteOffset::new(3)),
        AstNodeKind::Variable { name: Symbol::intern("+") },
    );
    
    let var_a = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(1)),
        AstNodeKind::Variable { name: Symbol::intern("a") },
    );
    
    let var_b = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(4), ByteOffset::new(5)),
        AstNodeKind::Variable { name: Symbol::intern("b") },
    );
    
    // Create: a + b (as function application)
    let add_body = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5)),
        AstNodeKind::Application {
            function: Box::new(plus_op),
            arguments: vec![var_a, var_b],
        },
    );
    
    let lambda_body = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(110), ByteOffset::new(150)),
        AstNodeKind::Lambda {
            parameters: vec![param_a, param_b],
            body: Box::new(add_body),
            effect_annotation: None,
        },
    );
    
    // Create type annotations for function type
    let param_type1 = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(3)),
        AstNodeKind::TypeReference {
            name: Symbol::intern("i32"),
            type_args: Vec::new(),
        },
    );
    
    let param_type2 = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(3)),
        AstNodeKind::TypeReference {
            name: Symbol::intern("i32"),
            type_args: Vec::new(),
        },
    );
    
    let return_type = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(3)),
        AstNodeKind::TypeReference {
            name: Symbol::intern("i32"),
            type_args: Vec::new(),
        },
    );
    
    let function_type = builder.build(
        Span::new(FileId::new(0), ByteOffset::new(105), ByteOffset::new(120)),
        AstNodeKind::FunctionType {
            parameters: vec![param_type1, param_type2],
            return_type: Box::new(return_type),
            effects: None,
        },
    );
    
    builder.build(
        Span::new(FileId::new(0), ByteOffset::new(100), ByteOffset::new(150)),
        AstNodeKind::ValueDef {
            name: Symbol::intern("add"),
            type_annotation: Some(Box::new(function_type)),
            body: Box::new(lambda_body),
            visibility: Visibility::Public,
            purity: Purity::Pure,
        },
    )
}

fn create_project_files(project_dir: &Path, name: &str) -> Result<()> {
    
    // Create project configuration
    let config_file = project_dir.join("x-lang.toml");
    let config_content = format!(r#"[project]
name = "{}"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]

[build]
syntax_style = "Rust"
optimization_level = 2
debug_info = true
source_maps = true

[targets.typescript]
enabled = true
module_system = "es2020"
emit_types = true
strict = true

[targets.wasm-component]
enabled = false
with_wit = true
generate_bindings = true
wit_package = "{}"

[lsp]
enabled = true
port = 9257
features = ["completions", "hover", "goto-definition", "find-references"]
"#, name, name);
    
    fs::write(&config_file, config_content)
        .with_context(|| format!("Failed to create config file: {}", config_file.display()))?;
    
    // Create README
    let readme_file = project_dir.join("README.md");
    let readme_content = format!(r#"# {}

A new x Language project.

## Getting Started

This project uses the x Language, which features direct AST manipulation and multiple syntax styles.

### Building

```bash
# Type check the project
cargo run --bin x -- check main.x

# Compile to TypeScript
cargo run --bin x -- compile main.x --target typescript

# Compile to WebAssembly Component
cargo run --bin x -- compile main.x --target wasm-component
```

### Development

```bash
# View the AST in Rust-like syntax
cargo run --bin x -- show main.x --format rustic

# Show the AST structure
cargo run --bin x -- show main.x --format tree

# Query for specific nodes
cargo run --bin x -- query main.x "type:ValueDef"

# Start the REPL
cargo run --bin x -- repl --preload main.x
```

### Converting Between Formats

```bash
# Convert to text formats for viewing
cargo run --bin x -- convert main.x --to rustic
cargo run --bin x -- convert main.x --to ocaml

# Convert to JSON for inspection
cargo run --bin x -- convert main.x --to json
```

## Project Structure

- `main.x` - Main source file (binary AST format)
- `x-lang.toml` - Project configuration
- `README.md` - This file

## Learn More

- [x Language Documentation](https://docs.x-lang.org)
- [AST Manipulation Guide](https://docs.x-lang.org/ast-guide)
- [Multi-Syntax Guide](https://docs.x-lang.org/syntax-guide)
"#, name);
    
    fs::write(&readme_file, readme_content)
        .with_context(|| format!("Failed to create README: {}", readme_file.display()))?;
    
    // Create .gitignore
    let gitignore_file = project_dir.join(".gitignore");
    let gitignore_content = r#"# x Language build artifacts
/dist/
/target/
*.x.map
*.wasm

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Logs
*.log
"#;
    
    fs::write(&gitignore_file, gitignore_content)
        .with_context(|| format!("Failed to create .gitignore: {}", gitignore_file.display()))?;
    
    Ok(())
}