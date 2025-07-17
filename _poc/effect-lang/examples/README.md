# EffectLang Code Examples

This directory contains examples demonstrating EffectLang's S-expression syntax.

## Syntax Style

**S-Expression Style** (`*.sexp.eff`) - Lisp-like syntax with parentheses

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

These examples can be parsed using EffectLang's S-expression parser:

```rust
use effect_lang::{parse_sexp};

let file_id = FileId::new(0);

// Parse S-expression code
let ast = parse_sexp(sexp_code, file_id)?;
```