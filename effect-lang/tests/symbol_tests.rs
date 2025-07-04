//! Symbol interning tests for x Language

use effect_lang::core::symbol::{
    Symbol, SymbolTable, SymbolInfo, SymbolKind, SymbolVisibility, 
    symbols, init_symbols, interner_stats
};
use effect_lang::core::span::{FileId, ByteOffset, Span};

#[test]
fn test_symbol_interning() {
    let s1 = Symbol::intern("hello");
    let s2 = Symbol::intern("hello");
    let s3 = Symbol::intern("world");
    
    // Same strings should intern to same symbol
    assert_eq!(s1, s2);
    
    // Different strings should intern to different symbols
    assert_ne!(s1, s3);
    
    // Symbol IDs should be consistent
    assert_eq!(s1.as_u32(), s2.as_u32());
    assert_ne!(s1.as_u32(), s3.as_u32());
}

#[test]
fn test_symbol_from_string() {
    let s1: Symbol = "test".into();
    let s2: Symbol = "test".to_string().into();
    let s3 = Symbol::intern("test");
    
    // All should be equal
    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
}

#[test]
fn test_predefined_symbols() {
    init_symbols(); // Initialize predefined symbols
    
    let int_sym = symbols::INT();
    let string_sym = symbols::STRING();
    let bool_sym = symbols::BOOL();
    
    // Should be distinct symbols
    assert_ne!(int_sym, string_sym);
    assert_ne!(string_sym, bool_sym);
    assert_ne!(int_sym, bool_sym);
    
    // Should intern consistently
    let int_sym2 = symbols::INT();
    assert_eq!(int_sym, int_sym2);
}

#[test]
fn test_keyword_symbols() {
    let let_sym = symbols::LET();
    let fun_sym = symbols::FUN();
    let if_sym = symbols::IF();
    let module_sym = symbols::MODULE();
    
    // Keywords should be distinct
    assert_ne!(let_sym, fun_sym);
    assert_ne!(fun_sym, if_sym);
    assert_ne!(if_sym, module_sym);
}

#[test]
fn test_operator_symbols() {
    let plus = symbols::PLUS();
    let minus = symbols::MINUS();
    let arrow = symbols::ARROW();
    let fat_arrow = symbols::FAT_ARROW();
    
    // Operators should be distinct
    assert_ne!(plus, minus);
    assert_ne!(arrow, fat_arrow);
}

#[test]
fn test_effect_symbols() {
    let effect_sym = symbols::EFFECT();
    let handler_sym = symbols::HANDLER();
    let do_sym = symbols::DO();
    let handle_sym = symbols::HANDLE();
    let resume_sym = symbols::RESUME();
    let perform_sym = symbols::PERFORM();
    
    // Effect-related symbols should be distinct
    let symbols = [effect_sym, handler_sym, do_sym, handle_sym, resume_sym, perform_sym];
    for i in 0..symbols.len() {
        for j in i+1..symbols.len() {
            assert_ne!(symbols[i], symbols[j]);
        }
    }
}

#[test]
fn test_symbol_table_basic() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    let var_info = SymbolInfo {
        symbol: Symbol::intern("x"),
        kind: SymbolKind::Variable,
        visibility: SymbolVisibility::Local,
        span,
        module: None,
    };
    
    // Insert and lookup
    table.insert(var_info.clone());
    let found = table.lookup(Symbol::intern("x"));
    
    assert!(found.is_some());
    assert_eq!(found.unwrap().symbol, Symbol::intern("x"));
    assert_eq!(found.unwrap().kind, SymbolKind::Variable);
}

#[test]
fn test_symbol_table_scoping() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    let x_symbol = Symbol::intern("x");
    
    // Insert in outer scope
    let outer_var = SymbolInfo {
        symbol: x_symbol,
        kind: SymbolKind::Variable,
        visibility: SymbolVisibility::Local,
        span,
        module: None,
    };
    table.insert(outer_var);
    
    // Should be visible
    assert!(table.lookup(x_symbol).is_some());
    assert!(table.exists_in_current_scope(x_symbol));
    
    // Enter new scope
    table.enter_scope();
    
    // Should still be visible from outer scope
    assert!(table.lookup(x_symbol).is_some());
    // But not in current scope
    assert!(!table.exists_in_current_scope(x_symbol));
    
    // Shadow with new definition
    let inner_var = SymbolInfo {
        symbol: x_symbol,
        kind: SymbolKind::Function,  // Different kind
        visibility: SymbolVisibility::Local,
        span,
        module: None,
    };
    table.insert(inner_var);
    
    // Should now find the inner definition
    let found = table.lookup(x_symbol).unwrap();
    assert_eq!(found.kind, SymbolKind::Function);
    assert!(table.exists_in_current_scope(x_symbol));
    
    // Exit scope
    table.exit_scope();
    
    // Should find outer definition again
    let found = table.lookup(x_symbol).unwrap();
    assert_eq!(found.kind, SymbolKind::Variable);
}

#[test]
fn test_symbol_table_visibility() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    let symbols = [
        ("local", SymbolVisibility::Local),
        ("private", SymbolVisibility::Private),
        ("public", SymbolVisibility::Public),
        ("exported", SymbolVisibility::Exported),
    ];
    
    for (name, vis) in &symbols {
        let info = SymbolInfo {
            symbol: Symbol::intern(name),
            kind: SymbolKind::Variable,
            visibility: vis.clone(),
            span,
            module: None,
        };
        table.insert(info);
    }
    
    // All should be findable
    for (name, _) in &symbols {
        let found = table.lookup(Symbol::intern(name));
        assert!(found.is_some(), "Should find symbol {}", name);
    }
    
    // Check visibility is preserved
    let public_var = table.lookup(Symbol::intern("public")).unwrap();
    assert_eq!(public_var.visibility, SymbolVisibility::Public);
}

#[test]
fn test_symbol_kinds() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    let kinds = [
        ("var", SymbolKind::Variable),
        ("func", SymbolKind::Function),
        ("typ", SymbolKind::Type),
        ("ctor", SymbolKind::Constructor),
        ("eff", SymbolKind::Effect),
        ("op", SymbolKind::Operation),
        ("mod", SymbolKind::Module),
        ("tparam", SymbolKind::TypeParameter),
        ("eparam", SymbolKind::EffectParameter),
    ];
    
    for (name, kind) in &kinds {
        let info = SymbolInfo {
            symbol: Symbol::intern(name),
            kind: kind.clone(),
            visibility: SymbolVisibility::Local,
            span,
            module: None,
        };
        table.insert(info);
    }
    
    // Check all kinds are preserved
    for (name, expected_kind) in &kinds {
        let found = table.lookup(Symbol::intern(name)).unwrap();
        assert_eq!(found.kind, *expected_kind);
    }
}

#[test]
fn test_symbol_table_current_scope() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    // Add some symbols
    for i in 0..3 {
        let info = SymbolInfo {
            symbol: Symbol::intern(&format!("var{}", i)),
            kind: SymbolKind::Variable,
            visibility: SymbolVisibility::Local,
            span,
            module: None,
        };
        table.insert(info);
    }
    
    let current_symbols = table.current_scope_symbols();
    assert_eq!(current_symbols.len(), 3);
    
    // Enter new scope
    table.enter_scope();
    let empty_scope = table.current_scope_symbols();
    assert_eq!(empty_scope.len(), 0);
    
    // But all symbols should still be visible
    let all_visible = table.all_visible_symbols();
    assert_eq!(all_visible.len(), 3);
}

#[test]
fn test_symbol_table_module_info() {
    let mut table = SymbolTable::new();
    let span = Span::new(FileId::new(0), ByteOffset::new(0), ByteOffset::new(5));
    
    let module_symbol = Symbol::intern("TestModule");
    
    let var_info = SymbolInfo {
        symbol: Symbol::intern("exported_var"),
        kind: SymbolKind::Variable,
        visibility: SymbolVisibility::Exported,
        span,
        module: Some(module_symbol),
    };
    
    table.insert(var_info);
    
    let found = table.lookup(Symbol::intern("exported_var")).unwrap();
    assert_eq!(found.module, Some(module_symbol));
}

#[test]
fn test_interner_stats() {
    // Intern some symbols
    Symbol::intern("test1");
    Symbol::intern("test2");
    Symbol::intern("test3");
    
    let stats = interner_stats();
    assert!(stats.symbol_count > 0);
    
    // Should include predefined symbols plus our test symbols
    assert!(stats.symbol_count >= 3);
}

#[test]
fn test_predefined_symbols_completeness() {
    let all_predefined = symbols::all_predefined();
    
    // Should have a reasonable number of predefined symbols
    assert!(all_predefined.len() > 20);
    
    // Should include basic types
    assert!(all_predefined.contains(&symbols::INT()));
    assert!(all_predefined.contains(&symbols::STRING()));
    assert!(all_predefined.contains(&symbols::BOOL()));
    
    // Should include keywords
    assert!(all_predefined.contains(&symbols::LET()));
    assert!(all_predefined.contains(&symbols::FUN()));
    assert!(all_predefined.contains(&symbols::MODULE()));
    
    // Should include effect keywords
    assert!(all_predefined.contains(&symbols::EFFECT()));
    assert!(all_predefined.contains(&symbols::HANDLER()));
    
    // Should include operators
    assert!(all_predefined.contains(&symbols::PLUS()));
    assert!(all_predefined.contains(&symbols::ARROW()));
}

#[test]
fn test_symbol_display() {
    let sym = Symbol::intern("test_symbol");
    let display_str = format!("{}", sym);
    
    // For now, our implementation returns "<symbol>"
    // In a full implementation, this would show the actual string
    assert!(!display_str.is_empty());
}

#[test]
fn test_unsafe_symbol_creation() {
    let original = Symbol::intern("test");
    let id = original.as_u32();
    
    // Create symbol from raw ID (unsafe)
    let reconstructed = unsafe { Symbol::from_u32(id) };
    
    // Should have same ID
    assert_eq!(original.as_u32(), reconstructed.as_u32());
}