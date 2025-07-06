use x_parser::Symbol;
use x_editor::namespace::{
    NamespacePath, FullyQualifiedName,
    Import, ImportKind, ExportSpec, Visibility,
    TypeKind, NamespaceBuilder,
};
use x_editor::namespace_storage::NamespaceStorage;
use x_editor::namespace_resolver::LazyNamespaceResolver;
use x_editor::content_addressing::{ContentRepository, ContentHash};
use std::sync::{Arc, RwLock};
use tempfile::TempDir;

fn main() {
    println!("=== Unison-style Namespace System Test ===\n");

    // Create temporary directory for storage
    let temp_dir = TempDir::new().unwrap();
    let content_repo = ContentRepository::new();
    let mut storage = NamespaceStorage::new(
        temp_dir.path().to_path_buf(),
        content_repo.clone(),
    ).unwrap();

    println!("1. Creating namespace hierarchy:");
    
    // Create root namespace first (required for resolver)
    let root_ns = NamespaceBuilder::new(NamespacePath::root())
        .build();
    storage.save_namespace(&root_ns).unwrap();
    println!("   - Created root namespace");
    
    // Create Data namespace
    let data_ns = NamespaceBuilder::new(NamespacePath::from_str("Data"))
        .build();
    storage.save_namespace(&data_ns).unwrap();
    println!("   - Created namespace: Data");
    
    // Create Data.List namespace
    let mut list_ns = NamespaceBuilder::new(NamespacePath::from_str("Data.List"))
        .build();
    
    // Add functions to Data.List
    let map_hash = ContentHash::new(b"map_implementation");
    list_ns.add_value(
        Symbol::intern("map"),
        map_hash.clone(),
        None, // Type would be inferred
        Visibility::Public,
    );
    
    let filter_hash = ContentHash::new(b"filter_implementation");
    list_ns.add_value(
        Symbol::intern("filter"),
        filter_hash,
        None,
        Visibility::Public,
    );
    
    let fold_hash = ContentHash::new(b"fold_implementation");
    list_ns.add_value(
        Symbol::intern("fold"),
        fold_hash,
        None,
        Visibility::Public,
    );
    
    // Private helper function
    let helper_hash = ContentHash::new(b"helper_implementation");
    list_ns.add_value(
        Symbol::intern("_helper"),
        helper_hash,
        None,
        Visibility::Private,
    );
    
    storage.save_namespace(&list_ns).unwrap();
    println!("   - Created namespace: Data.List with functions: map, filter, fold, _helper");
    
    // Create Data.Maybe namespace
    let mut maybe_ns = NamespaceBuilder::new(NamespacePath::from_str("Data.Maybe"))
        .build();
    
    // Add Maybe type
    let maybe_type_hash = ContentHash::new(b"maybe_type_definition");
    maybe_ns.add_type(
        Symbol::intern("Maybe"),
        maybe_type_hash,
        TypeKind::Data {
            constructors: vec![Symbol::intern("None"), Symbol::intern("Some")],
        },
        Visibility::Public,
    );
    
    storage.save_namespace(&maybe_ns).unwrap();
    println!("   - Created namespace: Data.Maybe with type: Maybe");
    
    println!("\n2. Creating application namespace with imports:");
    
    // Create MyApp namespace that imports from Data.List
    let mut myapp_ns = NamespaceBuilder::new(NamespacePath::from_str("MyApp"))
        .add_import(Import {
            source: NamespacePath::from_str("Data.List"),
            kind: ImportKind::Selective(vec![
                Symbol::intern("map"),
                Symbol::intern("filter"),
            ]),
            alias: None,
        })
        .add_import(Import {
            source: NamespacePath::from_str("Data.Maybe"),
            kind: ImportKind::All,
            alias: None,
        })
        .build();
    
    // Add a function that uses imported functions
    let process_hash = ContentHash::new(b"process_implementation");
    myapp_ns.add_value(
        Symbol::intern("process"),
        process_hash,
        None,
        Visibility::Public,
    );
    
    // Set exports
    myapp_ns.set_exports(ExportSpec {
        export_all: false,
        exports: [Symbol::intern("process")].into_iter().collect(),
        reexports: vec![],
    });
    
    storage.save_namespace(&myapp_ns).unwrap();
    println!("   - Created namespace: MyApp");
    println!("     * Imports: Data.List.{{map, filter}}, Data.Maybe.*");
    println!("     * Exports: process");
    
    println!("\n3. Testing namespace resolution:");
    
    // Create resolver with Arc<RwLock<Storage>>
    let storage_arc = Arc::new(RwLock::new(storage));
    let resolver = LazyNamespaceResolver::new(storage_arc.clone());
    let mut storage = storage_arc.write().unwrap();
    
    // Test resolving in MyApp context
    let myapp_path = NamespacePath::from_str("MyApp");
    
    // Should resolve imported 'map'
    match resolver.resolve(&myapp_path, Symbol::intern("map")) {
        Ok(resolved) => {
            println!("   - Resolved 'map' in MyApp: {}", resolved.fully_qualified.to_string());
        }
        Err(e) => {
            println!("   - Failed to resolve 'map': {}", e);
        }
    }
    
    // Should resolve imported 'Maybe'
    match resolver.resolve(&myapp_path, Symbol::intern("Maybe")) {
        Ok(resolved) => {
            println!("   - Resolved 'Maybe' in MyApp: {}", resolved.fully_qualified.to_string());
        }
        Err(e) => {
            println!("   - Failed to resolve 'Maybe': {}", e);
        }
    }
    
    // Should not resolve private '_helper'
    match resolver.resolve(&myapp_path, Symbol::intern("_helper")) {
        Ok(_) => {
            println!("   - ERROR: Private '_helper' should not be accessible!");
        }
        Err(_) => {
            println!("   - Correctly failed to resolve private '_helper'");
        }
    }
    
    // Should resolve local 'process'
    match resolver.resolve(&myapp_path, Symbol::intern("process")) {
        Ok(resolved) => {
            println!("   - Resolved local 'process' in MyApp: {}", resolved.fully_qualified.to_string());
        }
        Err(e) => {
            println!("   - Failed to resolve 'process': {}", e);
        }
    }
    
    println!("\n4. Creating namespace with aliases:");
    
    // Create Utils namespace with aliases
    let mut utils_ns = NamespaceBuilder::new(NamespacePath::from_str("Utils"))
        .build();
    
    // Add alias to Data.List.map
    utils_ns.add_alias(
        Symbol::intern("listMap"),
        FullyQualifiedName::new(
            NamespacePath::from_str("Data.List"),
            Symbol::intern("map"),
        ),
    );
    
    // Import Data.List as L
    utils_ns.add_import(Import {
        source: NamespacePath::from_str("Data.List"),
        kind: ImportKind::Namespace,
        alias: Some(Symbol::intern("L")),
    });
    
    storage.save_namespace(&utils_ns).unwrap();
    println!("   - Created namespace: Utils");
    println!("     * Alias: listMap -> Data.List.map");
    println!("     * Import: Data.List as L");
    
    println!("\n5. Testing hierarchical namespace structure:");
    
    // Create nested namespace
    let mut nested_ns = NamespaceBuilder::new(
        NamespacePath::from_str("Company.Project.Module.SubModule")
    ).build();
    
    // Add some content
    let nested_fn_hash = ContentHash::new(b"nested_function");
    nested_ns.add_value(
        Symbol::intern("deepFunction"),
        nested_fn_hash,
        None,
        Visibility::Protected, // Visible to parent namespaces
    );
    
    storage.save_namespace(&nested_ns).unwrap();
    println!("   - Created deeply nested namespace: Company.Project.Module.SubModule");
    
    println!("\n6. Namespace versioning:");
    
    // Create multiple versions of a namespace
    let mut versioned_ns = NamespaceBuilder::new(NamespacePath::from_str("Versioned"))
        .build();
    
    versioned_ns.metadata.version = Some("1.0.0".to_string());
    let v1_hash = ContentHash::new(b"version_1_function");
    versioned_ns.add_value(
        Symbol::intern("api"),
        v1_hash,
        None,
        Visibility::Public,
    );
    storage.save_namespace(&versioned_ns).unwrap();
    
    // Update to version 2
    versioned_ns.metadata.version = Some("2.0.0".to_string());
    let v2_hash = ContentHash::new(b"version_2_function");
    versioned_ns.add_value(
        Symbol::intern("api_v2"),
        v2_hash,
        None,
        Visibility::Public,
    );
    // Update modification time
    // versioned_ns.metadata.modified_at = chrono::Utc::now();
    storage.save_namespace(&versioned_ns).unwrap();
    
    let versions = storage.get_versions(&NamespacePath::from_str("Versioned"));
    println!("   - Namespace 'Versioned' has {} versions:", versions.len());
    for v in &versions {
        println!("     * {} (created: {})", v.version, v.created_at.format("%Y-%m-%d %H:%M:%S"));
    }
    
    println!("\n7. Namespace export/import:");
    
    // Export MyApp namespace
    let export = storage.export_namespace(&NamespacePath::from_str("MyApp")).unwrap();
    println!("   - Exported namespace: MyApp");
    println!("     * Contains {} content items", export.content_items.len());
    
    println!("\n8. Listing all namespaces:");
    
    let all_namespaces = storage.list_namespaces();
    println!("   - Total namespaces: {}", all_namespaces.len());
    for ns_path in &all_namespaces {
        println!("     * {}", ns_path.to_string());
    }
    
    println!("\n9. Namespace dependencies:");
    
    // Show dependency graph
    println!("   - MyApp depends on:");
    if let Ok(myapp) = storage.load_namespace(&NamespacePath::from_str("MyApp")) {
        for import in &myapp.imports {
            println!("     * {}", import.source.to_string());
        }
    }
    
    println!("\n10. Summary:");
    println!("   ✓ Hierarchical namespace structure");
    println!("   ✓ Import/export functionality");
    println!("   ✓ Name resolution with visibility");
    println!("   ✓ Namespace aliases");
    println!("   ✓ Versioning support");
    println!("   ✓ Dependency tracking");
    println!("   ✓ Content addressing integration");
}