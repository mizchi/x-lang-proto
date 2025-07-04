# EffectLang Code Examples

This directory contains examples demonstrating EffectLang's multi-syntax support. The same semantic program is written in four different syntax styles:

## Syntax Styles

1. **OCaml Style** (`*.ocaml.eff`) - Default syntax with ML-family features
2. **S-Expression Style** (`*.sexp.eff`) - Lisp-like syntax with parentheses
3. **Haskell Style** (`*.haskell.eff`) - Haskell-inspired functional syntax
4. **Rust Style** (`*.rust.eff`) - Rust-like syntax with explicit types

## Example Programs

### 1. Basic Features (`basic.*`)
- Visibility modifiers (pub, pub(crate), etc.)
- Simple value definitions
- Type definitions
- Pipeline syntax

### 2. Effects System (`effects.*`)
- Effect definitions
- Handler implementations
- Effect operations

### 3. WebAssembly Interface (`wasm_interface.*`)
- Component interface definitions
- WASI-style imports/exports
- Resource definitions
- Function signatures

### 4. Advanced Features (`advanced.*`)
- Complex type system features
- Module system
- Import/export declarations

## Usage

These examples can be parsed and converted between syntax styles using EffectLang's multi-syntax support:

```rust
use effect_lang::{MultiSyntax, SyntaxStyle, SyntaxConfig};

let mut multi = MultiSyntax::default();
let file_id = FileId::new(0);

// Parse OCaml-style code
let ast = multi.parse(ocaml_code, SyntaxStyle::OCaml, file_id)?;

// Print in Rust style
let config = SyntaxConfig { style: SyntaxStyle::RustLike, ..Default::default() };
let rust_code = multi.print(&ast, &config)?;
```

All examples demonstrate the same semantic content while showcasing the syntactic flexibility of EffectLang.