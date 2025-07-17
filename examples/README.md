# x Language Examples

This directory contains example programs demonstrating various features and syntax styles of x Language.

## Examples

### Basic Examples

- `hello.x` - Simple hello world program (S-expression syntax)
- `math.x` - Basic arithmetic and recursion (S-expression syntax)

### Syntax Style

- `sexp/` - S-expression syntax examples

### Feature Examples

- `effects/` - Algebraic effects and handlers
- `types/` - Advanced type system features
- `modules/` - Module system examples

## Running Examples

### Parse an example:
```bash
cargo run --bin x-lang -p x-compiler -- parse examples/hello.x
```

### Type check an example:
```bash
cargo run --bin x-lang -p x-compiler -- check examples/hello.x
```

### Compile to TypeScript:
```bash
cargo run --bin x-lang -p x-compiler -- compile examples/hello.x --target typescript --output output
```

### Compile to WebAssembly:
```bash
cargo run --bin x-lang -p x-compiler -- compile examples/hello.x --target wasm-gc --output output
```