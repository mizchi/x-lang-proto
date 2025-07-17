This is a prototyping language for considering languages for AI.

I plan to completely remake it. I'm not sure if this will be the repository.

# x-lang

An experimental programming language supporting multiple programming paradigms.

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/mizchi/x-lang-proto.git
cd x-lang-proto

# Build and install
cargo build --release
cargo install --path x-cli
cargo install --path x-compiler
```

## Architecture

### Core Crates

- `x-parser` - Text to AST conversion with S-expression syntax
- `x-checker` - Type checking and semantic analysis with effect system
- `x-compiler` - Code generation for multiple targets
- `x-editor` - Language service with direct AST manipulation for AI

### Key Features

#### Advanced Type System

- Effect System - Algebraic effects and handlers for controlled side effects
- Type Inference - Automatic type deduction with constraints
- Visibility Modifiers - Module privacy (`pub`, `pub(crate)`, etc.)

## Language Service & AI Integration

The `x-lang-editor` crate provides a language service designed for AI integration:

```rust
use x_editor::{XLangEditor, EditOperation, AstQuery};

let mut editor = XLangEditor::new(config);
let session_id = editor.start_session(source_code)?;

// Direct AST manipulation
let operation = EditOperation::insert(path, new_node);
editor.apply_operation(session_id, operation)?;

// Query AST structure
let query = AstQuery::find_by_type("FunctionDef");
let results = editor.query_ast(session_id, query)?;
```

## CLI Reference

```bash
# Compile source code
x-lang compile input.x --target typescript --output dist/

# Type check only
x-lang check input.x

# Parse and show AST
x-lang parse input.x --format json

# Show available targets
x-lang targets

# Create configuration file
x-lang init-config --output x-lang.toml

# Validate configuration
x-lang validate-config x-lang.toml
```

## Configuration

Create `x-lang.toml` for project settings:

```toml
[default]
optimization_level = 2
debug_info = true
source_maps = true

[targets.typescript]
enabled = true
module_system = "es2020"
emit_types = true
strict = true

[targets.wasm-component]
enabled = true
with_wit = true
generate_bindings = true
wit_package = "my-component"
```

## CLI Tools

### x-lang (Compiler)

Main compiler tool for x-lang.

```bash
x-lang compile <file> --target <target> --output <dir>
x-lang check <file>
x-lang parse <file>
x-lang targets
```

### x (AST Editor)

AST manipulation tool.

```bash
x convert <input> <output>
x show <file>
x query <file> <query>
x edit <file>
x rename <file> <old> <new>
```

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Roadmap

- [ ] LSP Server - Full Language Server Protocol support
- [ ] Package System - Module management and distribution
- [ ] Debugger - Source-level debugging support
- [ ] REPL - Interactive development environment
- [ ] More Backends - LLVM, C, Go compilation targets
- [ ] Async/Await - Built-in concurrency primitives
- [ ] Macros - Compile-time code generation

## Contributing

Contributions are welcome! We need help in the following areas:

- Language Features - New syntax constructs, type system improvements
- Backends - Additional compilation targets
- Tooling - IDE plugins, debugger support
- Documentation - Examples, tutorials, API docs

To submit a Pull Request:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by OCaml, Rust, and effect systems research
- Built with Rust and the amazing Rust ecosystem
- Thanks to all contributors
