# x Language Semantic API for AI

## Overview

The x Language provides a semantic API designed specifically for AI assistants to understand and manipulate code at a higher level than traditional line/character-based LSP operations.

## Key Design Principles

1. **AST-First**: All operations work directly on the Abstract Syntax Tree
2. **Content-Addressed**: Code elements are referenced by stable content hashes
3. **Semantic References**: Use path expressions like `module.items[2].body` instead of line:column
4. **Effect-Aware**: Track and reason about side effects explicitly

## Core Commands

### `x doc` - Semantic Code Analysis

```bash
# Get semantic summary of a module
x doc path/to/file.x --format semantic

# Filter by symbol kind
x doc . --kind function --include-private

# Get detailed type signatures
x doc . --format json
```

Output includes:
- Symbol definitions with unique IDs
- Type signatures and effect annotations
- Dependency graphs
- AST node references
- Content hashes for stable references

### AST Navigation API

Instead of line/column positions, use AST paths:

```json
{
  "ast_ref": {
    "file": "main.x",
    "node_path": "module.items[0].body.arms[1]",
    "node_type": "MatchArm",
    "content_hash": "sha256:abc123..."
  }
}
```

### Semantic Operations

#### 1. Query by Semantic Properties
```json
{
  "query": {
    "kind": "function",
    "effects": ["IO"],
    "is_pure": false,
    "has_tests": true
  }
}
```

#### 2. Transform by Pattern
```json
{
  "transform": {
    "pattern": {
      "type": "function_call",
      "name": "old_function"
    },
    "replacement": {
      "name": "new_function",
      "preserve_args": true
    }
  }
}
```

#### 3. Effect Analysis
```json
{
  "analyze_effects": {
    "function": "processData",
    "transitive": true
  }
}
```

## Test-Specific Syntax

The new test syntax provides rich metadata for AI understanding:

```x
test "user authentication" with tags ["auth", "security"] {
    setup {
        let db = create_test_db()
        let user = create_test_user(db)
        (db, user)
    }
    
    body {
        let result = authenticate(user, "password")
        assert(result.success)
    }
    
    teardown {
        cleanup_db(db)
    }
}
```

AI can query tests by:
- Tags
- Expected outcomes
- Setup/teardown requirements
- Performance constraints

## Benefits for AI

1. **No Parsing Ambiguity**: Direct AST manipulation eliminates syntax errors
2. **Semantic Understanding**: Query and transform based on meaning, not text
3. **Stable References**: Content hashes remain stable across edits
4. **Effect Tracking**: Understand side effects and purity automatically
5. **Type-Aware**: All operations respect the type system

## Example Workflow

1. **Discover**: `x doc . --format semantic` → Get semantic overview
2. **Query**: Find all functions that use IO effects
3. **Transform**: Add logging to all public functions
4. **Verify**: Check that transformations preserve types
5. **Test**: Run affected tests based on dependency graph

## Future Extensions

- Graph-based code navigation
- Semantic diff operations
- AI-specific query language
- Automated refactoring suggestions
- Effect inference and optimization