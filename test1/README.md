# test1

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
